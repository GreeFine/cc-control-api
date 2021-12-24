use crate::{
    functions::{go_to_position_orders, resume_or_create_plot_oders},
    utils::{Direction, Position},
    MiningPlots,
};

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Display, PartialEq, EnumString, Deserialize, Serialize)]
pub enum CommandName {
    Up,
    Down,
    Left,
    Right,
    Forward,
    // UpDig,
    // DownDig,
    ForwardDig,
    Sleep,
    Reboot,
    RefuelCheck,
    // Functions
    Home,
    MinePlot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    name: CommandName,
    argument: i32,
}

impl Command {
    pub fn new(name: CommandName, argument: i32) -> Self {
        Command { name, argument }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct Turtle {
    name: String,
    pub orders: Vec<Command>,
    pub infos: HashMap<String, String>,
    pos: Position,
    direction: Direction,
}

#[allow(dead_code)]
const IN_WORLD_DEFAULT_POSITION: Position = Position {
    x: -547,
    y: 63,
    z: -2615,
};
pub const IN_WORLD_CHEST_POSITION: Position = Position {
    x: -559,
    y: 63,
    z: -2767,
};
#[allow(dead_code)]
const IN_WORLD_SKY_POSITON: i32 = 73;

#[cfg(test)]
mod tests {
    use crate::{
        functions::go_to_position_orders,
        persistance,
        utils::{Direction, Position},
    };
    use std::collections::HashMap;

    use super::Turtle;
    #[actix_web::test]
    async fn test_go_to_order() {
        let (_, db_mining_plots) = persistance::connect().await;
        let pos = Position { x: 0, y: 0, z: 0 };

        let mut turtle = Turtle {
            pos,
            direction: Direction::North,
            name: "test".to_string(),
            orders: Vec::new(),
            infos: HashMap::new(),
        };
        let gotopos = Position { x: 4, y: 0, z: 0 };
        let gotoorders = go_to_position_orders(
            &mut turtle.pos.clone(),
            &mut Direction::North,
            &gotopos,
            &Direction::North,
        )
        .unwrap();
        println!("debug: orders= {:#?}", gotoorders);
        turtle.orders = gotoorders;

        turtle.orders(&db_mining_plots).await;
        assert_eq!(turtle.pos, gotopos);
        assert_eq!(turtle.direction, Direction::North);
    }
}

impl Turtle {
    pub fn default(name: String) -> Self {
        Turtle {
            orders: vec![Command::new(CommandName::Sleep, 2)],
            infos: HashMap::new(),
            pos: IN_WORLD_CHEST_POSITION,
            direction: Direction::North,
            name,
        }
    }

    #[allow(dead_code)]
    pub fn get_position(&self) -> String {
        format!("Position: {:#?}, Direction: {}", &self.pos, &self.direction)
    }

    #[allow(dead_code)]
    fn go_in_the_sky_orders(&mut self) -> Option<Command> {
        if self.pos.y < IN_WORLD_SKY_POSITON {
            Some(Command::new(
                CommandName::Up,
                IN_WORLD_SKY_POSITON - self.pos.y,
            ))
        } else {
            None
        }
    }

    pub async fn orders(&mut self, mining_plots: &MiningPlots) -> String {
        let mut result: Vec<String> = Vec::new();

        // We use temporary position for functions, but we are re-calculating position and stuff later
        let mut temporary_position = self.pos;
        let mut temporary_direction = self.direction.clone();
        let mut orders = if let Some(fuellevel) = self.infos.get("fuellevel") {
            let fuelvalue: i32 = fuellevel.parse().unwrap();
            if fuelvalue < 500 {
                println!("Turtle {} as low fuel !", self.name);
                go_to_position_orders(
                    &mut temporary_position,
                    &mut temporary_direction,
                    &IN_WORLD_CHEST_POSITION,
                    &Direction::North,
                )
                .unwrap_or_default()
            } else {
                self.orders.clone()
            }
        } else {
            self.orders.clone()
        };

        let mut indexs_to_replace = Vec::new();
        for (index, Command { name, argument: _ }) in orders.iter().enumerate() {
            match name {
                CommandName::Home => {
                    let orders = go_to_position_orders(
                        &mut temporary_position,
                        &mut temporary_direction,
                        &IN_WORLD_CHEST_POSITION,
                        &Direction::North,
                    );
                    indexs_to_replace.push((index, orders));
                }
                CommandName::MinePlot => {
                    let oders = resume_or_create_plot_oders(
                        &mut temporary_position,
                        &mut temporary_direction,
                        &self.name,
                        mining_plots,
                    )
                    .await;
                    indexs_to_replace.push((index, Some(oders)));
                }
                _ => {}
            }
        }

        let mut added_indexs = 0;
        for (index, commands) in indexs_to_replace {
            orders.remove(index + added_indexs);
            if let Some(mut commands) = commands {
                commands.reverse();
                let commands_size = commands.len();
                for command in commands {
                    orders.insert(index + added_indexs, command);
                }
                added_indexs += commands_size - 1;
            }
        }
        for Command { name, argument } in orders {
            let argument = argument as i32;
            let order_str = match name {
                CommandName::Up => {
                    self.pos.y += argument;
                    format!("{}({})", name, argument)
                }
                CommandName::Down => {
                    self.pos.y -= argument;
                    format!("{}({})", name, argument)
                }
                CommandName::Forward | CommandName::ForwardDig => {
                    match self.direction {
                        Direction::North => self.pos.z -= argument,
                        Direction::East => self.pos.x += argument,
                        Direction::South => self.pos.z += argument,
                        Direction::West => self.pos.x -= argument,
                    };
                    format!("{}({})", name, argument)
                }
                CommandName::Right => {
                    let u8facing: usize = self.direction.clone() as usize;
                    self.direction =
                        Direction::from_repr((u8facing + argument as usize) % 4).unwrap();
                    format!("{}({})", name, argument)
                }
                CommandName::Left => {
                    let u8facing: usize = self.direction.clone() as usize;
                    let newfacing = if u8facing == 0 {
                        3
                    } else {
                        (u8facing - argument as usize) % 4
                    };
                    self.direction = Direction::from_repr(newfacing).unwrap();
                    format!("{}({})", name, argument)
                }
                CommandName::Reboot | CommandName::Sleep | CommandName::RefuelCheck => {
                    format!("{}({})", name, argument)
                }
                _ => {
                    panic!("Order process missing: {}", name)
                }
            };
            result.push(order_str);
        }
        self.orders = vec![Command::new(CommandName::Sleep, 2)];
        result.join("\n")
    }
}
