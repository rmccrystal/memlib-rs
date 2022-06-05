use crate::*;

/// Represents a type that can attach to a process and return
/// a struct that implements MemoryRead, MemoryWrite, and ModuleList
pub trait ProcessAttach: Sized {
    /// The type of the resulting process after attaching
    type ProcessType;

    /// Attaches to a process of name process_name. If no process is found None is returned.
    /// If there is an error internally, this function should panic
    fn attach(&self, process_name: &str) -> Option<Self::ProcessType>;

    /// Attaches to a process by a pid. If the pid does not exist, this will return None
    fn attach_pid(&self, pid: u32) -> Option<Self::ProcessType>;

    /// Attaches to the current process
    fn attach_current(&self) -> Self::ProcessType;
}

/// Represents a type that can attach to a process and return
/// a struct that implements MemoryRead, MemoryWrite, and ModuleList while consuming Self
pub trait ProcessAttachInto: Sized {
    /// The type of the resulting process after attaching
    type ProcessType;

    /// Attaches to a process of name process_name. If no process is found None is returned.
    /// If there is an error internally, this function should panic
    fn attach_into(self, process_name: &str) -> Option<Self::ProcessType>;

    /// Attaches to a process by a pid. If the pid does not exist, this will return None
    fn attach_into_pid(self, pid: u32) -> Option<Self::ProcessType>;

    /// Attaches to the current process, consuming Self
    fn attach_into_current(self) -> Self::ProcessType;
}

impl<T, P> ProcessAttach for T
    where T: Clone + GetContext<Context=P>,
{
    type ProcessType = AttachedProcess<Self, P>;

    fn attach(&self, process_name: &str) -> Option<Self::ProcessType> {
        self.get_context_from_name(process_name)
            .map(|ctx| AttachedProcess::new(self.clone(), ctx))
    }

    fn attach_pid(&self, pid: u32) -> Option<Self::ProcessType> {
        self.get_context_from_pid(pid)
            .map(|ctx| AttachedProcess::new(self.clone(), ctx))
    }

    fn attach_current(&self) -> Self::ProcessType {
        let ctx = self.get_current_context();
        AttachedProcess::new(self.clone(), ctx)
    }
}

impl<T, P> ProcessAttachInto for T
    where
        T: GetContext<Context=P>,
{
    type ProcessType = AttachedProcess<Self, P>;

    fn attach_into(self, process_name: &str) -> Option<Self::ProcessType> {
        self.get_context_from_name(process_name)
            .map(|ctx| AttachedProcess::new(self, ctx))
    }

    fn attach_into_pid(self, pid: u32) -> Option<Self::ProcessType> {
        self.get_context_from_pid(pid)
            .map(|ctx| AttachedProcess::new(self, ctx))
    }

    fn attach_into_current(self) -> Self::ProcessType {
        let ctx = self.get_current_context();
        AttachedProcess::new(self, ctx)
    }
}

/// Gets the Pid type from a process name, a windows PID, or the current PID
pub trait GetContext {
    type Context;

    fn get_context_from_name(&self, process_name: &str) -> Option<Self::Context>;
    fn get_context_from_pid(&self, pid: u32) -> Option<Self::Context>;
    fn get_current_context(&self) -> Self::Context;
}

pub struct AttachedProcess<T, P> {
    pub api: T,
    pub context: P,
}

impl<T, P> AttachedProcess<T, P> {
    pub fn new(api: T, context: P) -> Self {
        Self { api, context }
    }
}

/// A trait that mirrors the MemoryRead trait but reads from a PID instead of directly from the implementor.
/// Note that the Pid type is not necessarily a Windows process ID. One may implement this using another form of identifier such as a dirbase.
pub trait MemoryReadPid: GetContext {
    /// Reads memory from the process with the given PID.
    fn try_read_bytes_into_pid(&self, ctx: &Self::Context, address: u64, buffer: &mut [u8]) -> Option<()>;
}

impl<T, P> MemoryRead for AttachedProcess<T, P>
    where
        T: MemoryReadPid<Context=P>
{
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        self.api.try_read_bytes_into_pid(&self.context, address, buffer)
    }
}

/// A trait that mirrors the MemoryWrite trait but writes to a PID instead of directly from the implementor.
/// Note that the Pid type is not necessarily a Windows process ID. One may implement this using another form of identifier such as a dirbase.
pub trait MemoryWritePid: GetContext {
    /// Writes memory to the process with the given PID.
    fn try_write_bytes_pid(&self, ctx: &Self::Context, address: u64, buffer: &[u8]) -> Option<()>;
}

impl<T, P> MemoryWrite for AttachedProcess<T, P>
    where
        T: MemoryWritePid<Context=P>,
{
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        self.api.try_write_bytes_pid(&self.context, address, buffer)
    }
}

/// A trait that mirrors the ModuleList trait by gets information from a PID instead of directly from the implementor.
/// Note that the Pid type is not necessarily a Windows process ID. One may implement this using another form of identifier such as a dirbase.
pub trait ModuleListPid: GetContext {
    /// Returns a list of all modules from a Pid. If the implementor can only
    /// provide a single module based on the name, this function should panic
    fn get_module_list(&self, pid: &Self::Context) -> Vec<Module>;

    /// Returns a single module by name from the Pid.
    /// If the module name does not exist, returns None
    fn get_module(&self, pid: &Self::Context, name: &str) -> Option<Module> {
        self.get_module_list(pid)
            .into_iter()
            .find(|m| m.name.to_lowercase() == name.to_lowercase())
    }

    /// Gets the main module from the Pid.
    fn get_main_module(&self, pid: &Self::Context) -> Module;
}

impl<T, P> ModuleList for AttachedProcess<T, P>
    where
        T: ModuleListPid<Context=P>,
{
    fn get_module_list(&self) -> Vec<Module> {
        self.api.get_module_list(&self.context)
    }

    fn get_module(&self, name: &str) -> Option<Module> {
        self.api.get_module(&self.context, name)
    }

    fn get_main_module(&self) -> Module {
        self.api.get_main_module(&self.context)
    }
}

/// A trait that mirrors the ProcessInfo trait by gets information from a PID instead of directly from the implementor.
/// Note that the Pid type is not necessarily a Windows process ID. One may implement this using another form of identifier such as a dirbase.
pub trait ProcessInfoPid: GetContext {
    fn process_name(&self, pid: &Self::Context) -> String;
    fn peb_base_address(&self, pid: &Self::Context) -> u64;
    fn pid(&self, pid: &Self::Context) -> u32;
}

impl<T, P> ProcessInfo for AttachedProcess<T, P>
    where
        T: ProcessInfoPid<Context=P>,
{
    fn process_name(&self) -> String {
        self.api.process_name(&self.context)
    }

    fn peb_base_address(&self) -> u64 {
        self.api.peb_base_address(&self.context)
    }

    fn pid(&self) -> u32 {
        self.api.pid(&self.context)
    }
}