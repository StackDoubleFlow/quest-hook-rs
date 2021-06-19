# quest_hook

A library used for writing mods for Oculus Quest Unity il2cpp games. Mods using this library may be loaded using [QuestLoader](https://github.com/sc2ad/QuestLoader).

[documentation](https://stackdoubleflow.github.io/quest-hook-rs/quest_hook/index.html)

## Example

```rust
use quest_hook::hook;
use quest_hook::libil2cpp::Il2CppObject;
use tracing_android::tracing::info;

#[no_mangle]
pub extern "C" fn setup() {
    quest_hook::setup();
}

#[hook("", "MainSettingsModelSO", "Load")]
fn on_enable(this: &mut Il2CppObject, forced: bool) {
    on_enable.original(this, forced);

    let field = this.class().find_field_unchecked("burnMarkTrailsEnabled").unwrap();
    let val: &mut Il2CppObject = field.load(this);
    let _: () = val.invoke("set_value", true).unwrap();
}

#[no_mangle]
pub extern "C" fn load() {
    info!("Installing burn_marks hooks!");

    on_enable.install();

    info!("Installed burn_marks hooks!");
}
```
