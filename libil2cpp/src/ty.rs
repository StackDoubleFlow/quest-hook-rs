use super::raw::{
    Il2CppTypeEnum_IL2CPP_TYPE_ARRAY, Il2CppTypeEnum_IL2CPP_TYPE_BOOLEAN,
    Il2CppTypeEnum_IL2CPP_TYPE_BYREF, Il2CppTypeEnum_IL2CPP_TYPE_CHAR,
    Il2CppTypeEnum_IL2CPP_TYPE_CLASS, Il2CppTypeEnum_IL2CPP_TYPE_ENUM,
    Il2CppTypeEnum_IL2CPP_TYPE_FNPTR, Il2CppTypeEnum_IL2CPP_TYPE_I, Il2CppTypeEnum_IL2CPP_TYPE_I1,
    Il2CppTypeEnum_IL2CPP_TYPE_I2, Il2CppTypeEnum_IL2CPP_TYPE_I4, Il2CppTypeEnum_IL2CPP_TYPE_I8,
    Il2CppTypeEnum_IL2CPP_TYPE_OBJECT, Il2CppTypeEnum_IL2CPP_TYPE_PTR,
    Il2CppTypeEnum_IL2CPP_TYPE_R4, Il2CppTypeEnum_IL2CPP_TYPE_R8,
    Il2CppTypeEnum_IL2CPP_TYPE_STRING, Il2CppTypeEnum_IL2CPP_TYPE_TYPEDBYREF,
    Il2CppTypeEnum_IL2CPP_TYPE_U, Il2CppTypeEnum_IL2CPP_TYPE_U1, Il2CppTypeEnum_IL2CPP_TYPE_U2,
    Il2CppTypeEnum_IL2CPP_TYPE_U4, Il2CppTypeEnum_IL2CPP_TYPE_U8,
    Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE, TYPE_ATTRIBUTE_ABSTRACT, TYPE_ATTRIBUTE_INTERFACE,
    TYPE_ATTRIBUTE_PUBLIC, TYPE_ATTRIBUTE_SEALED, TYPE_ATTRIBUTE_SERIALIZABLE,
};
use super::{raw, WrapRaw};

/// An il2cpp type
#[repr(transparent)]
pub struct Il2CppType(raw::Il2CppType);

impl Il2CppType {
    /// Whether the type is public
    pub fn is_public(&self) -> bool {
        self.raw().attrs() & TYPE_ATTRIBUTE_PUBLIC != 0
    }

    /// Whether the type is abstract
    pub fn is_abstract(&self) -> bool {
        self.raw().attrs() & TYPE_ATTRIBUTE_ABSTRACT != 0
    }

    /// Whether the type is sealed
    pub fn is_sealed(&self) -> bool {
        self.raw().attrs() & TYPE_ATTRIBUTE_SEALED != 0
    }

    /// Whether the type is an interface
    pub fn is_interface(&self) -> bool {
        self.raw().attrs() & TYPE_ATTRIBUTE_INTERFACE != 0
    }

    /// Whether the type is serializable
    pub fn is_serializable(&self) -> bool {
        self.raw().attrs() & TYPE_ATTRIBUTE_SERIALIZABLE != 0
    }

    /// Whether the type is a value type
    #[allow(non_upper_case_globals)]
    pub fn is_value_type(&self) -> bool {
        // TODO: Is this valid ???
        matches!(
            self.raw().type_(),
            Il2CppTypeEnum_IL2CPP_TYPE_VALUETYPE
                | Il2CppTypeEnum_IL2CPP_TYPE_I
                | Il2CppTypeEnum_IL2CPP_TYPE_I1
                | Il2CppTypeEnum_IL2CPP_TYPE_I2
                | Il2CppTypeEnum_IL2CPP_TYPE_I4
                | Il2CppTypeEnum_IL2CPP_TYPE_I8
                | Il2CppTypeEnum_IL2CPP_TYPE_U
                | Il2CppTypeEnum_IL2CPP_TYPE_U1
                | Il2CppTypeEnum_IL2CPP_TYPE_U2
                | Il2CppTypeEnum_IL2CPP_TYPE_U4
                | Il2CppTypeEnum_IL2CPP_TYPE_U8
                | Il2CppTypeEnum_IL2CPP_TYPE_R4
                | Il2CppTypeEnum_IL2CPP_TYPE_R8
                | Il2CppTypeEnum_IL2CPP_TYPE_BOOLEAN
                | Il2CppTypeEnum_IL2CPP_TYPE_CHAR
                | Il2CppTypeEnum_IL2CPP_TYPE_ENUM
                | Il2CppTypeEnum_IL2CPP_TYPE_PTR
                | Il2CppTypeEnum_IL2CPP_TYPE_FNPTR
        )
    }

    /// Whether the type is a reference type
    #[allow(non_upper_case_globals)]
    pub fn is_reference_type(&self) -> bool {
        // TODO: Is this valid ???
        matches!(
            self.raw().type_(),
            Il2CppTypeEnum_IL2CPP_TYPE_BYREF
                | Il2CppTypeEnum_IL2CPP_TYPE_OBJECT
                | Il2CppTypeEnum_IL2CPP_TYPE_ARRAY
                | Il2CppTypeEnum_IL2CPP_TYPE_STRING
                | Il2CppTypeEnum_IL2CPP_TYPE_CLASS
                | Il2CppTypeEnum_IL2CPP_TYPE_TYPEDBYREF
        )
    }
}

unsafe impl WrapRaw for Il2CppType {
    type Raw = raw::Il2CppType;
}
