use paste::paste;
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
        if let Some(name) = self.as_builtin().map(Builtin::name) {
            return name.into();
        }

        let name = unsafe { raw::type_get_name(self.raw()) };
        assert!(!name.is_null());
        unsafe { CStr::from_ptr(name) }.to_string_lossy()
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

impl fmt::Debug for Il2CppType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Il2CppType")
            .field("name", &self.name())
            .finish()
    }
}

impl fmt::Display for Il2CppType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&*self.name())
    }
}

macro_rules! builtins {
    ($($const:ident => ($variant:ident, $id:ident, $name:literal),)*) => {
        // Builtin C# types
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u32)]
        pub enum Builtin {
            $($variant = $const,)*
        }

        impl Il2CppType {
            /// Whether the type represents the given [`Builtin`]
            #[inline]
            pub fn is_builtin(&self, builtin: Builtin) -> bool {
                self.raw().type_() == builtin as u32
            }

            paste! {
                $(
                    #[doc = concat!("Whether the type represents a `", $name , "`")]
                    pub fn [<is_ $id>](&self) -> bool {
                        self.is_builtin(Builtin::$variant)
                    }
                )*
            }

            /// [`Builtin`] the type represents, if any
            pub fn as_builtin(&self) -> Option<Builtin> {
                #[allow(non_upper_case_globals)]
                match self.raw().type_() {
                    $($const => Some(Builtin::$variant),)*
                    _ => None
                }
            }
        }

        impl Builtin {
            /// Name of the builtin
            pub fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)*
                }
            }
        }
    }
}

builtins! {
    Il2CppTypeEnum_IL2CPP_TYPE_VOID => (Void, void, "void"),
    Il2CppTypeEnum_IL2CPP_TYPE_OBJECT => (Object, object, "object"),
    Il2CppTypeEnum_IL2CPP_TYPE_BOOLEAN => (Bool, bool, "bool"),
    Il2CppTypeEnum_IL2CPP_TYPE_CHAR => (Char, char, "char"),
    Il2CppTypeEnum_IL2CPP_TYPE_U1 => (Byte, byte, "byte"),
    Il2CppTypeEnum_IL2CPP_TYPE_I1 => (SByte, sbyte, "sbyte"),
    Il2CppTypeEnum_IL2CPP_TYPE_I2 => (Short, short, "short"),
    Il2CppTypeEnum_IL2CPP_TYPE_U2 => (UShort, ushort, "ushort"),
    Il2CppTypeEnum_IL2CPP_TYPE_I4 => (Int, int, "int"),
    Il2CppTypeEnum_IL2CPP_TYPE_U4 => (UInt, uint, "uint"),
    Il2CppTypeEnum_IL2CPP_TYPE_I8 => (Long, long, "long"),
    Il2CppTypeEnum_IL2CPP_TYPE_U8 => (ULong, ulong, "ulong"),
    Il2CppTypeEnum_IL2CPP_TYPE_R4 => (Single, single, "single"),
    Il2CppTypeEnum_IL2CPP_TYPE_R8 => (Double, double, "double"),
    Il2CppTypeEnum_IL2CPP_TYPE_STRING => (String, string, "string"),
}

impl fmt::Display for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
