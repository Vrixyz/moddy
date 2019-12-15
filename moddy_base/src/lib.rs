#[derive(Debug)]
pub enum Command {
    Light(bool),
    Beep,
}
#[derive(Debug)]
pub enum CommandCapability {
    Light,
    Beep,
}
#[derive(Debug)]
pub enum Event<E: Element> {
    Pushed(E, bool),
    // Read(Option<&str>),
}
#[derive(Debug)]
pub enum EventCapability {
    Pushed,
    // Read,
}

pub struct Capabilities {
    pub commands: Vec<CommandCapability>,
    pub events: Vec<EventCapability>,
}

pub trait ServerTrait<E: Element> {
    fn add_element(&mut self, capabilities: &Capabilities) -> Result<E, ()>;
    fn send_command(&mut self, e: &E, command: &Command);
}

pub trait Element {
    fn get_capabilities(&self) -> Capabilities;
    fn is_same(&self, e: &Self) -> bool;
}

pub trait Poller<E: Element> {
    fn poll_events(&mut self) -> Vec<Event<E>>;
}

pub trait Logic<E: Element, S: ServerTrait<E>> {
    fn init(&mut self, server: &mut S) -> Result<(), ()>;
    fn logic_loop(&mut self, server: &mut S, elapsed_seconds: f32);
    fn event_received(&mut self, server: &mut S, event: &Event<E>);
}