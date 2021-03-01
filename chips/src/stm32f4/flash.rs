use core::mem;
use core::slice;

use drone_core::reg::prelude::*;
use drone_cortexm::reg::prelude::*;
use drone_stm32_map::periph::flash::FlashPeriph;

pub struct Flash(FlashPeriph);

const KEY1: u32 = 0x45670123;
const KEY2: u32 = 0xCDEF89AB;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
    ProgrammingSequence,
    ProgrammingParallelism,
    ProgrammingAlignment,
    WriteProtection,
    Operation,
}

#[derive(Copy, Clone)]
pub struct Sector(u32);

impl Sector {
    pub fn new(index: usize) -> Option<Self> {
        match index {
            0..=11 => Some(Self(index as u32)),
            _ => None,
        }
    }

    pub fn from_address(address: usize) -> Option<Self> {
        let sector = match address {
            0x0800_0000 => 0,
            0x0800_4000 => 1,
            0x0800_8000 => 2,
            0x0800_C000 => 3,
            0x0801_0000 => 4,
            0x0802_0000 => 5,
            0x0804_0000 => 6,
            0x0806_0000 => 7,
            0x0808_0000 => 8,
            0x080A_0000 => 9,
            0x080C_0000 => 10,
            0x080E_0000 => 11,
            _ => return None,
        };
        Some(Self(sector))
    }

    pub fn address(&self) -> usize {
        match self.0 {
            0 => 0x0800_0000,
            1 => 0x0800_4000,
            2 => 0x0800_8000,
            3 => 0x0800_C000,
            4 => 0x0801_0000,
            5 => 0x0802_0000,
            6 => 0x0804_0000,
            7 => 0x0806_0000,
            8 => 0x0808_0000,
            9 => 0x080A_0000,
            10 => 0x080C_0000,
            11 => 0x080E_0000,
            _ => unreachable!(),
        }
    }

    pub fn size(&self) -> usize {
        match self.0 {
            0 => 16 * 1024,
            1 => 16 * 1024,
            2 => 16 * 1024,
            3 => 16 * 1024,
            4 => 64 * 1024,
            5..=11 => 128 * 1024,
            _ => unreachable!(),
        }
    }

    pub unsafe fn as_slice<T>(&self) -> &'static mut [T] {
        let address = self.address();
        let size = self.size();
        slice::from_raw_parts_mut(address as *mut T, size / mem::size_of::<T>())
    }
}

impl Flash {
    pub fn new(regs: FlashPeriph) -> Self {
        regs.flash_cr.modify(|r| r.set_errie().set_eopie());
        Self(regs)
    }

    fn unlock(&mut self) {
        self.0.flash_keyr.store(|r| r.write_key(KEY1));
        self.0.flash_keyr.store(|r| r.write_key(KEY2));
    }

    fn lock(&mut self) {
        self.0.flash_cr.modify(|r| r.set_lock())
    }

    fn status(&self) -> Result<(), Error> {
        let status = self.0.flash_sr.load();
        self.0.flash_sr.store(|r| r.set_pgserr().set_pgperr().set_wrperr().set_operr());
        match () {
            _ if status.pgserr() => Err(Error::ProgrammingSequence),
            _ if status.pgperr() => Err(Error::ProgrammingParallelism),
            _ if status.wrperr() => Err(Error::WriteProtection),
            _ if status.operr() => Err(Error::Operation),
            _ => Ok(()),
        }
    }

    fn wait_busy(&self) {
        while self.0.flash_sr.load().bsy() {}
    }

    pub fn erase(&mut self, sector: Sector) -> Result<(), Error> {
        self.unlock();
        cortex_m::interrupt::free(|_| {
            self.0.flash_cr.store(|r| r.write_snb(sector.0).write_psize(0b10).set_ser().set_strt());
            self.wait_busy();
        });
        self.lock();
        self.status()
    }

    pub fn program(&mut self, address: usize, words: &[u32]) -> Result<(), Error> {
        let mut result = Ok(());
        self.unlock();
        cortex_m::interrupt::free(|_| {
            self.0.flash_cr.store(|r| r.write_psize(0b10).set_pg());
            let dest = unsafe { slice::from_raw_parts_mut(address as *mut u32, words.len()) };
            for i in 0..words.len() {
                dest[i] = words[i];
                self.wait_busy();
                if let Some(error) = self.status().err() {
                    result = Err(error);
                    break;
                }
            }
        });
        self.lock();
        result
    }
}
