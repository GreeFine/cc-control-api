use crate::utils::{Facing, Position};
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
pub const PLOT_MAX_DEPTH_SEGMENT: u32 = ((IN_WORLD_MINING_POSITION.y + 32) / 2) as u32;
pub const PLOT_FACING: Facing = Facing::North;

const IN_WORLD_MINING_POSITION: Position = Position {
    x: -559,
    y: 48,
    z: -2777,
};

#[cfg(test)]
mod tests {
    use crate::mining_plots::mining_orders;

    #[test]
    fn it_works() {
        let result = mining_orders();
        println!("{}", result);
    }
}

pub fn mining_orders() -> String {
    let mut result = String::new();
    for turn in 1..PLOT_SIZE {
        let side = if turn % 2 == 0 { "Left" } else { "Right" };
        result.push_str(&*format!(
            "Forward({})\n{}(1)\nForward(1)\n{}(1)\n",
            PLOT_SIZE - 1,
            side,
            side
        ));
    }
    result.push_str(&*format!("Forward({})", PLOT_SIZE - 1));
    result
}

pub fn new_mining_position(total_plots: i32) -> Position {
    let x = (total_plots % PLOTS_WIDE) as i32 * PLOT_SIZE;
    let y = (total_plots / PLOTS_WIDE) as i32 * PLOT_SIZE;
    Position { x, y, z: 0 } + IN_WORLD_MINING_POSITION
}
