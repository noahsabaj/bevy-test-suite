//! Attribute macro implementation for #[bevy_test]
//!
//! Provides the #[bevy_test] attribute that the community has been asking for,
//! alongside our declarative testing approach.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, ItemFn, Result};

#[derive(Default)]
pub struct TestConfig {
    pub headless: bool,
    pub plugins: Vec<syn::Expr>,
    pub timeout_ms: Option<u64>,
}

impl Parse for TestConfig {
    fn parse(input: ParseStream) -> Result<Self> {
        parse_config(input)
    }
}

fn parse_config(input: ParseStream) -> Result<TestConfig> {
    let mut config = TestConfig::default();

    while !input.is_empty() {
        let ident: syn::Ident = input.parse()?;

        match ident.to_string().as_str() {
            "headless" => {
                config.headless = true;
            }
            "plugins" => {
                input.parse::<syn::Token![=]>()?;
                let content;
                syn::bracketed!(content in input);
                while !content.is_empty() {
                    config.plugins.push(content.parse()?);
                    content.parse::<syn::Token![,]>().ok();
                }
            }
            "timeout" => {
                input.parse::<syn::Token![=]>()?;
                let lit: syn::LitInt = input.parse()?;
                config.timeout_ms = Some(lit.base10_parse()?);
            }
            _ => {
                return Err(syn::Error::new(ident.span(), "Unknown configuration option"));
            }
        }

        input.parse::<syn::Token![,]>().ok();
    }

    Ok(config)
}

pub fn expand_bevy_test(config: TestConfig, function: ItemFn) -> TokenStream {
    let fn_name = &function.sig.ident;
    let fn_body = &function.block;
    let fn_args = &function.sig.inputs;

    // Determine what the test function expects
    let expects_app = fn_args.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            // Check if parameter type contains "App" or "TestApp"
            let type_str = quote!(#pat_type.ty).to_string();
            type_str.contains("App") || type_str.contains("TestApp")
        } else {
            false
        }
    });

    let setup_code = generate_setup(&config);
    let timeout_code = generate_timeout(&config);

    if expects_app {
        // Function expects an app parameter
        quote! {
            #[test]
            fn #fn_name() {
                #timeout_code

                // Create and configure test app
                let mut app = {
                    #setup_code
                };

                // Define test function with app parameter
                let test_fn = |app: &mut bevy::app::App| {
                    #fn_body
                };

                // Execute test
                test_fn(&mut app);
            }
        }
    } else {
        // Function is self-contained
        quote! {
            #[test]
            fn #fn_name() {
                #timeout_code

                // Create app in scope
                let mut app = {
                    #setup_code
                };

                // Make app available via thread-local if needed
                thread_local! {
                    static TEST_APP: std::cell::RefCell<Option<bevy::app::App>> = std::cell::RefCell::new(None);
                }

                TEST_APP.with(|a| {
                    *a.borrow_mut() = Some(app);
                });

                // Execute original test body
                #fn_body

                // Clean up
                TEST_APP.with(|a| {
                    *a.borrow_mut() = None;
                });
            }
        }
    }
}

fn generate_setup(config: &TestConfig) -> TokenStream {
    let plugins = if config.plugins.is_empty() {
        quote! {
            app.add_plugins(bevy::app::MinimalPlugins);
        }
    } else {
        let plugins = &config.plugins;
        quote! {
            app.add_plugins(bevy::app::MinimalPlugins);
            #(app.add_plugins(#plugins);)*
        }
    };

    quote! {
        let mut app = bevy::app::App::new();
        #plugins

        // Add common test resources
        app.insert_resource(bevy::time::Time::default());

        // Configure for headless if needed
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Note: headless configuration would go here if needed
        }

        app
    }
}

fn generate_timeout(config: &TestConfig) -> TokenStream {
    if let Some(timeout_ms) = config.timeout_ms {
        quote! {
            // Set test timeout
            // Note: This would need platform-specific implementation
            let _timeout = std::time::Duration::from_millis(#timeout_ms);
        }
    } else {
        quote! {}
    }
}

/// Generate helper functions that work with #[bevy_test]
pub fn generate_test_helpers() -> TokenStream {
    quote! {
        /// Extension trait for TestApp functionality
        pub trait TestApp {
            fn spawn<B: bevy::ecs::bundle::Bundle>(&mut self, bundle: B) -> bevy::ecs::entity::Entity;
            fn advance_time(&mut self, seconds: f32);
            fn advance_frames(&mut self, frames: usize);
            fn send_event<E: bevy::ecs::event::Event>(&mut self, event: E);
            fn query<Q: bevy::ecs::query::QueryData>(&self) -> TestQuery<Q>;
            fn resource<R: bevy::ecs::system::Resource>(&self) -> &R;
            fn resource_mut<R: bevy::ecs::system::Resource>(&mut self) -> &mut R;
        }

        impl TestApp for bevy::app::App {
            fn spawn<B: bevy::ecs::bundle::Bundle>(&mut self, bundle: B) -> bevy::ecs::entity::Entity {
                self.world_mut().spawn(bundle).id()
            }

            fn advance_time(&mut self, seconds: f32) {
                let frames = (seconds * 60.0) as usize;
                for _ in 0..frames {
                    self.update();
                }
            }

            fn advance_frames(&mut self, frames: usize) {
                for _ in 0..frames {
                    self.update();
                }
            }

            fn send_event<E: bevy::ecs::event::Event>(&mut self, event: E) {
                self.world_mut().send_event(event);
            }

            fn query<Q: bevy::ecs::query::QueryData>(&self) -> TestQuery<Q> {
                TestQuery {
                    world: self.world(),
                    _phantom: std::marker::PhantomData,
                }
            }

            fn resource<R: bevy::ecs::system::Resource>(&self) -> &R {
                self.world().resource::<R>()
            }

            fn resource_mut<R: bevy::ecs::system::Resource>(&mut self) -> &mut R {
                self.world_mut().resource_mut::<R>()
            }
        }

        /// Query wrapper for testing
        pub struct TestQuery<'w, Q: bevy::ecs::query::QueryData> {
            world: &'w bevy::ecs::world::World,
            _phantom: std::marker::PhantomData<Q>,
        }

        impl<'w, Q: bevy::ecs::query::QueryData> TestQuery<'w, Q> {
            pub fn single(&self) -> Q::Item<'w> {
                let mut query = self.world.query::<Q>();
                query.single(self.world)
            }

            pub fn is_empty(&self) -> bool {
                let mut query = self.world.query::<Q>();
                query.iter(self.world).count() == 0
            }

            pub fn count(&self) -> usize {
                let mut query = self.world.query::<Q>();
                query.iter(self.world).count()
            }

            pub fn iter(&self) -> impl Iterator<Item = Q::Item<'w>> {
                let mut query = self.world.query::<Q>();
                query.iter(self.world)
            }
        }
    }
}