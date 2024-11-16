use copilot_rs::{complete, FunctionImplTrait, FunctionTool, IntoPrompt, Structure};
use serde::{Deserialize, Serialize};
extern crate copilot_rs;

fn main() {
    let response = test("天津");
    println!("{}",response);
}

fn client() -> copilot_rs::ChatModel {
    let config = include_str!("../config.json");
    serde_json::from_str(config).unwrap()
    // let chat_model = copilot_rs::ChatModel::builder()
    //     .chat_api_base("chat_api_base".to_string())
    //     .chat_api_key("chat_api_key".to_string())
    //     .chat_model_default("chat_model_default".to_string())
    //     .build();
    // chat_model
}
// complete会将函数体和参数注入到函数中
#[complete(client="client", temperature=0.6, max_tokens=1000, tools = ["GetCurrentWeather"])]
fn test(name: &str) -> String {
    vec![format!("{}天气如何？", name).user()].chat()
}

#[derive(FunctionTool, Deserialize, Serialize)]
#[props(desc = "Get weather of an location, the user shoud supply a location first")]
struct GetCurrentWeather {
    #[props(desc = "The city and state, e.g. San Francisco, CA")]
    location: String,
}

impl FunctionImplTrait for GetCurrentWeather {
    fn exec(&self) -> String {
        "剧烈高温".to_string()
    }
}