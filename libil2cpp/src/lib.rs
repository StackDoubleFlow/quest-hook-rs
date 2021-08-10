#![feature(once_cell, generic_associated_types)]
#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#![allow(clippy::single_component_path_imports)]
#![doc(html_root_url = "https://stackdoubleflow.github.io/quest-hook-rs/libil2cpp")]

//! Wrappers and raw bindings for Unity's libil2cpp

#[cfg(not(any(feature = "unity2019", feature = "unity2018")))]
compile_error!("No Unity version selected");

#[cfg(feature = "trace")]
#[macro_use]
extern crate tracing;
#[cfg(feature = "trace")]
pub use tracing::{debug, instrument};

#[cfg(not(feature = "trace"))]
macro_rules! debug {
    ($_:tt) => {};
}
#[cfg(not(feature = "trace"))]
pub use quest_hook_proc_macros::identity as instrument;

mod array;
mod class;
mod exception;
mod field_info;
mod method_info;
mod object;
mod parameter_info;
pub mod raw;
mod string;
mod ty;
mod typecheck;

pub use array::Il2CppArray;
pub use class::{FindMethodError, Il2CppClass};
pub use exception::Il2CppException;
pub use field_info::FieldInfo;
pub use method_info::{Il2CppReflectionMethod, MethodInfo};
pub use object::Il2CppObject;
pub use parameter_info::ParameterInfo;
pub use raw::{unbox, WrapRaw};
pub use string::Il2CppString;
pub use ty::{Builtin, Il2CppReflectionType, Il2CppType};
pub use typecheck::callee::{Parameter, Parameters, Return, ThisParameter};
pub use typecheck::caller::{Argument, Arguments, Returned, ThisArgument};
pub use typecheck::ty::{Reference, Type, Value};
