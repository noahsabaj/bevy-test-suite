//! Property-based testing example
//!
//! This example demonstrates using property_test! macro to verify
//! invariants hold for all possible inputs.

use bevy::prelude::*;
use bevy_test_suite::{bevy_test_utils, property_test};

bevy_test_utils!();

// Game mechanics to test
#[derive(Component, Debug, Clone, PartialEq)]
struct Health {
    current: i32,
    max: i32,
}

impl Health {
    fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    fn damage(&mut self, amount: i32) {
        self.current = (self.current - amount).max(0);
    }

    fn heal(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }

    fn is_alive(&self) -> bool {
        self.current > 0
    }
}

// Property: Health should always be between 0 and max
property_test!(health_bounds_invariant {
    given: {
        max_health: 1..=1000,
        damage_amounts: vec(0..=500, 0..10),
        heal_amounts: vec(0..=500, 0..10)
    },
    invariants: [
        "Health current is never negative",
        "Health current never exceeds max",
        "Health max remains unchanged"
    ]
});

// Complex game state invariants
#[derive(Component, Debug, Clone)]
struct Nation {
    treasury: i32,
    population: u32,
    happiness: f32,
    tax_rate: f32,
}

impl Nation {
    fn collect_taxes(&mut self) {
        let tax_income = (self.population as f32 * self.tax_rate) as i32;
        self.treasury += tax_income;
        self.happiness *= 1.0 - (self.tax_rate * 0.5); // Higher taxes reduce happiness
    }
}

// Property: Economic invariants
property_test!(nation_economy_invariants {
    given: {
        initial_treasury: -1000..=100000,
        population: 100..=1000000,
        tax_rate: 0.0..=1.0,
        iterations: 1..=100
    },
    invariants: [
        "Population never goes negative",
        "Tax rate stays between 0 and 1",
        "Happiness stays between 0 and 1",
        "Treasury changes are proportional to population and tax rate"
    ]
});

// Inventory system invariants
#[derive(Component, Debug)]
struct Inventory {
    items: Vec<Item>,
    max_weight: f32,
}

#[derive(Debug, Clone)]
struct Item {
    name: String,
    weight: f32,
    stackable: bool,
    quantity: u32,
}

impl Inventory {
    fn add_item(&mut self, item: Item) -> bool {
        let total_weight = self.current_weight() + item.weight * item.quantity as f32;
        if total_weight <= self.max_weight {
            self.items.push(item);
            true
        } else {
            false
        }
    }

    fn current_weight(&self) -> f32 {
        self.items
            .iter()
            .map(|i| i.weight * i.quantity as f32)
            .sum()
    }
}

// Property: Inventory weight constraints
property_test!(inventory_weight_invariant {
    given: {
        max_weight: 10.0..=1000.0,
        items: vec(Item::arbitrary(), 0..100)
    },
    invariants: [
        "Total weight never exceeds max_weight",
        "All item quantities are positive",
        "Stackable items combine correctly"
    ]
});

// Combat system properties
#[derive(Component)]
struct CombatStats {
    attack: i32,
    defense: i32,
    critical_chance: f32,
}

fn calculate_damage(attacker: &CombatStats, defender: &CombatStats) -> i32 {
    let base_damage = attacker.attack - defender.defense;
    base_damage.max(1) // Always deal at least 1 damage
}

// Property: Combat always produces valid results
property_test!(combat_damage_properties {
    given: {
        attacker_attack: 1..=1000,
        attacker_defense: 1..=1000,
        defender_attack: 1..=1000,
        defender_defense: 1..=1000
    },
    invariants: [
        "Damage is always at least 1",
        "Higher attack produces equal or higher damage",
        "Higher defense produces equal or lower damage taken"
    ]
});

// Demonstration of property test failure detection
property_test!(find_edge_cases {
    given: {
        values: vec(-1000..=1000, 100)
    },
    invariants: [
        "Sum of squares is always positive" // This will find the edge case where all values are 0
    ]
});

fn main() {
    println!("Property testing examples - these would be run with cargo test");
    println!("Property tests automatically generate hundreds of test cases");
    println!("They help find edge cases and ensure invariants hold");
}
