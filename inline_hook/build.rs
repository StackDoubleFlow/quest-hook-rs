use cfg_if::cfg_if;

fn main() {
    cfg_if! {
        if #[cfg(all(target_arch = "aarch64", target_os = "android"))] {
            cc::Build::new()
                .file("beatsaber-hook/src/inline-hook/And64InlineHook.cpp")
                .compile("inline_hook");
        } else if #[cfg(all(target_arch = "arm", target_os = "android"))] {
            cc::Build::new()
                .file("beatsaber-hook/src/inline-hook/inlineHook.c")
                .compile("inline_hook");
        }
    }
}
