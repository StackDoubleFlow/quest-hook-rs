fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target == "aarch64-linux-android" {
        cc::Build::new()
            .file("beatsaber-hook/src/inline-hook/And64InlineHook.cpp")
            .compile("inline_hook");
    } else if target == "armv7-linux-androideabi" {
        cc::Build::new()
            .file("beatsaber-hook/src/inline-hook/inlineHook.c")
            .include("beatsaber-hook/shared/inline-hook")
            .compile("inline_hook");
    }
}
