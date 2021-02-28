#![warn(rust_2018_idioms)]
#![cfg_attr(feature = "strict", deny(warnings))]

pub mod inline_hook;
pub mod libil2cpp;

pub use quest_hook_proc_macros::hook;

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
