mod types;
use anyhow::{Context, Result};
pub use copilot_rs_core::*;
pub use copilot_rs_macro::{complete, FunctionTool};
use std::{collections::HashMap, marker::PhantomData, pin::Pin};
use types::{ChatCompletion, OpenAIRequest, Role};
pub use types::{Client, PromptMessage};
pub trait Structure {}

pub trait FunctionImplTrait {
    fn exec(&self) -> String;
}

type InjectionImpl = fn(std::collections::HashMap<String, serde_json::Value>) -> String;
type FunctionName = String;

struct NormalChat<T = String> {
    _marker: PhantomData<T>,
}

pub fn chat(
    model: &Client,
    messages: &[PromptMessage],
    chat_model: &str,
    temperature: f32,
    max_tokens: u32,
    functions: HashMap<FunctionName, (ToolImpl, InjectionImpl)>,
) -> String {
    match normal_chat(
        model,
        messages,
        chat_model,
        temperature,
        max_tokens,
        functions,
    ) {
        Ok(output) => output,
        Err(e) => e.to_string(),
    }
}

pub fn normal_chat(
    client: &Client,
    messages: &[PromptMessage],
    chat_model: &str,
    temperature: f32,
    max_tokens: u32,
    functions: HashMap<FunctionName, (ToolImpl, InjectionImpl)>,
) -> Result<String> {
    let tools: Vec<_> = functions.iter().map(|(_, (v, _))| v).collect();
    let requst_client = reqwest::blocking::Client::new();
    let url = format!("{}/chat/completions", client.api_base);
    let common_builder = requst_client.post(url).bearer_auth(&client.api_key);

    let chat_model = if chat_model.is_empty() {
        &client.model_default
    } else {
        chat_model
    };

    let json = OpenAIRequest {
        model: chat_model.to_string(),
        messages: messages.to_vec(),
        max_tokens,
        temperature,
        stream: false,
        tools: (!tools.is_empty()).then_some(tools),
    };

    let builder = common_builder
        .try_clone()
        .context("build request")?
        .json(&json);
    let res = builder.send()?.text()?;
    let res = serde_json::from_str::<ChatCompletion>(&res)?;
    if let Some(common_message) = res.choices.first().and_then(|v| v.message.as_ref()) {
        if let Some(tool_calls) = &common_message.tool_calls {
            let tool_messages = tool_calls
                .first()
                .map(|call| {
                    let call_name = &call.function.name;
                    let (_, call_func) = functions.get(call_name).unwrap();
                    let args = &call.function.arguments;
                    let args = args.replace("\\\"", "\"");
                    let args: HashMap<String, serde_json::Value> =
                        serde_json::from_str(&args).unwrap();
                    let result = call_func(args);
                    result.tool(call.id.clone())
                })
                .unwrap();
            let tool_messages = vec![common_message.clone(), tool_messages];
            let total_message = messages
                .iter()
                .chain(&tool_messages)
                .map(|v| v.clone())
                .collect::<Vec<_>>();
            let json = OpenAIRequest {
                model: client.model_default.to_string(),
                messages: total_message,
                max_tokens,
                temperature,
                stream: false,
                tools: None,
            };

            let builder = common_builder.json(&json);
            let res = builder.send()?.text()?;
            let res = serde_json::from_str::<ChatCompletion>(&res)?;
            let r = res
                .choices
                .first()
                .as_ref()
                .context("no choices")?
                .message
                .as_ref()
                .context("no message")?;
            Ok(r.content.clone())
        } else {
            Ok(common_message.content.clone())
        }
    } else {
        Ok("none".to_string())
    }
}

pub trait Chat {
    fn chat(&self) -> String {
        "chat".to_string()
    }

    fn async_chat(&self) -> Pin<Box<impl std::future::Future<Output = String>>> {
        Box::pin(async { "async_chat".to_string() })
    }
}

impl Chat for Vec<PromptMessage> {}
impl Chat for dyn AsRef<[PromptMessage]> {}

pub trait IntoPrompt
where
    Self: ToString,
{
    fn system(&self) -> PromptMessage {
        PromptMessage {
            role: Role::System,
            content: self.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }
    }
    fn user(&self) -> PromptMessage {
        PromptMessage {
            role: Role::User,
            content: self.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }
    }
    fn assistant(&self) -> PromptMessage {
        PromptMessage {
            role: Role::Assistant,
            content: self.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }
    }
    fn tool(&self, id: String) -> PromptMessage {
        PromptMessage {
            role: Role::Tool,
            content: self.to_string(),
            tool_calls: None,
            tool_call_id: Some(id),
        }
    }
}

impl IntoPrompt for &str {}

impl IntoPrompt for String {}
