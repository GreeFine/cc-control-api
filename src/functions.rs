use mongodb::bson::{self, doc};

use crate::{
    mining_plots::{
        mining_orders, new_mining_position, MiningPlot, PLOT_DEPTH, PLOT_DIRECTION,
        PLOT_MAX_DEPTH_SEGMENT,
    },
    turtle::{Command, CommandName},
    utils::{Direction, Position},
    MiningPlots,
};

fn rotate_to(current: &mut Direction, wanted: Direction) -> Option<Command> {
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
    departure: &mut Position,
    departure_direction: &mut Direction,
    destination: &Position,
    destination_direction: &Direction,
) -> Option<Vec<Command>> {
    if departure == destination && departure_direction == destination_direction {
        return None;
    }
    let mut orders_to_pos = Vec::new();
    // Don't go flying when you are near
    // if (departure.x - destination.y).abs() + (departure.x - destination.y).abs() > (PLOT_SIZE * 2) {
    //     if let Some(orders) = self.go_in_the_sky_orders() {
    //         orders_to_pos.push(orders);
    //     }
    // }
    let pos_diff = *departure - *destination;
    if pos_diff.x > 0 {
        if let Some(orders) = rotate_to(departure_direction, Direction::West) {
            orders_to_pos.push(orders);
        }
    } else if let Some(orders) = rotate_to(departure_direction, Direction::East) {
        orders_to_pos.push(orders);
    }
    orders_to_pos.push(Command::new(CommandName::Forward, pos_diff.x.abs()));
    if pos_diff.z > 0 {
        if let Some(orders) = rotate_to(departure_direction, Direction::North) {
            orders_to_pos.push(orders);
        }
    } else if let Some(orders) = rotate_to(departure_direction, Direction::South) {
        orders_to_pos.push(orders);
    }
    orders_to_pos.push(Command::new(CommandName::Forward, pos_diff.z.abs()));
    let command = if pos_diff.y > 0 {
        CommandName::Down
    } else {
        CommandName::Up
    };
    orders_to_pos.push(Command::new(command, pos_diff.y.abs()));
    if let Some(orders) = rotate_to(departure_direction, destination_direction.clone()) {
        orders_to_pos.push(orders);
    }
    *departure_direction = destination_direction.clone();
    *departure = *destination;
    Some(orders_to_pos)
}

fn mine_plot_orders(
    departure: &mut Position,
    departure_direction: &mut Direction,
    mining_plot: MiningPlot,
) -> Vec<Command> {
    let mut result = Vec::new();
    let mut current_plot_position_w_depth = mining_plot.position;
    current_plot_position_w_depth.y -= (mining_plot.mined_depth_segment * PLOT_DEPTH) as i32;
    if let Some(mut orders) = go_to_position_orders(
        departure,
        departure_direction,
        &current_plot_position_w_depth,
        &PLOT_DIRECTION,
    ) {
        result.append(&mut orders);
    }
    result.append(&mut mining_orders(departure, departure_direction));
    result
}

pub async fn resume_or_create_plot_oders(
    departure: &mut Position,
    departure_direction: &mut Direction,
    turtle_name: &str,
    mining_plots: &MiningPlots,
) -> Vec<Command> {
    let current_plot = mining_plots
        .find_one(doc! {"current_turtle": turtle_name}, None)
        .await
        .expect("Unable to find_one from mining plots");
    if let Some(mut current_plot) = current_plot {
        if current_plot.mined_depth_segment < PLOT_MAX_DEPTH_SEGMENT {
            current_plot.mined_depth_segment += 1;
            mining_plots
                .update_one(
                    doc! {"current_turtle": turtle_name},
                    doc! { "$set": { "mined_depth_segment": current_plot.mined_depth_segment } },
                    None,
                )
                .await
                .expect("Unable to update mined_depth_segment in plot");
            return mine_plot_orders(departure, departure_direction, current_plot);
        } else {
            mining_plots
                .update_one(
                    doc! {"current_turtle": turtle_name},
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
    let new_plot = MiningPlot::new(new_plot_position, turtle_name);
    mining_plots
        .insert_one(&new_plot, None)
        .await
        .expect("Unable to add new mining plot");
    mine_plot_orders(departure, departure_direction, new_plot)
}
