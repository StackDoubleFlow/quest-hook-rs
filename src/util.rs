use std::backtrace::Backtrace;
use std::panic::PanicInfo;

use cfg_if::cfg_if;
use tracing::error;
use tracing_error::SpanTrace;

/// Sets up Android logging with the provided tag and default settings using
/// [`tracing`]. Also sets up panic handling with backtrace and spantrace
/// capture enabled.
#[allow(clippy::needless_pass_by_value)]
pub fn setup(tag: impl ToString) {
    cfg_if! {
        if #[cfg(target_os = "android")] {
            tracing_android::init(tag);
        } else {
            let env = format!("LOG_{}", tag.to_string().to_ascii_uppercase());
            let filter = tracing_subscriber::filter::EnvFilter::from_env(env);
            tracing_subscriber::fmt().with_env_filter(filter).init();
        }
    }
    std::panic::set_hook(panic_hook(true, true));
}

/// Returns a panic handler, optionally with backtrace and spantrace capture.
pub fn panic_hook(
    backtrace: bool,
    spantrace: bool,
) -> Box<dyn Fn(&PanicInfo<'_>) + Send + Sync + 'static> {
    // Mostly taken from https://doc.rust-lang.org/src/std/panicking.rs.html
    Box::new(move |info| {
        let location = info.location().unwrap();
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<dyn Any>",
            },
        };

        error!(target: "panic", "panicked at '{}', {}", msg, location);
        if backtrace {
            error!(target: "panic", "{:?}", Backtrace::force_capture());
        }
        if spantrace {
            error!(target: "panic", "{:?}", SpanTrace::capture());
        }
    })
}
