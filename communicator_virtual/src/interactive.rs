use crate::EventLoopModification::*;
use cursive::views::ScrollView;
use moddy_base::Event;
use crate::EventLoopModification;
use crate::VirtualElement;
use cursive::traits::*;
use cursive::views::{
    Checkbox, Dialog, LinearLayout, ListView, TextView, Button, Canvas,
};
use cursive::Cursive;
use std::sync::mpsc;

// This example uses a ListView.
//
// ListView can be used to build forms, with a list of inputs.

pub struct Interactive {
    siv: Cursive,
    /// To receive updates to adapt interface
    recv_update: mpsc::Receiver<EventLoopModification>,
    /// To send events
    send_event: mpsc::Sender<Event<VirtualElement>>,
}

impl Interactive {
    pub fn new(r: mpsc::Receiver<EventLoopModification>, s: mpsc::Sender<Event<VirtualElement>>) -> Self {
        let mut siv = Cursive::default();
        siv.set_autorefresh(true);
        siv.add_layer(
            Dialog::around(
                LinearLayout::vertical()
                    .with_id("elem_list")
                    .scrollable()
                    .scroll_x(false)
            ),
        );
        Interactive {
            siv,
            recv_update: r,
            send_event: s,
        }
    }

    pub fn run(&mut self) {
        while self.siv.is_running() {
            while let Ok(event) = self.recv_update.try_recv() {
                match event {
                    UpdateElement(e) => {
                        if let Some(mut button) = self.siv.find_id::<Button>(&e.uuid.to_string()) {
                            button.set_label(if e.is_lightened {"Button (light)"} else {"Button"});
                        }
                        else {// TODO: check if we don't have this element, create it if we don't have it
                        // TODO: update the element if we have it
                        let mut linear_layout = self.siv.find_id::<LinearLayout>("elem_list").unwrap();
                        let send_event = self.send_event.clone();
                        let button_text = if e.is_lightened {"Button (light)"} else {"Button"};
                        let id = e.uuid;
                        linear_layout
                            .add_child(
                                Button::new(button_text, move |s| {
                                    // TODO: send event to push/unpush button with correct id
                                    send_event.send(Event::Pushed(VirtualElement {uuid:e.uuid, is_lightened: e.is_lightened}, true)).unwrap();
                                    // We can add a layer popup to choose interaction options
                                    // s.add_layer(Dialog::info("Ah"));
                                })
                                .with_id(id.to_string()));
                        }
                    },
                    _ => {
                        // TODO: handle stopping
                    }
                }
            }
            self.siv.step();
        }
    }
}