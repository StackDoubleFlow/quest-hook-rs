#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#![cfg_attr(feature = "strict", deny(warnings))]
#![feature(backtrace)]
#![doc(html_root_url = "https://stackdoubleflow.github.io/quest-hook-rs/quest_hook")]

//! A library used for writing mods for Oculus Quest Unity il2cpp games. Mods using this library may be loaded using [QuestLoader](https://github.com/sc2ad/QuestLoader).

mod hook;
mod setup;

pub use hook::*;
pub use setup::*;

pub use libil2cpp;
pub use quest_hook_proc_macros::hook;
pub use tracing_android::tracing;

#[doc(hidden)]
pub use inline_hook;
#[doc(hidden)]
pub use tracing_android;
