use std::iter::once;

use copilot_rs::{
    complete, IntoPrompt, PromptMessage,
};

fn main() {
    let mut cathy =
        ConversableAgent::new("cathy","Your name is Cathy and you are a part of a duo of comedians.",talk_0);
    let mut joe =
        ConversableAgent::new("joe","Your name is Joe and you are a part of a duo of comedians.",talk_1);
    cathy.initiate_chat(&mut joe, "讲个笑话", 3);
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

#[complete(client = "client", temperature = 0.6, max_tokens = 4096)]
fn talk_0(system: &str, history: Vec<PromptMessage>) -> String {
    let history = once(system.system()).chain(history);
    let history: Vec<_> = history.collect();
    history.chat()
}


#[complete(client = "client", temperature = 0.8, max_tokens = 4096)]
fn talk_1(system: &str, history: Vec<PromptMessage>) -> String {
    let history = once(system.system()).chain(history);
    let history: Vec<_> = history.collect();
    history.chat()
}

struct ConversableAgent {
    name:&'static str,
    system: &'static str,
    history: Vec<PromptMessage>,
    talk: fn(system: &str, history: Vec<PromptMessage>) -> String,
}

impl ConversableAgent {
    fn new(name:&'static str,system: &'static str,talk: fn(system: &str, history: Vec<PromptMessage>) -> String) -> ConversableAgent {
        ConversableAgent {
            name,
            system,
            history: vec![],
            talk,
        }
    }
}

impl ConversableAgent {
    fn initiate_chat(&mut self, agent: &mut ConversableAgent, message: &str, max_turns: u32) {
        let mut flow_message = message.to_string();
        for _ in 0..max_turns {
            agent.history.push(flow_message.user());
            flow_message = (agent.talk)(agent.system, agent.history.clone());
            agent.history.push(flow_message.assistant());
            println!("{} ==> {}:\n{}\n",agent.name,self.name, flow_message);

            self.history.push(flow_message.user());
            flow_message = (self.talk)(self.system, self.history.clone());
            self.history.push(flow_message.assistant());
            println!("{} ==> {}:\n{}\n",self.name, agent.name, flow_message);
        }
    }
}
