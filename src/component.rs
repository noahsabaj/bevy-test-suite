//! Component testing macro implementation
//!
//! Provides the test_component! macro for testing component state transitions
//! and behavior with minimal boilerplate.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, Expr, Ident, Result, Token};

pub struct ComponentTest {
    name: Ident,
    given: Expr,
    operations: Vec<Operation>,
}

struct Operation {
    method: Ident,
    args: Option<Expr>,
    expected_state: Expr,
}

impl Parse for ComponentTest {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;

        let content;
        syn::braced!(content in input);

        // Parse given
        let given_ident: Ident = content.parse()?;
        if given_ident != "given" {
            return Err(syn::Error::new(given_ident.span(), "Expected 'given'"));
        }
        content.parse::<Token![:]>()?;
        let given = content.parse()?;
        content.parse::<Token![,]>().ok();

        // Parse operations
        let ident: Ident = content.parse()?;
        if ident != "operations" {
            return Err(syn::Error::new(ident.span(), "Expected 'operations'"));
        }
        content.parse::<Token![:]>()?;

        let operations_content;
        syn::bracketed!(operations_content in content);
        let operations = parse_operations(&operations_content)?;

        Ok(ComponentTest { name, given, operations })
    }
}

fn parse_operations(input: ParseStream) -> Result<Vec<Operation>> {
    let mut operations = Vec::new();

    while !input.is_empty() {
        // Parse method call
        let method: Ident = input.parse()?;

        // Parse optional arguments
        let args = if input.peek(syn::token::Paren) {
            let args_content;
            syn::parenthesized!(args_content in input);
            Some(args_content.parse()?)
        } else {
            None
        };

        // Parse arrow =>
        input.parse::<Token![=>]>()?;

        // Parse expected state
        let expected_state: Expr = input.parse()?;

        operations.push(Operation { method, args, expected_state });

        input.parse::<Token![,]>().ok();
    }

    Ok(operations)
}

impl ComponentTest {
    pub fn expand(&self) -> TokenStream {
        let test_name = &self.name;
        let initial_state = &self.given;
        let test_steps = self.generate_test_steps();

        quote! {
            #[test]
            fn #test_name() {
                use bevy::prelude::*;

                // Create the component with initial state
                let mut component = #initial_state;

                // Execute operations and verify state transitions
                #test_steps
            }
        }
    }

    fn generate_test_steps(&self) -> TokenStream {
        let mut steps = TokenStream::new();

        for (i, op) in self.operations.iter().enumerate() {
            let method = &op.method;
            let expected = &op.expected_state;
            let step_num = i + 1;

            let method_call = if let Some(args) = &op.args {
                quote! { component.#method(#args) }
            } else {
                quote! { component.#method() }
            };

            steps.extend(quote! {
                // Operation #step_num
                #method_call;
                assert_eq!(
                    component,
                    #expected,
                    "After operation {}: component state mismatch",
                    #step_num
                );
            });
        }

        steps
    }
}