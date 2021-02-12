use crate::config;
use crate::config::setter::{Setter, Value};
use crate::config::yaml::ToYAML;
use crate::sys::fs::OpenOptions;

pub fn show() {
    println!("{}", config::get());
}

pub fn set(line: &str) {
    let mut split = line.split(' ');
    split.next();
    if let Some(path) = split.next() {
        let mut config = config::get().clone();
        match config.set(&mut path.split('.'), Value(split.next())) {
            Ok(()) => (),
            Err(e) => println!("{}", e),
        }
        config::replace(config);
    }
}

pub fn save() {
    let option = OpenOptions { create: true, write: true, truncate: true, ..Default::default() };
    if let Some(mut file) = option.open("sdcard://config.yml").ok() {
        config::get().write_to(0, &mut file).ok();
        file.close();
    }
}
