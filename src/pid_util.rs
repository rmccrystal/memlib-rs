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
    where T: Clone + GetPid<Pid=P>,
{
    type ProcessType = AttachedProcess<Self, P>;

    fn attach(&self, process_name: &str) -> Option<Self::ProcessType> {
        self.get_pid_from_name(process_name)
            .map(|pid| AttachedProcess::new(self.clone(), pid))
    }

    fn attach_pid(&self, pid: u32) -> Option<Self::ProcessType> {
        self.get_pid_from_windows_pid(pid)
            .map(|win_pid| AttachedProcess::new(self.clone(), win_pid))
    }

    fn attach_current(&self) -> Self::ProcessType {
        let pid = self.get_current_pid();
        AttachedProcess::new(self.clone(), pid)
    }
}

impl<T, P> ProcessAttachInto for T
    where
        T: GetPid<Pid=P>,
{
    type ProcessType = AttachedProcess<Self, P>;

    fn attach_into(self, process_name: &str) -> Option<Self::ProcessType> {
        self.get_pid_from_name(process_name)
            .map(|pid| AttachedProcess::new(self, pid))
    }

    fn attach_into_pid(self, pid: u32) -> Option<Self::ProcessType> {
        self.get_pid_from_windows_pid(pid)
            .map(|win_pid| AttachedProcess::new(self, win_pid))
    }

    fn attach_into_current(self) -> Self::ProcessType {
        let pid = self.get_current_pid();
        AttachedProcess::new(self, pid)
    }
}

/// Gets the Pid type from a process name, a windows PID, or the current PID
pub trait GetPid {
    type Pid;

    fn get_pid_from_name(&self, process_name: &str) -> Option<Self::Pid>;
    fn get_pid_from_windows_pid(&self, pid: u32) -> Option<Self::Pid>;
    fn get_current_pid(&self) -> Self::Pid;
}

pub struct AttachedProcess<T, P> {
    api: T,
    pid: P,
}

impl<T, P> AttachedProcess<T, P> {
    pub fn new(api: T, pid: P) -> Self {
        Self { api, pid }
    }
}

/// A trait that mirrors the MemoryRead trait but reads from a PID instead of directly from the implementor.
/// Note that the Pid type is not necessarily a Windows process ID. One may implement this using another form of identifier such as a dirbase.
pub trait MemoryReadPid: GetPid {
    /// Reads memory from the process with the given PID.
    fn try_read_bytes_into_pid(&self, pid: &Self::Pid, address: u64, buffer: &mut [u8]) -> Option<()>;
}

impl<T, P> MemoryRead for AttachedProcess<T, P>
    where
        T: MemoryReadPid<Pid=P>
{
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        self.api.try_read_bytes_into_pid(&self.pid, address, buffer)
    }
}

/// A trait that mirrors the MemoryWrite trait but writes to a PID instead of directly from the implementor.
/// Note that the Pid type is not necessarily a Windows process ID. One may implement this using another form of identifier such as a dirbase.
pub trait MemoryWritePid: GetPid {
    /// Writes memory to the process with the given PID.
    fn try_write_bytes_pid(&self, pid: &Self::Pid, address: u64, buffer: &[u8]) -> Option<()>;
}

impl<T, P> MemoryWrite for AttachedProcess<T, P>
    where
        T: MemoryWritePid<Pid=P>,
{
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        self.api.try_write_bytes_pid(&self.pid, address, buffer)
    }
}

/// A trait that mirrors the ModuleList trait by gets information from a PID instead of directly from the implementor.
/// Note that the Pid type is not necessarily a Windows process ID. One may implement this using another form of identifier such as a dirbase.
pub trait ModuleListPid: GetPid {
    /// Returns a list of all modules from a Pid. If the implementor can only
    /// provide a single module based on the name, this function should panic
    fn get_module_list(&self, pid: &Self::Pid) -> Vec<Module>;

    /// Returns a single module by name from the Pid.
    /// If the module name does not exist, returns None
    fn get_module(&self, pid: &Self::Pid, name: &str) -> Option<Module> {
        self.get_module_list(pid)
            .into_iter()
            .find(|m| m.name.to_lowercase() == name.to_lowercase())
    }

    /// Gets the main module from the Pid.
    fn get_main_module(&self, pid: &Self::Pid) -> Module;
}

impl<T, P> ModuleList for AttachedProcess<T, P>
    where
        T: ModuleListPid<Pid=P>,
{
    fn get_module_list(&self) -> Vec<Module> {
        self.api.get_module_list(&self.pid)
    }

    fn get_module(&self, name: &str) -> Option<Module> {
        self.api.get_module(&self.pid, name)
    }

    fn get_main_module(&self) -> Module {
        self.api.get_main_module(&self.pid)
    }
}

/// A trait that mirrors the ProcessInfo trait by gets information from a PID instead of directly from the implementor.
/// Note that the Pid type is not necessarily a Windows process ID. One may implement this using another form of identifier such as a dirbase.
pub trait ProcessInfoPid: GetPid {
    fn process_name(&self, pid: &Self::Pid) -> String;
    fn peb_base_address(&self, pid: &Self::Pid) -> u64;
    fn pid(&self, pid: &Self::Pid) -> u32;
}

impl<T, P> ProcessInfo for AttachedProcess<T, P>
    where
        T: ProcessInfoPid<Pid=P>,
{
    fn process_name(&self) -> String {
        self.api.process_name(&self.pid)
    }

    fn peb_base_address(&self) -> u64 {
        self.api.peb_base_address(&self.pid)
    }

    fn pid(&self) -> u32 {
        self.api.pid(&self.pid)
    }
}