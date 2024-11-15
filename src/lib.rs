use std::{borrow::Cow, collections::HashMap};

pub use copilot_rs_macro::{complete, FunctionTool};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use typed_builder::TypedBuilder;
pub trait FunctionTool {
    fn key() -> String;
    fn desc() -> String;
    fn inject(args: HashMap<String, serde_json::Value>) -> String;
}

pub trait FunctiomImplTrait {
    fn exec(&self) -> String;
}

#[derive(TypedBuilder, Debug, Serialize, Deserialize)]
pub struct ChatModel {
    pub chat_api_base: String,
    pub chat_model_default: String,
    pub chat_api_key: String,
}

pub fn chat(
    model: &ChatModel,
    messages: &[PromptMessage],
    chat_model: &str,
    temperature: f32,
    max_tokens: u32,
    tools: Vec<String>,
    keys: Vec<String>,
    functions: Vec<fn(std::collections::HashMap<String, serde_json::Value>) -> String>,
) -> String {
    let func_map =
        keys.clone()
            .into_iter()
            .zip(functions.iter())
            .fold(HashMap::new(), |mut acc, (k, v)| {
                acc.insert(k, v);
                acc
            });
    let tools: Vec<serde_json::Value> = tools
        .iter()
        .map(|v| serde_json::from_str(v).unwrap())
        .collect();
    let client = reqwest::blocking::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", model.chat_api_key).parse().unwrap(),
    );
    let url = format!("{}/chat/completions", model.chat_api_base);
    let common_builder = client.post(url).headers(headers);

    let json = json!({
        "model": model.chat_model_default,
        "messages": messages,
        "temperature": temperature,
        "max_tokens": max_tokens,
        "stream":false,
        "tools":tools
    });

    let builder = common_builder.try_clone().unwrap().json(&json);
    let res = builder.send().unwrap().text().unwrap();
    let res = serde_json::from_str::<ChatCompletion>(&res).unwrap();
    if let Some(common_message) = res.choices.first().and_then(|v| v.message.as_ref()) {
        if let Some(tool_calls) = &common_message.tool_calls {
            let tool_messages = tool_calls
                .iter()
                .map(|call| {
                    let call_name = &call.function.name;
                    let call_func = func_map.get(call_name).unwrap();
                    let args = call.function.arguments.clone();
                    let result = call_func(args);
                    result.tool(call.id.clone())
                })
                .collect::<Vec<_>>();
            let total_message = messages.iter().chain(&tool_messages).collect::<Vec<_>>();

            let json = json!({
                "model": model.chat_model_default,
                "messages": total_message,
                "temperature": temperature,
                "max_tokens": max_tokens,
                "stream":false,
            });

            let builder = common_builder.json(&json);
            let res = builder.send().unwrap().text().unwrap();
            let res = serde_json::from_str::<ChatCompletion>(&res).unwrap();
            let r = res
                .choices
                .first()
                .as_ref()
                .unwrap()
                .message
                .as_ref()
                .unwrap();
            r.content.clone()
        } else {
            common_message.content.clone()
        }
    } else {
        "none".to_string()
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
    #[serde(deserialize_with = "deserialize_map")]
    arguments: HashMap<String, serde_json::Value>,
}

fn deserialize_map<'de, D>(deserializer: D) -> Result<HashMap<String, serde_json::Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let json_string: String = Deserialize::deserialize(deserializer)?;
    let s = json_string.replace("\\", "");
    let map: HashMap<String, serde_json::Value> = serde_json::from_str(&s).unwrap();
    Ok(map)
}
pub trait Chat {
    fn chat(&self) -> String {
        "chat".to_string()
    }
}

impl Chat for Vec<PromptMessage> {}

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
