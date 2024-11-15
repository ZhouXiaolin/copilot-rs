use copilot_rs::{complete, FunctiomImplTrait, FunctionTool, IntoPrompt};
use serde::{Deserialize, Serialize};
extern crate copilot_rs;

fn main() {
    let a = test("深圳");
    println!("{a}");

}


fn client() -> copilot_rs::ChatModel {
    let config = include_str!("../config.json");
    serde_json::from_str(config).unwrap()
}
// complete会将函数体和参数注入到函数中
#[complete(client="client", temperature=0.6, max_tokens=1000, tools=["GetCurrentWeather","Add"])]
fn test(name: &str) -> String {
    vec![format!("{}今天天气怎么样",name).user()].chat()
}


#[derive(FunctionTool, Deserialize, Serialize)]
#[property(desc = "Get weather of an location, the user shoud supply a location first")]
struct GetCurrentWeather {
    #[property(desc = "The city and state, e.g. San Francisco, CA")]
    location: String,
}

impl FunctiomImplTrait for GetCurrentWeather {
    fn exec(&self) -> String {
        "大暴雨，由于雨势太大，可能发生洪灾".to_string()
    }
}


#[derive(FunctionTool, Deserialize, Serialize)]
#[property(desc = "add two number")]
struct Add {
    #[property(desc = "first number")]
    first: f64,
    #[property(desc = "second number")]
    second: f64,
}

impl FunctiomImplTrait for Add {
    fn exec(&self) -> String {
        (self.first + self.second).to_string()
    }
}