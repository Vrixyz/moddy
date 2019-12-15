use moddy_base::*;

use rand::*;

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
    rng : rand::rngs::ThreadRng,
}

impl<E: Element> LogicWhackAMole<E> {
    pub fn new() -> Self {
        LogicWhackAMole {
            all_buttons: vec![],
            current_button_target: None,
            current_button_cycle: vec![],
            rng: rand::thread_rng(),
        }
    }
    fn update_random_button<S: ServerTrait<E>>(&mut self, server: &mut S) {
        let len = self.all_buttons.len();
        if len > 0 {
            let index = if len > 1 { self.rng.gen::<usize>() % (self.all_buttons.len() - 1) } else { 0 };
            if let Some(current_button) = self.current_button_target.take() {
                server.send_command(&current_button.e, &Command::Light(false));
                server.send_command(&current_button.e, &Command::Beep);
                self.all_buttons.push(current_button);
            }
            let new_button = self.all_buttons.remove(index);
            server.send_command(&new_button.e, &Command::Light(true));
            server.send_command(&new_button.e, &Command::Beep);
            self.current_button_target = Some(new_button);
        }
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
        self.update_random_button(server);
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
                            self.update_random_button(server);
                        }
                    },
                    _ => {},
                };
            },
            _ => {},
        }
    }
}