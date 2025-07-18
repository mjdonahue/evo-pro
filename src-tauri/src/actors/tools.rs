use macros::askable;
use rig::tool::ToolError;
use serde::ser::StdError;
use std::{borrow::Cow, collections::HashMap, io, ops::Deref, path::PathBuf, sync::Arc};

use color_eyre::eyre::eyre;
use futures_util::{StreamExt, future::BoxFuture};
use kameo::prelude::*;
use schemars::{JsonSchema, schema::RootSchema, schema_for};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, LossyError, Result};

#[derive(Actor)]
pub struct ToolExecutorActor {
    pub tools: HashMap<Cow<'static, str>, ToolWrapper<Arc<dyn ToolDyn + 'static>>>,
}

#[askable]
impl Message<UseTool> for ToolExecutorActor {
    type Reply = DelegatedReply<Result<String>>;

    async fn handle(&mut self, msg: UseTool, ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        let (delegated, sender) = ctx.reply_sender();
        let Some(tx) = sender else {
            return delegated;
        };
        let Some(tool) = self.tools.get(&msg.name).cloned() else {
            tx.send(Err(AppError::ToolCallError(LossyError::Lossless(
                ToolError::ToolCallError(eyre!("Could not call tool!").into()),
            ))));
            return delegated;
        };
        tokio::spawn(async move { tx.send(tool.call(msg.args).await) });
        delegated
    }
}

#[askable]
impl Message<GetTools> for ToolExecutorActor {
    type Reply = Vec<rig::completion::ToolDefinition>;

    async fn handle(
        &mut self,
        _msg: GetTools,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.tools
            .values()
            .map(|tool| tool.to_rig_tool())
            .collect::<Vec<_>>()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GetTools;

#[derive(Clone, Serialize, Deserialize)]
pub struct UseTool {
    pub name: Cow<'static, str>,
    pub args: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub params: RootSchema,
    pub returns: Option<RootSchema>,
}

impl From<ToolDefinition> for rig::completion::ToolDefinition {
    fn from(def: ToolDefinition) -> Self {
        rig::completion::ToolDefinition {
            name: def.name.into(),
            description: def.description.into(),
            parameters: match serde_json::to_value(&def.params) {
                Ok(params) => params,
                _ => serde_json::Value::Null,
            },
        }
    }
}

#[derive(Clone)]
pub struct ToolWrapper<T>(pub T);

impl<T> Deref for ToolWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Tool> Tool for ToolWrapper<T> {
    type Error = T::Error;
    type Args = T::Args;
    type Output = T::Output;

    const NAME: &'static str = T::NAME;

    fn definition(&self, prompt: String) -> ToolDefinition {
        T::definition(&self.0, prompt)
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send + Sync + 'static {
        T::call(&self.0, args)
    }
}

impl<T: Tool> rig::tool::Tool for ToolWrapper<T>
where
    <T as Tool>::Error: StdError,
{
    const NAME: &'static str = T::NAME;

    type Error = T::Error;
    type Args = T::Args;
    type Output = T::Output;

    async fn definition(&self, prompt: String) -> rig::completion::ToolDefinition {
        T::definition(&self.0, prompt).into()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(T::call(&self.0, args).await?)
    }
}

pub trait Tool: Sized + Send + Sync {
    type Error: Into<AppError> + Send + Sync + 'static;
    type Args: for<'a> Deserialize<'a> + Send + Sync;
    type Output: Serialize;

    const NAME: &'static str;

    fn definition(&self, prompt: String) -> ToolDefinition;
    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send + Sync + 'static;
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(Self::NAME)
    }
}

pub trait ToolDyn: Send + Sync {
    fn name(&self) -> Cow<'static, str>;

    fn definition(&self, prompt: String) -> ToolDefinition;

    fn call(&self, args: String) -> BoxFuture<Result<String, AppError>>;

    fn to_rig_tool(&self) -> rig::completion::ToolDefinition {
        let def = self.definition(String::new());
        rig::completion::ToolDefinition {
            name: def.name.into(),
            description: def.description.into(),
            parameters: match serde_json::to_value(&def.params) {
                Ok(params) => params,
                _ => serde_json::Value::Null,
            },
        }
    }
}

impl<T: Tool> ToolDyn for T {
    fn name(&self) -> Cow<'static, str> {
        self.name()
    }

    fn definition(&self, prompt: String) -> ToolDefinition {
        self.definition(prompt)
    }

    fn call(&self, args: String) -> BoxFuture<Result<String, AppError>> {
        Box::pin(async move {
            let args: T::Args = serde_json::from_str(&args)?;
            let output = T::call(self, args).await.map_err(|e| e.into())?;
            Ok(serde_json::to_string(&output)?)
        })
    }

    fn to_rig_tool(&self) -> rig::completion::ToolDefinition {
        let def = self.definition(String::new());
        rig::completion::ToolDefinition {
            name: def.name.into(),
            description: def.description.into(),
            parameters: match serde_json::to_value(&def.params) {
                Ok(params) => params,
                _ => serde_json::Value::Null,
            },
        }
    }
}

pub struct ReadFile;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ReadFileArgs {
    pub path: PathBuf,
}

impl Tool for ReadFile {
    type Error = io::Error;
    type Args = ReadFileArgs;
    type Output = String;

    const NAME: &'static str = "read_file";

    fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Reads the entire contents of a file on disk.".into(),
            params: schema_for!(ReadFileArgs),
            returns: Some(schema_for!(String)),
        }
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send + Sync + 'static {
        // Use an `async move` block to create a 'static future
        async move { Ok(tokio::fs::read_to_string(&args.path).await?) }
    }
}

pub struct WriteFile;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct WriteFileArgs {
    pub path: PathBuf,
    pub content: String,
}

impl Tool for WriteFile {
    type Error = io::Error;
    type Args = WriteFileArgs;
    type Output = ();

    const NAME: &'static str = "write_file";

    fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Reads the entire contents of a file on disk.".into(),
            params: schema_for!(WriteFileArgs),
            returns: None,
        }
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send + Sync + 'static {
        async move { Ok(tokio::fs::write(&args.path, &args.content).await?) }
    }
}
