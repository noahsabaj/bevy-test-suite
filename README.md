# bevy-test-suite

[![Crates.io](https://img.shields.io/crates/v/bevy-test-suite.svg)](https://crates.io/crates/bevy-test-suite)
[![Documentation](https://docs.rs/bevy-test-suite/badge.svg)](https://docs.rs/bevy-test-suite)
[![License](https://img.shields.io/crates/l/bevy-test-suite.svg)](https://github.com/noahsabaj/bevy-test-suite)

**TWO ways to test Bevy apps**: The `#[bevy_test]` attribute you asked for, plus declarative `test_scenario!` macros for complex integration tests. 80% less boilerplate, 100% less pain.

## Why This Exists

The Bevy community has been asking for `#[bevy::test]` to eliminate testing boilerplate. We deliver that AND more:

### Problems We Solve

1. **Boilerplate Hell** (30+ lines → 5-10 lines)
2. **No Standard Patterns** (given/when/then structure)
3. **Time Manipulation** (advance by frames/seconds/days)
4. **Resource Management** (automatic Time, Events, etc.)
5. **Test Readability** (tests read like documentation)

### Two Complementary Approaches

- **`#[bevy_test]`** - The attribute macro the community requested for simple tests
- **`test_scenario!`** - Declarative testing for complex integration scenarios

## Quick Start

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
bevy-test-suite = "0.1"
bevy = "0.16"
```

## Usage

### Approach 1: `#[bevy_test]` Attribute

The familiar Rust testing experience with zero boilerplate:

```rust
use bevy::prelude::*;
use bevy_test_suite::{bevy_test, bevy_test_utils};

// Generate test utilities including TestApp trait
bevy_test_utils!();

#[bevy_test]
fn test_player_movement(app: &mut TestApp) {
    let player = app.spawn(Player { position: Vec3::ZERO });
    app.advance_time(1.0);
    assert!(app.query::<&Position>().single().x > 0.0);
}

// With configuration options
#[bevy_test(headless, timeout = 1000)]
fn test_for_ci(app: &mut TestApp) {
    // Runs without GPU/window requirements
}
```

### Approach 2: Declarative Testing

For complex scenarios that read like specifications:

```rust
use bevy::prelude::*;
use bevy_test_suite::{test_scenario, bevy_test_utils};

// Generate test utilities
bevy_test_utils!();

test_scenario!(player_takes_damage {
    given: {
        resources: [Time::default()],
        entities: [
            Player {
                type_name: Player,
                fields: {
                    health: 100,
                    armor: 10
                }
            }
        ]
    },
    when: {
        event: DamageEvent { target: entity_0, amount: 30 },
        advance: 1.second()
    },
    then: {
        entity_0.get::<Player>().unwrap().health == 73
    }
});
```

### System Testing

Test individual systems in isolation:

```rust
test_system!(movement_system_test {
    setup: {
        resources: [Time::default()],
        player: (Position(Vec3::ZERO), Velocity(Vec3::X * 10.0))
    },
    call: movement_system,
    expect: {
        app.world().entity(player).get::<Position>().unwrap().0.x > 0.0
    }
});
```

### Component Testing

Test component behavior and state transitions:

```rust
test_component!(health_component {
    given: Health(100),
    operations: [
        take_damage(30) => Health(70),
        heal(20) => Health(90),
        take_damage(100) => Health(0)
    ]
});
```

### Time Manipulation

Easily control time in your tests:

```rust
test_scenario!(test_over_time {
    // ...
    when: {
        advance: 10.frames(),    // Advance 10 frames
        advance: 5.seconds(),    // Advance 5 seconds
        advance: 2.days()        // Advance 2 game days
    },
    // ...
});
```

### Property Testing

Automatically generate test cases to verify invariants:

```rust
property_test!(health_invariants {
    given: {
        max_health: 1..=1000,
        damage_amounts: vec(0..=500, 0..10)
    },
    invariants: [
        "Health never negative",
        "Health never exceeds max",
        "Healing beyond max is capped"
    ]
});
```

### Important: Using `#[bevy_test]`

When using the `#[bevy_test]` attribute macro, you must first generate the TestApp trait:

```rust
// At the top of your test file
bevy_test_utils!();  // This generates TestApp and other utilities

#[bevy_test]
fn my_test(app: &mut TestApp) {
    // Your test code
}
```

### Mock Builders

Create test worlds and inputs with builder patterns:

```rust
let world = MockWorld::new()
    .with_entities(100)
    .with_random_components::<Transform>()
    .with_resource(GameSettings::default())
    .build();

let input = MockInput::new()
    .press(KeyCode::Space)
    .wait(0.5)
    .mouse_move(Vec2::new(100.0, 200.0))
    .click(MouseButton::Left)
    .apply_to(&mut app);
```

### Rich Assertions

Use powerful assertion macros beyond simple equality:

```rust
assert_entity_count!(app, Player, 1);
assert_component_changed!(app, Transform);
assert_event_sent!(app, CollisionEvent);
assert_resource_exists!(app, GameSettings);
assert_query_empty!(app, Query<&Dead>);
assert_parent_child!(app, parent_entity, child_entity);
assert_approx_eq!(position.x, 100.0, 0.001);
```

## Comparison: Three Ways to Test

### 1. Manual Testing (The Old Way - 30+ lines)

```rust
#[test]
fn test_damage_manual() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Time::default());
    app.add_event::<DamageEvent>();
    app.add_systems(Update, damage_system);

    let player = app.world_mut().spawn(Player {
        health: 100,
        position: Vec3::ZERO,
    }).id();

    app.world_mut().send_event(DamageEvent {
        target: player,
        amount: 30,
    });

    app.update();

    let player_health = app.world()
        .entity(player)
        .get::<Player>()
        .unwrap()
        .health;
    assert_eq!(player_health, 70);
}
```

### 2. With `#[bevy_test]` (10 lines - 66% reduction)

```rust
#[bevy_test]
fn test_damage_attribute(app: &mut TestApp) {
    let player = app.spawn(Player { health: 100, position: Vec3::ZERO });
    app.send_event(DamageEvent { target: player, amount: 30 });
    app.update();
    assert_eq!(app.query::<&Player>().single().health, 70);
}
```

### 3. With `test_scenario!` (5 lines - 83% reduction)

```rust
test_scenario!(test_damage_declarative {
    given: { entities: [Player { health: 100 }] },
    when: { event: DamageEvent { amount: 30 } },
    then: { Player[0].health == 70 }
});
```

## Examples

Check out the `examples/` directory for comprehensive examples:

- `dual_approach.rs` - **Shows both `#[bevy_test]` and declarative testing**
- `comparison.rs` - **Side-by-side comparison of all three approaches**
- `basic_scenario.rs` - Introduction to scenario testing
- `system_testing.rs` - Testing individual systems
- `property_testing.rs` - Property-based testing patterns

Run examples with:

```bash
cargo test --example basic_scenario
```

## Performance

`bevy-test-suite` adds **zero runtime overhead**. The macros expand at compile time to generate the same code you would write manually. Your tests run at the same speed, but you write 80% less code.

## Compatibility

- Bevy 0.16+ (required)
- Works with all Bevy plugins
- Supports headless testing for CI/CD
- Cross-platform (Windows, macOS, Linux)

## When to Use Each Approach

| Use `#[bevy_test]` When | Use `test_scenario!` When |
|------------------------|-------------------------|
| Testing algorithms | Testing game scenarios |
| Need fine control | Want readable specs |
| Prefer imperative style | Prefer declarative style |
| Simple unit tests | Complex integration tests |
| Migrating from manual | Writing new test suites |

## Real Talk: What This Actually Solves

**What the community asked for**: `#[bevy::test]` attribute macro

**What we deliver**:
- `#[bevy_test]` - The attribute you wanted
- `test_scenario!` - Declarative testing for complex scenarios
- 66-83% less code than manual testing
- Time manipulation utilities
- Rich assertions and mock builders

**What we DON'T solve**:
- Not built into Bevy core (we're a third-party crate)
- Still need to import our crate
- Learning curve for declarative syntax

## Philosophy

We give you TWO ways to test because different tests need different approaches:

1. **`#[bevy_test]`** - Familiar, imperative, great for unit tests
2. **`test_scenario!`** - Declarative, readable, perfect for integration tests
3. **Both approaches** - 66-83% less boilerplate than manual testing
4. **Zero runtime overhead** - Compiles to the code you'd write by hand

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Credits

Special thanks to the Bevy community for feedback and suggestions.

---

*Making Bevy testing as pleasant as using Bevy itself.*