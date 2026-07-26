use super::RoundRobinScheduler;
use super::TaskId;

/// Priority-based preemptive scheduler.
/// [Research Prototype]
pub struct PriorityScheduler {
    levels: [RoundRobinScheduler; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    RealTime = 3,
}

impl PriorityScheduler {
    pub const fn new() -> Self {
        Self {
            levels: [
                RoundRobinScheduler::new(),
                RoundRobinScheduler::new(),
                RoundRobinScheduler::new(),
                RoundRobinScheduler::new(),
            ],
        }
    }

    pub fn add(&self, tid: TaskId, prio: Priority) {
        self.levels[prio as usize].add_task(tid);
    }

    pub fn schedule(&self) -> Option<TaskId> {
        for level in self.levels.iter().rev() {
            if let Some(tid) = level.schedule() {
                return Some(tid);
            }
        }
        None
    }
}
