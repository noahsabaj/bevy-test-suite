//! Test scenario macro implementation
//!
//! Provides the core test_scenario! macro that creates complete test scenarios
//! with given/when/then structure.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, Expr, Ident, Result, Token};

pub struct TestScenario {
    name: Ident,
    given: GivenClause,
    when: WhenClause,
    then: ThenClause,
}

struct GivenClause {
    resources: Vec<Expr>,
    events: Vec<Expr>,
    systems: Vec<Expr>,
    entities: Vec<EntityDef>,
}

struct WhenClause {
    actions: Vec<Action>,
}

struct ThenClause {
    assertions: Vec<Assertion>,
}

struct EntityDef {
    type_name: Ident,
    fields: Vec<(Ident, Expr)>,
}

enum Action {
    Event(Expr),
    Advance(TimeAdvance),
    Input(Expr),
}

enum TimeAdvance {
    Frames(u32),
    Seconds(f32),
    Days(u32),
}

enum Assertion {
    ComponentCheck(Expr),
    EventsReceived(Vec<Ident>),
    Snapshot(Expr),
}

impl Parse for TestScenario {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;

        let content;
        syn::braced!(content in input);

        // Parse given clause
        let given_ident: Ident = content.parse()?;
        if given_ident != "given" {
            return Err(syn::Error::new(given_ident.span(), "Expected 'given'"));
        }
        content.parse::<Token![:]>()?;
        let given_content;
        syn::braced!(given_content in content);
        let given = parse_given_clause(&given_content)?;
        content.parse::<Token![,]>().ok();

        // Parse when clause
        let when_ident: Ident = content.parse()?;
        if when_ident != "when" {
            return Err(syn::Error::new(when_ident.span(), "Expected 'when'"));
        }
        content.parse::<Token![:]>()?;
        let when_content;
        syn::braced!(when_content in content);
        let when = parse_when_clause(&when_content)?;
        content.parse::<Token![,]>().ok();

        // Parse then clause
        let then_ident: Ident = content.parse()?;
        if then_ident != "then" {
            return Err(syn::Error::new(then_ident.span(), "Expected 'then'"));
        }
        content.parse::<Token![:]>()?;
        let then_content;
        syn::braced!(then_content in content);
        let then = parse_then_clause(&then_content)?;

        Ok(TestScenario {
            name,
            given,
            when,
            then,
        })
    }
}

fn parse_given_clause(input: ParseStream) -> Result<GivenClause> {
    let mut resources = Vec::new();
    let mut events = Vec::new();
    let mut systems = Vec::new();
    let mut entities = Vec::new();

    while !input.is_empty() {
        let field_name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        match field_name.to_string().as_str() {
            "resources" => {
                let content;
                syn::bracketed!(content in input);
                while !content.is_empty() {
                    resources.push(content.parse()?);
                    content.parse::<Token![,]>().ok();
                }
            }
            "events" => {
                let content;
                syn::bracketed!(content in input);
                while !content.is_empty() {
                    events.push(content.parse()?);
                    content.parse::<Token![,]>().ok();
                }
            }
            "systems" => {
                let content;
                syn::bracketed!(content in input);
                while !content.is_empty() {
                    systems.push(content.parse()?);
                    content.parse::<Token![,]>().ok();
                }
            }
            "entities" => {
                let content;
                syn::bracketed!(content in input);
                while !content.is_empty() {
                    let type_name = content.parse()?;
                    let fields_content;
                    syn::braced!(fields_content in content);

                    let mut fields = Vec::new();
                    while !fields_content.is_empty() {
                        let field_name = fields_content.parse()?;
                        fields_content.parse::<Token![:]>()?;
                        let field_value = fields_content.parse()?;
                        fields.push((field_name, field_value));
                        fields_content.parse::<Token![,]>().ok();
                    }

                    entities.push(EntityDef { type_name, fields });
                    content.parse::<Token![,]>().ok();
                }
            }
            _ => return Err(syn::Error::new(field_name.span(), "Unknown given field")),
        }

        input.parse::<Token![,]>().ok();
    }

    Ok(GivenClause {
        resources,
        events,
        systems,
        entities,
    })
}

fn parse_when_clause(input: ParseStream) -> Result<WhenClause> {
    let mut actions = Vec::new();

    while !input.is_empty() {
        let action_type: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        match action_type.to_string().as_str() {
            "event" => {
                let event_expr = input.parse()?;
                actions.push(Action::Event(event_expr));
            }
            "advance" => {
                // Parse time advance expression properly
                let advance_expr: Expr = input.parse()?;

                // Analyze the expression to determine time type
                let time_advance = parse_time_advance(&advance_expr);
                actions.push(Action::Advance(time_advance));
            }
            "input" => {
                let input_expr = input.parse()?;
                actions.push(Action::Input(input_expr));
            }
            _ => return Err(syn::Error::new(action_type.span(), "Unknown when action")),
        }

        input.parse::<Token![,]>().ok();
    }

    Ok(WhenClause { actions })
}

fn parse_time_advance(expr: &Expr) -> TimeAdvance {
    // Parse expressions like "1.second()", "10.frames()", "5.days()"
    match expr {
        Expr::MethodCall(method_call) => match method_call.method.to_string().as_str() {
            "frames" | "frame" => {
                if let Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(n),
                    ..
                }) = &*method_call.receiver
                {
                    TimeAdvance::Frames(n.base10_parse().unwrap_or(1))
                } else {
                    TimeAdvance::Frames(1)
                }
            }
            "seconds" | "second" => {
                if let Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Float(f),
                    ..
                }) = &*method_call.receiver
                {
                    TimeAdvance::Seconds(f.base10_parse().unwrap_or(1.0))
                } else if let Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(n),
                    ..
                }) = &*method_call.receiver
                {
                    TimeAdvance::Seconds(n.base10_parse::<f32>().unwrap_or(1.0))
                } else {
                    TimeAdvance::Seconds(1.0)
                }
            }
            "days" | "day" => {
                if let Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(n),
                    ..
                }) = &*method_call.receiver
                {
                    TimeAdvance::Days(n.base10_parse().unwrap_or(1))
                } else {
                    TimeAdvance::Days(1)
                }
            }
            _ => TimeAdvance::Frames(1),
        },
        _ => TimeAdvance::Frames(1),
    }
}

fn parse_then_clause(input: ParseStream) -> Result<ThenClause> {
    let mut assertions = Vec::new();

    while !input.is_empty() {
        // Check for events_received keyword
        if input.peek(Ident) {
            let lookahead = input.fork();
            let ident: Ident = lookahead.parse()?;

            if ident == "events_received" {
                input.parse::<Ident>()?; // Consume events_received
                input.parse::<Token![:]>()?;

                let content;
                syn::bracketed!(content in input);
                let mut events = Vec::new();
                while !content.is_empty() {
                    events.push(content.parse()?);
                    content.parse::<Token![,]>().ok();
                }

                assertions.push(Assertion::EventsReceived(events));
            } else if ident == "snapshot" {
                input.parse::<Ident>()?; // Consume snapshot
                input.parse::<Token![:]>()?;
                let expr = input.parse()?;
                assertions.push(Assertion::Snapshot(expr));
            } else {
                // Regular component check
                let expr = input.parse()?;
                assertions.push(Assertion::ComponentCheck(expr));
            }
        } else {
            // Default to component check
            let expr = input.parse()?;
            assertions.push(Assertion::ComponentCheck(expr));
        }

        input.parse::<Token![,]>().ok();
    }

    Ok(ThenClause { assertions })
}

impl TestScenario {
    pub fn expand(&self) -> TokenStream {
        let test_name = &self.name;
        let setup = self.generate_setup();
        let actions = self.generate_actions();
        let assertions = self.generate_assertions();

        quote! {
            #[test]
            fn #test_name() {
                use bevy::prelude::*;

                // Create test app
                let mut app = App::new();
                app.add_plugins(MinimalPlugins);

                #setup

                // Execute when clause
                #actions

                // Verify then clause
                #assertions
            }
        }
    }

    fn generate_setup(&self) -> TokenStream {
        let mut setup = TokenStream::new();

        // Add resources
        for resource in &self.given.resources {
            setup.extend(quote! {
                app.insert_resource(#resource);
            });
        }

        // Add events
        for event in &self.given.events {
            setup.extend(quote! {
                app.add_event::<#event>();
            });
        }

        // Add systems
        for system in &self.given.systems {
            setup.extend(quote! {
                app.add_systems(Update, #system);
            });
        }

        // Spawn entities
        for (idx, entity) in self.given.entities.iter().enumerate() {
            let type_name = &entity.type_name;
            let mut field_inits = TokenStream::new();

            for (field_name, field_value) in &entity.fields {
                field_inits.extend(quote! {
                    #field_name: #field_value,
                });
            }

            let var_name = quote::format_ident!("entity_{}", idx);
            setup.extend(quote! {
                let #var_name = app.world_mut().spawn(#type_name {
                    #field_inits
                    ..Default::default()
                }).id();
            });
        }

        setup
    }

    fn generate_actions(&self) -> TokenStream {
        let mut actions = TokenStream::new();

        for action in &self.when.actions {
            match action {
                Action::Event(event) => {
                    actions.extend(quote! {
                        app.world_mut().send_event(#event);
                    });
                }
                Action::Advance(advance) => {
                    match advance {
                        TimeAdvance::Frames(n) => {
                            actions.extend(quote! {
                                for _ in 0..#n {
                                    app.update();
                                }
                            });
                        }
                        TimeAdvance::Days(n) => {
                            // Advance by days - assumes 10 updates per day
                            let total_updates = n * 10;
                            actions.extend(quote! {
                                for _ in 0..#total_updates {
                                    app.update();
                                }
                            });
                        }
                        TimeAdvance::Seconds(s) => {
                            // Advance time by seconds
                            actions.extend(quote! {
                                // Advance by seconds - 60 FPS assumed
                                let frames = (#s * 60.0) as usize;
                                for _ in 0..frames {
                                    app.update();
                                }
                            });
                        }
                    }
                }
                Action::Input(input_expr) => {
                    actions.extend(quote! {
                        // Send input to the app (e.g., keyboard, mouse events)
                        app.world_mut().send_event(#input_expr);
                    });
                }
            }
        }

        actions
    }

    fn generate_assertions(&self) -> TokenStream {
        let mut assertions = TokenStream::new();

        for assertion in &self.then.assertions {
            match assertion {
                Assertion::ComponentCheck(expr) => {
                    assertions.extend(quote! {
                        assert!(#expr, "Assertion failed: {}", stringify!(#expr));
                    });
                }
                Assertion::EventsReceived(events) => {
                    for event in events {
                        assertions.extend(quote! {
                            // Verify event was sent
                            let event_reader = app.world().resource::<Events<#event>>();
                            assert!(!event_reader.is_empty(), "Event {} should have been received", stringify!(#event));
                        });
                    }
                }
                Assertion::Snapshot(expr) => {
                    assertions.extend(quote! {
                        // Snapshot testing (simplified - would integrate with insta or similar)
                        let snapshot_value = #expr;
                        // In real implementation, this would compare with stored snapshot
                        println!("Snapshot: {:?}", snapshot_value);
                    });
                }
            }
        }

        assertions
    }
}

// Stub implementations for other test types
pub struct BenchmarkScenario;
impl Parse for BenchmarkScenario {
    fn parse(_input: ParseStream) -> Result<Self> {
        Ok(BenchmarkScenario)
    }
}
impl BenchmarkScenario {
    pub fn expand(&self) -> TokenStream {
        quote! {}
    }
}

pub struct PropertyTest;
impl Parse for PropertyTest {
    fn parse(_input: ParseStream) -> Result<Self> {
        Ok(PropertyTest)
    }
}
impl PropertyTest {
    pub fn expand(&self) -> TokenStream {
        quote! {}
    }
}
