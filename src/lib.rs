extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Literal, TokenTree};
use quote::quote;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Brace,
    Expr, Ident, LitStr, Result,
};

struct SqlInput {
    parts: Vec<SqlPart>,
}

enum SqlPart {
    Keyword(String),
    Interpolation(Expr),
    StringLiteral(String),
    Punct(char),
    Other(String),
}

impl Parse for SqlInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut parts = Vec::new();

        while !input.is_empty() {
            if input.peek(Brace) {
                let content;
                let _ = syn::braced!(content in input);
                let expr: Expr = content.parse()?;
                parts.push(SqlPart::Interpolation(expr));
            } else if input.peek(Ident) {
                let ident: Ident = input.parse()?;
                parts.push(SqlPart::Keyword(ident.to_string()));
            } else if input.peek(LitStr) {
                let lit_str: LitStr = input.parse()?;
                parts.push(SqlPart::StringLiteral(lit_str.value()));
            } else {
                let tt = input.parse::<TokenTree>()?;
                match tt {
                    TokenTree::Punct(punct) => {
                        parts.push(SqlPart::Punct(punct.as_char()));
                    }
                    _ => {
                        parts.push(SqlPart::Other(tt.to_string()));
                    }
                }
            }
        }

        Ok(SqlInput { parts })
    }
}

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let SqlInput { parts } = parse_macro_input!(input as SqlInput);

    let mut query_string = String::new();
    let mut query_string_for_parsing = String::new();
    let mut interpolations = Vec::new();

    let no_space_before = vec!['.', ',', ')', ';'];
    let _no_space_after = vec!['.', ',', '(', ';'];

    let mut last_char: Option<char> = None;

    for part in parts {
        match part {
            SqlPart::Keyword(s) => {
                if let Some(c) = last_char {
                    if !c.is_whitespace() && !no_space_before.contains(&c) {
                        query_string.push(' ');
                        query_string_for_parsing.push(' ');
                    }
                }
                query_string.push_str(&s);
                query_string_for_parsing.push_str(&s);
                last_char = s.chars().last();
            }
            SqlPart::Interpolation(expr) => {
                if let Some(c) = last_char {
                    if !c.is_whitespace() && !no_space_before.contains(&c) {
                        query_string.push(' ');
                        query_string_for_parsing.push(' ');
                    }
                }
                query_string.push_str("{}");
                interpolations.push(expr);

                query_string_for_parsing.push_str("dummy_value");

                last_char = Some('}');
            }
            SqlPart::StringLiteral(s) => {
                if let Some(c) = last_char {
                    if !c.is_whitespace() && !no_space_before.contains(&c) {
                        query_string.push(' ');
                        query_string_for_parsing.push(' ');
                    }
                }
                query_string.push('"');
                query_string.push_str(&s);
                query_string.push('"');

                query_string_for_parsing.push('\'');
                query_string_for_parsing.push_str(&s);
                query_string_for_parsing.push('\'');

                last_char = Some('"');
            }
            SqlPart::Punct(ch) => {
                if !no_space_before.contains(&ch) {
                    if let Some(c) = last_char {
                        if !c.is_whitespace() && !no_space_before.contains(&c) {
                            query_string.push(' ');
                            query_string_for_parsing.push(' ');
                        }
                    }
                }
                query_string.push(ch);
                query_string_for_parsing.push(ch);

                last_char = Some(ch);
            }
            SqlPart::Other(s) => {
                if let Some(c) = last_char {
                    if !c.is_whitespace() && !no_space_before.contains(&c) {
                        query_string.push(' ');
                        query_string_for_parsing.push(' ');
                    }
                }
                query_string.push_str(&s);
                query_string_for_parsing.push_str(&s);

                last_char = s.chars().last();
            }
        }
    }

    let mut query_string = query_string.trim().to_string();
    let query_string_for_parsing = query_string_for_parsing.trim().to_string();

    query_string = query_string.replace("\"", "'");

    let dialect = GenericDialect {};
    match Parser::parse_sql(&dialect, &query_string_for_parsing) {
        Ok(_) => {}
        Err(e) => {
            return syn::Error::new_spanned(
                Literal::string(&query_string),
                format!("Erro de sintaxe na query SQL: {}", e),
            )
            .to_compile_error()
            .into();
        }
    }

    let query_literal = Literal::string(&query_string);

    let expanded = if interpolations.is_empty() {
        quote! {
            #query_literal.to_string()
        }
    } else {
        quote! {
            format!(#query_literal, #(#interpolations),*)
        }
    };

    TokenStream::from(expanded)
}
