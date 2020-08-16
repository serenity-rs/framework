use serenity::prelude::{Context as SerenityContext, RwLock};
use serenity::model::prelude::Message;
use serenity::futures::future::BoxFuture;

use std::sync::Arc;
use std::error::Error as StdError;
use std::marker::PhantomData;
use std::collections::HashMap;

pub type DefaultError = Box<dyn StdError + Send + Sync>;

#[derive(Clone)]
pub struct Context<D = ()> {
    pub data: Arc<RwLock<D>>,
    pub msg: Message,
    parent_ctx: SerenityContext,
}

pub type CommandResult<E = DefaultError> = std::result::Result<(), E>;
pub type CommandFn<D, E> = fn(ctx: Context<D>) -> BoxFuture<'static, CommandResult<E>>;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandId(pub u64);

#[derive(Debug, Clone)]
pub struct Command<D, E> {
    pub function: CommandFn<D, E>,
    pub names: Vec<String>,
    pub subcommands: Vec<CommandId>,
}

#[derive(Debug, Clone)]
pub struct Framework<D = (), E = DefaultError> {
    data: Arc<RwLock<D>>,
    command_name_to_id: HashMap<String, CommandId>,
    commands: HashMap<CommandId, Command<D, E>>,
    _error: PhantomData<E>,
}

impl<D, E> Default for Framework<D, E>
where
    D: Default
{
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(D::default())),
            command_name_to_id: HashMap::default(),
            commands: HashMap::default(),
            _error: PhantomData,
        }
    }
}

impl<D, E> Framework<D, E>
where
    D: Default
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<D, E> Framework<D, E> {
    pub fn with_data(data: D) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
            command_name_to_id: HashMap::default(),
            commands: HashMap::default(),
            _error: PhantomData,
        }
    }

    pub fn command(&mut self, command: fn() -> Command<D, E>) -> &mut Self {
        let command = command();

        let id = CommandId(command.function as u64);

        for name in &command.names {
            self.command_name_to_id.insert(name.clone(), id);
        }

        self.commands.insert(id, command);

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestData {
        text: String,
    }

    fn ping(ctx: Context<TestData>) -> BoxFuture<'static, CommandResult> {
        async move {
            println!("Hello world!");
            println!("{:?}", ctx.data.read().await.text);
            Ok(())
        }.boxed()
    }

    #[tokio::test]
    async fn construction() {
        let _framework: Framework = Framework::new();
        let _framework: Framework<(), DefaultError> = Framework::new();
        let _framework: Framework<TestData> = Framework::new();
        let mut framework: Framework<TestData> = Framework::with_data(TestData {
            text: "42 is the answer to life, the universe, and everything.".to_string(),
        });

        framework.command(|| Command {
            function: ping,
            names: vec!["ping".to_string()],
            subcommands: Vec::new(),
        });
    }
}
