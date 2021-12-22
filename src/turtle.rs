use crate::{
    mining_plots::{
        mining_orders, new_mining_position, MiningPlot, PLOT_FACING, PLOT_MAX_DEPTH_SEGMENT,
        PLOT_SIZE,
    },
    utils::{Facing, Position},
    MiningPlots,
};

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
    argument: u32,
}

impl Command {
    pub fn new(name: CommandName, argument: u32) -> Self {
        Command { name, argument }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct Turtle {
    name: String,
    pub orders: Vec<Command>,
    pub infos: HashMap<String, String>,
    pos: Position,
    facing: Facing,
}

#[allow(dead_code)]
const IN_WORLD_DEFAULT_POSITION: Position = Position {
    x: -547,
    y: 63,
    z: -2615,
};
const IN_WORLD_CHEST_POSITION: Position = Position {
    x: -559,
    y: 63,
    z: -2767,
};
const IN_WORLD_SKY_POSITON: i32 = 73;

impl Turtle {
    pub fn default(name: String) -> Self {
        Turtle {
            orders: vec![Command::new(CommandName::Sleep, 2)],
            infos: HashMap::new(),
            pos: IN_WORLD_CHEST_POSITION,
            facing: Facing::North,
            name,
        }
    }

    #[allow(dead_code)]
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
        if self.pos == position && self.facing == facing {
            return String::new();
        }
        let mut orders_to_pos = String::new();
        // Don't go flying with you are near
        if (self.pos.x - position.y).abs() + (self.pos.x - position.y).abs() > (PLOT_SIZE * 2) {
            if let Some(orders) = self.go_in_the_sky_orders() {
                orders_to_pos.push_str(&*orders);
            }
        }
        let pos_diff = self.pos - position;
        if pos_diff.x > 0 {
            if let Some(orders) = self.rotate_to(Facing::West) {
                orders_to_pos.push_str(&*orders);
            }
        } else if let Some(orders) = self.rotate_to(Facing::East) {
            orders_to_pos.push_str(&*orders);
        }
        orders_to_pos.push_str(&*format!("Forward({})\n", pos_diff.x.abs()));
        if pos_diff.z > 0 {
            if let Some(orders) = self.rotate_to(Facing::North) {
                orders_to_pos.push_str(&*orders);
            }
        } else if let Some(orders) = self.rotate_to(Facing::South) {
            orders_to_pos.push_str(&*orders);
        }
        orders_to_pos.push_str(&*format!("Forward({})\n", pos_diff.z.abs()));
        let verticality = if pos_diff.y > 0 { "Down" } else { "Up " };
        orders_to_pos.push_str(&*format!("{}({})\n", verticality, pos_diff.y));
        if let Some(orders) = self.rotate_to(facing) {
            orders_to_pos.push_str(&*orders);
        }
        self.pos = position;
        orders_to_pos.trim_end_matches('\n').to_string()
    }

    fn mine_plot_orders(&mut self, mining_plot: MiningPlot) -> String {
        let mut current_plot_position_w_depth = mining_plot.position;
        current_plot_position_w_depth.y -= (mining_plot.mined_depth_segment * 2) as i32;
        let mut result = self.go_to_position_orders(current_plot_position_w_depth, PLOT_FACING);
        result.push('\n');
        result.push_str(&*mining_orders());
        result
    }

    async fn resume_or_create_plot_oders(&mut self, mining_plots: &MiningPlots) -> String {
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
        if let Some(fuellevel) = self.infos.get("fuellevel") {
            let fuelvalue: i32 = fuellevel.parse().unwrap();
            if fuelvalue < 500 {
                println!("Turtle {} as low fuel !", self.name);
                return self.go_to_position_orders(IN_WORLD_CHEST_POSITION, Facing::North);
            }
        }
        let orders = self.orders.clone();
        let mut indexs_to_replace = Vec::new();
        for (index, Command { name, argument: _ }) in orders.iter().enumerate() {
            match name {
                CommandName::Home => {
                    let orders = self.go_to_position_orders(IN_WORLD_CHEST_POSITION, Facing::North);
                    indexs_to_replace.push((index, orders));
                }
                CommandName::MinePlot => {
                    let oders = self.resume_or_create_plot_oders(mining_plots).await;
                    indexs_to_replace.push((index, oders));
                }
                _ => {}
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
                CommandName::Right => {
                    let u8facing: usize = self.facing.clone() as usize;
                    self.facing = Facing::from_repr((u8facing + 1) % 4).unwrap();
                    format!("{}({})", name, argument)
                }
                CommandName::Left => {
                    let u8facing: usize = self.facing.clone() as usize;
                    let newfacing = if u8facing == 0 { 3 } else { (u8facing - 1) % 4 };
                    self.facing = Facing::from_repr(newfacing).unwrap();
                    format!("{}({})", name, argument)
                }
                CommandName::Forward => {
                    match self.facing {
                        Facing::North => self.pos.z -= argument,
                        Facing::East => self.pos.x += argument,
                        Facing::South => self.pos.z += argument,
                        Facing::West => self.pos.x -= argument,
                    };
                    format!("{}({})", name, argument)
                }
                CommandName::Reboot | CommandName::Sleep | CommandName::RefuelCheck => {
                    format!("{}({})", name, argument)
                }
                _ => {
                    panic!("Order process missing")
                }
            };
            result.push(order_str);
        }
        self.orders = vec![Command::new(CommandName::Sleep, 2)];
        result.join("\n")
    }
}
