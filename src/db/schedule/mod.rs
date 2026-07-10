use std::{
    collections::BinaryHeap,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use crate::{
    db::user::I2PAddress,
    types::{Hash, PublicKey, Timestamp, Topic},
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
        self.when <= Timestamp::now()
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

#[derive(Debug)]
pub struct Scheduler {
    // TODO: We should probably use other data structure
    heap: BinaryHeap<Schedule>,
    delay: Pin<Box<tokio::time::Interval>>,
}

impl Scheduler {
    pub fn new() -> Self {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        Self {
            heap: BinaryHeap::new(),
            delay: Box::pin(interval),
        }
    }

    /// Adds schedule to scheduler
    pub fn schedule(&mut self, schedule: Schedule) {
        self.heap.push(schedule);
    }

    /// Kinda expensive as it has to iterate over the entire heap
    pub fn remove(&mut self, schedule: Schedule) {
        self.heap.retain(|s| s != &schedule);
    }

    /// Returns [`None`] if the schedule is not overdue or if the scheduler is
    /// empty
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

impl Future for &mut Scheduler {
    type Output = Schedule;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        match this.delay.as_mut().poll_tick(cx) {
            Poll::Ready(_) => match this.heap.peek() {
                Some(schedule) if schedule.is_overdue() => Poll::Ready(this.heap.pop().unwrap()),
                _ => Poll::Pending,
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
