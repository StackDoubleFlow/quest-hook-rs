fn main() {
    cc::Build::new()
        .file("And64InlineHook/And64InlineHook.cpp")
        .include("And64InlineHook")
        .compile("inline_hook");
}
