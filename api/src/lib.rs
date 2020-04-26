#![cfg_attr(all(target_arch = "wasm32", target_os = "wasi"), feature(vec_into_raw_parts))]
pub extern crate bincode;

use serde::{Serialize, Deserialize};

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
mod wasm_internal {
    // Export this function so that the host is able to get a buffer in WASM address space that wont
    // interfere with other things
    #[no_mangle]
    pub extern "C" fn __feather_buffer_allocate(size: usize) -> *mut u8 {
        let mut buf = vec![0; size];

        let (ptr, _, _) = buf.into_raw_parts();
        ptr
    }
}

#[cfg(feature = "wasm")]
pub mod wasm;

#[derive(Serialize, Deserialize, Debug)]
pub struct SomeEvent {
    pub foo: f64,
    pub bar: u32,
    pub baz: u8
}

#[derive(Serialize, Deserialize)]
pub struct CommonStruct {
    pub a: u32,
    pub b: u32,
    pub c: String,
}