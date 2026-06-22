use serde::{Deserialize, Serialize};

use super::error::QeosError;
use super::Result;

use crate::device::DeviceTree;
use crate::drivers::DriverManager;
use crate::fs::FileSystemManager;
use crate::io::IOManager;
use crate::memory::MemoryManager;
use crate::net::NetworkStack;
use crate::sched::{Context, ProcessManager, Scheduler};
use crate::security::SecurityManager;
use crate::sync::HwSemaphore;
use crate::syscall::SyscallHandler;

use alloc::sync::Arc;

pub struct QuantumKernel {
    pub memory: Arc<MemoryManager>,
    pub scheduler: Arc<Scheduler>,
    pub processes: Arc<ProcessManager>,
    pub devices: Arc<DeviceTree>,
    pub drivers: Arc<DriverManager>,
    pub fs: Arc<FileSystemManager>,
    pub net: Arc<NetworkStack>,
    pub security: Arc<SecurityManager>,
    pub io: Arc<IOManager>,
    pub syscalls: Arc<SyscallHandler>,
}

impl QuantumKernel {
    pub fn new(boot_info: &BootInfo) -> Result<Self> {
        let memory = MemoryManager::init(boot_info)?;
        let processes = ProcessManager::init()?;
        let scheduler = Scheduler::new(processes.clone())?;
        let devices = DeviceTree::parse(boot_info)?;
        let drivers = DriverManager::new(devices.clone())?;
        let fs = FileSystemManager::new()?;
        let net = NetworkStack::new()?;
        let security = SecurityManager::new()?;
        let io = IOManager::new()?;
        let syscalls = SyscallHandler::new()?;
        Ok(Self {
            memory: Arc::new(memory),
            scheduler: Arc::new(scheduler),
            processes: Arc::new(processes),
            devices: Arc::new(devices),
            drivers: Arc::new(drivers),
            fs: Arc::new(fs),
            net: Arc::new(net),
            security: Arc::new(security),
            io: Arc::new(io),
            syscalls: Arc::new(syscalls),
        })
    }

    pub fn init(&self) -> Result<()> {
        self.memory.init()?;
        self.drivers.probe_all()?;
        self.fs.mount_all()?;
        self.net.init()?;
        self.security.init()?;
        Ok(())
    }

    pub fn run(self: Arc<Self>) -> ! {
        let kernel = self.clone();
        loop {
            let ctx = kernel.scheduler.tick();
            if let Some(task) = ctx {
                kernel.execute_task(task);
            }
            core::hint::spin_loop();
        }
    }

    fn execute_task(&self, task: &Task) {
        let tid = task.id;
        self.syscalls.handle(task);
        if task.exit {
            self.processes.remove(tid);
        }
    }
}
