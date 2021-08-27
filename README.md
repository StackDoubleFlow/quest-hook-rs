# quest_hook

A library for writing (mostly) memory safe mods for Unity il2cpp games

[![Docs](https://img.shields.io/github/workflow/status/StackDoubleFlow/quest-hook-rs/Docs?color=blue&label=docs&style=for-the-badge)](https://stackdoubleflow.github.io/quest-hook-rs/quest_hook/) [![Tests](https://img.shields.io/github/workflow/status/StackDoubleFlow/quest-hook-rs/Tests?label=tests&style=for-the-badge)](https://github.com/StackDoubleFlow/quest-hook-rs/actions/workflows/tests.yml)

## Platform support

Despite its name and initial target and scope, this library supports modding most il2cpp games, as long as you have a way to load the mods.

### Unity versions

- Unity 2019
- Unity 2018

### Targets

- Android ARMv8
- Android ARMv7
- Windows x64
- Windows x86
- Linux x64
- Linux x86

## Usage

Simply add the library as a dependency to your `Cargo.toml` and set the crate type to a C dynamic library. You will need to use a nightly version in order to compile `quest_hook`. **Don't forget to select a Unity version**.

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
quest_hook = { git = "https://github.com/StackDoubleFlow/quest-hook-rs.git", features = ["unity2019"] }
```

This library is still under heavy development and breaking changes are frequent. To avoid dealing with those, you can [pin the dependency to a specific commit or tag](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories).

## Example

```rust
use quest_hook::hook;
use quest_hook::libil2cpp::{Il2CppObject, Il2CppString};
use tracing::debug;

#[hook("UnityEngine.SceneManagement", "SceneManager", "SetActiveScene")]
fn set_active_scene(scene: &mut Il2CppObject) -> bool {
    let name: &Il2CppString = scene.invoke("get_name", ()).unwrap();
    debug!("Hello, {}!", name);

    set_active_scene.original(scene)
}

#[no_mangle]
pub extern "C" fn load() {
    set_active_scene.install().unwrap();
}

#[no_mangle]
pub extern "C" fn setup() {
    quest_hook::setup("hello world");
}
```

Check out the [`examples`](./examples/) directory for more examples.

## Cargo features

- `unity2019`, `unity2018` - Unity version the targetted game uses
- `util` - Adds small utility functions for setting up logging and the like
- `cache` - Enables class and method caching to greatly improve lookup speed at the cost of slightly higher memory consumption
- `serde` - Implement `Serialize` and `Deserialize` for il2cpp types where it makes sense
- `trace` - Adds `tracing` instrumentation to many internal functions

## Contributing

Contributions are welcome, especially to the documentation and examples. Most of the discussions regarding the development of this library happen in the `#quest-mod-dev` channel of the [BSMG Discord server](https://discord.gg/beatsabermods).

Everything that can be reasonably be done in Rust should be done in Rust. The reasons behind this choice are improving readability and providing a more Rust-friendly API, and not safety. This library is, by nature, extremely unsafe, and contains a lot of unsafe code.

A decent understanding of both Rust and C++ is required for most work on the library. The main reference used for development is libil2cpp, which is written in C++. Another excellent resource is [beatsaber-hook](https://github.com/sc2ad/beatsaber-hook), also written in C++.

This library is mainly developed using Visual Studio Code with [rust-analyzer](https://rust-analyzer.github.io/). Code style, quality and documentation are enforced using rustfmt and clippy via GitHub Actions. Due to the nature of this library, we can sadly not really unit test most of the features, but are open to suggestions to improve this aspect.

### Project structure

- `libil2cpp` - Abstractions and raw bindings for libil2cpp. This is where most of the code and functionality lives.
- `inline_hook` - Cross-platform function hooking abstraction. This is where support for more targets can be added.
- `proc_macros` - Home of the `hook` macro implementation, and of various internally used ones.

## License

quest_hook is licensed under the [MIT License](./LICENSE).

## Credits

This library wouldn't exist without the invaluable help, feedback and previous work from [Sc2ad](https://github.com/sc2ad).
