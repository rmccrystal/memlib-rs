pub macro generate_process_attach_tests($make_attach:expr) {
use $crate::tests::process_attach_tests::ProcessAttachTests;

    fn get_tester() -> ProcessAttachTests<impl $crate::ProcessAttach<ProcessType: $crate::MemoryRead + $crate::MemoryWrite + $crate::ModuleList + $crate::ProcessInfo>> {
        let attach = $make_attach();
        ProcessAttachTests::new(attach)
    }

    #[test]
    fn test_attach() {
        get_tester().test_attach();
    }

    #[test]
    fn test_main_module() {
        get_tester().test_main_module();
    }

    #[test]
    fn test_module_list() {
        get_tester().test_module_list();
    }

    #[test]
    fn test_read() {
        get_tester().test_read();
    }

    #[test]
    fn test_dump_module() {
        get_tester().test_dump_module();
    }

    #[test]
    fn test_module_coverage() {
        get_tester().test_module_coverage();
    }

    /*
    #[test]
    fn test_write() {
        get_tester().test_write();
    }
     */

    #[test]
    fn test_process_info() {
        get_tester().test_process_info();
    }
}

#[cfg(test)]
pub mod process_attach_tests {
    use crate::{MemoryRead, MemoryWrite, ModuleList, ProcessAttach, ProcessAttachInto, ProcessInfo};
    use log::LevelFilter;
    use std::process;

    struct TestProcess {
        proc: process::Child,
    }

    const TEST_PROCESS: &str = "winver.exe";

    impl TestProcess {
        pub fn new() -> Self {
            Self {
                proc: process::Command::new(TEST_PROCESS).spawn().unwrap(),
            }
        }

        pub fn name(&self) -> String {
            TEST_PROCESS.to_string()
        }
    }

    impl Drop for TestProcess {
        fn drop(&mut self) {
            self.proc.kill().unwrap()
        }
    }

    fn init_tests() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Debug)
            .try_init();
    }

    pub struct ProcessAttachTests<T: ProcessAttachInto<ProcessType: MemoryRead + MemoryWrite + ModuleList + ProcessInfo> + Clone> {
        api: T,
        proc: TestProcess,
    }

    impl<T: ProcessAttachInto<ProcessType: MemoryRead + MemoryWrite + ModuleList + ProcessInfo> + Clone> ProcessAttachTests<T> {
        pub fn new(api: T) -> Self {
            init_tests();
            Self {
                api,
                proc: TestProcess::new(),
            }
        }

        fn attach(&self) -> T::ProcessType {
            self.api.attach_pid(self.proc.proc.id()).unwrap()
        }

        pub fn test_attach(&self) {
            let _ = self.attach();
        }

        pub fn test_main_module(&self) {
            let handle = self.attach();
            let main_module = handle.get_main_module();
            dbg!(&main_module);
            assert_eq!(main_module.name, self.proc.name());
        }

        pub fn test_module_list(&self) {
            let handle = self.attach();
            let modules = handle.get_module_list();
            dbg!(&modules);
            assert!(modules.len() > 5);
            assert!(modules.iter().any(|m| m.name.to_lowercase() == "ntdll.dll"));
        }

        pub fn test_read(&self) {
            let handle = self.attach();
            let main_module = handle.get_main_module();
            let header = handle
                .try_read_bytes(main_module.base, 2)
                .expect("Could not read bytes from the process");
            assert_eq!(header[0], 0x4D);
            assert_eq!(header[1], 0x5A);
        }

        pub fn test_dump_module(&self) {
            let handle = self.attach();
            let main_module = handle.get_main_module();
            let range = main_module.memory_range();
            let _ = handle
                .dump_memory(range)
                .expect("Could not dump memory range");
        }

        pub fn test_module_coverage(&self) {
            let handle = self.attach();
            let range = handle.get_main_module().memory_range();

            const CHUNK_SIZE: usize = 0x8;

            let mut total = 0;
            let mut good = 0;
            for i in range.step_by(CHUNK_SIZE) {
                let valid = handle.try_read_bytes(i, CHUNK_SIZE).is_some();
                total += 1;
                if valid {
                    good += 1;
                }
            }

            eprintln!(
                "{}/{} ({:.1}%) chunks of {:#X} could be read",
                good,
                total,
                (good as f32 / total as f32) * 100.0,
                CHUNK_SIZE
            );
            assert_eq!(good, total);
        }

        pub fn test_write(&self) {
            let handle = self.attach();
            let main_module = handle.get_main_module();
            let base = main_module.base;

            let test_data = [1u8, 2, 3, 4, 5, 6];

            let orig_data = handle.try_read_bytes(base, test_data.len()).unwrap();

            handle.try_write_bytes(base, &test_data).unwrap();

            let actual_data = handle.try_read_bytes(base, test_data.len()).unwrap();

            handle.try_write_bytes(base, &orig_data).unwrap();

            assert_eq!(&test_data[..], &actual_data);
        }

        pub fn test_process_info(&self) {
            let handle = self.attach();
            dbg!(handle.peb_base_address());
            assert_ne!(handle.peb_base_address(), 0);
            assert_eq!(handle.pid(), self.proc.proc.id());
            assert_eq!(handle.process_name(), self.proc.name());
        }
    }
}
