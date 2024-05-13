use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::fmt;
use teloxide::utils::command::BotCommands;
use teloxide::RequestError;

pub type HandleResult = Result<(), AppError>;

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "See all available commands")]
    Help,
    #[command(description = "set system prompt.")]
    Prompt(String),
    #[command(description = "chat with ai.")]
    Chat(String),
    #[command(description = "chat with ai.")]
    Askgpt(String),
    #[command(description = "view chat histories.")]
    View,
    #[command(description = "clear history chats.")]
    Clear,
    #[command(description = "clear history chats.")]
    Reset,
    #[command(description = "source.")]
    Source,
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Teloxide error")]
    Teloxide(#[from] RequestError),

    #[error("HTTP error")]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct APIResponse {
    pub(crate) code: i32,
    pub(crate) message: String,
    pub(crate) content: Option<String>,
    pub(crate) images: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    #[default]
    User,
    Assistant,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::System => write!(f, "System"),
            Role::User => write!(f, "User"),
            Role::Assistant => write!(f, "Assistant"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Builder)]
#[builder(name = "ChatCompletionRequestMessageArgs")]
#[builder(pattern = "mutable")]
#[builder(setter(into, strip_option), default)]
#[builder(derive(Debug))]
#[builder(build_fn(error = "RequestError"))]
pub struct ChatCompletionRequestMessage {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

