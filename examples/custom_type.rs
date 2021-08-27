#![feature(generic_associated_types, once_cell)]

use libil2cpp::{Il2CppArray, Type};
use quest_hook::hook;
use quest_hook::libil2cpp::{unsafe_impl_reference_type, unsafe_impl_value_type, Il2CppObject};
use tracing::debug;

#[derive(Debug)]
#[repr(C)]
pub struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}
unsafe_impl_value_type!(in quest_hook::libil2cpp for Vector3 => UnityEngine.Vector3);

#[hook("UnityEngine", "RigidBody", "set_position")]
fn set_position(this: &mut Il2CppObject, new_position: Vector3) {
    let old_position: Vector3 = this.invoke("get_position", ()).unwrap();
    debug!("{:?} -> {:?}", old_position, new_position);

    set_position.original(this, new_position)
}

#[no_mangle]
pub extern "C" fn setup() {
    quest_hook::setup("custom type");
}

#[no_mangle]
pub extern "C" fn load() {
    set_position.install().unwrap();
}

#[repr(C)]
pub struct List<T: Type> {
    object: Il2CppObject,
    items: *mut Il2CppArray<T>,
    size: i32,
}
unsafe_impl_reference_type!(in quest_hook::libil2cpp for List<T> => System.Collections.Generic.List<T>);
