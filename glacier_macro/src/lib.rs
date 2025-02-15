// #![allow(unused)]

extern crate proc_macro;

use std::sync::{LazyLock, Mutex};

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::parse_quote;
use syn::token::Comma;
use syn::Stmt;

// #[glacier(GET, "/")]
struct Route {
    method: syn::Ident,
    path: syn::LitStr,
}

impl Parse for Route {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method = input.parse()?;
        let _comma: Comma = input.parse()?;
        let path = input.parse()?;

        Ok(Route { method, path })
    }
}

//////////////////////////////

static STMTS: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| {
    let mut stmts = Vec::new();

    let first_stmt: Stmt = parse_quote! {
        let mut routes = Routes::new();
    };

    let first_stmt = first_stmt.to_token_stream().to_string();
    stmts.push(first_stmt);

    Mutex::new(stmts)
});
//////////////////////////////

#[proc_macro_attribute]
pub fn glacier(args: TokenStream, input: TokenStream) -> TokenStream {
    // 解析函数声明
    let ast = syn::parse_macro_input!(input as syn::ItemFn);
    let args = syn::parse_macro_input!(args as Route);

    gen_glacier(ast, args)
}

fn gen_glacier(ast: syn::ItemFn, args: Route) -> TokenStream {
    // 原函数的 ast 结构
    let func_name = ast.sig.ident;
    let func_inputs = ast.sig.inputs;
    let func_body_stmts = ast.block.stmts;

    // 宏标记接收到的参数
    let _method = args.method;
    let path = args.path;

    // 临时变量用于 insert
    let stmt: Stmt = parse_quote! {
        routes.add(#path, #func_name);
    };
    let stmt = stmt.to_token_stream().to_string();
    STMTS.lock().unwrap().push(stmt);

    // 转换后的函数
    let gen = quote! {

        fn #func_name (#func_inputs)
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
    let func_name = &ast.sig.ident;
    let func_inputs = ast.sig.inputs;
    let func_body_stmts = ast.block.stmts;

    // 处理临时变量

    let stmts = STMTS.lock().unwrap().clone();
    let stmts: Vec<Stmt> = stmts
        .into_iter()
        .map(|stmt| {
            let stmt = syn::parse_str(&stmt).unwrap();
            stmt
        })
        .collect();

    // 转换后的函数
    let gen = quote! {

        fn #func_name (#func_inputs)
        {
            # (#stmts) *

            # (#func_body_stmts) *
        }
    };

    gen.into()
}
