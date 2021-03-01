use crate::hal::flash::Flash;

const ACTIVE: u32 = 0x4E565241;
const EMPTY: u32 = 0xFFFFFFFF;

fn offset(sector: &[u32]) -> Option<usize> {
    if sector[1] == EMPTY {
        return None;
    }
    let mut index = 1;
    loop {
        let length = sector[index] as usize;
        if sector[index + length - 1] == EMPTY {
            break; // possibly partial write
        }
        if sector[index + length] == EMPTY {
            break;
        }
        index += length;
    }
    Some(index)
}

pub struct NVRAM<F> {
    flash: F,
    sectors: [&'static mut [u32]; 2],
    active_sector: usize,
    offset: Option<usize>,
}

impl<E, F: Flash<u32, Error = E>> NVRAM<F> {
    pub fn new(mut flash: F, sectors: [&'static mut [u32]; 2]) -> Result<Self, E> {
        if sectors[0][0] != ACTIVE && sectors[0][0] != EMPTY {
            flash.erase(&sectors[0][0] as *const _ as usize)?;
        }
        if sectors[1][0] != ACTIVE && sectors[1][0] != EMPTY {
            flash.erase(&sectors[1][0] as *const _ as usize)?;
        }
        let active_sector = match (sectors[0][0], sectors[1][0]) {
            (ACTIVE, ACTIVE) => {
                let sector1 = &sectors[0];
                let mut active_sector = 0;
                if sector1[1] != EMPTY {
                    if sector1[sector1[1] as usize] != EMPTY {
                        active_sector = 1;
                    }
                }
                flash.erase(&sectors[active_sector ^ 1][0] as *const _ as usize)?;
                active_sector
            }
            (ACTIVE, _) => 0,
            (_, ACTIVE) => 1,
            (_, _) => 0,
        };
        if sectors[active_sector][0] != ACTIVE {
            flash.program(&sectors[active_sector][0] as *const _ as usize, &[ACTIVE])?;
        }
        let offset = offset(sectors[active_sector]);
        Ok(Self { flash, sectors, active_sector, offset })
    }

    pub fn load<'a, T: From<&'a [u32]> + Default>(&'a self) -> Result<T, E> {
        let offset = match self.offset {
            Some(offset) => offset,
            None => return Ok(T::default()),
        };
        let sector = &self.sectors[self.active_sector];
        let length = sector[offset];
        Ok(T::from(&sector[offset + 1..offset + 1 + length as usize]))
    }

    pub fn store<'a, T: AsRef<[u32]>>(&mut self, t: T) -> Result<(), E> {
        let sector = &self.sectors[self.active_sector];
        let mut offset = self.offset.unwrap_or(1);
        while offset < sector.len() && sector[offset] != EMPTY {
            offset += sector[offset] as usize + 1;
        }
        let words = t.as_ref();
        if offset + 1 + words.len() > sector.len() {
            self.active_sector = self.active_sector ^ 1;
            let next_sector = &self.sectors[self.active_sector];
            self.flash.program(&next_sector[0] as *const _ as usize, &[ACTIVE])?;
            self.flash.program(&next_sector[1] as *const _ as usize, &[words.len() as u32])?;
            self.flash.program(&next_sector[2] as *const _ as usize, words)?;
            self.flash.erase(&sector[0] as *const _ as usize)?;
            self.offset = Some(1);
        } else {
            self.flash.program(&sector[offset] as *const _ as usize, &[words.len() as u32])?;
            self.flash.program(&sector[offset + 1] as *const _ as usize, words)?;
            self.offset = Some(offset);
        }
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), E> {
        self.flash.erase(&self.sectors[self.active_sector][0] as *const _ as usize)?;
        self.flash.program(&self.sectors[self.active_sector][0] as *const _ as usize, &[ACTIVE])?;
        self.offset = None;
        Ok(())
    }
}

mod test {
    use core::slice;

    use crate::hal::flash::Flash;

    #[derive(Default)]
    pub struct DummyFlash(());

    impl Flash<u32> for DummyFlash {
        type Error = ();
        fn erase(&mut self, address: usize) -> Result<(), ()> {
            unsafe { *(address as *mut u32) = super::EMPTY }
            Ok(())
        }

        fn program(&mut self, address: usize, words: &[u32]) -> Result<(), ()> {
            let dest = unsafe { slice::from_raw_parts_mut(address as *mut u32, words.len()) };
            dest.copy_from_slice(words);
            Ok(())
        }
    }

    #[derive(Copy, Clone, Debug, Default, PartialEq)]
    pub struct Data([u32; 2]);

    impl AsRef<[u32]> for Data {
        fn as_ref(&self) -> &[u32] {
            &self.0[..]
        }
    }

    impl From<&[u32]> for Data {
        fn from(words: &[u32]) -> Self {
            let mut data = Self::default();
            data.0.copy_from_slice(words);
            data
        }
    }

    #[test]
    fn test_nvram() {
        let sectors = [
            &mut Box::leak(Box::new([super::EMPTY; 8]))[..],
            &mut Box::leak(Box::new([super::EMPTY; 8]))[..],
        ];
        let flash = DummyFlash::default();
        let mut nvram = super::NVRAM::new(flash, sectors).unwrap();
        assert_eq!(nvram.sectors[0][0], super::ACTIVE);
        assert_eq!(nvram.load(), Ok(&[][..]));
        let expect = Data([3, 4]);
        nvram.store(expect).expect("store");
        assert_eq!(expect, nvram.load().expect("load"));
        let expect = Data([5, 6]);
        nvram.store(expect).expect("store");
        assert_eq!(expect, nvram.load().expect("load"));
        let expect = Data([7, 8]);
        nvram.store(expect).expect("store");
        assert_eq!(expect, nvram.load().expect("load"));
        assert_eq!(nvram.sectors[0][0], super::EMPTY);
        let expect = Data([9, 10]);
        nvram.store(expect).expect("store");
        assert_eq!(expect, nvram.load().expect("load"));
        nvram.sectors[0][1] = super::EMPTY;
        let expect = Data([11, 12]);
        nvram.store(expect).expect("store");
        assert_eq!(expect, nvram.load().expect("load"));
    }
}
