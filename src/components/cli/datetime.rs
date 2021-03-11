use chrono::naive::NaiveDateTime;

use crate::sys::time;

pub fn date(line: &str) {
    if line == "" {
        println!("{}", time::now());
        return;
    }
    match NaiveDateTime::parse_from_str(line, "%Y-%m-%d %H:%M:%S") {
        Ok(datetime) => match time::update(&datetime) {
            Ok(_) => println!("ok."),
            Err(err) => println!("{:?}", err),
        },
        Err(_) => println!("Malformed datetime: {}", line),
    };
}
