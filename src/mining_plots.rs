use crate::{
    turtle::{Command, CommandName},
    utils::{Direction, Position},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MiningPlot {
    pub position: Position,
    pub mined_depth_segment: u32,
    pub current_turtle: Option<String>,
}

impl MiningPlot {
    pub fn new(position: Position, turtle_name: &str) -> Self {
        MiningPlot {
            position,
            mined_depth_segment: 0,
            current_turtle: Some(turtle_name.to_string()),
        }
    }
}

const PLOTS_WIDE: i32 = 6;
pub const PLOT_SIZE: i32 = 6;
pub const PLOT_DEPTH: u32 = 3;
pub const PLOT_MAX_DEPTH_SEGMENT: u32 =
    ((IN_WORLD_MINING_POSITION.y + 32) / PLOT_DEPTH as i32) as u32;
pub const PLOT_DIRECTION: Direction = Direction::North;

const IN_WORLD_MINING_POSITION: Position = Position {
    x: -559,
    y: 48,
    z: -2777,
};

#[cfg(test)]
mod tests {
    // use crate::utils::Position;

    // use super::mining_orders;

    #[test]
    fn test_mining_orders() {
        // let result = mining_orders(
        //     &mut Position { x: 0, y: 0, z: 0 },
        //     &mut crate::utils::Direction::East,
        // );
        // println!("{:#?}", result);
    }
}

pub fn mining_orders() -> Vec<Command> {
    let mut result = Vec::new();
    for turn in 1..PLOT_SIZE {
        let side = if turn % 2 == 0 {
            CommandName::Left
        } else {
            CommandName::Right
        };
        result.append(&mut vec![
            Command::new(CommandName::ForwardDig, PLOT_SIZE - 1),
            Command::new(side.clone(), 1),
            Command::new(CommandName::ForwardDig, 1),
            Command::new(side, 1),
        ]);
    }
    result.push(Command::new(CommandName::ForwardDig, PLOT_SIZE - 1));
    result
}

pub fn new_mining_position(total_plots: i32) -> Position {
    let x = (total_plots % PLOTS_WIDE) as i32 * PLOT_SIZE;
    let y = (total_plots / PLOTS_WIDE) as i32 * PLOT_SIZE;
    Position { x, y, z: 0 } + IN_WORLD_MINING_POSITION
}
