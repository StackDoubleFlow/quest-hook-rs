use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt;

use crate::raw::{
    self, Il2CppTypeEnum_IL2CPP_TYPE_BOOLEAN, Il2CppTypeEnum_IL2CPP_TYPE_CHAR,
    Il2CppTypeEnum_IL2CPP_TYPE_I1, Il2CppTypeEnum_IL2CPP_TYPE_I2, Il2CppTypeEnum_IL2CPP_TYPE_I4,
    Il2CppTypeEnum_IL2CPP_TYPE_I8, Il2CppTypeEnum_IL2CPP_TYPE_OBJECT,
    Il2CppTypeEnum_IL2CPP_TYPE_R4, Il2CppTypeEnum_IL2CPP_TYPE_R8,
    Il2CppTypeEnum_IL2CPP_TYPE_STRING, Il2CppTypeEnum_IL2CPP_TYPE_U1,
    Il2CppTypeEnum_IL2CPP_TYPE_U2, Il2CppTypeEnum_IL2CPP_TYPE_U4, Il2CppTypeEnum_IL2CPP_TYPE_U8,
    Il2CppTypeEnum_IL2CPP_TYPE_VOID,
};
use crate::{Il2CppClass, WrapRaw};

/// An il2cpp type
#[repr(transparent)]
pub struct Il2CppType(raw::Il2CppType);

impl Il2CppType {
    /// Class of the type
    pub fn class(&self) -> &Il2CppClass {
        unsafe { Il2CppClass::wrap(raw::class_from_il2cpp_type(self.raw())) }
    }

    /// Name of the type
    pub fn name(&self) -> Cow<'_, str> {
        let name = raw::type_get_name(self.raw());
        assert!(!name.is_null());
        unsafe { CStr::from_ptr(name) }.to_string_lossy()
    }

    /// Whether the type represents the given builtin
    pub fn is_builtin(&self, builtin: Builtin) -> bool {
        self.raw().type_() == builtin as i32
    }
}

unsafe impl WrapRaw for Il2CppType {
    type Raw = raw::Il2CppType;
}

impl PartialEq for Il2CppType {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.raw().data.klassIndex == other.raw().data.klassIndex }
    }
}
impl Eq for Il2CppType {}

impl fmt::Display for Il2CppType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&*self.name())
    }
}

/// Builtin C# types
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Builtin {
    Void = Il2CppTypeEnum_IL2CPP_TYPE_VOID,
    Object = Il2CppTypeEnum_IL2CPP_TYPE_OBJECT,
    Bool = Il2CppTypeEnum_IL2CPP_TYPE_BOOLEAN,
    Char = Il2CppTypeEnum_IL2CPP_TYPE_CHAR,
    Byte = Il2CppTypeEnum_IL2CPP_TYPE_U1,
    SByte = Il2CppTypeEnum_IL2CPP_TYPE_I1,
    Short = Il2CppTypeEnum_IL2CPP_TYPE_I2,
    UShort = Il2CppTypeEnum_IL2CPP_TYPE_U2,
    Int = Il2CppTypeEnum_IL2CPP_TYPE_I4,
    UInt = Il2CppTypeEnum_IL2CPP_TYPE_U4,
    Long = Il2CppTypeEnum_IL2CPP_TYPE_I8,
    ULong = Il2CppTypeEnum_IL2CPP_TYPE_U8,
    Single = Il2CppTypeEnum_IL2CPP_TYPE_R4,
    Double = Il2CppTypeEnum_IL2CPP_TYPE_R8,
    String = Il2CppTypeEnum_IL2CPP_TYPE_STRING,
}

impl fmt::Display for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Builtin::Void => "void",
            Builtin::Object => "object",
            Builtin::Bool => "bool",
            Builtin::Char => "char",
            Builtin::Byte => "byte",
            Builtin::SByte => "sbyte",
            Builtin::Short => "short",
            Builtin::UShort => "ushort",
            Builtin::Int => "int",
            Builtin::UInt => "uint",
            Builtin::Long => "long",
            Builtin::ULong => "ulong",
            Builtin::Single => "single",
            Builtin::Double => "double",
            Builtin::String => "string",
        })
    }
}
