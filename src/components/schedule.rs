use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ptr::{read_volatile, write_volatile};

pub type Rate = usize;

pub trait Schedulable {
    fn pre_schedule(&mut self) {}
    fn schedule(&mut self) -> bool;
    fn post_schedule(&mut self) {}
    fn rate(&self) -> Rate;
}

pub struct TaskInfo {
    counter: usize,
    interval: usize,
}

pub struct Scheduler {
    schedulables: Vec<Box<dyn Schedulable>>,
    rate: Rate,
    task_infos: Vec<TaskInfo>,
    running: bool,
}

impl Scheduler {
    pub fn new(schedulables: Vec<Box<dyn Schedulable>>, rate: Rate) -> Self {
        let mut task_infos: Vec<TaskInfo> = Vec::with_capacity(schedulables.len());
        for schedulable in schedulables.iter() {
            let interval = rate / schedulable.rate();
            task_infos.push(TaskInfo { counter: 0, interval });
        }
        Self { schedulables, rate, task_infos, running: false }
    }
}

impl Schedulable for Scheduler {
    fn schedule(&mut self) -> bool {
        for i in 0..self.schedulables.len() {
            let task_info = &mut self.task_infos[i];
            task_info.counter += 1;
        }

        if unsafe { read_volatile(&self.running) } {
            // in case of re-enter
            return true;
        }

        unsafe { write_volatile(&mut self.running, true) };
        let mut tasks: Vec<(usize, &mut dyn Schedulable)> =
            Vec::with_capacity(self.schedulables.len());
        for (i, schedulable) in self.schedulables.iter_mut().enumerate() {
            let task_info = &mut self.task_infos[i];
            if task_info.counter < task_info.interval {
                continue;
            }
            tasks.push((i, schedulable.as_mut()));
        }
        for (_, schedulable) in tasks.iter_mut() {
            schedulable.pre_schedule();
        }
        for (i, schedulable) in tasks.iter_mut() {
            if schedulable.schedule() {
                self.task_infos[*i].counter = 0;
            }
        }
        for (_, schedulable) in tasks.iter_mut() {
            schedulable.post_schedule();
        }
        unsafe { write_volatile(&mut self.running, false) };
        true
    }

    fn rate(&self) -> Rate {
        self.rate
    }
}
