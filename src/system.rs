//! System testing macro implementation
//!
//! Provides the test_system! macro for testing individual Bevy systems in isolation
//! with minimal boilerplate.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, Expr, Ident, Result, Token};

pub struct SystemTest {
    name: Ident,
    setup: SetupBlock,
    call: SystemCall,
    expect: ExpectBlock,
}

struct SetupBlock {
    resources: Vec<Expr>,
    entities: Vec<EntitySetup>,
    events: Vec<Expr>,
}

struct EntitySetup {
    var_name: Ident,
    components: Expr,
}

struct SystemCall {
    system: Expr,
    params: Option<Expr>,
}

struct ExpectBlock {
    assertions: Vec<Expr>,
}

impl Parse for SystemTest {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;

        let content;
        syn::braced!(content in input);

        // Parse setup block
        let setup_ident: Ident = content.parse()?;
        if setup_ident != "setup" {
            return Err(syn::Error::new(setup_ident.span(), "Expected 'setup'"));
        }
        content.parse::<Token![:]>()?;
        let setup_content;
        syn::braced!(setup_content in content);
        let setup = parse_setup_block(&setup_content)?;
        content.parse::<Token![,]>().ok();

        // Parse call block
        let call_ident: Ident = content.parse()?;
        if call_ident != "call" {
            return Err(syn::Error::new(call_ident.span(), "Expected 'call'"));
        }
        content.parse::<Token![:]>()?;
        let call = parse_system_call(&content)?;
        content.parse::<Token![,]>().ok();

        // Parse expect block
        let expect_ident: Ident = content.parse()?;
        if expect_ident != "expect" {
            return Err(syn::Error::new(expect_ident.span(), "Expected 'expect'"));
        }
        content.parse::<Token![:]>()?;
        let expect_content;
        syn::braced!(expect_content in content);
        let expect = parse_expect_block(&expect_content)?;

        Ok(SystemTest { name, setup, call, expect })
    }
}

fn parse_setup_block(input: ParseStream) -> Result<SetupBlock> {
    let mut resources = Vec::new();
    let mut entities = Vec::new();
    let mut events = Vec::new();

    while !input.is_empty() {
        if input.peek(Ident) {
            let ident: Ident = input.fork().parse()?;

            match ident.to_string().as_str() {
                "resources" => {
                    input.parse::<Ident>()?;
                    input.parse::<Token![:]>()?;
                    let content;
                    syn::bracketed!(content in input);
                    while !content.is_empty() {
                        resources.push(content.parse()?);
                        content.parse::<Token![,]>().ok();
                    }
                }
                "events" => {
                    input.parse::<Ident>()?;
                    input.parse::<Token![:]>()?;
                    let content;
                    syn::bracketed!(content in input);
                    while !content.is_empty() {
                        events.push(content.parse()?);
                        content.parse::<Token![,]>().ok();
                    }
                }
                _ => {
                    // Entity setup: var_name: ComponentBundle { ... }
                    let var_name: Ident = input.parse()?;
                    input.parse::<Token![:]>()?;
                    let components: Expr = input.parse()?;
                    entities.push(EntitySetup { var_name, components });
                }
            }
        }

        input.parse::<Token![,]>().ok();
    }

    Ok(SetupBlock { resources, entities, events })
}

fn parse_system_call(input: ParseStream) -> Result<SystemCall> {
    let system = input.parse()?;

    let params = if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
        if !input.is_empty() {
            Some(input.parse()?)
        } else {
            None
        }
    } else {
        None
    };

    Ok(SystemCall { system, params })
}

fn parse_expect_block(input: ParseStream) -> Result<ExpectBlock> {
    let mut assertions = Vec::new();

    while !input.is_empty() {
        assertions.push(input.parse()?);
        input.parse::<Token![,]>().ok();
    }

    Ok(ExpectBlock { assertions })
}

impl SystemTest {
    pub fn expand(&self) -> TokenStream {
        let test_name = &self.name;
        let setup = self.generate_setup();
        let system_call = self.generate_system_call();
        let assertions = self.generate_assertions();

        quote! {
            #[test]
            fn #test_name() {
                use bevy::prelude::*;

                // Create test app with minimal setup
                let mut app = App::new();
                app.add_plugins(MinimalPlugins);

                // Setup phase
                #setup

                // Add and run the system under test
                #system_call

                // Run one update cycle
                app.update();

                // Assertions
                #assertions
            }
        }
    }

    fn generate_setup(&self) -> TokenStream {
        let mut setup = TokenStream::new();

        // Add resources
        for resource in &self.setup.resources {
            setup.extend(quote! {
                app.insert_resource(#resource);
            });
        }

        // Add events
        for event in &self.setup.events {
            setup.extend(quote! {
                app.add_event::<#event>();
            });
        }

        // Spawn entities
        for entity in &self.setup.entities {
            let var_name = &entity.var_name;
            let components = &entity.components;

            setup.extend(quote! {
                let #var_name = app.world_mut().spawn(#components).id();
            });
        }

        setup
    }

    fn generate_system_call(&self) -> TokenStream {
        let system = &self.call.system;

        if let Some(params) = &self.call.params {
            quote! {
                app.add_systems(Update, #system.pipe(#params));
            }
        } else {
            quote! {
                app.add_systems(Update, #system);
            }
        }
    }

    fn generate_assertions(&self) -> TokenStream {
        let mut assertions = TokenStream::new();

        for assertion in &self.expect.assertions {
            assertions.extend(quote! {
                assert!(#assertion, "Assertion failed: {}", stringify!(#assertion));
            });
        }

        assertions
    }
}