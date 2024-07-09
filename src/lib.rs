extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, token, Expr, Ident,
};

struct SqlInput {
    parts: Vec<SqlPart>,
}

enum SqlPart {
    Keyword(Ident),
    Interpolation(Expr),
    Other(TokenTree),
}

impl Parse for SqlInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parts = Vec::new();

        while !input.is_empty() {
            if input.peek(token::Brace) {
                let content;
                let _ = syn::braced!(content in input);
                let expr: Expr = content.parse()?;
                parts.push(SqlPart::Interpolation(expr));
            } else if input.peek(Ident) {
                let ident: Ident = input.parse()?;
                parts.push(SqlPart::Keyword(ident));
            } else {
                let token_tree: TokenTree = input.parse()?;
                parts.push(SqlPart::Other(token_tree));
            }
        }

        Ok(SqlInput { parts })
    }
}

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let SqlInput { parts } = parse_macro_input!(input as SqlInput);

    let mut query_string = String::new();
    let mut interpolations = TokenStream2::new();

    for part in parts {
        match part {
            SqlPart::Keyword(ident) => {
                query_string.push_str(&format!("{} ", ident));
            }
            SqlPart::Interpolation(expr) => {
                query_string.push_str("{} ");
                interpolations.extend(quote! { , #expr });
            }
            SqlPart::Other(token) => {
                query_string.push_str(&format!("{} ", token));
            }
        }
    }

    let query_literal = TokenTree::from(proc_macro2::Literal::string(&query_string.trim()));

    let expanded = quote! {
        {
            let query = format!(#query_literal #interpolations);
            let query = query.replace("{", "").replace("}", "");
            query
        }
    };

    TokenStream::from(expanded)
}
