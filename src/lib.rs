use std::{borrow::Cow, collections::HashMap, iter::once, marker::PhantomData};

pub use copilot_rs_macro::{complete, FunctionTool};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use typed_builder::TypedBuilder;
use anyhow::{Context, Result};
pub trait FunctionTool {
    fn key() -> String;
    fn desc() -> String;
    fn inject(args: HashMap<String, serde_json::Value>) -> String;
}
pub trait Structure {}

pub trait FunctionImplTrait {
    fn exec(&self) -> String;
}

#[derive(TypedBuilder, Debug, Serialize, Deserialize)]
pub struct Client {
    pub api_base: String,
    pub api_key: String,
    pub model_default: String,
}
type FuncImpl = fn(std::collections::HashMap<String, serde_json::Value>) -> String;

struct NormalChat<T = String> {
    _marker: PhantomData<T>,
}

pub fn chat(
    model: &Client,
    messages: &[PromptMessage],
    chat_model: &str,
    temperature: f32,
    max_tokens: u32,
    functions: HashMap<String, (String, FuncImpl)>,
) -> String{
    match normal_chat(model, messages, chat_model, temperature, max_tokens, functions) {
        Ok(output) => output,
        Err(e) => e.to_string(),
    }
}


type FunctionName = String;
type FunctionDesc = String;
struct OpenAIRequest {
    model: String,
    messages: Vec<PromptMessage>,
    max_tokens: u32,
    temperature: f32,
    functions: HashMap<FunctionName, (FunctionDesc, FuncImpl)>,
    presence_penalty: f32,
    frequency_penalty: f32,
    stop: Vec<String>,
    stream: bool, 
}

pub fn normal_chat(
    client: &Client,
    messages: &[PromptMessage],
    chat_model: &str,
    temperature: f32,
    max_tokens: u32,
    functions: HashMap<String, (String, FuncImpl)>,
) -> Result<String> {
    let tools: Vec<serde_json::Value> = functions
        .iter()
        .map(|(_, (v, _))| serde_json::from_str(v).unwrap())
        .collect();
    let requst_client = reqwest::blocking::Client::new();
    let url = format!("{}/chat/completions", client.api_base);
    let common_builder = requst_client.post(url).bearer_auth(&client.api_key);

    let chat_model = if chat_model.is_empty() {
        &client.model_default
    } else {
        chat_model
    };
    let mut json = json!({
        "model":chat_model,
        "messages": messages,
        "temperature": temperature,
        "max_tokens": max_tokens,
        "stream":false,
    });
    if !tools.is_empty() {
        json["tools"] = serde_json::Value::Array(tools);
    }

    let builder = common_builder.try_clone().context("build request")?.json(&json);
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
            let total_message = messages.iter().chain(&tool_messages).collect::<Vec<_>>();

            let json = json!({
                "model": client.model_default,
                "messages": total_message,
                "temperature": temperature,
                "max_tokens": max_tokens,
                "stream":false,
            });

            let builder = common_builder.json(&json);
            let res = builder.send()?.text()?;
            let res = serde_json::from_str::<ChatCompletion>(&res)?;
            let r = res
                .choices
                .first()
                .as_ref().context("no choices")?
                .message
                .as_ref().context("no message")?;
            Ok(r.content.clone())
        } else {
            Ok(common_message.content.clone())
        }
    } else {
        Ok("none".to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PromptMessage {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    ty: String,
    function: Function,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    name: String,
    arguments: String,
}

pub trait Chat {
    fn chat(&self) -> String {
        "chat".to_string()
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

#[derive(Debug, Deserialize, Default)]
pub struct ChatCompletion {
    choices: Vec<Choice>,
    created: u64,
    id: String,
    model: String,
    object: String,
}

impl ChatCompletion {
    pub fn get_content(&self) -> Cow<str> {
        if let Some(content) = self.choices[0]
            .delta
            .as_ref()
            .and_then(|v| v.content.as_ref())
        {
            Cow::Borrowed(content)
        } else if let Some(msg) = self.choices[0].message.as_ref() {
            Cow::Borrowed(&msg.content)
        } else {
            Cow::Borrowed("")
        }
    }
}

#[derive(Debug, Deserialize)]
struct Choice {
    delta: Option<Delta>,
    message: Option<PromptMessage>,
    finish_reason: Option<String>,
    index: u32,
}

#[derive(Debug, Deserialize)]
struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}
