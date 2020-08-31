use crate::context::CheckContext;
use crate::{DefaultData, DefaultError};

use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;

use std::fmt;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Reason {
    Unknown,
    User(String),
    Log(String),
    UserAndLog { user: String, log: String },
}

pub type CheckResult<T = ()> = std::result::Result<T, Reason>;

pub type CheckFunction<D = DefaultData, E = DefaultError> =
    for<'fut> fn(&'fut CheckContext<'_, D, E>, &'fut Message) -> BoxFuture<'fut, CheckResult<()>>;

pub type CheckConstructor<D = DefaultData, E = DefaultError> = fn() -> Check<D, E>;

#[non_exhaustive]
pub struct Check<D = DefaultData, E = DefaultError> {
    pub name: String,
    pub function: CheckFunction<D, E>,
    pub check_in_help: bool,
    pub display_in_help: bool,
}

impl<D, E> Clone for Check<D, E> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            function: self.function,
            check_in_help: self.check_in_help,
            display_in_help: self.display_in_help,
        }
    }
}

impl<D, E> Default for Check<D, E> {
    fn default() -> Self {
        Self {
            name: String::default(),
            function: |_, _| Box::pin(async move { Ok(()) }),
            check_in_help: true,
            display_in_help: true,
        }
    }
}

impl<D, E> fmt::Debug for Check<D, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Check")
            .field("name", &self.name)
            .field("function", &"<fn>")
            .field("check_in_help", &self.check_in_help)
            .field("display_in_help", &self.display_in_help)
            .finish()
    }
}

impl<D, E> Check<D, E> {
    pub fn builder() -> CheckBuilder<D, E> {
        CheckBuilder::default()
    }
}

pub struct CheckBuilder<D, E> {
    inner: Check<D, E>,
}

impl<D, E> CheckBuilder<D, E> {
    pub fn name<I>(mut self, name: I) -> Self
    where
        I: Into<String>,
    {
        self.inner.name = name.into();
        self
    }

    pub fn function(mut self, function: CheckFunction<D, E>) -> Self {
        self.inner.function = function;
        self
    }

    pub fn check_in_help(mut self, check_in_help: bool) -> Self {
        self.inner.check_in_help = check_in_help;
        self
    }

    pub fn display_in_help(mut self, display_in_help: bool) -> Self {
        self.inner.display_in_help = display_in_help;
        self
    }

    pub fn build(self) -> Check<D, E> {
        self.inner
    }
}

impl<D, E> Clone for CheckBuilder<D, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<D, E> Default for CheckBuilder<D, E> {
    fn default() -> Self {
        Self {
            inner: Check::default(),
        }
    }
}

impl<D, E> fmt::Debug for CheckBuilder<D, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CheckBuilder")
            .field("inner", &self.inner)
            .finish()
    }
}
