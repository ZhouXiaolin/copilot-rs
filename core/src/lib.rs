use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
pub trait FunctionTool {
    fn key() -> String;
    fn desc() -> ToolImpl;
    fn inject(args: std::collections::HashMap<String, serde_json::Value>) -> String;
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "function")]
pub enum ToolImpl {
    #[serde(rename = "function")]
    Function {
        name: String,
        description: String,
        parameters: Parameters,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Parameters {
    #[serde(default = "default_type")]
    pub r#type: String,
    pub properties: HashMap<String, Property>,
    pub required: Vec<String>,
}

const DEFAULT_TYPE: &str = "object";

pub fn default_type() -> String {
    DEFAULT_TYPE.to_string()
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Property {
    pub r#type: String,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,
    pub description: String,
}

impl ToTokens for Property {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let type_str = &self.r#type;
        let description = &self.description;
        let choices = &self.choices;

        let choices_tokens = if let Some(choices) = choices {
            quote! {
                Some(vec![#(#choices.to_string()),*])
            }
        } else {
            quote! {
                None
            }
        };

        let expanded = quote! {
            ::copilot_rs_core::Property {
                r#type: #type_str.to_string(),
                choices: #choices_tokens,
                description: #description.to_string(),
            }
        };
        tokens.extend(expanded);
    }
}

impl ToTokens for Parameters {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let type_str = &self.r#type;
        let properties = &self.properties;
        let required = &self.required;

        let property_tokens = properties.iter().map(|(key, value)| {
            let key_str = key.to_string();
            quote! {
                properties.insert(#key_str.to_string(), #value);
            }
        });

        let expanded = quote! {
            {
                let mut parameters = ::copilot_rs_core::Parameters {
                    r#type: #type_str.to_string(),
                    properties: std::collections::HashMap::new(),
                    required: vec![#(#required.to_string()),*],
                };
                let mut properties = std::collections::HashMap::new();
                #(#property_tokens)*
                parameters.properties = properties;
                parameters
            }
        };
        tokens.extend(expanded);
    }
}

impl ToTokens for ToolImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ToolImpl::Function {
                name,
                description,
                parameters,
            } => {
                let expanded = quote! {
                    ToolImpl::Function {
                        name: #name.to_string(),
                        description: #description.to_string(),
                        parameters: #parameters,
                    }
                };
                tokens.extend(expanded);
            }
        }
    }
}
