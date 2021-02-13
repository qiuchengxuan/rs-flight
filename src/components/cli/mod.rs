mod config;
pub mod memory;

use git_version::git_version;

use crate::components::logger;
use crate::drivers::terminal::Terminal;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const REVISION: &'static str = git_version!();
const PROMPT: &'static str = "cli> ";

pub struct Command {
    name: &'static str,
    description: &'static str,
    action: fn(&str),
}

impl Command {
    pub fn new(name: &'static str, description: &'static str, action: fn(&str)) -> Self {
        Self { name, description, action }
    }
}

macro_rules! command {
    ($name:literal, $description: literal, $action:expr) => {
        Command { name: $name, description: $description, action: $action }
    };
}

const BUILTIN_CMDS: [Command; 9] = [
    command!("dump", "Dump memory address", |line| memory::dump(line)),
    command!("logread", "Read log", |_| print!("{}", logger::get())),
    command!("read", "Read memory address", |line| memory::read(line)),
    command!("readx", "Read memory address in hex", |line| memory::read(line)),
    command!("write", "Write memory address", |line| memory::write(line)),
    command!("save", "Save config", |_| config::save()),
    command!("set", "Set config entry", |line| config::set(line)),
    command!("show", "Show config", |_| config::show()),
    command!("version", "Get version", |_| println!("{}-{}", VERSION, REVISION)),
];

pub struct CLI<'a> {
    terminal: Terminal,
    commands: &'a [Command],
}

impl<'a> CLI<'a> {
    pub fn new(commands: &'a [Command]) -> Self {
        CLI { terminal: Terminal::new(), commands }
    }

    pub fn receive(&mut self, bytes: &[u8]) {
        let line = match self.terminal.receive(bytes) {
            Some(line) => line,
            None => return,
        };
        if line.starts_with('#') {
            print!("\r{}", PROMPT);
            return;
        }
        let first_word = match line.split(' ').next() {
            Some(word) => word,
            None => {
                print!("\r{}", PROMPT);
                return;
            }
        };
        match first_word {
            "" => (),
            "help" => {
                for command in BUILTIN_CMDS.iter() {
                    println!("{}\t{}", command.name, command.description);
                }
                for command in self.commands.iter() {
                    println!("{}\t{}", command.name, command.description);
                }
            }
            _ => {
                let mut cmd = BUILTIN_CMDS.iter().find(|cmd| cmd.name == first_word);
                if cmd.is_none() {
                    cmd = self.commands.iter().find(|cmd| cmd.name == first_word);
                }
                match cmd {
                    Some(cmd) => (cmd.action)(line),
                    None => println!("Unknown command: {}", first_word),
                }
                print!("\r{}", PROMPT);
            }
        }
        print!("\r{}", PROMPT);
    }
}
