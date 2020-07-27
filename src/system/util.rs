lazy_static::lazy_static! {
    static ref HANDLE: tokio::runtime::Handle = {
        let mut runtime = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();
        let handle = runtime.handle().clone();
        std::thread::spawn(move || {
            runtime.block_on(async { loop { futures::pending!(); } });
        });
        handle
    };
}
/// Runs an async function from a non async context
pub fn run_async<T>(future: impl std::future::Future<Output=T>) -> T {
    // tokio::runtime::Runtime::new().unwrap().block_on(future)
    HANDLE.block_on(future)
}
