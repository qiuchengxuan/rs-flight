use alloc::vec::Vec;

use ascii::{AsciiChar, ToAsciiChar};

pub struct Terminal {
    buffer: Vec<u8>,
    eol: bool,
}

const BACKSPACE: &str = "\x08 \x08";

impl Terminal {
    pub fn new() -> Self {
        Self { buffer: Vec::with_capacity(80), eol: false }
    }

    pub fn receive(&mut self, bytes: &[u8]) -> Option<&str> {
        if self.eol {
            self.eol = false;
            self.buffer.truncate(0);
        }
        let mut skip = false;
        for &b in bytes.iter() {
            if skip {
                skip = false;
                continue;
            }
            let ch = unsafe { b.to_ascii_char_unchecked() };
            match ch {
                AsciiChar::BackSpace => {
                    if let Some(_) = self.buffer.pop() {
                        print!("{}", &BACKSPACE);
                    }
                }
                AsciiChar::DEL => {
                    if let Some(_) = self.buffer.pop() {
                        print!("{}", &BACKSPACE);
                    }
                }
                AsciiChar::CarriageReturn => {
                    self.eol = true;
                    println!("");
                    return Some(unsafe { core::str::from_utf8_unchecked(self.buffer.as_slice()) });
                }
                AsciiChar::ETB => {
                    // ^W or CTRL+W
                    while self.buffer.len() > 0 {
                        if self.buffer.pop().unwrap() == ' ' as u8 {
                            break;
                        }
                        print!("{}", &BACKSPACE);
                    }
                }
                AsciiChar::ESC => {
                    skip = true;
                }
                _ => {
                    self.buffer.push(b);
                    print!("{}", ch);
                }
            }
        }
        None
    }
}
