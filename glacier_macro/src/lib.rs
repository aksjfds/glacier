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
static mut CONTAIN_PATH: Vec<String> = Vec::new();

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
    let middles = args.middles;

    /* ------------------------------ // contain_uri 分支 ------------------------------ */
    let arm: syn::Arm = parse_quote! {
        #path => true,
    };
    let arm = arm.to_token_stream().to_string();
    unsafe { CONTAIN_PATH.push(arm) };

    /* ------------------------------ // match1 分支 ------------------------------ */
    let middles = middles.map(|middles| {
        middles
            .elems
            .into_iter()
            .map(|expr| match expr {
                syn::Expr::Call(mut call) => {
                    call.args.insert(0, parse_quote!(req));
                    syn::Expr::Call(call)
                }
                _ => parse_quote!( #expr(req) ),
            })
            .collect::<Vec<_>>()
    });

    let arm: syn::Arm = match middles {
        Some(middles) => parse_quote! {
            #path => {
                # ( req = #middles.await?; )*
                # (#func_body_stmts) *
            }
        },
        None => parse_quote! {
            #path => {
                # (#func_body_stmts) *
            }
        },
    };

    let arm = arm.into_token_stream().to_string();
    unsafe { ARMS.push(arm) };

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

fn gen_main(mut ast: syn::ItemFn) -> TokenStream {
    /* ------------------------------ // contain_uri 分支 ------------------------------ */
    let arms = unsafe { CONTAIN_PATH.clone() };
    let mut arms: Vec<Arm> = arms
        .into_iter()
        .map(|stmt| {
            let stmt = syn::parse_str(&stmt).unwrap();
            stmt
        })
        .collect();
    arms.push(parse_quote! {
        _ => false,
    });
    let mut match_expr: syn::ExprMatch = parse_quote! {
        match path {}
    };
    match_expr.arms = arms;
    let contain_path: syn::ItemFn = parse_quote! {
        fn contain_path(path: &str) -> bool {
            #match_expr
        }
    };

    ast.block.stmts.insert(
        0,
        parse_quote! {
            unsafe { CONTAIN_PATH = contain_path; }
        },
    );

    /* ------------------------------ // 处理 match1 ------------------------------ */
    let arms = unsafe { ARMS.clone() };
    let mut arms: Vec<Arm> = arms
        .into_iter()
        .map(|stmt| syn::parse_str(&stmt).unwrap())
        .collect();

    let _arm = parse_quote! {
        _ => {
            if unsafe { !DIR_PATH.is_empty() } {
                let pos = req.path_for_routes().rfind("/").unwrap_or(0);
                let req_dir_path = &req.path_for_routes()[..pos];
                if req_dir_path != unsafe { DIR_PATH } {
                    req.respond_404().await?;
                    return Ok(req);
                }

                let file_path = String::from(&req.path_for_routes()[1..]);
                if let Err(e) = req.respond_buf(file_path).await {
                    req.respond_404().await?;
                }
            }

        }
    };
    arms.push(_arm);

    let routes_func: syn::ItemFn = parse_quote! {
        async fn routes(mut req: OneRequest) -> Result<OneRequest> {
            let path = req.path_for_routes();

            match path {
                # ( #arms ) *
            }

            Ok(req)
        }
    };

    // 转换后的函数
    let gen = quote! {

        #contain_path
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
