pub use quest_hook_proc_macros::hook;
pub use inline_hook::*;

trait Hook {
    fn namespace(&self) -> &'static str;
    fn class_name(&self) -> &'static str;
    fn method_name(&self) -> &'static str;
    fn arg_count(&self) -> usize;
}
