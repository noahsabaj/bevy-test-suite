//! Dual approach example - showing both #[bevy_test] and declarative testing
//!
//! This example demonstrates when to use each testing approach and how they
//! complement each other.

use bevy::prelude::*;
use bevy_test_suite::{test_scenario, test_system, bevy_test, bevy_test_utils};

// Generate test utilities
bevy_test_utils!();

// ============================================================================
// APPROACH 1: #[bevy_test] Attribute (What the Community Asked For)
// ============================================================================
// Best for: Simple unit tests, familiar syntax, quick assertions

/// Simple test using the #[bevy_test] attribute
#[bevy_test]
fn test_entity_spawning(app: &mut TestApp) {
    // Spawn an entity
    let player = app.spawn((
        Name::new("Player"),
        Transform::default(),
        GlobalTransform::default(),
    ));

    // Verify it exists
    assert!(app.world().entities().contains(player));
    assert_eq!(app.query::<&Name>().single().as_str(), "Player");
}

/// Test with time advancement
#[bevy_test]
fn test_movement_with_time(app: &mut TestApp) {
    // Setup
    let entity = app.spawn((
        Position(Vec3::ZERO),
        Velocity(Vec3::X * 10.0),
    ));

    // Add movement system
    app.add_systems(Update, movement_system);

    // Advance time
    app.advance_time(1.0);

    // Check position changed
    let position = app.world().entity(entity).get::<Position>().unwrap();
    assert!(position.0.x > 0.0);
}

/// Test with headless configuration for CI
#[bevy_test(headless)]
fn test_for_ci(app: &mut TestApp) {
    // This runs without GPU/window requirements
    app.insert_resource(GameSettings::default());
    assert!(app.world().contains_resource::<GameSettings>());
}

/// Test with custom plugins
#[bevy_test(plugins = [TransformPlugin, HierarchyPlugin])]
fn test_with_plugins(app: &mut TestApp) {
    let parent = app.spawn(Transform::default());
    let child = app.spawn(Transform::default());

    // Test hierarchy operations
    app.world_mut().entity_mut(child).set_parent(parent);

    // Verify relationship
    assert_parent_child!(app, parent, child);
}

// ============================================================================
// APPROACH 2: Declarative Testing (Our Innovation)
// ============================================================================
// Best for: Complex scenarios, integration tests, readable specifications

/// Complex integration test with given/when/then
test_scenario!(combat_scenario {
    given: {
        resources: [Time::default(), CombatSettings::default()],
        systems: [damage_system, death_system],
        events: [DamageEvent, DeathEvent],
        entities: [
            Player {
                type_name: Player,
                fields: {
                    health: 100,
                    defense: 10
                }
            },
            Enemy {
                type_name: Enemy,
                fields: {
                    attack: 30
                }
            }
        ]
    },
    when: {
        event: DamageEvent {
            attacker: entity_1,
            target: entity_0,
            base_damage: 30
        },
        advance: 0.5.seconds()
    },
    then: {
        // 30 damage - 10 defense = 20 damage taken
        entity_0.get::<Player>().unwrap().health == 80,
        events_received: []
    }
});

/// Test with multiple time steps
test_scenario!(regeneration_over_time {
    given: {
        resources: [Time::default()],
        systems: [regeneration_system],
        entities: [
            Player {
                type_name: Player,
                fields: {
                    health: 50,
                    max_health: 100,
                    regen_rate: 5
                }
            }
        ]
    },
    when: {
        advance: 3.seconds()
    },
    then: {
        // 3 seconds * 5 health/second = 15 health gained
        entity_0.get::<Player>().unwrap().health == 65
    }
});

/// Testing system in isolation
test_system!(economy_system_test {
    setup: {
        resources: [Time::default(), EconomySettings { tax_rate: 0.2 }],
        nation: Nation { treasury: 1000, population: 100 }
    },
    call: collect_taxes_system,
    expect: {
        // 100 population * 0.2 tax rate = 20 gold collected
        app.world().entity(nation).get::<Nation>().unwrap().treasury == 1020
    }
});

// ============================================================================
// CHOOSING THE RIGHT APPROACH
// ============================================================================

/*
When to use #[bevy_test]:
- Simple unit tests
- Testing individual functions or systems
- You want familiar Rust test syntax
- Quick smoke tests
- CI/CD pipeline tests
- You're migrating from manual tests

When to use declarative (test_scenario!):
- Complex integration scenarios
- Multi-step workflows
- Tests that should document behavior
- Testing emergent behavior
- Time-based simulations
- Tests with rich setup requirements

Both approaches can be used in the same test suite!
*/

// ============================================================================
// COMPARISON: Same Test, Both Approaches
// ============================================================================

// Approach 1: Using #[bevy_test]
#[bevy_test]
fn test_health_potion_imperative(app: &mut TestApp) {
    // Manual setup
    let player = app.spawn(Player {
        health: 50,
        max_health: 100,
        inventory: vec![Item::HealthPotion]
    });

    // Add systems
    app.add_systems(Update, use_item_system);
    app.add_event::<UseItemEvent>();

    // Send event
    app.send_event(UseItemEvent {
        entity: player,
        item: Item::HealthPotion,
    });

    // Update
    app.update();

    // Assert
    let player_health = app.world()
        .entity(player)
        .get::<Player>()
        .unwrap()
        .health;
    assert_eq!(player_health, 100);
}

// Approach 2: Using test_scenario!
test_scenario!(test_health_potion_declarative {
    given: {
        systems: [use_item_system],
        events: [UseItemEvent],
        entities: [
            Player {
                type_name: Player,
                fields: {
                    health: 50,
                    max_health: 100,
                    inventory: vec![Item::HealthPotion]
                }
            }
        ]
    },
    when: {
        event: UseItemEvent {
            entity: entity_0,
            item: Item::HealthPotion
        }
    },
    then: {
        entity_0.get::<Player>().unwrap().health == 100,
        entity_0.get::<Player>().unwrap().inventory.is_empty()
    }
});

// ============================================================================
// SHARED TEST COMPONENTS AND SYSTEMS
// ============================================================================

#[derive(Component, Debug, Clone, PartialEq)]
struct Player {
    health: i32,
    max_health: i32,
    defense: i32,
    inventory: Vec<Item>,
    regen_rate: i32,
}

#[derive(Component, Debug, Clone)]
struct Enemy {
    attack: i32,
}

#[derive(Component, Debug, Clone)]
struct Nation {
    treasury: i32,
    population: u32,
}

#[derive(Component, Debug, Clone, PartialEq)]
struct Position(Vec3);

#[derive(Component, Debug, Clone)]
struct Velocity(Vec3);

#[derive(Debug, Clone, PartialEq)]
enum Item {
    HealthPotion,
}

#[derive(Event)]
struct DamageEvent {
    attacker: Entity,
    target: Entity,
    base_damage: i32,
}

#[derive(Event)]
struct DeathEvent {
    entity: Entity,
}

#[derive(Event)]
struct UseItemEvent {
    entity: Entity,
    item: Item,
}

#[derive(Resource, Default)]
struct GameSettings;

#[derive(Resource, Default)]
struct CombatSettings;

#[derive(Resource)]
struct EconomySettings {
    tax_rate: f32,
}

// Example systems
fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    for (mut pos, vel) in &mut query {
        pos.0 += vel.0 * time.delta_secs();
    }
}

fn damage_system(
    mut events: EventReader<DamageEvent>,
    mut players: Query<&mut Player>,
    mut death_events: EventWriter<DeathEvent>,
) {
    for event in events.read() {
        if let Ok(mut player) = players.get_mut(event.target) {
            let damage = event.base_damage - player.defense;
            player.health -= damage.max(0);

            if player.health <= 0 {
                death_events.send(DeathEvent { entity: event.target });
            }
        }
    }
}

fn death_system(
    mut commands: Commands,
    mut events: EventReader<DeathEvent>,
) {
    for event in events.read() {
        commands.entity(event.entity).despawn();
    }
}

fn regeneration_system(
    time: Res<Time>,
    mut players: Query<&mut Player>,
) {
    for mut player in &mut players {
        let regen = (player.regen_rate as f32 * time.delta_secs()) as i32;
        player.health = (player.health + regen).min(player.max_health);
    }
}

fn collect_taxes_system(
    settings: Res<EconomySettings>,
    mut nations: Query<&mut Nation>,
) {
    for mut nation in &mut nations {
        let tax = (nation.population as f32 * settings.tax_rate) as i32;
        nation.treasury += tax;
    }
}

fn use_item_system(
    mut events: EventReader<UseItemEvent>,
    mut players: Query<&mut Player>,
) {
    for event in events.read() {
        if let Ok(mut player) = players.get_mut(event.entity) {
            if let Some(index) = player.inventory.iter().position(|i| i == &event.item) {
                player.inventory.remove(index);

                match event.item {
                    Item::HealthPotion => {
                        player.health = player.max_health;
                    }
                }
            }
        }
    }
}

fn main() {
    println!("Run tests with: cargo test --example dual_approach");
    println!("\nThis example shows both testing approaches:");
    println!("1. #[bevy_test] - The familiar attribute macro the community wants");
    println!("2. test_scenario! - Our declarative approach for complex scenarios");
    println!("\nBoth approaches work together in the same test suite!");
}