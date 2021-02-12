mod config;
pub mod memory;

use alloc::vec::Vec;
use git_version::git_version;

use crate::alloc;
use crate::components::logger;
use crate::components::telemetry::TelemetryData;
use crate::datastructures::data_source::StaticData;
use crate::drivers::terminal::Terminal;
use crate::sys::timer::SysTimer;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const REVISION: &'static str = git_version!();
const PROMPT: &'static str = "cli> ";

pub struct CLI<T> {
    vec: Vec<u8>,
    timer: SysTimer,
    telemetry: T,
    terminal: Terminal,
    reboot: fn() -> !,
    bootloader: fn() -> !,
    free: fn() -> (usize, usize),
}

impl<T: StaticData<TelemetryData>> CLI<T> {
    pub fn new(
        telemetry: T,
        reboot: fn() -> !,
        bootloader: fn() -> !,
        free: fn() -> (usize, usize),
    ) -> Self {
        CLI {
            vec: Vec::with_capacity(80),
            timer: SysTimer::new(),
            terminal: Terminal::new(),
            telemetry,
            reboot,
            bootloader,
            free,
        }
    }

    pub fn receive(&mut self, bytes: &[u8]) {
        let line = match self.terminal.receive(bytes) {
            Some(line) => line,
            None => return,
        };
        if !line.starts_with('#') {
            if let Some(first_word) = line.split(' ').next() {
                match first_word {
                    "bootloader" => (self.bootloader)(),
                    "dump" => memory::dump(line),
                    "free" => {
                        let (used, free) = (self.free)();
                        println!("Used: {}, free: {}", used, free);
                    }
                    "logread" => print!("{}", logger::get()),
                    "read" | "readx" => memory::read(line),
                    "reboot" => (self.reboot)(),
                    "set" => config::set(line),
                    "show" => config::show(),
                    "save" => config::save(),
                    "telemetry" => println!("{}", self.telemetry.read()),
                    "version" => println!("{}-{}", VERSION, REVISION),
                    "write" => memory::write(line, &mut self.timer),
                    "" => (),
                    _ => println!("Unknown command"),
                }
            }
        }
        print!("{}", PROMPT);
        self.vec.truncate(0);
    }
}
