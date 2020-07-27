pub fn get_system() -> &dyn system_host::SystemHandleInterface {
    test().wait()
}
