# quest_hook

A Rust library used for writing (mostly) memory safe mods for Oculus Quest Unity il2cpp games. Mods using this library may be loaded using [QuestLoader](https://github.com/sc2ad/QuestLoader).

[![Docs](https://img.shields.io/github/workflow/status/StackDoubleFlow/quest-hook-rs/Docs?color=blue&label=docs&style=for-the-badge)](https://stackdoubleflow.github.io/quest-hook-rs/quest_hook/) [![Tests](https://img.shields.io/github/workflow/status/StackDoubleFlow/quest-hook-rs/Tests?label=tests&style=for-the-badge)](https://github.com/StackDoubleFlow/quest-hook-rs/actions/workflows/tests.yml)

## Usage

Simply add the library as a dependency to your `Cargo.toml` and set the crate type to a C dynamic library.

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
quest_hook = { git = "https://github.com/StackDoubleFlow/quest-hook-rs.git" }
```

This library is still under heavy development and breaking changes are frequent. To avoid dealing with those, you can [pin the dependency to a specific commit or tag](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories).

It is also recommended to use [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk) to simplify the build process.

## Example

```rust
use quest_hook::hook;
use quest_hook::libil2cpp::Il2CppObject;
use quest_hook::tracing::info;

#[no_mangle]
pub extern "C" fn setup() {
    quest_hook::setup!();
}

#[hook("", "MainSettingsModelSO", "Load")]
fn on_enable(this: &mut Il2CppObject, forced: bool) {
    on_enable.original(this, forced);

    let burn_mark_trails_enabled: &mut Il2CppObject = this.load("burnMarkTrailsEnabled").unwrap();
    burn_mark_trails_enabled.invoke_void("set_value", true).unwrap();
}

#[no_mangle]
pub extern "C" fn load() {
    info!("Installing burn_marks hooks!");

    on_enable.install();

    info!("Installed burn_marks hooks!");
}
```

## Contributing

Contributions are welcome, especially to the documentation and examples. Most of the discussions regarding the development of this library happen in the `#quest-mod-dev` channel of the [BSMG Discord server](https://discord.gg/beatsabermods).

Everything that can be reasonably be done in Rust should be done in Rust. The reasons behind this choice are improving readability and providing a more Rust-friendly API, and not safety. This library is, by nature, extremely unsafe, and contains a lot of unsafe code.

A decent understanding of both Rust and C++ is required for most work on the library. The main reference used for development is libil2cpp, which is written in C++. Another excellent resource is [beatsaber-hook](https://github.com/sc2ad/beatsaber-hook), also written in C++.

This library is mainly developed using Visual Studio Code with [rust-analyzer](https://rust-analyzer.github.io/) and [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk). Code style, quality and documentation are enforced using rustfmt and clippy via GitHub Actions. Due to the nature of this library, we can sadly not really unit test most of the features, but are open to suggestions to improve this aspect.

## License

quest_hook is licensed under the [MIT License](./LICENSE). It also contains third party code licensed under different terms.

- [And64InlineHook](./inline_hook/And64InlineHook/) - MIT License

## Credits

This library wouldn't exist without the invaluable help, feedback and previous work from [Sc2ad](https://github.com/sc2ad).
