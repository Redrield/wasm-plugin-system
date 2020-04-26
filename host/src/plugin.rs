use wasmer_runtime::*;
use wasmer_runtime::error::CallResult;

/// A struct around an instantiated plugin that provides functions for dealing
/// with the plugin's memory
pub struct PluginInstance {
    pub inst: Instance,
    buf_ptr_base: u32,
    buf_pool_size: usize,
    write_offset: isize,
}

impl PluginInstance {
    pub fn new(inst: Instance) -> PluginInstance {
        // The exported memory is used by the plugins libstd allocator for heap structures
        // so the best way to make sure that a region of memory wont interfere with structures
        // that a plugin may be allocating is to let it give us the pointer, and make sure to 
        // remain in the asked for bounds when interacting with plugin memory
        let alloc = inst.func::<u32, u32>("__feather_buffer_allocate").unwrap();
        let ptr = alloc.call(1024).unwrap();
        println!("Host has 0x{:x}..0x{:x} to play with", ptr, ptr + 1024);

        PluginInstance {
            inst,
            buf_ptr_base: ptr,
            buf_pool_size: 1024,
            write_offset: 0,
        }
    }

    /// Writes the given slice into the plugin's address space, returning 
    /// the memory offset and length of the written data
    pub fn write_object_to_memory(&mut self, buf: &[u8]) -> (u32, u32) {
        // Make sure we wont overflow our buffer
        if self.buf_ptr_base + self.write_offset as u32 > self.buf_ptr_base + self.buf_pool_size as u32 {
            panic!("Out of mem");
        }
        let mem = self.inst.context().memory(0);
        unsafe {
            // Get the write pointer based on our offset into WASM memory, and the offset into our buffer we've written to
            //let ptr = mem.data_ptr().offset(self.buf_ptr_base as isize + self.write_offset);
            let ptr = WasmPtr::<u8, Array>::new(self.buf_ptr_base + self.write_offset as u32);
        
            let mut wasm_buf = ptr.deref_mut(mem, 0, buf.len() as u32).unwrap();
            for (i, b) in wasm_buf.iter().enumerate() {
                b.set(buf[i]);
            }
        }
        // Keep track of where the newly written data starts, and increment the write offset by the data written
        let ptr_handle = self.buf_ptr_base + self.write_offset as u32;
        self.write_offset += buf.len() as isize;
        (ptr_handle, buf.len() as u32)
    }

    /// Resets the write offset to the head of this pointer
    /// FIXME: This doesn't work at all for a proper heap allocator
    pub fn delete(&mut self, ptr: u32) {
        self.write_offset = (ptr - self.buf_ptr_base) as isize;
    }

    /// Dynamically calls a function with the given name and args
    pub fn call_function(&self, name: &str, args: &[Value]) -> CallResult<Vec<Value>> {
        self.inst.dyn_func(name).unwrap().call(args)
    }
}
