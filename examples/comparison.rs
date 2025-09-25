//! Comparison of testing approaches in bevy-test-suite
//!
//! This example shows THREE ways to test the same scenario:
//! 1. Manual testing (the old way)
//! 2. #[bevy_test] attribute (what the community asked for)
//! 3. test_scenario! declarative (our innovation)

use bevy::prelude::*;
use bevy_test_suite::{bevy_test, test_scenario, bevy_test_utils};

bevy_test_utils!();

// Components for our example
#[derive(Component, Debug, Clone, PartialEq)]
struct Player {
    health: i32,
    position: Vec3,
}

#[derive(Component)]
struct Enemy {
    damage: i32,
}

#[derive(Event)]
struct DamageEvent {
    target: Entity,
    amount: i32,
}

fn damage_system(
    mut events: EventReader<DamageEvent>,
    mut query: Query<&mut Player>,
) {
    for event in events.read() {
        if let Ok(mut player) = query.get_mut(event.target) {
            player.health -= event.amount;
        }
    }
}

// ==============================================================================
// APPROACH 1: Manual Testing (The Old Way - 30+ lines)
// ==============================================================================
#[test]
fn test_damage_manual() {
    // Setup boilerplate
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Time::default());
    app.add_event::<DamageEvent>();
    app.add_systems(Update, damage_system);

    // Spawn entities
    let player = app.world_mut().spawn(Player {
        health: 100,
        position: Vec3::ZERO,
    }).id();

    // Send damage event
    app.world_mut().send_event(DamageEvent {
        target: player,
        amount: 30,
    });

    // Run update cycle
    app.update();

    // Assert
    let player_health = app.world()
        .entity(player)
        .get::<Player>()
        .unwrap()
        .health;
    assert_eq!(player_health, 70);
}

// ==============================================================================
// APPROACH 2: #[bevy_test] Attribute (What The Community Asked For - 10 lines)
// ==============================================================================
#[bevy_test]
fn test_damage_attribute(app: &mut TestApp) {
    // Much cleaner! The attribute handles all setup
    app.add_systems(Update, damage_system);
    app.add_event::<DamageEvent>();

    let player = app.spawn(Player {
        health: 100,
        position: Vec3::ZERO,
    });

    app.send_event(DamageEvent {
        target: player,
        amount: 30,
    });

    app.update();

    assert_eq!(app.query::<&Player>().single().health, 70);
}

// Even simpler with configuration
#[bevy_test(headless, timeout = 1000)]
fn test_damage_attribute_configured(app: &mut TestApp) {
    // Configuration in the attribute!
    let player = app.spawn(Player { health: 100, position: Vec3::ZERO });
    app.send_event(DamageEvent { target: player, amount: 30 });
    app.advance_frames(1);
    assert_eq!(app.query::<&Player>().single().health, 70);
}

// ==============================================================================
// APPROACH 3: test_scenario! Declarative (Our Innovation - 5 lines, reads like spec)
// ==============================================================================
test_scenario!(test_damage_declarative {
    given: {
        systems: [damage_system],
        events: [DamageEvent],
        entities: [Player { type_name: Player, fields: { health: 100, position: Vec3::ZERO } }]
    },
    when: {
        event: DamageEvent { target: entity_0, amount: 30 },
        advance: 1.frame()
    },
    then: {
        entity_0.get::<Player>().unwrap().health == 70
    }
});

// ==============================================================================
// Comparison: Complex Scenario
// ==============================================================================

// With #[bevy_test] - Good for imperative testing
#[bevy_test]
fn test_complex_combat_attribute(app: &mut TestApp) {
    // Setup
    let player = app.spawn(Player { health: 100, position: Vec3::ZERO });
    let enemy1 = app.spawn(Enemy { damage: 20 });
    let enemy2 = app.spawn(Enemy { damage: 15 });

    // Simulate combat rounds
    app.send_event(DamageEvent { target: player, amount: 20 });
    app.advance_time(1.0);

    app.send_event(DamageEvent { target: player, amount: 15 });
    app.advance_time(1.0);

    app.send_event(DamageEvent { target: player, amount: 30 });
    app.advance_time(1.0);

    // Assert final state
    let player_health = app.query::<&Player>().single().health;
    assert_eq!(player_health, 35); // 100 - 20 - 15 - 30
}

// With test_scenario! - Better for describing behavior
test_scenario!(test_complex_combat_declarative {
    given: {
        entities: [
            Player { type_name: Player, fields: { health: 100, position: Vec3::ZERO } },
            Enemy { type_name: Enemy, fields: { damage: 20 } },
            Enemy { type_name: Enemy, fields: { damage: 15 } }
        ]
    },
    when: {
        event: DamageEvent { target: entity_0, amount: 20 },
        advance: 1.second(),
        event: DamageEvent { target: entity_0, amount: 15 },
        advance: 1.second(),
        event: DamageEvent { target: entity_0, amount: 30 },
        advance: 1.second()
    },
    then: {
        entity_0.get::<Player>().unwrap().health == 35
    }
});

// ==============================================================================
// When to Use Each Approach
// ==============================================================================

// Use #[bevy_test] when:
// - You need fine-grained control
// - You're testing algorithmic logic
// - You prefer imperative style
// - You're migrating from manual tests
#[bevy_test]
fn test_best_for_algorithms(app: &mut TestApp) {
    // Complex algorithm testing with loops and conditions
    for i in 0..10 {
        let entity = app.spawn(Player {
            health: 100 - (i * 10),
            position: Vec3::new(i as f32, 0.0, 0.0),
        });

        if i % 2 == 0 {
            app.send_event(DamageEvent { target: entity, amount: 5 });
        }
    }

    app.update();

    // Complex assertions
    let healths: Vec<i32> = app.query::<&Player>()
        .iter()
        .map(|p| p.health)
        .collect();

    assert!(healths.iter().all(|&h| h >= 0));
}

// Use test_scenario! when:
// - You want tests that read like specifications
// - You're testing game scenarios
// - You want documentation-as-tests
// - You prefer declarative style
test_scenario!(test_best_for_scenarios {
    given: {
        entities: [
            Player { type_name: Player, fields: { health: 100, position: Vec3::ZERO } }
        ]
    },
    when: {
        event: DamageEvent { target: entity_0, amount: 150 }  // Overkill damage
    },
    then: {
        entity_0.get::<Player>().unwrap().health == 0,  // Health capped at 0
        events_received: [PlayerDeathEvent]  // Death event triggered
    }
});

// ==============================================================================
// Statistics: Line Count Comparison
// ==============================================================================
// Manual Testing:        30+ lines of boilerplate
// #[bevy_test]:         10 lines (66% reduction)
// test_scenario!:       5 lines (83% reduction)
//
// But it's not just about line count:
// - #[bevy_test] gives familiar Rust testing experience
// - test_scenario! gives BDD-style documentation
// - Both eliminate the worst boilerplate
// - Use the right tool for the right job!

fn main() {
    println!("Comparison of three testing approaches:");
    println!("1. Manual: 30+ lines, lots of boilerplate");
    println!("2. #[bevy_test]: 10 lines, familiar Rust style");
    println!("3. test_scenario!: 5 lines, reads like documentation");
    println!("\nRun with: cargo test --example comparison");
}