mod interactive;

use std::sync::Arc;
use std::sync::Mutex;
use moddy_base::*;

use std::thread;
use std::sync::mpsc;

#[derive(Clone, Debug)] 
pub struct VirtualElement {
    uuid: i32,
    is_lightened : bool,
}

impl Element for VirtualElement {
    fn get_capabilities(&self) -> Capabilities {
        Capabilities {
            commands: vec![CommandCapability::Light],
            events: vec![EventCapability::Pushed],
        }
    }
    fn is_same(&self, e: &VirtualElement) -> bool {
        self.uuid == e.uuid
    }
}

pub enum EventLoopModification {
    Stop,
    UpdateElement(VirtualElement),
}

pub struct VirtualCommunicator {
    elems: Arc<Mutex<Vec<VirtualElement>>>,
    recv_events: Option<mpsc::Receiver<Event<VirtualElement>>>,
    send_to_event_loop: Option<mpsc::Sender<EventLoopModification>>,
}

impl VirtualCommunicator {
    pub fn new() -> Self {
        Self {
            elems: Arc::new(Mutex::new(vec![])),
            recv_events: None,
            send_to_event_loop: None,
        }
    }
    pub fn run_event_loop(&mut self) {

        // event loop send msg event, that we receive in `poll_events`
        // event loop can receive events such as "Stop" or "Add button"
        // add 2 props "channel":
        // - recv_from_event_loop OK
        // - send_to_event_loop
        // Their double is moved inside the thread (recv_from_main_thread and send_to_main_thread)
        let (events_tx, rx_events) = mpsc::channel();
        self.recv_events = Some(rx_events);
        let (tx_main_thread, rx) = mpsc::channel();
        self.send_to_event_loop = Some(tx_main_thread);
        let elems = self.elems.clone();
        // TODO: here we can have multiple event managers (physical, local, internet, ...), we receive all events the same way, but for each command/event impact, we have to adress them specifically.
        thread::spawn(move || {
            let mut session = interactive::Interactive::new(rx, events_tx);
            session.run();
        });
    }
}

impl ServerTrait<VirtualElement> for VirtualCommunicator {
    fn add_element(&mut self, capabilities: &Capabilities) -> Result<VirtualElement, ()> {
        // TODO: return a correct device accepting all these capabilities. 
        // TODO: return an error if these capabilities are not possible.
        loop {
            if let Ok(mut elems) = self.elems.lock() {
                let new_elem = VirtualElement{uuid: elems.len() as i32, is_lightened: false};
                let ret = new_elem.clone();
                elems.push(new_elem);
                let sender = self.send_to_event_loop.clone().unwrap();
                sender.send(EventLoopModification::UpdateElement(ret.clone())).unwrap();
                return Ok(ret);
            }
        }
        Err(())
    }
    fn send_command(&mut self, e: &VirtualElement, command: &Command) {
        match command {
            Command::Light(v) => {
                let mut elem = None;
                if let Ok(mut elems) = self.elems.lock() {
                    if let Some(real_e) = elems.iter().position(|x| x.is_same(e)) {
                        elems[real_e].is_lightened = *v;
                        elem = Some(elems[real_e].clone());
                    }
                }
                if let Some(cloned_e) = elem {
                    let sender = self.send_to_event_loop.clone().unwrap();
                    sender.send(EventLoopModification::UpdateElement(cloned_e)).unwrap();
                }
            },
            _ => {

            }
        }
        println!("{:?}: {:?}", e.uuid, command);
    }
}
impl Poller<VirtualElement> for VirtualCommunicator {
    fn poll_events(&mut self) -> Vec<Event<VirtualElement>> {
        let mut res = vec![];
        if let Some(rx) = &self.recv_events {
            while let Ok(event) = rx.try_recv() {
                res.push(event);
            }
        }
        res
    }
}