use std::borrow::Cow;

use copilot_rs_core::ToolImpl;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, Debug, Serialize, Deserialize)]
pub struct Client {
    pub api_base: String,
    pub api_key: String,
    pub model_default: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAIRequest<'a> {
    pub model: String,
    pub messages: Vec<PromptMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<&'a ToolImpl>>,
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
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub function: Function,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ChatCompletion {
    pub choices: Vec<Choice>,
    pub created: u64,
    pub id: String,
    pub model: String,
    pub object: String,
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
pub struct Choice {
    pub delta: Option<Delta>,
    pub message: Option<PromptMessage>,
    pub finish_reason: Option<String>,
    pub index: u32,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}
