#![cfg_attr(not(target_os = "android"), feature(once_cell))]
#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#![doc(html_root_url = "https://stackdoubleflow.github.io/quest-hook-rs/inline_hook")]

//! A cross platform function hooking abstraction, working across Windows,
//! Linux, macOS and Android

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(target_arch = "aarch64", target_os = "android"))] {
        mod aarch64_linux_android;
        pub use crate::aarch64_linux_android::*;
    } else if #[cfg(all(target_arch = "arm", target_os = "android"))] {
        mod armv7_linux_androideabi;
        pub use crate::armv7_linux_androideabi::*;
    } else {
        mod detour;
        pub use crate::detour::*;
    }
}

#[cfg(test)]
mod tests {
    use std::mem::transmute;

    use super::Hook;

    #[test]
    fn target_and_original() {
        static HOOK: Hook = Hook::new();

        #[inline(never)]
        fn add(n1: usize, n2: usize) -> usize {
            n1 + n2
        }

        #[inline(never)]
        fn mul(n1: usize, n2: usize) -> usize {
            n1 * n2
        }

        assert_eq!(add(2, 3), 5);
        assert_eq!(mul(2, 3), 6);

        assert!(unsafe { HOOK.install(add as _, mul as _) } && HOOK.is_installed());

        assert_eq!(add(2, 3), mul(2, 3));

        let original =
            unsafe { transmute::<*const (), fn(usize, usize) -> usize>(HOOK.original().unwrap()) };
        assert_eq!(original(2, 3), 5);
    }
}
