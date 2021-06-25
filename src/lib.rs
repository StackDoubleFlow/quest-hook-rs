#![warn(rust_2018_idioms)]
#![cfg_attr(feature = "strict", deny(warnings))]
#![feature(backtrace)]

#[doc(hidden)]
pub mod inline_hook;
pub mod libil2cpp;

pub use quest_hook_proc_macros::hook;

use std::ops::Deref;
use std::panic;
use tracing_android::tracing::error;

/// Trait implemented by all hooks to facilitate generic programming
pub trait Hook {
    /// Installs the hook
    fn install(&self);

    /// Namespace of the hooked method's class
    fn namespace(&self) -> &'static str;

    /// Name of the hooked method's class
    fn class_name(&self) -> &'static str;

    /// Name of the hooked method
    fn method_name(&self) -> &'static str;

    /// Number of parameters the hooked method takes
    fn parameters_count(&self) -> usize;

    /// Pointer to the hook function
    fn hook(&self) -> *mut ();

    /// Pointer to the hooked method
    fn original(&self) -> *mut ();
}

pub fn setup() {
    tracing_android::subscriber(env!("CARGO_PKG_NAME")).init();

    panic::set_hook(Box::new(|panic_info| {
        let (filename, line) = panic_info
            .location()
            .map(|loc| (loc.file(), loc.line()))
            .unwrap_or(("<unknown>", 0));

        let cause = panic_info
            .payload()
            .downcast_ref::<String>()
            .map(String::deref);

        let cause = cause.unwrap_or_else(|| {
            panic_info
                .payload()
                .downcast_ref::<&str>()
                .cloned()
                .unwrap_or("<cause unknown>")
        });

        error!("A panic occurred at {}:{}: {}", filename, line, cause);

        error!("{:?}", std::backtrace::Backtrace::force_capture());
    }));
}
