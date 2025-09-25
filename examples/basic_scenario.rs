//! Basic scenario testing example
//!
//! This example demonstrates how to use the test_scenario! macro for
//! integration testing with given/when/then structure.

use bevy::prelude::*;
use bevy_test_suite::{test_scenario, bevy_test_utils};

// Generate test utilities (MockWorld, MockInput, assertions)
bevy_test_utils!();

// Example components for testing
#[derive(Component, Debug, Clone, PartialEq)]
struct Player {
    health: i32,
    position: Vec3,
}

#[derive(Component, Debug, Clone)]
struct Enemy {
    damage: i32,
}

#[derive(Event)]
struct DamageEvent {
    target: Entity,
    amount: i32,
}

#[derive(Event)]
struct PlayerDeathEvent;

// Example system to test
fn damage_system(
    mut damage_events: EventReader<DamageEvent>,
    mut players: Query<&mut Player>,
    mut death_events: EventWriter<PlayerDeathEvent>,
) {
    for event in damage_events.read() {
        if let Ok(mut player) = players.get_mut(event.target) {
            player.health -= event.amount;
            if player.health <= 0 {
                death_events.send(PlayerDeathEvent);
            }
        }
    }
}

// Declarative test using test_scenario!
test_scenario!(player_takes_damage {
    given: {
        resources: [Time::default()],
        systems: [damage_system],
        events: [DamageEvent, PlayerDeathEvent],
        entities: [
            Player {
                type_name: Player,
                fields: {
                    health: 100,
                    position: Vec3::ZERO
                }
            }
        ]
    },
    when: {
        event: DamageEvent { target: entity_0, amount: 30 },
        advance: 1.frame()
    },
    then: {
        entity_0.get::<Player>().unwrap().health == 70,
        events_received: []
    }
});

// Test player death scenario
test_scenario!(player_dies_from_damage {
    given: {
        resources: [Time::default()],
        systems: [damage_system],
        events: [DamageEvent, PlayerDeathEvent],
        entities: [
            Player {
                type_name: Player,
                fields: {
                    health: 20,
                    position: Vec3::ZERO
                }
            }
        ]
    },
    when: {
        event: DamageEvent { target: entity_0, amount: 50 },
        advance: 1.frame()
    },
    then: {
        entity_0.get::<Player>().unwrap().health <= 0,
        events_received: [PlayerDeathEvent]
    }
});

// Test time-based scenario
test_scenario!(health_regeneration_over_time {
    given: {
        resources: [Time::default()],
        systems: [regen_system],
        entities: [
            Player {
                type_name: Player,
                fields: {
                    health: 50,
                    position: Vec3::ZERO
                }
            }
        ]
    },
    when: {
        advance: 5.seconds()
    },
    then: {
        entity_0.get::<Player>().unwrap().health > 50
    }
});

fn regen_system(
    time: Res<Time>,
    mut players: Query<&mut Player>,
) {
    for mut player in &mut players {
        // Regenerate 5 health per second
        player.health = (player.health + (5.0 * time.delta_secs()) as i32).min(100);
    }
}

fn main() {
    println!("Run with: cargo test --example basic_scenario");
}