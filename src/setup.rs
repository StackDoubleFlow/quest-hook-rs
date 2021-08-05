/// Properly sets up logging and panic handling using [`tracing`]
#[macro_export]
macro_rules! setup {
    () => {
        $crate::tracing_android::subscriber(env!("CARGO_PKG_NAME")).init();

        ::std::panic::set_hook(Box::new(|panic_info| {
            let (filename, line) = panic_info
                .location()
                .map(|loc| (loc.file(), loc.line()))
                .unwrap_or(("<unknown>", 0));

            let cause = panic_info
                .payload()
                .downcast_ref::<String>()
                .map(::std::ops::Deref::deref);

            let cause = cause.unwrap_or_else(|| {
                panic_info
                    .payload()
                    .downcast_ref::<&str>()
                    .cloned()
                    .unwrap_or("<cause unknown>")
            });

            $crate::tracing::error!("A panic occurred at {}:{}: {}", filename, line, cause);
            $crate::tracing::error!("{:?}", $crate::backtrace::Backtrace::force_capture());
        }));
    };
}
