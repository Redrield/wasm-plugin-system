use api::wasm::*;
use api::SomeEvent;
use std::env::args;

pub extern "C" fn event_handler(ev_bytes: &[u8]) {
    let ev = api::bincode::deserialize::<api::SomeEvent>(ev_bytes);
    println!("Handling event {:?}", ev);
}

fn main() {
    let plinit = init(args());
    plinit.register_event_handler(event_handler);
}
