use copilot_rs::{complete, FunctionImplTrait, FunctionTool, IntoPrompt, Structure};
use serde::{Deserialize, Serialize};
extern crate copilot_rs;

fn main() {
    let a = test("class DataModel(BaseModel):
   name: str
   email: str
   cost: float  # Answer to the reasoning problem, stored as a float
   experience: list[str]
   skills: list[str]");
    println!("{a}");
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
#[complete(client="client", temperature=0.6, max_tokens=1000, response_format="Answer")]
fn test(name: &str) -> String {
    vec![format!("对输入按照指定的格式输出").system(),format!("John Doe is a freelance software engineer. He charges a base rate of $50 per
hour for the first 29 hours of work each week. For any additional hours, he
charges 1.7 times his base hourly rate. This week, John worked on a project
for 38 hours. How much will John Doe charge his client for the project this
week? 输出{}", name).user()].chat()
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

#[derive(FunctionTool, Deserialize, Serialize)]
#[props(desc = "add two number")]
struct Add {
    #[props(desc = "first number")]
    first: f64,
    #[props(desc = "second number")]
    second: f64,
}

impl FunctionImplTrait for Add {
    fn exec(&self) -> String {
        (self.first + self.second).to_string()
    }
}

// #[derive(Structure, Debug, Serialize, Deserialize)]
// #[props(doc = "Which is the highest mountain in the world? Mount Everest.")]
// struct Answer {
//     #[serde(default = "Which is the highest mountain in the world?")]
//     question: String,
//     #[props(default = "Mount Everest")]
//     answer: String,
// }