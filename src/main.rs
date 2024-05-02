#![allow(non_snake_case)]

mod api;
mod constants;
mod types;
use crate::{
    api::ChatCompletion,
    constants::{getBotId, AKENO},
    types::{
        AppError, ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs, Command,
        HandleResult, Role,
    },
};
use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::prelude::*;
type ChatMessages = Vec<ChatCompletionRequestMessage>;
type ChatHistories = HashMap<UserId, ChatMessages>;
type State = Arc<Mutex<ChatHistories>>;

async fn completeChat(content: String, bot: Bot, state: State, msg: Message) -> HandleResult {
    let hists;
    if content.is_empty() {
        return Ok(());
    };
    {
        let mut guard = state.lock().unwrap();
        let messages = guard.entry(msg.from().unwrap().id).or_default();
        if messages.is_empty() {
            let userName = msg
                .from()
                .and_then(|user| Some(user.first_name.clone()))
                .unwrap_or_default();
            let systemPrompt: String = AKENO.replace("[name]", &userName);
            messages.push(
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content(systemPrompt)
                    .build()?,
            )
        };
        messages.push(
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(content)
                .build()?,
        );
        hists = messages.clone();
    }

    let response = bot
        .send_message(msg.chat.id, "💭")
        .reply_to_message_id(msg.id)
        .await?;

    let msg_id = response.id;
    let content = ChatCompletion(hists).await?;
    bot.edit_message_text(msg.chat.id, msg_id, content.clone())
        .await?;

    {
        let mut guard = state.lock().unwrap();
        let messages = guard.entry(msg.from().unwrap().id).or_default();
        messages.push(
            ChatCompletionRequestMessageArgs::default()
                .role(Role::Assistant)
                .content(content.clone())
                .build()?,
        );
    }
    Ok(())
}

async fn setPrompt(prompt: String, bot: Bot, state: State, msg: Message) -> HandleResult {
    {
        let userName = msg
            .from()
            .and_then(|user| Some(user.first_name.clone()))
            .unwrap_or_default();
        let systemPrompt: String = if prompt.contains("[name]") {
            prompt.replace("[name]", &userName)
        } else {
            prompt
        };
        let mut guard = state.lock().unwrap();
        let messages = guard.entry(msg.from().unwrap().id).or_default();
        messages.clear();
        messages.push(
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(systemPrompt)
                .build()?,
        );
    };

    bot.send_message(msg.chat.id, "Prompt set.")
        .reply_to_message_id(msg.id)
        .await?;

    Ok(())
}

async fn viewHistories(bot: Bot, state: State, msg: Message) -> HandleResult {
    let content = {
        let mut guard = state.lock().unwrap();
        let messages = guard.entry(msg.from().unwrap().id).or_default();
        if messages.is_empty() {
            "Empty chat history.".to_owned()
        } else {
            messages
                .iter()
                .map(|msg| format!("[{}]: {}", msg.role, msg.content.trim()))
                .collect::<Vec<String>>()
                .join("\n\n")
        }
    };

    bot.send_message(msg.chat.id, content)
        .reply_to_message_id(msg.id)
        .await?;

    Ok(())
}

async fn clearHistory(bot: Bot, state: State, msg: Message) -> HandleResult {
    {
        let mut guard = state.lock().unwrap();
        let messages = guard.entry(msg.from().unwrap().id).or_default();
        messages.clear();
    }

    bot.send_message(msg.chat.id, "Chat histories cleared.")
        .reply_to_message_id(msg.id)
        .await?;

    Ok(())
}

async fn handleCommand(bot: Bot, state: State, msg: Message, cmd: Command) -> HandleResult {
    match cmd {
        Command::Prompt(prompt) => {
            setPrompt(prompt, bot, state, msg).await?;
        }
        Command::Chat(content) => {
            completeChat(content, bot, state, msg).await?;
        }
        Command::Askgpt(content) => {
            completeChat(content, bot, state, msg).await?;
        }
        Command::View => {
            viewHistories(bot, state, msg).await?;
        }
        Command::Clear => {
            clearHistory(bot, state, msg).await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    let TOKEN: &str = &dotenv::var("BOT_TOKEN").expect("BOT_TOKEN is not set.");
    let BOT_ID = UserId(getBotId(TOKEN));
    let bot = Bot::new(TOKEN);

    let state = Arc::new(Mutex::new(ChatHistories::new()));

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(handleCommand),
        )
        .branch(
            dptree::filter_async(move |msg: Message| async move {
                msg.text().unwrap().to_lowercase().starts_with("akeno")
            })
            .endpoint(move |msg: Message, bot: Bot, state| async move {
                let content = msg.text().unwrap_or_default();
                if content.is_empty() {
                    return Ok(());
                }
                completeChat(content.to_string(), bot, state, msg).await?;
                Ok(())
            }),
        )
        .branch(
            dptree::filter_async(move |msg: Message| async move {
                msg.reply_to_message().is_some()
                    && msg.reply_to_message().unwrap().from().unwrap().id == BOT_ID
            })
            .endpoint(move |msg: Message, bot: Bot, state| async move {
                let content = msg.text().unwrap_or_default();
                if content.is_empty() {
                    return Ok(());
                }
                completeChat(content.to_string(), bot, state, msg).await?;
                Ok(())
            }),
        )
        as Handler<'_, DependencyMap, Result<(), AppError>, DpHandlerDescription>;

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
