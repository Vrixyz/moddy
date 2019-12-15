use moddy_base::*;

#[derive(Clone)]
struct Button<E: Element> {
    e: E,
}

///
/// Mod
/// Good condidates to implement mod:
/// - Rust, obviously
/// - Rhai: https://github.com/jonathandturner/rhai
///
pub struct LogicWhackAMole<E:Element> {
    all_buttons : Vec<Button<E>>,
    current_button_target : Option<Button<E>>,
    current_button_cycle : Vec<Button<E>>,
}

impl<E: Element> LogicWhackAMole<E> {
    pub fn new() -> Self {
        LogicWhackAMole {
            all_buttons: vec![],
            current_button_target: None,
            current_button_cycle: vec![],
        }
    }
    fn get_random_button(&mut self) -> Option<Button<E>> {
//        self.current_button_cycle.take_random()
        None
    }
}

impl<E: Element + Clone + std::fmt::Debug, S: ServerTrait<E>> Logic<E, S> for LogicWhackAMole<E> {
    fn init(&mut self, server: &mut S) -> Result<(), ()> {
        let caps_push_light_beep = Capabilities {
            events: vec![EventCapability::Pushed],
            commands: vec![CommandCapability::Beep, CommandCapability::Light],
        };
        for _ in 0..5 {
            match server.add_element(&caps_push_light_beep) {
                Ok(e) => self.all_buttons.push(Button {e}),
                _ => break,
            }
        }
        Ok(())
    }
    fn logic_loop(&mut self, server: &mut S, elapsed_seconds: f32) {
        // TODO: change current button after 10 seconds for example
    }
    fn event_received(&mut self, server: &mut S, event: &Event<E>) {
        println!("event received: {:?}", event);
        match event {
            Event::Pushed(e, true) => {
                match &self.current_button_target {
                    Some(current_button_target) => {
                        if current_button_target.e.is_same(&e) {
                            server.send_command(&e, &Command::Light(false));
                            server.send_command(&e, &Command::Beep);
                            self.current_button_target = self.get_random_button();
                            match &self.current_button_target {
                                Some(button) => {
                                    server.send_command(&button.e, &Command::Light(true));
                                },
                                None => {
                                    self.current_button_cycle = self.all_buttons.clone();
                                    self.current_button_target = self.get_random_button();
                                },
                            }
                        }
                    },
                    _ => {},
                };
            },
            _ => {},
        }
    }
}