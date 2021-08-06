use std::backtrace::Backtrace;

use tracing_android::tracing::error;

/// Properly sets up logging and panic handling using
/// [`tracing`](tracing_android::tracing)
pub fn setup(name: &str) {
    tracing_android::subscriber(name).init();

    std::panic::set_hook(Box::new(|panic_info| {
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

        error!("a panic occurred at {}:{}: {}", filename, line, cause);
        error!("{:?}", Backtrace::force_capture());
    }));
}
