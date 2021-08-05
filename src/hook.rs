use libil2cpp::{Parameter, Parameters, Return, ThisParameter};

/// Trait implemented by all hooks to facilitate generic programming
pub trait Hook<const N: usize> {
    /// Type of this for the hooked method
    type This: ThisParameter;
    /// Type of the parameters for the hooked method
    type Parameters: Parameters<N>;
    /// Type of the return for the hooked method
    type Return: Return;

    /// Installs the hook
    fn install(&self);

    /// Namespace of the hooked method's class
    fn namespace(&self) -> &'static str;

    /// Name of the hooked method's class
    fn class_name(&self) -> &'static str;

    /// Name of the hooked method
    fn method_name(&self) -> &'static str;

    /// Number of parameters the hooked method takes
    fn parameters_count(&self) -> usize;

    /// Pointer to the hook function
    fn hook(&self) -> *mut ();

    /// Pointer to the hooked method
    fn original(&self) -> *mut ();
}

/// Trait implemented by types which can be used as a this parameter in Rust
/// hook signatures
pub trait HookThis {
    /// Actual type of the this parameter
    type Actual: ThisParameter;

    /// Converts the actual type into the desired one
    fn from_actual(actual: Self::Actual) -> Self;
}

/// Trait implemented by types which can be used as a parameter in Rust hook
/// signatures
pub trait HookParameter {
    /// Actual type of the parameter
    type Actual: Parameter;

    /// Converts the actual type into the desired one
    fn from_actual(actual: Self::Actual) -> Self;
}

/// Trait implemented by types which can be used as a return value in Rust hook
/// signatures
pub trait HookResult {
    /// Actual type of the return value
    type Actual: HookResult;

    /// Converts the type into the actual one
    fn into_actual(self) -> Self::Actual;
}

default impl<T> HookThis for T
where
    T: ThisParameter,
{
    default type Actual = Self;

    #[inline]
    default fn from_actual(actual: Self::Actual) -> Self {
        actual
    }
}

impl<T> HookThis for T
where
    Option<T>: ThisParameter,
{
    type Actual = Option<T>;

    #[inline]
    fn from_actual(actual: Self::Actual) -> Self {
        actual.unwrap()
    }
}
