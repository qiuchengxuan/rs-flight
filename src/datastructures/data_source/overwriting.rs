use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::num::Wrapping;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::{DataWriter, OptionData, StaticData, WithCapacity};

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
