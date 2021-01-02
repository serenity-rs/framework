//! Functions and types relating to checks.
//!
//! A check is a function that can be plugged into a [command] to allow/deny
//! a user's access. The check returns a [`Result`] that indicates whether
//! it succeeded or failed. In the case of failure, additional information
//! can be given, a reason, that describes the failure.
//!
//! [command]: crate::command

use crate::context::CheckContext;
use crate::DefaultData;

use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;

use std::error::Error as StdError;
use std::fmt::{self, Display};

/// The reason describing why a check failed.
///
/// # Notes
///
/// This information is not handled by the framework; it is only propagated
/// to the consumer of the framework.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Reason {
    /// There is no information.
    Unknown,
    /// Information for the user.
    User(String),
    /// Information for logging purposes.
    Log(String),
    /// Information both for the user and logging purposes.
    UserAndLog {
        /// Information for the user.
        user: String,
        /// Information for logging purposes.
        log: String,
    },
}

impl Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => f.write_str("Unknown"),
            Self::User(msg) => write!(f, "User: {}", msg),
            Self::Log(msg) => write!(f, "Log: {}", msg),
            Self::UserAndLog { user, log } => write!(f, "User: {}; Log: {}", user, log),
        }
    }
}

impl StdError for Reason {}

/// The result type of a [check function][fn]
///
/// [fn]: CheckFn
pub type CheckResult<T = ()> = std::result::Result<T, Reason>;

/// The definition of a check function.
pub type CheckFn<D = DefaultData> =
    for<'fut> fn(&'fut CheckContext<'_, D>, &'fut Message) -> BoxFuture<'fut, CheckResult<()>>;

/// A constructor of the [`Check`] type provided by the consumer of the framework.
pub type CheckConstructor<D = DefaultData> = fn() -> Check<D>;

/// Data relating to a check.
///
/// Refer to the [module-level documentation][docs]
///
/// [docs]: crate::check
#[non_exhaustive]
pub struct Check<D = DefaultData> {
    /// Name of the check.
    ///
    /// Used in help commands.
    pub name: String,
    /// The function of this check.
    pub function: CheckFn<D>,
    /// A boolean indicating whether the check can apply in help commands.
    pub check_in_help: bool,
    /// A boolean indicating whether the check can be displayed in help commands.
    pub display_in_help: bool,
}

impl<D> Clone for Check<D> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            function: self.function,
            check_in_help: self.check_in_help,
            display_in_help: self.display_in_help,
        }
    }
}

impl<D> Default for Check<D> {
    fn default() -> Self {
        Self {
            name: String::default(),
            function: |_, _| Box::pin(async move { Ok(()) }),
            check_in_help: true,
            display_in_help: true,
        }
    }
}

impl<D> fmt::Debug for Check<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Check")
            .field("name", &self.name)
            .field("function", &"<fn>")
            .field("check_in_help", &self.check_in_help)
            .field("display_in_help", &self.display_in_help)
            .finish()
    }
}

impl<D> Check<D> {
    /// Constructs a builder that will be used to create a check from scratch.
    ///
    /// Argument is the main name of the check.
    pub fn builder<I>(name: I) -> CheckBuilder<D>
    where
        I: Into<String>,
    {
        CheckBuilder::new(name)
    }
}

/// A builder type for creating a [`Check`] from scratch.
pub struct CheckBuilder<D> {
    inner: Check<D>,
}

impl<D> CheckBuilder<D> {
    /// Constructs a new instance of the builder.
    ///
    /// Argument is the main name of the check.
    pub fn new<I>(name: I) -> Self
    where
        I: Into<String>,
    {
        CheckBuilder {
            inner: Check {
                name: name.into(),
                ..Default::default()
            },
        }
    }
    /// Assigns the function to this function.
    pub fn function(mut self, function: CheckFn<D>) -> Self {
        self.inner.function = function;
        self
    }

    /// Assigns the indicator to this function.
    pub fn check_in_help(mut self, check_in_help: bool) -> Self {
        self.inner.check_in_help = check_in_help;
        self
    }

    /// Assigns the indicator to this function.
    pub fn display_in_help(mut self, display_in_help: bool) -> Self {
        self.inner.display_in_help = display_in_help;
        self
    }

    /// Complete building a check.
    pub fn build(self) -> Check<D> {
        self.inner
    }
}

impl<D> Clone for CheckBuilder<D> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<D> Default for CheckBuilder<D> {
    fn default() -> Self {
        Self {
            inner: Check::default(),
        }
    }
}

impl<D> fmt::Debug for CheckBuilder<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CheckBuilder")
            .field("inner", &self.inner)
            .finish()
    }
}
