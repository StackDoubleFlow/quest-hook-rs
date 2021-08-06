#![feature(once_cell)]
#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#![cfg_attr(feature = "strict", deny(warnings))]
#![doc(html_root_url = "https://stackdoubleflow.github.io/quest-hook-rs/libil2cpp")]

//! Wrappers and raw bindings for Unity's libil2cpp

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
