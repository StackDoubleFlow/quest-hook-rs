use quest_hook::hook;
use quest_hook::libil2cpp::{Il2CppObject, Il2CppString};
use quest_hook::tracing::debug;

#[no_mangle]
pub extern "C" fn setup() {
    quest_hook::setup("hello world");
}

#[hook("UnityEngine.SceneManagement", "SceneManager", "SetActiveScene")]
fn set_active_scene(scene: &mut Il2CppObject) -> bool {
    let name: &Il2CppString = scene.invoke("get_name", ()).unwrap();
    debug!("Hello, {}!", name);

    set_active_scene.original(scene)
}

#[no_mangle]
pub extern "C" fn load() {
    set_active_scene.install();
}
