use std::collections::HashMap;
use crate::plugin::PluginInstance;
use uuid::Uuid;
use wasmer_runtime::*;
use wasmer_runtime::types::TableIndex;
use wasmer_runtime_core::structures::TypedIndex;
use api::SomeEvent;

/// A struct to keep track of instantiated wasm modules as well as event handlers that they have registered
pub struct FeatherCtx {
    instances: HashMap<Uuid, PluginInstance>,
    event_handlers: Vec<(Uuid, u32)>,
}

impl FeatherCtx {
    pub fn new() -> FeatherCtx {
        FeatherCtx {
            instances: HashMap::new(),
            event_handlers: Vec::new(),
        }
    }

    pub fn add_instance(&mut self, id: Uuid, inst: PluginInstance) -> Uuid {
        self.instances.insert(id, inst);
        id
    }

    pub fn register_event(&mut self, plugin_id: Uuid, ev: u32) {
        self.event_handlers.push((plugin_id, ev));
    }

    pub fn post(&mut self, ev: SomeEvent) {
        let buf = &api::bincode::serialize(&ev).unwrap()[..];

        //TODO: Forcing a clone isn't the best, maybe use a refcell or something
        for (id, f) in self.event_handlers.clone().iter() {
            // Write the event object into the address space of the module, then call the event handler
            let inst = self.instances.get_mut(&id).unwrap();
            let (ptr, len) = inst.write_object_to_memory(buf);
            let cx = inst.inst.context_mut();
            cx.call_with_table_index(TableIndex::new(*f as usize), &[Value::I32(ptr as i32), Value::I32(len as i32)]).unwrap();
            inst.delete(ptr);
        }
    }
}