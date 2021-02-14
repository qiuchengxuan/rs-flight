use chrono::naive::NaiveDateTime;

use crate::sys::time;

pub fn date(line: &str) {
    if line == "" {
        println!("{}", time::now());
        return;
    }
    if let Some(datetime) = NaiveDateTime::parse_from_str(line, "%Y-%m-%d %H:%M:%S").ok() {
        if let Some(err) = time::update(&datetime).err() {
            println!("{}", err)
        }
    } else {
        println!("Malformed datetime: {}", line);
    }
}
