use communicator_virtual::*;
use logic_whackamole::*;
///
/// Base Library
///
use moddy_base::*;

fn main() -> Result<(), ()> {
    let mut logic = LogicWhackAMole::new();
    let mut communicator = VirtualCommunicator::new();
    use std::{thread, time};

    communicator.run_event_loop();
    logic.init(&mut communicator)?;
    loop {
        thread::sleep(time::Duration::from_millis(100));
        for event in communicator.poll_events() {
            logic.event_received(&mut communicator, &event);
        }
        // TODO: make a proper deltatime
        logic.logic_loop(&mut communicator, 0.1);
    }
}
