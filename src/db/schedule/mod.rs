use std::collections::BinaryHeap;

use crate::{
    db::{Timestamp, comments::Topic, user::I2PAddress},
    hash::{Hash, PublicKey},
    helpers::now_timestamp,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleType {
    FullSync(PublicKey),
    SyncMangaContent(Hash),
    SyncPost(Topic),
}

#[derive(Debug, Clone)]
pub struct Schedule {
    pub when: Timestamp,
    pub address: I2PAddress,
    pub schedule_type: ScheduleType,
    pub last_sync: Timestamp,
}

impl Schedule {
    pub fn is_overdue(&self) -> bool {
        self.when <= now_timestamp()
    }
}

impl Ord for Schedule {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.when.cmp(&other.when).reverse()
    }
}

impl PartialOrd for Schedule {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other).reverse())
    }
}

impl PartialEq for Schedule {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.schedule_type == other.schedule_type
    }
}

impl Eq for Schedule {}

#[derive(Debug, Clone)]
pub struct Scheduler {
    // TODO: We should probably use other data structure
    heap: BinaryHeap<Schedule>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub fn schedule(&mut self, schedule: Schedule) {
        self.heap.push(schedule);
    }

    /// Kinda expensive as it has to iterate over the entire heap
    pub fn remove(&mut self, schedule: Schedule) {
        self.heap.retain(|s| s != &schedule);
    }

    /// Returns [`None`] if the schedule is not overdue or if the scheduler is empty
    pub fn try_next(&mut self) -> Option<Schedule> {
        let Some(schedule) = self.heap.peek() else {
            return None;
        };
        if schedule.is_overdue() {
            self.heap.pop()
        } else {
            None
        }
    }
}
