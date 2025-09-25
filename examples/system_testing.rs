//! System testing example
//!
//! This example demonstrates testing individual systems in isolation
//! using the test_system! macro.

use bevy::prelude::*;
use bevy_test_suite::{bevy_test_utils, test_system};

bevy_test_utils!();

// Components for testing
#[derive(Component, Debug, Clone, PartialEq)]
struct Position(Vec3);

#[derive(Component, Debug, Clone, PartialEq)]
struct Velocity(Vec3);

#[derive(Resource, Default)]
struct GameSpeed(f32);

// System to test
fn movement_system(
    time: Res<Time>,
    speed: Res<GameSpeed>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    for (mut pos, vel) in &mut query {
        pos.0 += vel.0 * time.delta_secs() * speed.0;
    }
}

// Test the movement system
test_system!(test_basic_movement {
    setup: {
        resources: [Time::default(), GameSpeed(1.0)],
        player: (Position(Vec3::ZERO), Velocity(Vec3::X * 10.0))
    },
    call: movement_system,
    expect: {
        // After one update, position should have moved
        app.world().entity(player).get::<Position>().unwrap().0.x > 0.0
    }
});

// Test with multiple entities
test_system!(test_multiple_entities_movement {
    setup: {
        resources: [Time::default(), GameSpeed(2.0)],
        entity1: (Position(Vec3::ZERO), Velocity(Vec3::X * 5.0)),
        entity2: (Position(Vec3::ZERO), Velocity(Vec3::Y * 5.0))
    },
    call: movement_system,
    expect: {
        app.world().entity(entity1).get::<Position>().unwrap().0.x > 0.0,
        app.world().entity(entity2).get::<Position>().unwrap().0.y > 0.0
    }
});

// Test resource interactions
fn score_system(mut score: ResMut<Score>, enemies: Query<Entity, With<Enemy>>) {
    score.0 = enemies.iter().count() as u32 * 100;
}

#[derive(Resource, Default, Debug, PartialEq)]
struct Score(u32);

#[derive(Component)]
struct Enemy;

test_system!(test_score_calculation {
    setup: {
        resources: [Score::default()],
        enemy1: Enemy,
        enemy2: Enemy,
        enemy3: Enemy
    },
    call: score_system,
    expect: {
        *app.world().resource::<Score>() == Score(300)
    }
});

// Test event handling
#[derive(Event)]
struct CollisionEvent {
    entity_a: Entity,
    entity_b: Entity,
}

fn collision_handler(mut events: EventReader<CollisionEvent>, mut commands: Commands) {
    for event in events.read() {
        // Mark entities as collided
        commands.entity(event.entity_a).insert(Collided);
        commands.entity(event.entity_b).insert(Collided);
    }
}

#[derive(Component)]
struct Collided;

test_system!(test_collision_handling {
    setup: {
        events: [CollisionEvent],
        ball: Position(Vec3::ZERO),
        wall: Position(Vec3::X)
    },
    call: collision_handler,
    expect: {
        // After sending collision event, both entities should be marked
        app.world_mut().send_event(CollisionEvent {
            entity_a: ball,
            entity_b: wall
        });
        app.update(); // Process the event

        app.world().entity(ball).contains::<Collided>() &&
        app.world().entity(wall).contains::<Collided>()
    }
});

fn main() {
    println!("Run with: cargo test --example system_testing");
}
