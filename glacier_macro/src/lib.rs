#![allow(unused)]
#![allow(static_mut_refs)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::token::Comma;
use syn::{parse_quote, Arm};

// #[glacier(GET, "/")]
struct RouteArgs {
    method: syn::Ident,
    path: syn::LitStr,
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method = input.parse()?;
        let _comma: Comma = input.parse()?;
        let path = input.parse()?;

        Ok(RouteArgs { method, path })
    }
}

//////////////////////////////

static mut STMTS: Vec<String> = Vec::new();
//////////////////////////////

#[proc_macro_attribute]
pub fn glacier(args: TokenStream, input: TokenStream) -> TokenStream {
    // 解析函数声明
    let ast = syn::parse_macro_input!(input as syn::ItemFn);
    let args = syn::parse_macro_input!(args as RouteArgs);

    gen_glacier(ast, args)
}

fn gen_glacier(ast: syn::ItemFn, args: RouteArgs) -> TokenStream {
    // 原函数的 ast 结构
    let func_async = ast.sig.asyncness.expect("no async signature");
    let func_name = ast.sig.ident;
    let func_inputs = ast.sig.inputs;
    let func_body_stmts = ast.block.stmts;

    // 宏标记接收到的参数
    let method = args.method;
    let path = args.path;

    /* ------------------------------ // match1 分支 ------------------------------ */
    let mut arm: syn::Arm = parse_quote! {
        #path => {
                # (#func_body_stmts) *
        }
    };

    /* ------------------------------ // Static 处理 ------------------------------ */
    if "Static" == method.to_string() {
        arm = parse_quote! {
            _ => {
                let pos = req.path().rfind("/").unwrap_or(0);
                let dir_path = &req.path()[..pos];
                if dir_path != #path { return Ok(req) }

                let file_path = String::from(&req.path()[1..]);
                req.respond(file_path).await?;
            }
        };
    }

    let arm = arm.into_token_stream().to_string();
    unsafe { STMTS.push(arm) };

    // 转换后的函数
    let gen = quote! {

        #func_async fn #func_name (#func_inputs) -> Result<OneRequest>
        {
            # (#func_body_stmts) *

            Ok(req)
        }

    };
    gen.into()
}

#[proc_macro_attribute]
pub fn main(_args: TokenStream, input: TokenStream) -> TokenStream {
    // 解析函数声明
    let ast = syn::parse_macro_input!(input as syn::ItemFn);

    gen_main(ast)
}

fn gen_main(ast: syn::ItemFn) -> TokenStream {
    /* ------------------------------ // 处理 match1 ------------------------------ */
    let arms = unsafe { STMTS.clone() };
    let arms: Vec<Arm> = arms
        .into_iter()
        .map(|stmt| {
            let stmt = syn::parse_str(&stmt).unwrap();
            stmt
        })
        .collect();

    let mut match_expr: syn::ExprMatch = parse_quote! {
        match path {}
    };
    match_expr.arms = arms;
    let routes_func: syn::ItemFn = parse_quote! {
        async fn routes(mut req: OneRequest) -> Result<OneRequest> {
            let path = req.path();
            #match_expr

            Ok(req)
        }
    };

    // 转换后的函数
    let gen = quote! {

        #routes_func

        #[tokio::main]
        #ast
    };

    gen.into()
}
