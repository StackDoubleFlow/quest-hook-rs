use cfg_if::cfg_if;

fn main() {
    cfg_if! {
        if #[cfg(target = "aarch64-linux-android")] {
            cc::Build::new()
                .file("beatsaber-hook/src/inline-hook/And64InlineHook.cpp")
                .compile("inline_hook");
        } else if #[cfg(target = "armv7-linux-androideabi")] {
            cc::Build::new()
                .file("beatsaber-hook/src/inline-hook/inlineHook.c")
                .compile("inline_hook");
        }
    }
}
