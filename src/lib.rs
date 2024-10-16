extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Literal, TokenTree};
use quote::quote;
use regex::Regex;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Brace,
    Expr, Ident, LitInt, LitStr, Result,
};

struct SqlInput {
    parts: Vec<SqlPart>,
}

enum SqlPart {
    Keyword(String),
    Interpolation(Expr),
    StringLiteral(String),
    Placeholder(String),
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
            } else if input.peek(syn::token::Dollar) {
                // Handle $ placeholders
                let _: syn::token::Dollar = input.parse()?;
                if input.peek(LitInt) {
                    let lit_int: LitInt = input.parse()?;
                    let placeholder = format!("${}", lit_int.base10_digits());
                    parts.push(SqlPart::Placeholder(placeholder));
                } else {
                    // If $ is not followed by an integer, treat it as a punct
                    parts.push(SqlPart::Punct('$'));
                }
            } else {
                let tt = input.parse::<TokenTree>()?;
                match tt {
                    TokenTree::Punct(punct) => {
                        parts.push(SqlPart::Punct(punct.as_char()));
                    }
                    TokenTree::Literal(literal) => {
                        let s = literal.to_string();
                        if s.starts_with('"') && s.ends_with('"') {
                            // String literal
                            let s = s[1..s.len() - 1].to_string();
                            parts.push(SqlPart::StringLiteral(s));
                        } else {
                            // Other literals (e.g., numbers)
                            parts.push(SqlPart::Other(s));
                        }
                    }
                    TokenTree::Ident(ident) => {
                        parts.push(SqlPart::Keyword(ident.to_string()));
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

    let no_space_before = vec!['.', ',', ';'];
    let no_space_after = vec!['.', '$', ';'];

    let mut last_char: Option<char> = None;

    for part in parts {
        let (current_token_str, parsing_token_str, curr_first_char, curr_last_char) = match &part {
            SqlPart::Keyword(s) => (s.clone(), s.clone(), s.chars().next(), s.chars().last()),
            SqlPart::Interpolation(_) => (
                "{}".to_string(),
                "dummy_value".to_string(),
                Some('{'),
                Some('}'),
            ),
            SqlPart::StringLiteral(s) => {
                let token_str = format!("\"{}\"", s);
                let parsing_str = format!("'{}'", s);
                (token_str, parsing_str, Some('"'), Some('"'))
            }
            SqlPart::Placeholder(s) => (
                s.clone(),
                "'dummy_value'".to_string(),
                s.chars().next(),
                s.chars().last(),
            ),
            SqlPart::Punct(ch) => (ch.to_string(), ch.to_string(), Some(*ch), Some(*ch)),
            SqlPart::Other(s) => (s.clone(), s.clone(), s.chars().next(), s.chars().last()),
        };

        // Determine if we need to insert a space before the current token
        if let Some(prev_char) = last_char {
            if !prev_char.is_whitespace() && !no_space_after.contains(&prev_char) {
                if let Some(curr_char) = curr_first_char {
                    if !curr_char.is_whitespace() && !no_space_before.contains(&curr_char) {
                        query_string.push(' ');
                        query_string_for_parsing.push(' ');
                    }
                }
            }
        }

        // Append the current token to the query strings
        query_string.push_str(&current_token_str);
        query_string_for_parsing.push_str(&parsing_token_str);

        // Update last_char
        last_char = curr_last_char;

        // Handle interpolations
        if let SqlPart::Interpolation(expr) = part {
            interpolations.push(expr);
        }
    }

    let mut query_string = query_string.trim().to_string();
    let query_string_for_parsing = query_string_for_parsing.trim().to_string();

    // Replace double quotes with single quotes in the final query string
    query_string = query_string.replace("\"", "'");

    // Use regex to replace $ placeholders with a valid SQL literal
    let re = Regex::new(r"\$\s\d+").unwrap();
    let query_string_for_parsing_replaced = re
        .replace_all(&query_string_for_parsing, |_: &regex::Captures| {
            "'dummy_value'".to_string()
        });

    let dialect = GenericDialect {};
    match Parser::parse_sql(&dialect, &query_string_for_parsing_replaced) {
        Ok(_) => {}
        Err(e) => {
            return syn::Error::new_spanned(
                Literal::string(&query_string),
                format!(
                    "Erro de sintaxe na query SQL: {}\n\n{}",
                    e, query_string_for_parsing_replaced
                ),
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
