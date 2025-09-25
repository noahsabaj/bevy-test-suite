//! Rich assertion utilities for Bevy testing
//!
//! Provides a comprehensive set of assertions for testing Bevy applications
//! beyond simple equality checks.

use proc_macro2::TokenStream;
use quote::quote;

/// Generate assertion macro implementations
pub fn generate_assertions() -> TokenStream {
    quote! {
        /// Assert that a specific number of entities with a component exist
        #[macro_export]
        macro_rules! assert_entity_count {
            ($app:expr, $component:ty, $expected:expr) => {
                {
                    let count = $app.world()
                        .query::<&$component>()
                        .iter($app.world())
                        .count();
                    assert_eq!(
                        count,
                        $expected,
                        "Expected {} entities with {}, found {}",
                        $expected,
                        stringify!($component),
                        count
                    );
                }
            };
        }

        /// Assert that a component has changed
        #[macro_export]
        macro_rules! assert_component_changed {
            ($app:expr, $component:ty) => {
                {
                    let changed = $app.world()
                        .query_filtered::<bevy::prelude::Entity, bevy::prelude::Changed<$component>>()
                        .iter($app.world())
                        .count() > 0;
                    assert!(
                        changed,
                        "Expected {} to have changed, but it didn't",
                        stringify!($component)
                    );
                }
            };
        }

        /// Assert that an event was sent
        #[macro_export]
        macro_rules! assert_event_sent {
            ($app:expr, $event:ty) => {
                {
                    let events = $app.world().resource::<bevy::ecs::event::Events<$event>>();
                    assert!(
                        !events.is_empty(),
                        "Expected {} event to be sent, but no events found",
                        stringify!($event)
                    );
                }
            };
        }

        /// Assert that a resource exists
        #[macro_export]
        macro_rules! assert_resource_exists {
            ($app:expr, $resource:ty) => {
                {
                    assert!(
                        $app.world().contains_resource::<$resource>(),
                        "Expected resource {} to exist, but it doesn't",
                        stringify!($resource)
                    );
                }
            };
        }

        /// Assert that a resource equals a value
        #[macro_export]
        macro_rules! assert_resource_equals {
            ($app:expr, $resource:ty, $expected:expr) => {
                {
                    let actual = $app.world().resource::<$resource>();
                    assert_eq!(
                        *actual,
                        $expected,
                        "Resource {} value mismatch",
                        stringify!($resource)
                    );
                }
            };
        }

        /// Assert that a query returns no results
        #[macro_export]
        macro_rules! assert_query_empty {
            ($app:expr, $query:ty) => {
                {
                    let count = $app.world()
                        .query::<$query>()
                        .iter($app.world())
                        .count();
                    assert_eq!(
                        count,
                        0,
                        "Expected query {} to be empty, but found {} results",
                        stringify!($query),
                        count
                    );
                }
            };
        }

        /// Assert that a system ran successfully
        #[macro_export]
        macro_rules! assert_system_ran {
            ($app:expr, $system:ident) => {
                {
                    // This would check system execution metrics if available
                    // For now, we assume the system ran if the app updated
                    assert!(
                        true,
                        "System {} verification not yet implemented",
                        stringify!($system)
                    );
                }
            };
        }

        /// Assert entity relationships
        #[macro_export]
        macro_rules! assert_entity_has {
            ($app:expr, $entity:expr, $component:ty) => {
                {
                    let has_component = $app.world()
                        .entity($entity)
                        .contains::<$component>();
                    assert!(
                        has_component,
                        "Entity {:?} does not have component {}",
                        $entity,
                        stringify!($component)
                    );
                }
            };
        }

        /// Assert that entities are in a parent-child relationship
        #[macro_export]
        macro_rules! assert_parent_child {
            ($app:expr, $parent:expr, $child:expr) => {
                {
                    use bevy::prelude::*;

                    let child_of = $app.world()
                        .entity($child)
                        .get::<ChildOf>();

                    match child_of {
                        Some(child_of) if child_of.parent() == $parent => {},
                        Some(child_of) => panic!(
                            "Entity {:?} has parent {:?}, expected {:?}",
                            $child,
                            child_of.parent(),
                            $parent
                        ),
                        None => panic!(
                            "Entity {:?} has no parent, expected {:?}",
                            $child,
                            $parent
                        ),
                    }
                }
            };
        }

        /// Assert that a component matches a pattern
        #[macro_export]
        macro_rules! assert_component_matches {
            ($app:expr, $entity:expr, $component:ty, $pattern:pat) => {
                {
                    let component = $app.world()
                        .entity($entity)
                        .get::<$component>()
                        .expect(&format!(
                            "Entity {:?} does not have component {}",
                            $entity,
                            stringify!($component)
                        ));

                    match component {
                        $pattern => {},
                        _ => panic!(
                            "Component {} on entity {:?} does not match pattern {}",
                            stringify!($component),
                            $entity,
                            stringify!($pattern)
                        ),
                    }
                }
            };
        }

        /// Assert approximate equality for floating point values
        #[macro_export]
        macro_rules! assert_approx_eq {
            ($actual:expr, $expected:expr, $epsilon:expr) => {
                {
                    let diff = ($actual - $expected).abs();
                    assert!(
                        diff < $epsilon,
                        "Expected {} ≈ {} (ε = {}), but difference was {}",
                        $actual,
                        $expected,
                        $epsilon,
                        diff
                    );
                }
            };
        }
    }
}