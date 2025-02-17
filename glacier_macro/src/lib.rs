#![allow(unused)]

extern crate proc_macro;

use std::sync::{LazyLock, Mutex};

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::token::Comma;
use syn::Stmt;
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

static STMTS: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| {
    let stmts = Vec::new();

    Mutex::new(stmts)
});
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
    let _method = args.method;
    let path = args.path;

    // match 分支
    let arm: syn::Arm = parse_quote! {
        #path => {
            // println!("{:#?}", #path);
            let task = async {
                # (#func_body_stmts) *
            };
            task.await;
        }
    };
    let arm = arm.into_token_stream().to_string();
    STMTS.lock().unwrap().push(arm);

    // 转换后的函数
    let gen = quote! {

        #func_async fn #func_name (#func_inputs)
        {
            # (#func_body_stmts) *
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
    // 原函数的 ast 结构
    let func_async = &ast.sig.asyncness.unwrap();
    let func_name = &ast.sig.ident;
    let func_inputs = ast.sig.inputs;
    let func_body_stmts = ast.block.stmts;

    // 处理 match
    let arms = STMTS.lock().unwrap().clone();
    let mut arms: Vec<Arm> = arms
        .into_iter()
        .map(|stmt| {
            let stmt = syn::parse_str(&stmt).unwrap();
            stmt
        })
        .collect();
    arms.push(parse_quote! {
        _ => {
            // println!("{:#?}", "_");
            let mut not_found = Response::from(res).await;
            not_found.respond().await;
        }
    });

    let mut match_expr: syn::ExprMatch = parse_quote! {
        match path {}
    };
    match_expr.arms = arms;
    let routes_func: syn::ItemFn = parse_quote! {
        async fn routes(mut req: Request, mut res: Response) {
            let path = req.path();
            #match_expr
        }
    };

    // 转换后的函数
    let gen = quote! {

        #routes_func

        #[tokio::main]
        #func_async fn #func_name (#func_inputs)
        {
            # (#func_body_stmts) *
        }
    };

    gen.into()
}
