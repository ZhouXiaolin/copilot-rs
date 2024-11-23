use copilot_rs::{complete, FunctionImplTrait, FunctionTool, IntoPrompt, ToolImpl};
use serde::{Deserialize, Serialize};
extern crate copilot_rs;

fn main() {
    let response = test("天津");
    println!("{}", response);
}

fn client() -> copilot_rs::Client {
    let config = include_str!("../config.json");
    serde_json::from_str(config).unwrap()
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
