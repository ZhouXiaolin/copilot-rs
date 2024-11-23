use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Result;
use copilot_rs_core::{default_type, Parameters, Property, ToolImpl};
use darling::{ast::NestedMeta, FromMeta};
use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Ident};
use syn::{Expr, ItemFn, LitStr, Stmt};
#[proc_macro_attribute]
pub fn complete(attr: TokenStream, item: TokenStream) -> proc_macro::TokenStream {
    match common_simple(attr, item) {
        Ok(output) => output,
        Err(e) => TokenStream::from_str(e.to_string().as_str()).unwrap(),
    }
}
#[derive(Debug, FromMeta)]
struct MacroArgs {
    client: String,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    tools: Option<Vec<LitStr>>,
    response_format: Option<String>,
}

fn common_simple(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let attr_args = NestedMeta::parse_meta_list(attr.into())?;
    let args = MacroArgs::from_list(&attr_args).unwrap();

    let client = Ident::new(&args.client, proc_macro::Span::call_site().into());

    let mut item: ItemFn = syn::parse(item)?;

    let method_name = item.sig.ident.to_string();
    let mut is_async = item.sig.asyncness.is_some();
    let mut block = item.block;

    let new_chat_method = format!("chat_{}", method_name);

    if let Stmt::Expr(expr, _) = block.stmts.last_mut().unwrap() {
        if let Expr::Await(m) = expr {
            if let Expr::MethodCall(m) = m.base.as_mut() {
                let method = &m.method;
                if method == "async_chat" {
                    let ident = Ident::new(&new_chat_method, method.span());
                    m.method = ident;
                }
            }
        }
        if let Expr::MethodCall(m) = expr {
            let method = &m.method;
            if method == "chat" {
                let ident = Ident::new(&new_chat_method, method.span());
                m.method = ident;
                is_async = false;
            }
        }
    }

    // 更新函数体
    item.block = block;

    let new_chat_method_ident = Ident::new(&new_chat_method, proc_macro::Span::call_site().into());

    let new_chat_trait_name_ident = Ident::new(
        &format!("Chat{}", fastrand::u32(..)),
        proc_macro::Span::call_site().into(),
    );

    let client_model = client;
    let model = args.model.clone().unwrap_or_default();
    let temperature = args.temperature.unwrap_or(0.7);
    let max_tokens = args.max_tokens.unwrap_or(1024);
    let functions = args
        .tools
        .as_ref()
        .map(|v| v.iter().map(|v| Ident::new(v.value().as_str(), v.span())))
        .map(|tools|quote! {
            {
                let mut hm = std::collections::HashMap::new();
                #(hm.insert(#tools::key(),(#tools::desc(),#tools::inject as fn(std::collections::HashMap<String, serde_json::Value>) -> String));)*
                hm
            }
        }).unwrap_or(quote! { std::collections::HashMap::new() });
    if is_async {
        let trait_def = quote! {
            trait #new_chat_trait_name_ident {
                async fn #new_chat_method_ident(&self) -> String;
            }
        };
        let impl_def = quote! {
            impl #new_chat_trait_name_ident for Vec<copilot_rs::PromptMessage> {
                async fn #new_chat_method_ident(&self) -> String {
                    let model = #client_model();
                    copilot_rs::async_chat(&model, &self).await
                }
            }
        };
        let expanded = quote! {
            #item

            #trait_def

            #impl_def
        };

        Ok(expanded.into())
    } else {
        let trait_def = quote! {
            trait #new_chat_trait_name_ident {
                fn #new_chat_method_ident(&self) -> String;
            }
        };

        let impl_def = quote! {
            impl #new_chat_trait_name_ident for Vec<copilot_rs::PromptMessage> {
                fn #new_chat_method_ident(&self) -> String {
                    let client = #client_model();
                    let model = #model;
                    let temperature = #temperature;
                    let max_tokens = #max_tokens;
                    let functions = #functions;
                    copilot_rs::chat(&client,&self,model,temperature, max_tokens,functions)
                }
            }
        };

        let expanded = quote! {
            #item

            #trait_def

            #impl_def
        };

        Ok(expanded.into())
    }
}

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(props), forward_attrs(allow, deny))]
struct FunctionToolOptions {
    ident: Ident,
    data: darling::ast::Data<(), FunctionToolProperties>,
    #[darling(default)]
    desc: String,
}

#[derive(Debug, FromField)]
#[darling(attributes(props), forward_attrs(allow, deny))]
struct FunctionToolProperties {
    ident: Option<Ident>,
    ty: syn::Type,
    desc: String,
    #[darling(default)]
    choices: Vec<LitStr>,
}

#[proc_macro_derive(FunctionTool, attributes(props))]
pub fn derive_function_tool(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let parsed = FunctionToolOptions::from_derive_input(&input).unwrap();

    let struct_name = &parsed.ident;
    let struct_desc = parsed.desc;

    let properties = parsed
        .data
        .take_struct()
        .map(|v| v.fields)
        .map(|v| {
            v.iter().fold(HashMap::new(), |mut acc, field| {
                let name = field
                    .ident
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                let ty = match &field.ty {
                    syn::Type::Path(p) => p
                        .path
                        .segments
                        .first()
                        .map(|seg| seg.ident.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    _ => "unknown".to_string(),
                };
                let mut prop = Property::default();
                prop.r#type = ty.to_lowercase();
                prop.description = field.desc.clone();
                prop.choices = if field.choices.is_empty() {
                    None
                } else {
                    Some(field.choices.iter().map(|v| v.value()).collect())
                };
                acc.insert(name, prop);
                acc
            })
        })
        .unwrap_or_default();
    let required = properties
        .iter()
        .filter(|(_k, v)| v.choices.is_none())
        .map(|(k, _v)| k.clone())
        .collect();
    let struct_str = struct_name.to_string();
    let desc_impl = ToolImpl::Function {
        name: struct_str.clone(),
        description: struct_desc,
        parameters: Parameters {
            r#type: default_type(),
            properties,
            required,
        },
    };

    let ret = quote! {
        impl FunctionTool for #struct_name {
            fn key() -> String {
                #struct_str.to_string()
            }
            fn desc() -> ToolImpl {
                #desc_impl
            }
            fn inject(args: std::collections::HashMap<String, serde_json::Value>) -> String {
                let args = serde_json::to_string(&args).unwrap();
                let c : #struct_name = serde_json::from_str(&args).unwrap();
                c.exec()
            }
        }
    };
    ret.into()
}