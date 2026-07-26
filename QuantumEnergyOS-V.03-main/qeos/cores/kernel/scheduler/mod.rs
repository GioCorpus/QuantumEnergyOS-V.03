use alloc::collections::VecDeque;
use spin::Mutex;

/// Simple round-robin task scheduler.
/// [Research Prototype]
pub struct RoundRobinScheduler {
    ready_queue: Mutex<VecDeque<TaskId>>,
    current: Mutex<Option<TaskId>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(pub usize);

impl RoundRobinScheduler {
    pub const fn new() -> Self {
        Self {
            ready_queue: Mutex::new(VecDeque::new()),
            current: Mutex::new(None),
        }
    }

    pub fn add_task(&self, tid: TaskId) {
        self.ready_queue.lock().push_back(tid);
    }

    pub fn schedule(&self) -> Option<TaskId> {
        let mut q = self.ready_queue.lock();
        let next = q.pop_front();
        if let Some(tid) = next {
            let prev = self.current.replace(tid);
            if let Some(p) = prev {
                q.push_back(p);
            }
        }
        next
    }
}
