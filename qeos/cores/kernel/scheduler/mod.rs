#![warn(missing_docs)]

//! QuantumEnergyOS Kernel Scheduler
//!
//! Provides cooperative and priority-based task scheduling primitives.
//! [Research Prototype] — not a production preemptive scheduler.

use std::collections::VecDeque;
use spin::Mutex;

/// Unique task identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub usize);

impl TaskId {
    /// Creates a new task identifier.
    pub const fn new(id: usize) -> Self {
        Self(id)
    }
}

impl core::fmt::Display for TaskId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "T{}", self.0)
    }
}

/// Scheduling policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingPolicy {
    /// First-in, first-out within a priority level.
    RoundRobin,
    /// static priority with preemption.
    Priority,
}

/// Scheduler trait — abstracts over scheduling algorithms.
pub trait Scheduler {
    fn add_task(&self, tid: TaskId);
    fn schedule(&self) -> Option<TaskId>;
}

/// Simple round-robin task scheduler.
///
/// Uses a FIFO ready queue. The currently running task is stored
/// separately and re-queued at the tail on the next `schedule()` call.
///
/// # Thread safety
///
/// Safe to share between threads via `&self` because interior mutability
/// is protected by `spin::Mutex`. **Not safe for No-MMU no-atomic contexts.**
pub struct RoundRobinScheduler {
    ready_queue: Mutex<VecDeque<TaskId>>,
    current: Mutex<Option<TaskId>>,
}

impl RoundRobinScheduler {
    /// Creates an empty scheduler.
    pub const fn new() -> Self {
        Self {
            ready_queue: Mutex::new(VecDeque::new()),
            current: Mutex::new(None),
        }
    }

    /// Enqueues a task for execution.
    pub fn add_task(&self, tid: TaskId) {
        self.ready_queue.lock().push_back(tid);
    }
}

impl Default for RoundRobinScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler for RoundRobinScheduler {
    fn add_task(&self, tid: TaskId) {
        self.ready_queue.lock().push_back(tid);
    }

    fn schedule(&self) -> Option<TaskId> {
        let mut q = self.ready_queue.lock();
        let next = q.pop_front()?;
        let mut current = self.current.lock();
        let prev = current.replace(next);
        if let Some(p) = prev {
            q.push_back(p);
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_robin_rotates_tasks() {
        let sched = RoundRobinScheduler::new();
        sched.add_task(TaskId(1));
        sched.add_task(TaskId(2));

        assert_eq!(sched.schedule(), Some(TaskId(1)));
        assert_eq!(sched.schedule(), Some(TaskId(2)));
        assert_eq!(sched.schedule(), Some(TaskId(1)));
    }

    #[test]
    fn empty_schedule_returns_none() {
        let sched = RoundRobinScheduler::new();
        assert!(sched.schedule().is_none());
    }
}
