use libil2cpp::{Parameters, Return, ThisParameter};

/// Trait implemented by all hooks to facilitate generic programming
pub trait Hook {
    /// Type of this for the hooked method
    type This: ThisParameter;
    /// Type of the parameters for the hooked method
    type Parameters: Parameters;
    /// Type of the return for the hooked method
    type Return: Return;

    /// Namespace of the hooked method's class
    const NAMESPACE: &'static str;
    /// Name of the hooked method's class
    const CLASS_NAME: &'static str;
    /// Name of the hooked method
    const METHOD_NAME: &'static str;

    /// Installs the hook
    fn install(&self) -> Result<(), HookInstallError>;

    /// Pointer to the hook function
    fn hook(&self) -> *mut ();
    /// Pointer to the hooked method
    fn original(&self) -> *mut ();
}

/// Possible errors when installing a hook
#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookInstallError {
    /// Hook already installed
    #[error("hook already installed")]
    AlreadyInstalled,

    /// Class not found
    #[error("class not found")]
    ClassNotFound,

    /// Method not found
    #[error("method not found")]
    MethodNotFound,
}
