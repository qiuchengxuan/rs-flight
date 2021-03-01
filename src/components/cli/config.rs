use crate::config;
use crate::config::setter::{Setter, Value};
use crate::config::yaml::ToYAML;
use crate::sys::fs::{File, OpenOptions};

pub fn show() {
    println!("{}", config::get());
}

pub fn set(line: &str) {
    let mut split = line.split(' ');
    if let Some(path) = split.next() {
        let mut config = config::get().clone();
        match config.set(&mut path.split('.'), Value(split.next())) {
            Ok(()) => (),
            Err(e) => println!("{}", e),
        }
        config::replace(config);
    }
}

pub fn reset() {
    config::replace(config::Config::default());
}

pub fn import(line: &str) {
    let path = match line.split(' ').next() {
        Some(string) => string,
        None => {
            println!("Path must be specified");
            return;
        }
    };
    match File::open(path) {
        Ok(mut file) => {
            config::load(&mut file);
            file.close();
        }
        Err(e) => {
            println!("Import failed: {:?}", e);
        }
    };
}

pub fn export(line: &str) {
    let path = match line.split(' ').next() {
        Some(string) => string,
        None => {
            println!("Path must be specified");
            return;
        }
    };
    let option = OpenOptions { create: true, write: true, truncate: true, ..Default::default() };
    if let Some(mut file) = option.open(path).ok() {
        config::get().write_to(0, &mut file).ok();
        file.close();
    }
}
