#![feature(backtrace, doc_cfg)]
#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#![doc(html_root_url = "https://stackdoubleflow.github.io/quest-hook-rs/quest_hook")]
#![doc = include_str!("../README.md")]

#[macro_use]
mod cfg;

mod hook;
pub use hook::*;

feature! { #[feature = "util"]
    mod util;
    pub use util::*;
}

pub use libil2cpp;
pub use quest_hook_proc_macros::hook;

#[doc(hidden)]
pub use inline_hook;
