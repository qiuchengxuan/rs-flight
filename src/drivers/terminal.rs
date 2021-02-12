use alloc::vec::Vec;

use ascii::{AsciiChar, ToAsciiChar};

pub struct Terminal(Vec<u8>);

const BACKSPACE: &str = "\x08 \x08";

impl Terminal {
    pub fn new() -> Self {
        Self(Vec::with_capacity(80))
    }

    pub fn receive(&mut self, bytes: &[u8]) -> Option<&str> {
        let mut skip = false;
        for &b in bytes.iter() {
            if skip {
                skip = false;
                continue;
            }
            let ch = unsafe { b.to_ascii_char_unchecked() };
            match ch {
                AsciiChar::BackSpace => {
                    if let Some(_) = self.0.pop() {
                        print!("{}", &BACKSPACE);
                    }
                }
                AsciiChar::DEL => {
                    if let Some(_) = self.0.pop() {
                        print!("{}", &BACKSPACE);
                    }
                }
                AsciiChar::CarriageReturn => {
                    println!("{}");
                    return Some(unsafe { core::str::from_utf8_unchecked(self.0.as_slice()) });
                }
                AsciiChar::ETB => {
                    // ^W or CTRL+W
                    while self.0.len() > 0 {
                        if self.0.pop().unwrap() == ' ' as u8 {
                            break;
                        }
                        print!("{}", &BACKSPACE);
                    }
                }
                AsciiChar::ESC => {
                    skip = true;
                }
                _ => {
                    self.0.push(b);
                    print!("{}", ch);
                }
            }
        }
        None
    }
}
