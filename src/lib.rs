#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#![cfg_attr(feature = "strict", deny(warnings))]
#![feature(backtrace)]
#![doc(html_root_url = "https://stackdoubleflow.github.io/quest-hook-rs/quest_hook")]

//! A library used for writing mods for Oculus Quest Unity il2cpp games. Mods using this library may be loaded using [QuestLoader](https://github.com/sc2ad/QuestLoader).

pub use libil2cpp;
pub use quest_hook_proc_macros::hook;

#[doc(hidden)]
pub use inline_hook;
#[doc(hidden)]
pub use std::backtrace::Backtrace as StdBacktrace;
#[doc(hidden)]
pub use tracing_android;

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

/// Properly sets up logging and panic handling using [`tracing_android`]
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

            $crate::tracing_android::tracing::error!(
                "A panic occurred at {}:{}: {}",
                filename,
                line,
                cause
            );

            $crate::tracing_android::tracing::error!("{:?}", $crate::StdBacktrace::force_capture());
        }));
    };
}
