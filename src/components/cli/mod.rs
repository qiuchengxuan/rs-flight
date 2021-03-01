mod config;
mod datetime;
pub mod memory;

use alloc::boxed::Box;

use git_version::git_version;

use crate::components::logger;
use crate::drivers::terminal::Terminal;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const REVISION: &'static str = git_version!();
const PROMPT: &'static str = "cli> ";

pub struct FnCommand {
    name: &'static str,
    description: &'static str,
    action: fn(&str),
}

macro_rules! command {
    ($name:literal, $description: literal, $action:expr) => {
        FnCommand { name: $name, description: $description, action: $action }
    };
}

const BUILTIN_CMDS: [FnCommand; 12] = [
    command!("date", "Show date", |line| datetime::date(line)),
    command!("dump", "Dump memory address", |line| memory::dump(line)),
    command!("logread", "Read log", |_| print!("{}", logger::get())),
    command!("read", "Read memory address", |line| memory::read(line)),
    command!("readx", "Read memory address in hex", |line| memory::readx(line)),
    command!("writex", "Write memory address in hex", |line| memory::writex(line)),
    command!("import", "Import config", |line| config::import(line)),
    command!("export", "Export config", |line| config::export(line)),
    command!("set", "Set config entry", |line| config::set(line)),
    command!("reset", "Reset config", |_| config::reset()),
    command!("show", "Show config", |_| config::show()),
    command!("version", "Get version", |_| println!("{}-{}", VERSION, REVISION)),
];

pub struct Command {
    name: &'static str,
    description: &'static str,
    action: Box<dyn FnMut(&str)>,
}

impl Command {
    pub fn new(name: &'static str, desc: &'static str, action: impl FnMut(&str) + 'static) -> Self {
        Self { name, description: desc, action: Box::new(action) }
    }
}

pub struct CLI<'a> {
    terminal: Terminal,
    commands: &'a mut [Command],
}

fn prompt() {
    print!("\r{}", PROMPT);
}

impl<'a> CLI<'a> {
    pub fn new(commands: &'a mut [Command]) -> Self {
        CLI { terminal: Terminal::new(), commands }
    }

    pub fn receive(&mut self, bytes: &[u8]) {
        let line = match self.terminal.receive(bytes) {
            Some(line) => line,
            None => return,
        };
        if line.starts_with('#') {
            return prompt();
        }
        let mut split = line.splitn(2, ' ');
        let first_word = match split.next() {
            Some(word) => word,
            None => return prompt(),
        };
        let remain = split.next().unwrap_or("");
        match first_word {
            "" => return prompt(),
            "help" => {
                for command in BUILTIN_CMDS.iter() {
                    println!("{}\t\t{}", command.name, command.description);
                }
                for command in self.commands.iter() {
                    println!("{}\t\t{}", command.name, command.description);
                }
            }
            _ => {}
        };
        if let Some(cmd) = BUILTIN_CMDS.iter().find(|cmd| cmd.name == first_word) {
            (cmd.action)(remain);
        } else if let Some(cmd) = self.commands.iter_mut().find(|cmd| cmd.name == first_word) {
            (cmd.action)(remain);
        } else {
            println!("Unknown command: {}", first_word);
        }
        print!("\r{}", PROMPT);
    }
}
