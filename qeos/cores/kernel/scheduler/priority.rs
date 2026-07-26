use super::{RoundRobinScheduler, Scheduler, TaskId};

/// Priority level for real-time scheduling.
///
/// Higher numeric values indicate higher priority. The scheduler
/// always selects from the highest non-empty level first.
///
/// # Stability
///
/// [Research Prototype] — priority inversion mitigation not implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// Background / maintenance tasks.
    Low = 0,
    /// Normal user tasks.
    Normal = 1,
    /// Interactive / UI tasks.
    High = 2,
    /// Hard real-time (control loops, DMA ISR).
    RealTime = 3,
}

impl Priority {
    /// Total number of priority levels.
    pub const LEVELS: usize = 4;

    /// Returns the backing index for this priority.
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Priority-based preemptive scheduler.
///
/// Delegates each priority level to an independent `RoundRobinScheduler`.
/// On `schedule()`, the highest-priority non-empty level is polled first.
/// This is a **work-preserving** scheduler: lower-priority tasks are
/// never stolen by higher-priority levels.
pub struct PriorityScheduler {
    levels: [RoundRobinScheduler; Priority::LEVELS],
}

impl PriorityScheduler {
    /// Creates an empty priority scheduler with `Priority::LEVELS` queues.
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

    /// Enqueues `tid` at the specified priority level.
    pub fn add(&self, tid: TaskId, prio: Priority) {
        self.levels[prio.index()].add_task(tid);
    }

    /// Returns the number of tasks at `prio`.
    pub fn level_len(&self, prio: Priority) -> usize {
        // Access through queue metric if exposed; otherwise hard to count.
        // Leave as planned API for future metrics.
        let _ = prio;
        0
    }
}

impl Default for PriorityScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler for PriorityScheduler {
    fn schedule(&self) -> Option<TaskId> {
        for level in self.levels.iter().rev() {
            if let Some(tid) = (level as &dyn Scheduler).schedule() {
                return Some(tid);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_priority_runs_before_low() {
        let sched = PriorityScheduler::new();
        sched.add(TaskId(1), Priority::Low);
        sched.add(TaskId(2), Priority::High);

        assert_eq!(sched.schedule(), Some(TaskId(2)));
        assert_eq!(sched.schedule(), Some(TaskId(1)));
    }

    #[test]
    fn empty_levels_skip() {
        let sched = PriorityScheduler::new();
        sched.add(TaskId(1), Priority::RealTime);
        assert!(matches!(sched.schedule(), Some(TaskId(1))));
    }
}
