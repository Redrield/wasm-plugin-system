use crate::SomeEvent;

extern "C" {
    #[allow(improper_ctypes)]
    fn feather_register_event_handler(uuid: &str, funcptr: u32);
}


pub struct PluginInitializer {
    id: String,
}

pub fn init(mut args: std::env::Args) -> PluginInitializer {
    let id = args.next().unwrap();

    PluginInitializer {
        id
    }
}

impl PluginInitializer {
    pub fn register_event_handler(&self, f: extern "C" fn(&[u8])) {
        unsafe { feather_register_event_handler(&self.id, f as usize as u32); }
    }
}
