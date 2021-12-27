use crate::mining_plots::{
    mining_orders, new_mining_position, MiningPlot, PLOT_DEPTH, PLOT_DIRECTION,
    PLOT_MAX_DEPTH_SEGMENT,
};
use crate::{
    utils::{Direction, Position},
    MiningPlots,
};
use log::debug;
use mongodb::bson::{self, doc};
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
    DepositItem,
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
    pub pos: Position,
    pub direction: Direction,
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
        let gotoorders = turtle
            .go_to_position_orders(&gotopos, &Direction::North)
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

    fn rotate_to(&self, current: &mut Direction, wanted: Direction) -> Option<Command> {
        if wanted == *current {
            return None;
        }
        let diff_facing = wanted.clone() as i32 - current.clone() as i32;
        *current = wanted;
        if diff_facing > 0 {
            Some(Command::new(CommandName::Right, diff_facing))
        } else {
            Some(Command::new(CommandName::Left, diff_facing.abs()))
        }
    }

    pub fn go_to_position_orders(
        &self,
        destination: &Position,
        destination_direction: &Direction,
    ) -> Option<Vec<Command>> {
        if self.pos == *destination && self.direction == *destination_direction {
            return None;
        }
        let mut orders_to_pos = Vec::new();
        // Don't go flying when you are near
        // if (self.pos.x - destination.y).abs() + (self.pos.x - destination.y).abs() > (PLOT_SIZE * 2) {
        //     if let Some(orders) = self.go_in_the_sky_orders() {
        //         orders_to_pos.push(orders);
        //     }
        // }
        let mut tmp_direction = self.direction.clone();
        let pos_diff = self.pos - *destination;
        if pos_diff.x > 0 {
            if let Some(orders) = self.rotate_to(&mut tmp_direction, Direction::West) {
                orders_to_pos.push(orders);
            }
        } else if let Some(orders) = self.rotate_to(&mut tmp_direction, Direction::East) {
            orders_to_pos.push(orders);
        }
        orders_to_pos.push(Command::new(CommandName::Forward, pos_diff.x.abs()));
        if pos_diff.z > 0 {
            if let Some(orders) = self.rotate_to(&mut tmp_direction, Direction::North) {
                orders_to_pos.push(orders);
            }
        } else if let Some(orders) = self.rotate_to(&mut tmp_direction, Direction::South) {
            orders_to_pos.push(orders);
        }
        orders_to_pos.push(Command::new(CommandName::Forward, pos_diff.z.abs()));
        let command = if pos_diff.y > 0 {
            CommandName::Down
        } else {
            CommandName::Up
        };
        orders_to_pos.push(Command::new(command, pos_diff.y.abs()));
        if let Some(orders) = self.rotate_to(&mut tmp_direction, destination_direction.clone()) {
            orders_to_pos.push(orders);
        }
        Some(orders_to_pos)
    }

    fn mine_plot_orders(&self, mining_plot: MiningPlot) -> Vec<Command> {
        let mut result = Vec::new();
        let mut current_plot_position_w_depth = mining_plot.position;
        current_plot_position_w_depth.y -= (mining_plot.mined_depth_segment * PLOT_DEPTH) as i32;
        if let Some(mut orders) =
            self.go_to_position_orders(&current_plot_position_w_depth, &PLOT_DIRECTION)
        {
            result.append(&mut orders);
        }
        result.append(&mut mining_orders());
        result
    }

    pub async fn resume_or_create_plot_oders(
        &mut self,
        mining_plots: &MiningPlots,
    ) -> Vec<Command> {
        let current_plot = mining_plots
            .find_one(doc! {"current_turtle": &self.name}, None)
            .await
            .expect("Unable to find_one from mining plots");
        if let Some(mut current_plot) = current_plot {
            if current_plot.mined_depth_segment < PLOT_MAX_DEPTH_SEGMENT {
                current_plot.mined_depth_segment += 1;
                mining_plots
                .update_one(
                    doc! {"current_turtle": &self.name},
                    doc! { "$set": { "mined_depth_segment": current_plot.mined_depth_segment } },
                    None,
                )
                .await
                .expect("Unable to update mined_depth_segment in plot");
                return self.mine_plot_orders(current_plot);
            } else {
                mining_plots
                .update_one(
                    doc! {"current_turtle": &self.name},
                    doc! { "$set": { "current_turtle": bson::to_bson(&None::<String>).unwrap() } },
                    None,
                )
                .await
                .expect("Unable to remove clear name in plot");
            }
        };
        let document_count = mining_plots
            .count_documents(None, None)
            .await
            .expect("Unable to get document count for mining_plots");
        let new_plot_position = new_mining_position(document_count as i32);
        let new_plot = MiningPlot::new(new_plot_position, &self.name);
        mining_plots
            .insert_one(&new_plot, None)
            .await
            .expect("Unable to add new mining plot");
        self.mine_plot_orders(new_plot)
    }

    pub async fn orders(&mut self, mining_plots: &MiningPlots) -> String {
        let mut result: Vec<String> = Vec::new();

        let mut orders = None;
        if let Some(fuellevel) = self.infos.get("fuellevel") {
            let fuelvalue: i32 = fuellevel.parse().unwrap();
            debug!("fuelvalue: {}", fuelvalue);
            if fuelvalue < 500 {
                println!("Turtle {} as low fuel !", self.name);
                orders = Some(
                    self.go_to_position_orders(&IN_WORLD_CHEST_POSITION, &Direction::North)
                        .unwrap_or_else(|| vec![Command::new(CommandName::Sleep, 2)]),
                );
            }
        } else if let Some(is_full) = self.infos.get("isFull") {
            if is_full == "true" {
                let mut result = self
                    .go_to_position_orders(&IN_WORLD_CHEST_POSITION, &Direction::North)
                    .unwrap_or_else(|| vec![Command::new(CommandName::Sleep, 2)]);
                let mut tmp_direction = self.direction.clone();
                if let Some(order) = self.rotate_to(&mut tmp_direction, Direction::West) {
                    result.push(order);
                }
                result.push(Command::new(CommandName::DepositItem, 1));
                if let Some(order) = self.rotate_to(&mut tmp_direction, Direction::East) {
                    result.push(order);
                }
                result.push(Command::new(CommandName::DepositItem, 1));
                orders = Some(result);
            };
        };
        if orders.is_none() {
            orders = Some(self.orders.clone())
        };

        for Command { name, argument } in orders.unwrap() {
            let sub_orders: Option<Vec<_>> = match name {
                CommandName::Home => {
                    self.go_to_position_orders(&IN_WORLD_CHEST_POSITION, &Direction::North)
                }
                CommandName::MinePlot => Some(self.resume_or_create_plot_oders(mining_plots).await),
                _ => Some(vec![Command { name, argument }]),
            };
            if let Some(sub_orders) = sub_orders {
                for Command { name, argument } in sub_orders {
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
            }
        }
        self.orders = vec![Command::new(CommandName::MinePlot, 1)];
        result.join("\n")
    }
}
