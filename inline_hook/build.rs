fn main() {
    cc::Build::new()
        .file("beatsaber-hook/src/inline-hook/And64InlineHook.cpp")
        .compile("inline_hook");
}
