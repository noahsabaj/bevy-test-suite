//! Test builder utilities
//!
//! Provides builder patterns for creating test worlds, mock inputs, and other
//! test fixtures with minimal boilerplate.

use proc_macro2::TokenStream;
use quote::quote;

/// Generate MockWorld builder implementation
pub fn generate_mock_world() -> TokenStream {
    quote! {
        /// Builder for creating test worlds with entities and components
        pub struct MockWorld {
            app: bevy::app::App,
        }

        impl MockWorld {
            /// Create a new mock world
            pub fn new() -> Self {
                let mut app = bevy::app::App::new();
                app.add_plugins(bevy::app::MinimalPlugins);
                Self { app }
            }

            /// Add a specific number of entities to the world
            pub fn with_entities(mut self, count: usize) -> Self {
                for _ in 0..count {
                    self.app.world_mut().spawn_empty();
                }
                self
            }

            /// Add entities with random components of a specific type
            pub fn with_random_components<T: bevy::prelude::Component + Default>(mut self) -> Self {
                let entities: Vec<_> = self.app.world()
                    .query::<bevy::prelude::Entity>()
                    .iter(self.app.world())
                    .collect();

                for entity in entities {
                    self.app.world_mut()
                        .entity_mut(entity)
                        .insert(T::default());
                }
                self
            }

            /// Add a resource to the world
            pub fn with_resource<R: bevy::prelude::Resource>(mut self, resource: R) -> Self {
                self.app.insert_resource(resource);
                self
            }

            /// Add a system to the world
            pub fn with_system<M>(mut self, system: impl bevy::ecs::system::IntoSystem<(), (), M>) -> Self {
                self.app.add_systems(bevy::app::Update, system);
                self
            }

            /// Build the mock world into a Bevy App
            pub fn build(self) -> bevy::app::App {
                self.app
            }
        }
    }
}

/// Generate MockInput builder implementation
pub fn generate_mock_input() -> TokenStream {
    quote! {
        /// Builder for simulating input events in tests
        pub struct MockInput {
            events: Vec<InputEvent>,
        }

        enum InputEvent {
            KeyPress(bevy::prelude::KeyCode),
            KeyRelease(bevy::prelude::KeyCode),
            MouseMove(bevy::math::Vec2),
            MouseClick(bevy::input::mouse::MouseButton),
            Wait(f32),
        }

        impl MockInput {
            /// Create a new mock input sequence
            pub fn new() -> Self {
                Self { events: Vec::new() }
            }

            /// Simulate pressing a key
            pub fn press(mut self, key: bevy::prelude::KeyCode) -> Self {
                self.events.push(InputEvent::KeyPress(key));
                self
            }

            /// Simulate releasing a key
            pub fn release(mut self, key: bevy::prelude::KeyCode) -> Self {
                self.events.push(InputEvent::KeyRelease(key));
                self
            }

            /// Simulate moving the mouse
            pub fn mouse_move(mut self, position: bevy::math::Vec2) -> Self {
                self.events.push(InputEvent::MouseMove(position));
                self
            }

            /// Simulate clicking the mouse
            pub fn click(mut self, button: bevy::input::mouse::MouseButton) -> Self {
                self.events.push(InputEvent::MouseClick(button));
                self
            }

            /// Wait for a duration (in seconds)
            pub fn wait(mut self, duration: f32) -> Self {
                self.events.push(InputEvent::Wait(duration));
                self
            }

            /// Apply the input sequence to a Bevy App
            pub fn apply_to(self, app: &mut bevy::app::App) {
                for event in self.events {
                    match event {
                        InputEvent::KeyPress(key) => {
                            // Send key press event
                            app.world_mut().send_event(bevy::input::keyboard::KeyboardInput {
                                logical_key: bevy::input::keyboard::Key::Character(format!("{:?}", key).into()),
                                key_code: key,
                                state: bevy::input::ButtonState::Pressed,
                                window: bevy::prelude::Entity::PLACEHOLDER,
                            });
                        }
                        InputEvent::KeyRelease(key) => {
                            // Send key release event
                            app.world_mut().send_event(bevy::input::keyboard::KeyboardInput {
                                logical_key: bevy::input::keyboard::Key::Character(format!("{:?}", key).into()),
                                key_code: key,
                                state: bevy::input::ButtonState::Released,
                                window: bevy::prelude::Entity::PLACEHOLDER,
                            });
                        }
                        InputEvent::MouseMove(pos) => {
                            // Update cursor position
                            app.world_mut().send_event(bevy::window::CursorMoved {
                                window: bevy::prelude::Entity::PLACEHOLDER,
                                position: pos,
                                delta: None,
                            });
                        }
                        InputEvent::MouseClick(button) => {
                            // Send mouse click event
                            app.world_mut().send_event(bevy::input::mouse::MouseButtonInput {
                                button,
                                state: bevy::input::ButtonState::Pressed,
                                window: bevy::prelude::Entity::PLACEHOLDER,
                            });
                        }
                        InputEvent::Wait(duration) => {
                            // Advance time
                            let frames = (duration * 60.0) as usize;
                            for _ in 0..frames {
                                app.update();
                            }
                        }
                    }
                    // Update after each event
                    app.update();
                }
            }
        }
    }
}

/// Generate TestFixture trait and implementations
pub fn generate_fixtures() -> TokenStream {
    quote! {
        /// Trait for reusable test fixtures
        pub trait TestFixture {
            /// Create the fixture and apply it to an app
            fn apply_to(self, app: &mut bevy::app::App);
        }

        /// Macro for defining fixtures
        #[macro_export]
        macro_rules! fixture {
            ($name:ident { $($field:ident : $value:expr),* $(,)? }) => {
                pub struct $name;

                impl TestFixture for $name {
                    fn apply_to(self, app: &mut bevy::app::App) {
                        $(
                            // Apply each field to the app
                            // This would be expanded based on field type
                            $value.apply_to(app);
                        )*
                    }
                }
            };
        }
    }
}