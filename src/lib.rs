//! # Bevy Test Suite
//!
//! A declarative testing framework for Bevy applications that eliminates boilerplate
//! and makes tests readable, maintainable, and impossible to mess up.
//!
//! ## Features
//!
//! - **Declarative test scenarios** - Describe what you want to test, not how
//! - **Automatic app setup** - No more boilerplate MinimalPlugins configuration
//! - **Time control** - Advance frames, seconds, or game days declaratively
//! - **Rich assertions** - Component, resource, and event assertions built-in
//! - **Property testing** - Integration with proptest for edge case discovery
//! - **Performance benchmarks** - Built-in performance testing support
//!
//! ## Example
//!
//! ```rust
//! test_scenario!(player_movement {
//!     given: {
//!         resources: [Time::default()],
//!         entities: [Player { position: Vec3::ZERO, speed: 5.0 }]
//!     },
//!     when: {
//!         input: MoveRight,
//!         advance: 1.second()
//!     },
//!     then: {
//!         Player[0].position.x > 0.0
//!     }
//! });
//! ```

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod scenario;
mod system;
mod component;
mod builders;
mod assertions;
mod attribute;

/// Creates a complete test scenario with given/when/then structure.
///
/// # Example
/// ```
/// test_scenario!(law_enactment {
///     given: {
///         resources: [LawRegistry::default()],
///         events: [LawEnactmentEvent],
///         systems: [propose_laws_system],
///         entities: [Nation { name: "Test Nation" }]
///     },
///     when: {
///         event: ProposeLawEvent { law_id: TaxReform },
///         advance: 10.days()
///     },
///     then: {
///         Nation[0].laws.contains(TaxReform),
///         events_received: [LawEnactmentEvent]
///     }
/// });
/// ```
#[proc_macro]
pub fn test_scenario(input: TokenStream) -> TokenStream {
    let scenario = parse_macro_input!(input as scenario::TestScenario);
    scenario.expand().into()
}

/// Generates utility code for test builders and assertions.
/// Call this once in your test module to get access to MockWorld, MockInput, TestApp trait, and assertions.
///
/// # Example
/// ```
/// bevy_test_utils!();
///
/// // Now you can use:
/// // - TestApp trait for #[bevy_test]
/// // - MockWorld for world building
/// // - MockInput for input simulation
/// // - Rich assertion macros
/// ```
#[proc_macro]
pub fn bevy_test_utils(_input: TokenStream) -> TokenStream {
    use quote::quote;
    let mut output = quote! {};

    // Add builder utilities
    output.extend(builders::generate_mock_world());
    output.extend(builders::generate_mock_input());
    output.extend(builders::generate_fixtures());

    // Add assertion utilities
    output.extend(assertions::generate_assertions());

    // Add test helpers for #[bevy_test]
    output.extend(attribute::generate_test_helpers());

    output.into()
}

/// Tests a single system in isolation with inputs and expected outputs.
///
/// # Example
/// ```
/// test_system!(calculate_damage {
///     setup: {
///         attacker: Unit { strength: 10 },
///         defender: Unit { defense: 5 }
///     },
///     call: calculate_damage_system,
///     expect: {
///         defender.health < 100
///     }
/// });
/// ```
#[proc_macro]
pub fn test_system(input: TokenStream) -> TokenStream {
    let system_test = parse_macro_input!(input as system::SystemTest);
    system_test.expand().into()
}

/// Tests component behavior and state transitions.
///
/// # Example
/// ```
/// test_component!(health_component {
///     given: Health(100),
///     operations: [
///         take_damage(30) => Health(70),
///         heal(20) => Health(90),
///         take_damage(100) => Health(0)
///     ]
/// });
/// ```
#[proc_macro]
pub fn test_component(input: TokenStream) -> TokenStream {
    let component_test = parse_macro_input!(input as component::ComponentTest);
    component_test.expand().into()
}

/// Creates a performance benchmark for a scenario.
///
/// # Example
/// ```
/// benchmark_scenario!(world_update {
///     setup: {
///         world: generate_world(1_000_000),
///         nations: 50
///     },
///     measure: world_update_system,
///     max_time: 16.ms(),
///     iterations: 100
/// });
/// ```
#[proc_macro]
pub fn benchmark_scenario(input: TokenStream) -> TokenStream {
    let benchmark = parse_macro_input!(input as scenario::BenchmarkScenario);
    benchmark.expand().into()
}

/// Defines a property test with invariants.
///
/// # Example
/// ```
/// property_test!(law_invariants {
///     given: {
///         laws: vec(any::<LawId>(), 0..100),
///         nation: any::<Nation>()
///     },
///     invariants: [
///         "No conflicting laws active",
///         "Effects within bounds"
///     ]
/// });
/// ```
#[proc_macro]
pub fn property_test(input: TokenStream) -> TokenStream {
    let property = parse_macro_input!(input as scenario::PropertyTest);
    property.expand().into()
}

/// The #[bevy_test] attribute that the community has been asking for!
///
/// This provides a more familiar testing experience while still eliminating boilerplate.
///
/// # Example
/// ```
/// #[bevy_test]
/// fn test_player_movement(app: &mut TestApp) {
///     let player = app.spawn(Player { position: Vec3::ZERO });
///     app.advance_time(1.0);
///     assert!(app.query::<&Position>().single().x > 0.0);
/// }
///
/// // Or without app parameter
/// #[bevy_test(headless)]
/// fn test_simple() {
///     // App is created automatically
///     assert_eq!(2 + 2, 4);
/// }
/// ```
#[proc_macro_attribute]
pub fn bevy_test(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the function from input
    let function = parse_macro_input!(input as syn::ItemFn);

    // Parse configuration from args if provided
    let config = if args.is_empty() {
        attribute::TestConfig::default()
    } else {
        parse_macro_input!(args as attribute::TestConfig)
    };

    attribute::expand_bevy_test(config, function).into()
}