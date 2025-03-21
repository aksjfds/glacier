#![allow(static_mut_refs)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::token::Comma;
use syn::{parse_quote, Arm};

#[allow(unused)]
struct RouteArgs {
    method: syn::Ident,
    path: syn::LitStr,
    middles: Option<syn::ExprArray>,
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method = input.parse()?;
        let _comma: Comma = input.parse()?;
        let path = input.parse()?;

        let comma: Result<Comma, syn::Error> = input.parse();
        if comma.is_err() {
            Ok(RouteArgs {
                method,
                path,
                middles: None,
            })
        } else {
            let middles: syn::ExprArray = input.parse()?;

            Ok(RouteArgs {
                method,
                path,
                middles: Some(middles),
            })
        }
    }
}

//////////////////////////////

static mut ARMS: Vec<String> = Vec::new();

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
    let func_name = &ast.sig.ident;

    // 宏标记接收到的参数
    // let method = args.method;
    let path = args.path;
    // let middles = args.middles;

    /* ------------------------------ // match1 分支 ------------------------------ */
    let arm: syn::Arm = parse_quote! {
        #path => #func_name(req).await,
    };

    let arm = arm.into_token_stream().to_string();
    unsafe { ARMS.push(arm) };

    // 转换后的函数
    let gen = quote! {

        #ast

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
    let arms = unsafe { ARMS.clone() };
    let mut arms: Vec<Arm> = arms
        .iter()
        .map(AsRef::as_ref)
        .map(syn::parse_str)
        .collect::<Result<_, _>>()
        .unwrap();

    let _arm = parse_quote! {
        _ => handle_404(req).await,
    };
    arms.push(_arm);

    let routes_func: syn::ItemFn = parse_quote! {
        async fn routes(
            mut req: Request<RecvStream>,
        ) -> HttpResponse {

            let path = req.uri().path();

            let res = match path {
                # ( #arms ) *
            };

            handle_error(res).await
        }
    };

    // 转换后的函数
    let gen = quote! {

        /// 由宏生成的函数, 每个请求都会进入这个函数, 通过`req.path()`进入不同的match分支
        /// # Examples
        /// ```
        /// async fn routes(mut req: OneRequest) -> Result<OneRequest> {
        ///    let path = req.path();
        ///    match path {
        ///        "/" => {
        ///            req.respond_hello().await;
        ///        }
        ///        ...
        ///    }
        ///    Ok(req)
        ///}
        /// ```
        #routes_func

        #[tokio::main]
        #ast
    };

    gen.into()
}
