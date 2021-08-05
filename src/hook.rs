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
