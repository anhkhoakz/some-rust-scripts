use bracket_lib::prelude::*;

struct State {}

struct Weapon {
    name: &'static str,
    damage: i32,
    durability: i32,
}

struct Monster {
    name: &'static str,
    health: i32,
    damage: i32,
    level: i32,
    gold: i32,
}

struct Player {
    name: &'static str,
    health: i32,
    damage: i32,
    level: i32,
    gold: i32,
    weapon: Weapon,
}

struct Location {
    x: i32,
    y: i32,
}

const WEAPONS: [Weapon; 4] = [
    Weapon {
        name: "Stick",
        damage: 5,
        durability: 10,
    },
    Weapon {
        name: "Dagger",
        damage: 30,
        durability: 15,
    },
    Weapon {
        name: "Claw Hammer",
        damage: 50,
        durability: 12,
    },
    Weapon {
        name: "Sword",
        damage: 100,
        durability: 20,
    },
];

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(10, "RGB Roguelike with bracket-lib!");
        ctx.print_centered(12, "Press ESC to quit.");
    }
}

fn main() {
    let context: BTerm = BTermBuilder::simple80x50()
        .with_title("RGB Roguelike")
        .build()
        .unwrap();
    main_loop(context, State {}).unwrap();
}
