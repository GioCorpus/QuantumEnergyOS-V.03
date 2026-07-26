use crate::error::KernelError;
use crate::process::ProcessId;
use crate::regs::TaskRegisters;

#[derive(Debug, Clone)]
pub struct Context {
    pub regs: TaskRegisters,
    pub pid: ProcessId,
    pub privilege: PrivilegeLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeLevel {
    Kernel,
    User,
}
