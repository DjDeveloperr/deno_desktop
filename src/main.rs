#[macro_use]
extern crate lazy_static;

mod runtime;
mod event_loop;
mod util;
mod window;
mod extra;

use winit_main::EventLoopHandle;
use winit_main::EventReceiver;

use std::sync::Arc;
use std::sync::Mutex;

lazy_static! {
    pub static ref EVENT_LOOP: Mutex<Option<Arc<Mutex<EventLoopHandle>>>> = Mutex::new(None);
    pub static ref EVENT_RECEIVER: Mutex<Option<Arc<Mutex<EventReceiver>>>> = Mutex::new(None);
}

#[winit_main::main]
fn main(event_loop: EventLoopHandle, events: EventReceiver) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let arc_events = Arc::new(Mutex::new(events));

    rt.block_on(async {
        *EVENT_LOOP.lock().unwrap() = Some(Arc::new(Mutex::new(event_loop)));
        *EVENT_RECEIVER.lock().unwrap() = Some(arc_events);

        match runtime::start().await {
            Err(err) => eprintln!("{}", err.to_string()),
            _ => {},
        }
    });
}
