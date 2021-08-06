use quest_hook::hook;
use quest_hook::libil2cpp::unsafe_impl_value_type;
use quest_hook::tracing::debug;

#[no_mangle]
pub extern "C" fn setup() {
    quest_hook::setup("custom type");
}

#[derive(Debug)]
#[repr(C)]
struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}
unsafe_impl_value_type!(Vector3, "UnityEngine", "Vector3");

#[hook("UnityEngine", "RigidBody", "set_position")]
fn set_position(this: &mut Il2CppObject, new_position: Vector3) {
    let old_position: Vector3 = this.invoke("get_position", ()).unwrap();
    debug!("{:?} -> {:?}", old_position, new_position);

    set_position.original(this, position)
}

#[no_mangle]
pub extern "C" fn load() {
    set_position.install().unwrap();
}
