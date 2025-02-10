// #![allow(unused)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    token::Comma,
    Block, Stmt,
};

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

#[proc_macro_attribute]
pub fn glacier(args: TokenStream, input: TokenStream) -> TokenStream {
    // 解析函数声明
    let ast = syn::parse_macro_input!(input as syn::ItemFn);
    let args = syn::parse_macro_input!(args as Route);

    gen_glacier(ast, args)
}

static mut GLACIER_ID: u8 = 0;
fn gen_glacier(ast: syn::ItemFn, args: Route) -> TokenStream {
    // 原函数的 ast 结构
    let func_name = ast.sig.ident;
    let func_inputs = ast.sig.inputs;
    let func_body_stmts = ast.block.stmts;

    // 宏标记接收到的参数
    let method = args.method;
    let method = format_ident!("GLACIER_{}", method);
    let path = args.path;

    // 临时变量用于 insert
    let id = unsafe { GLACIER_ID };
    unsafe { GLACIER_ID += 1 };
    let glacier_temp = format_ident!("GLACIER_TEMP_{}", id);

    // 转换后的函数
    let gen = quote! {

        static #glacier_temp: LazyLock<u8> = LazyLock::new(|| {
            let lock = unsafe { #method.lock() };
            let mut lock = lock.unwrap();
            lock.insert(#path, #func_name);
            1
        });

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
    let mut block: Block = parse_quote! {
        {}
    };
    let ids = unsafe { GLACIER_ID };
    for id in 0..ids {
        let ident = format_ident!("GLACIER_TEMP_{}", id);
        let stmt: Stmt = parse_quote! {
            #ident.add(0);
        };
        block.stmts.push(stmt);
    }

    // 转换后的函数
    let gen = quote! {

        static GLACIER_GET: LazyLock<Mutex<HashMap<&str, fn(Request<'_>)>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

        fn #func_name (#func_inputs)
        {
            #block

            # (#func_body_stmts) *
        }
    };

    gen.into()
}
