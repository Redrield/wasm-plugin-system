

mod plugin;
use plugin::PluginInstance;

mod ctx;
use ctx::FeatherCtx;

use wasmer_runtime::*;
use wasmer_wasi::generate_import_object;
use uuid::Uuid;
use std::fs;
use std::sync::{Arc, Mutex};

/// Body of the syscall to track event handlers
fn add_event_handler(cx: Arc<Mutex<FeatherCtx>>, vmcx: &mut Ctx, id_ptr: u32, id_len: u32, func_ptr: u32) {
    use std::str::FromStr;

    println!("Function pointer gave {}", func_ptr);
    let ptr = WasmPtr::<u8, Array>::new(id_ptr);
    let id = ptr.get_utf8_string(vmcx.memory(0), id_len).unwrap();
    let id = Uuid::from_str(id).unwrap();
    cx.lock().unwrap().register_event(id, func_ptr);
}

fn main() {
    let wasm_bytes = fs::read("./target/wasm32-wasi/debug/plugin.wasm").unwrap();
    let module = compile(&wasm_bytes[..]).unwrap();

    let cx = Arc::new(Mutex::new(FeatherCtx::new()));

    let syscall_cx = cx.clone();
    let env_imports = imports! {
        "env" => {
            "feather_register_event_handler" => func!(move |vmcx: &mut Ctx, id_ptr: u32, id_len: u32, func_ptr: u32| {
                // weirdly need to clone a second time
                add_event_handler(syscall_cx.clone(), vmcx, id_ptr, id_len, func_ptr);
            }),
        },
    };

    let plid = Uuid::new_v4();
    let id_str = plid.to_hyphenated_ref().to_string();
    // Pass plugin ID to plugin via argv
    let mut wasi = generate_import_object(vec![id_str.into_bytes()], Vec::new(), Vec::new(), Vec::new());
    wasi.extend(env_imports);

    let inst = module.instantiate(&wasi)
        .unwrap();
    
    // Instantiate the module, run _start to let it run initialization steps, then add it to context
    let inst = PluginInstance::new(inst);
    inst.call_function("_start", &[]).unwrap();
    cx.lock().unwrap().add_instance(plid, inst);

    // Send an event
    cx.lock().unwrap().post(api::SomeEvent { foo: 3.0, bar: 5, baz: 69 });
}