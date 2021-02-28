#![feature(once_cell)]
#![warn(rust_2018_idioms)]
#![cfg_attr(feature = "strict", deny(warnings))]

mod array;
mod class;
mod method_info;
mod object;
mod parameter_info;
pub mod raw;
mod string;
mod ty;

pub use class::Il2CppClass;
pub use method_info::MethodInfo;
pub use object::Il2CppObject;
pub use parameter_info::ParameterInfo;
pub use raw::WrapRaw;
pub use string::Il2CppString;
pub use ty::Il2CppType;
