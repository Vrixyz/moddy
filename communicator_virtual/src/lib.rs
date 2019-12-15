use moddy_base::*;

use std::thread;
use std::sync::mpsc;

use std::borrow::Cow::{self, Borrowed, Owned};

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::{Cmd, CompletionType, Config, Context, EditMode, Editor, KeyPress};
use rustyline_derive::Helper;
use std::collections::HashMap;

struct MyCompleter {
    elements_interactions: HashMap<i32,Vec<EventCapability>>,
}

#[derive(Helper)]
struct MyHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for MyHelper {
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

#[derive(Clone, Debug)] 
pub struct VirtualElement {
    uuid: i32,
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

enum EventLoopModification {
    Stop,
    AddButton(i32),
}

pub struct VirtualCommunicator {
    recv_events: Option<mpsc::Receiver<Event<VirtualElement>>>,
    send_to_event_loop: Option<mpsc::Sender<EventLoopModification>>,
}

impl VirtualCommunicator {
    pub fn new() -> Self {
        Self {
            recv_events: None,
            send_to_event_loop: None,
        }
    }
    pub fn run_event_loop(&mut self) {

        // TODO: Use message passing: https://doc.rust-lang.org/book/ch16-02-message-passing.html
        // event loop send msg event, that we receive in `poll_events`
        // event loop can receive events such as "Stop" or "Add button"
        // add 2 props "channel":
        // - recv_from_event_loop OK
        // - send_to_event_loop
        // Their double is moved inside the thread (recv_from_main_thread and send_to_main_thread)
        let (tx, rx_main_thread) = mpsc::channel();
        self.recv_events = Some(rx_main_thread);
        let (tx_main_thread, rx) = mpsc::channel();
        self.send_to_event_loop = Some(tx_main_thread);
        thread::spawn(move || {
            let config = Config::builder()
                .history_ignore_space(true)
                .completion_type(CompletionType::List)
                .edit_mode(EditMode::Emacs)
                .output_stream(OutputStreamType::Stdout)
                .build();
            let h = MyHelper {
                completer: FilenameCompleter::new(),
                highlighter: MatchingBracketHighlighter::new(),
                hinter: HistoryHinter {},
                colored_prompt: "".to_owned(),
            };
            let mut rl = Editor::with_config(config);
            rl.set_helper(Some(h));
            rl.bind_sequence(KeyPress::Meta('N'), Cmd::HistorySearchForward);
            rl.bind_sequence(KeyPress::Meta('P'), Cmd::HistorySearchBackward);
            if rl.load_history("history.txt").is_err() {
                println!("No previous history.");
            }
            loop {
                let p = format!("> ");
                rl.helper_mut().expect("No helper").colored_prompt = format!("\x1b[1;32m{}\x1b[0m", p);
                let readline = rl.readline(&p);
                match readline {
                    Ok(line) => {
                        rl.add_history_entry(line.as_str());
                        println!("Line: {}", line);
                        let words: Vec<&str> = line.split_whitespace().collect();
                        if words.len() != 2 {
                            println!("Usage: 
 > ElementId Command
                            ");
                            continue;
                        }
                        if let Ok(element_id) = words[0].parse::<i32>() {
                            match words[1] {
                                "true" => {
                                    tx.send(Event::Pushed(VirtualElement{uuid:element_id}, true)).unwrap();
                                },
                                "false" => {
                                    tx.send(Event::Pushed(VirtualElement{uuid:element_id}, false)).unwrap();
                                },
                                _ => {
                                    println!("Command not understood.");
                                }
                            }
                        }
                    }
                    Err(ReadlineError::Interrupted) => {
                        println!("CTRL-C");
                        break;
                    }
                    Err(ReadlineError::Eof) => {
                        println!("CTRL-D");
                        break;
                    }
                    Err(err) => {
                        println!("Error: {:?}", err);
                        break;
                    }
                }
            }
        });
    }
}

impl ServerTrait<VirtualElement> for VirtualCommunicator {
    fn add_element(&mut self, capabilities: &Capabilities) -> Result<VirtualElement, ()> {
        // TODO: return a correct device accepting all these capabilities. 
        // TODO: return an error if these capabilities are not possible.
        Ok(VirtualElement{uuid: 42})
    }
    fn send_command(&mut self, e: &VirtualElement, command: &Command) {
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