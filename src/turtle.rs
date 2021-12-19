use std::{collections::HashMap, ops::Sub};

use strum_macros::{Display, EnumString, FromRepr};

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

#[derive(Display, FromRepr, Clone, PartialEq)]
pub enum Facing {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

#[derive(Clone, Display, PartialEq, EnumString)]
pub enum Command {
    Up,
    Down,
    Left,
    Right,
    Forward,
    Sleep,
    Reboot,
    RefuelCheck,
    Home,
}

#[derive(Clone)]
pub struct Turtle {
    pub orders: Vec<(Command, u32)>,
    pub infos: HashMap<String, String>,
    pos: Position,
    facing: Facing,
}

const IN_WORLD_DEFAULT_POSITION: Position = Position {
    x: -547,
    y: 63,
    z: -2615,
};
const IN_WORLD_SKY_POSITON: i32 = 73;

impl Turtle {
    pub fn default() -> Self {
        Turtle {
            orders: vec![(Command::Sleep, 2)],
            infos: HashMap::new(),
            pos: IN_WORLD_DEFAULT_POSITION,
            facing: Facing::North,
        }
    }
    pub fn get_position(&self) -> String {
        format!("Position: {:#?}, Facing: {}", &self.pos, &self.facing)
    }

    fn go_in_the_sky_orders(&mut self) -> Option<String> {
        if self.pos.y < IN_WORLD_SKY_POSITON {
            let result = Some(format!("Up({})\n", IN_WORLD_SKY_POSITON - self.pos.y));
            self.pos.y = IN_WORLD_SKY_POSITON;
            result
        } else {
            None
        }
    }

    fn rotate_to(&mut self, facing: Facing) -> Option<String> {
        if facing == self.facing {
            return None;
        }
        let mut result = String::new();
        let diff_facing = facing.clone() as i32 - self.facing.clone() as i32;
        if diff_facing > 0 {
            result.push_str(&*format!("Right({})\n", diff_facing))
        } else {
            result.push_str(&*format!("Left({})\n", diff_facing.abs()))
        }
        self.facing = facing;
        Some(result)
    }

    pub fn go_to_position_orders(&mut self, position: Position, facing: Facing) -> String {
        let mut orders_to_home = String::new();
        if let Some(orders) = self.go_in_the_sky_orders() {
            orders_to_home.push_str(&*orders);
        }
        let home_pos_diff = self.pos - position;
        if home_pos_diff.x > 0 {
            if let Some(orders) = self.rotate_to(Facing::West) {
                orders_to_home.push_str(&*orders);
            }
        } else if let Some(orders) = self.rotate_to(Facing::East) {
            orders_to_home.push_str(&*orders);
        }
        orders_to_home.push_str(&*format!("Forward({})\n", home_pos_diff.x.abs()));
        if home_pos_diff.z > 0 {
            if let Some(orders) = self.rotate_to(Facing::North) {
                orders_to_home.push_str(&*orders);
            }
        } else if let Some(orders) = self.rotate_to(Facing::South) {
            orders_to_home.push_str(&*orders);
        }
        orders_to_home.push_str(&*format!("Forward({})\n", home_pos_diff.z.abs()));
        orders_to_home.push_str(&*format!("Down({})\n", home_pos_diff.y));
        if let Some(orders) = self.rotate_to(facing) {
            orders_to_home.push_str(&*orders);
        }
        log::debug!("diff: {:#?}, result: {}", home_pos_diff, orders_to_home);
        self.pos = position;
        orders_to_home.trim_end_matches('\n').to_string()
    }

    pub fn orders(&mut self) -> String {
        let mut result: Vec<String> = Vec::new();
        let orders = self.orders.clone();
        for (order, x) in orders {
            let x_ = x as i32;
            let order_str = match order {
                Command::Up => {
                    self.pos.y += x_;
                    format!("{}({})", order, x)
                }
                Command::Down => {
                    self.pos.y -= x_;
                    format!("{}({})", order, x)
                }
                Command::Right => {
                    let u8facing: usize = self.facing.clone() as usize;
                    self.facing = Facing::from_repr((u8facing + 1) % 4).unwrap();
                    format!("{}({})", order, x)
                }
                Command::Left => {
                    let u8facing: usize = self.facing.clone() as usize;
                    let newfacing = if u8facing == 0 { 3 } else { (u8facing - 1) % 4 };
                    self.facing = Facing::from_repr(newfacing).unwrap();
                    format!("{}({})", order, x)
                }
                Command::Forward => {
                    match self.facing {
                        Facing::North => self.pos.z -= x_,
                        Facing::East => self.pos.x += x_,
                        Facing::South => self.pos.z += x_,
                        Facing::West => self.pos.x -= x_,
                    };
                    format!("{}({})", order, x)
                }
                Command::Home => {
                    self.go_to_position_orders(IN_WORLD_DEFAULT_POSITION, Facing::North)
                }
                Command::Reboot | Command::Sleep | Command::RefuelCheck => {
                    format!("{}({})", order, x)
                }
            };
            result.push(order_str);
        }
        self.orders = vec![(Command::Sleep, 2)];
        result.join("\n")
    }
}
