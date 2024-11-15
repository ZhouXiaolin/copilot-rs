# Copilot Rust SDK

## Overview
Copilot Rust SDK is a Rust library for interacting with the chat model.
It provides integration with the chat API, support for custom function tools, and message handling.

## Features
- **Chat model integration**: Interact with the chat API through the `ChatModel` structure.
- **Custom function tool**: Implement custom function tools using `FunctionTool` and `FunctiomImplTrait`.
- **Macro support**: Simplify function tool injection and configuration using `complete` macro.

## Installation
Add the following dependencies to your `Cargo.toml`:
```toml
copilot-rs = "0.1.0"
```

## Usage
### Chat model integration
To interact with the chat model, you need to create a `ChatModel` instance with the necessary configuration.

normally, you can use the `ChatModel::builder()` method to create a `ChatModel` instance.

or you can use serde to deserialize a `ChatModel` instance from a JSON string.

then use complete macro to inject paramaters and function tools into the chat function.


```rust
fn client() -> copilot_rs::ChatModel {
    ...
}

#[complete(client="client", temperature=0.6, max_tokens=1000, tools=["GetCurrentWeather","Add"])]
fn test(name: &str) -> String {
    vec![format!("{}今天天气怎么样",name).user()].chat()
}
```

### Custom function tool
You can define your own function tool by implementing the `FunctionTool` and `FunctiomImplTrait` traits.
also, you need implement serde's `Deserialize` and `Serialize` traits. beacuse copilot-rs will use serde to deserialize the function tool from a JSON string. 
```rust
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
```

more detail, please see the example in the `src/main.rs` file.

### Macro support
copilot-rs provides a macro called `complete` to simplify the process of injecting parameters and function tools into the chat function.

the `complete` macro takes the following parameters:
- `client`: the name of the function that returns a `ChatModel` instance.
- `temperature`: the temperature of the chat model. default is 0.7.
- `max_tokens`: the maximum number of tokens to generate. default is 1000.
- `tools`: a list of function tools to inject into the chat function.


if you have some issues with it, please feel free to open an issue or submit a pull request.