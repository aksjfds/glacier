#![allow(static_mut_refs)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Stmt};

#[allow(unused)]
struct RouteArgs {
    param: Option<syn::Ident>,
    body: Option<syn::Ident>,
}

impl Parse for RouteArgs {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

#[proc_macro]
pub fn glacier(input: TokenStream) -> TokenStream {
    // 解析函数声明
    let ast = syn::parse_macro_input!(input as syn::ExprArray);

    gen_glacier(ast)
}

fn gen_glacier(ast: syn::ExprArray) -> TokenStream {
    // 原函数的 ast 结构
    let mut func_array: Vec<_> = ast.elems.into_iter().collect();
    let func = func_array.pop().expect("func_array is none");

    let arm_block = if let _len @ 1.. = func_array.len() {
        let mut arm_block: syn::ExprAsync = parse_quote!(async {});
        let mut stmts: Vec<Stmt> = func_array
            .into_iter()
            .map(|middle| match middle {
                syn::Expr::Call(middle) => {
                    let args: Vec<_> = middle.args.into_iter().collect();
                    let middle = middle.func;
                    parse_quote! {
                            req = #middle(req,
                                #( #args, ) *
                            ).await?;
                    }
                }
                syn::Expr::Path(middle) => parse_quote! {
                    req = #middle(req).await?;
                },
                _ => todo!(),
            })
            .collect();

        stmts.push(syn::Stmt::Expr(
            parse_quote! {
                #func(req).await
            },
            None,
        ));

        arm_block.block.stmts = stmts;
        arm_block
    } else {
        parse_quote! {
                async { #func(req).await }
        }
    };

    // 转换后的函数
    let gen = quote! {
        #arm_block.await
    };
    gen.into()
}
