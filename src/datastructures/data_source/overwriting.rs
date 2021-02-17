use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::fmt;
use core::num::Wrapping;
use core::str::from_utf8_unchecked;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::{DataWriter, OptionData, StaticData, WithCapacity};
use crate::hal::io;

pub struct OverwritingData<T> {
    buffer: UnsafeCell<Vec<T>>,
    write: AtomicUsize,
    written: AtomicUsize,
}

impl<T: Copy + Clone> OverwritingData<T> {
    pub fn new(vec: Vec<T>) -> Self {
        Self {
            buffer: UnsafeCell::new(vec),
            write: AtomicUsize::new(0),
            written: AtomicUsize::new(0),
        }
    }
}

impl<T: Copy + Clone + Default> OverwritingData<T> {
    pub fn sized(size: usize) -> Self {
        Self::new(vec![T::default(); size])
    }
}

impl<T> DataWriter<T> for OverwritingData<T> {
    fn write(&self, value: T) {
        let buffer = unsafe { &mut *self.buffer.get() };
        let size = buffer.len();
        if size > 0 {
            let write = self.write.load(Ordering::Relaxed) as usize;
            let next_write = (Wrapping(write) + Wrapping(1)).0;
            self.write.store(next_write, Ordering::Relaxed);
            buffer[write % size] = value;
            self.written.store(next_write, Ordering::Release);
        }
    }
}

pub struct OverwritingDataSource<T> {
    ring: Rc<OverwritingData<T>>,
    read: usize,
}

impl<T> OverwritingDataSource<T> {
    pub fn new(ring: &Rc<OverwritingData<T>>) -> Self {
        Self { ring: Rc::clone(ring), read: ring.write.load(Ordering::Relaxed) }
    }
}

impl<T> WithCapacity for OverwritingDataSource<T> {
    fn capacity(&self) -> usize {
        let buffer = unsafe { &*self.ring.buffer.get() };
        buffer.len()
    }
}

impl<T: Copy + Clone> OptionData<T> for OverwritingDataSource<T> {
    fn read(&mut self) -> Option<T> {
        let buffer = unsafe { &*self.ring.buffer.get() };
        let mut written = self.ring.written.load(Ordering::Acquire);
        let mut delta = written.wrapping_sub(self.read);
        if delta == 0 {
            return None;
        }
        loop {
            delta = written.wrapping_sub(self.read);
            if delta > buffer.len() {
                self.read = written.wrapping_sub(buffer.len());
            }
            let value = buffer[self.read % buffer.len()];
            let write = self.ring.write.load(Ordering::Relaxed);
            if write.wrapping_sub(self.read) <= buffer.len() {
                self.read = self.read.wrapping_add(1);
                return Some(value);
            }
            written = self.ring.written.load(Ordering::Acquire);
            self.read = self.read.wrapping_add(1);
        }
    }
}

impl<T: Copy> StaticData<T> for OverwritingDataSource<T> {
    fn read(&mut self) -> T {
        let buffer = unsafe { &*self.ring.buffer.get() };
        let written = self.ring.written.load(Ordering::Relaxed);
        buffer[written.wrapping_sub(1) % buffer.len()]
    }
}

impl<T> Clone for OverwritingDataSource<T> {
    fn clone(&self) -> Self {
        Self { ring: Rc::clone(&self.ring), read: self.ring.write.load(Ordering::Relaxed) }
    }
}

impl fmt::Write for OverwritingData<u8> {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let buffer = unsafe { &mut *self.buffer.get() };
        let size = buffer.len();
        if size == 0 {
            return Ok(());
        }
        let mut buf = [0u8; 2];
        let bytes = c.encode_utf8(&mut buf).as_bytes();
        let write = self.write.load(Ordering::Relaxed) as usize;
        let next_write = write.wrapping_add(bytes.len());
        self.write.store(next_write, Ordering::Relaxed);
        buffer[write % size] = bytes[0];
        if bytes.len() > 1 {
            buffer[write.wrapping_add(1) % size] = bytes[1];
        }
        self.written.store(next_write, Ordering::Relaxed);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        let buffer = unsafe { &mut *self.buffer.get() };
        let mut bytes = s.as_bytes();
        if buffer.len() <= bytes.len() {
            bytes = &bytes[..buffer.len()];
        }
        let size = buffer.len();
        let write = self.write.load(Ordering::Relaxed) as usize % size;
        let next_write = write.wrapping_add(bytes.len()) % size;
        self.write.store(next_write, Ordering::Relaxed);

        if size - write > bytes.len() {
            buffer[write..write + bytes.len()].copy_from_slice(bytes);
            self.written.store(next_write, Ordering::Relaxed);
            return Ok(());
        }

        let partial_size = size - write;
        buffer[write..size].copy_from_slice(&bytes[..partial_size]);
        buffer[..bytes.len() - partial_size].copy_from_slice(&bytes[partial_size..]);
        self.written.store(next_write, Ordering::Relaxed);
        Ok(())
    }
}

impl fmt::Display for OverwritingDataSource<u8> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let buffer = unsafe { &*self.ring.buffer.get() };
        let written = self.ring.written.load(Ordering::Relaxed);
        if written <= buffer.len() {
            return write!(f, "{}", unsafe { from_utf8_unchecked(&buffer[..written]) });
        }
        let size = core::cmp::min(written.wrapping_sub(self.read), buffer.len());
        let index = written % buffer.len();
        write!(f, "{}", unsafe { from_utf8_unchecked(&buffer[index..]) })?;
        write!(f, "{}", unsafe { from_utf8_unchecked(&buffer[..size - (buffer.len() - index)]) })
    }
}

impl OverwritingDataSource<u8> {
    pub fn write<E>(&mut self, writer: &mut impl io::Write<Error = E>) -> Result<usize, E> {
        let buffer = unsafe { &*self.ring.buffer.get() };
        let written = self.ring.written.load(Ordering::Relaxed);
        if written.wrapping_sub(self.read) > buffer.len() {
            self.read = written.wrapping_sub(buffer.len());
        }
        let size = written.wrapping_sub(self.read);
        let index = written % buffer.len();
        if size + index <= buffer.len() {
            return writer.write(&buffer[index..index + size]);
        }
        writer.write(&buffer[index..])?;
        writer.write(&buffer[..size - (buffer.len() - index)])
    }
}

mod test {
    #[test]
    fn test_ring_buffer() {
        use alloc::rc::Rc;

        use super::{DataWriter, OptionData, OverwritingData, OverwritingDataSource};

        let ring: Rc<OverwritingData<usize>> = Rc::new(OverwritingData::sized(32));
        let mut reader = OverwritingDataSource::new(&ring);

        assert_eq!(reader.read(), None);

        ring.write(10086);
        assert_eq!(reader.read(), Some(10086));

        ring.write(10010);
        assert_eq!(reader.read(), Some(10010));

        for i in 1..33 {
            ring.write(i);
        }

        assert_eq!(reader.read(), Some(1));
        assert_eq!(reader.read(), Some(2));
    }

    #[test]
    fn test_ring_buffer_as_static() {
        use alloc::rc::Rc;
        use core::sync::atomic::Ordering;

        use super::{DataWriter, OverwritingData, OverwritingDataSource, StaticData};

        let ring: Rc<OverwritingData<usize>> = Rc::new(OverwritingData::sized(32));
        let mut reader = OverwritingDataSource::new(&ring);

        ring.write.store(usize::MAX, Ordering::Relaxed);
        ring.write(10010);
        ring.write(10086);
        assert_eq!(reader.read(), 10086);
    }
}
