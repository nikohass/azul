use crate::*;
use std::fmt::Write as _;

use self::wall::WALL_COLOR_MASKS;

pub fn bag_to_string(bag: &Bag) -> String {
    let mut string = String::new();
    for (color, number_of_tiles_left) in bag.iter().enumerate() {
        write!(
            string,
            "{} {}\t",
            TileColor::from(color),
            number_of_tiles_left
        )
        .unwrap();
    }
    string.push('\n');
    string
}

pub fn factories_to_string(factories: &Factories) -> String {
    let mut string = String::new();

    let mut factory_strings = Vec::new();
    let mut total_length = 0;
    for (factory_index, factory) in factories.iter().enumerate() {
        let mut factory_string = String::new();
        let tile_count: usize = factory.iter().sum::<u8>() as usize;

        if factory_index == CENTER_FACTORY_INDEX {
            total_length += tile_count + 1 + 4; // +1 for min space + 4 for "[n] "
            factory_string.push_str("[C] ");
        } else {
            total_length += 4 + 1 + 4; // always 4 because placeholders
            write!(factory_string, "[{}] ", factory_index + 1).unwrap();
        }

        for (color, number_of_tiles) in factory.iter().enumerate() {
            factory_string.push_str(
                &TileColor::from(color)
                    .to_string()
                    .repeat(*number_of_tiles as usize),
            );
        }

        if factory_index != NUM_FACTORIES - 1 {
            factory_string.push_str(&".".repeat(4 - tile_count));
        }

        factory_strings.push(factory_string);
    }

    let maximum_size = (NUM_PLAYERS * 33).max(total_length);
    let spaces_between_factories = (maximum_size - total_length) / NUM_FACTORIES;

    for factory_string in &factory_strings {
        string.push_str(factory_string);
        string.push_str(&" ".repeat(spaces_between_factories));
    }
    let remaining_space =
        maximum_size - total_length - spaces_between_factories * (NUM_FACTORIES - 1);
    let leading_spaces = remaining_space / 2;
    format!("{}{}\n", " ".repeat(leading_spaces), string,)
}

fn player_wall_to_string(game_state: &GameState, player_index: usize) -> String {
    let mut string = String::new();

    for y in 0..5 {
        for x in 0..5 {
            let bit: u32 = 1 << (y * 6 + x);
            let mut found = false;

            for (color, color_mask) in WALL_COLOR_MASKS.iter().enumerate() {
                let wall = game_state.get_walls()[player_index];
                if wall & color_mask & bit > 0 {
                    string.push_str(&TileColor::from(color).to_string());
                    found = true;
                    break;
                }
            }

            if !found && game_state.get_walls()[player_index] & bit == 0 {
                for (color, wall_color) in WALL_COLOR_MASKS.iter().enumerate() {
                    if wall_color & bit > 0 {
                        let (start, end) = TileColor::from(color).get_color_string();
                        write!(string, "{}.{}", start, end).unwrap();
                        break;
                    }
                }
            }

            string.push(' ');
        }
        string.push('\n');
    }

    string
}

fn player_pattern_board_to_string(game_state: &GameState, player_index: usize) -> String {
    let mut string = String::new();

    let pattern_line_occupancy = game_state.get_pattern_lines_occupancy()[player_index];
    let pattern_colors = game_state.get_pattern_line_colors()[player_index];
    for pattern_index in 0..5 {
        let pattern_color: Option<TileColor> = pattern_colors[pattern_index];
        let color = if let Some(pattern_color) = pattern_color {
            pattern_color.get_color_string()
        } else {
            ("".to_string(), "".to_string())
        };
        write!(string, " {}{} ", color.0, pattern_index + 1).unwrap();
        let leading_whitespace = (4 - pattern_index) * 2;
        for _ in 0..leading_whitespace {
            string.push(' ');
        }
        let count = pattern_line_occupancy[pattern_index] as usize;
        let missing = pattern_index + 1 - count;
        for _ in 0..missing {
            string.push_str(". ");
        }
        for _ in 0..count {
            if let Some(c) = pattern_color {
                string.push_str(&format!("{} ", c));
            } else {
                string.push_str("X ");
            }
        }
        string.push_str(&color.1);
        string.push(' ');
        string.push('\n');
    }

    string
}

pub fn display_gamestate(game_state: &GameState, player_names: Option<&Vec<String>>) -> String {
    // Empty line for spacing
    let mut empty_line = " ".to_string();
    for _ in 0..NUM_PLAYERS - 1 {
        empty_line.push_str(&" ".repeat(28));
        empty_line.push_str("|  ");
    }
    empty_line.push_str(&" ".repeat(27));
    empty_line.push('\n');

    let separator_line = empty_line.replace(' ', "-").replace('|', "+");

    let mut string = String::new();
    string.push_str(&format!("BAG: {}", bag_to_string(&game_state.get_bag())));
    string.push_str(&format!(
        "OUT OF BAG: {}",
        bag_to_string(&game_state.get_out_of_bag())
    ));
    string.push('\n');
    string.push_str(&factories_to_string(game_state.get_factories()));
    string.push('\n');

    // Player header
    string.push(' ');
    for player_index in 0..NUM_PLAYERS {
        if usize::from(game_state.get_current_player()) == player_index {
            string.push_str("\x1b[30m\x1b[47m");
        }

        let player_name = if let Some(player_names) = &player_names {
            let name = player_names[player_index].to_string();
            if name.len() > 23 {
                let mut name = name[..22].to_string();
                name.push('…');
                name
            } else {
                name
            }
        } else {
            format!("Player {}", player_index)
        };
        string.push_str(&format!(
            "{:23} {:3}\x1b[0m ",
            player_name,
            game_state.get_scores()[player_index]
        ));
        if player_index != NUM_PLAYERS - 1 {
            string.push_str("|  ");
        }
    }
    string.push('\n');

    string.push_str(&separator_line);
    string.push_str(&empty_line);

    // Compute max lines
    let mut max_lines = 0;
    for player_index in 0..NUM_PLAYERS {
        let pattern_string = player_pattern_board_to_string(game_state, player_index);
        let wall_string = player_wall_to_string(game_state, player_index);
        max_lines = max_lines.max(pattern_string.lines().count());
        max_lines = max_lines.max(wall_string.lines().count());
    }

    // Player pattern boards next to walls
    for line in 0..max_lines {
        for player_index in 0..NUM_PLAYERS {
            let pattern_string = player_pattern_board_to_string(game_state, player_index);
            let wall_string = player_wall_to_string(game_state, player_index);

            let pattern_line = pattern_string.lines().nth(line).unwrap_or("");
            let wall_line = wall_string.lines().nth(line).unwrap_or("");

            string.push_str(pattern_line);
            string.push_str("->  "); // separator between pattern and wall
            string.push_str(wall_line);
            if player_index != NUM_PLAYERS - 1 {
                string.push_str(" | ");
            }
        }
        string.push('\n');
    }

    string.push_str(&empty_line);
    string.push_str(&separator_line);

    string.push(' ');
    // Player floor lines
    for player_index in 0..NUM_PLAYERS {
        let mut floor_line = String::new();
        let progress = game_state.get_floor_line_progress()[player_index]
            .min(FLOOR_LINE_PENALTY.len() as u8 - 1) as usize;

        let mut previous_penalty = 0;
        for (i, relative_penalty) in FLOOR_LINE_PENALTY.iter().enumerate().skip(1) {
            let penalty = *relative_penalty as i16 - previous_penalty;
            if i > progress {
                floor_line.push_str("\u{001b}[02m");
            } else if i == 1
                && game_state.get_next_round_starting_player()
                    == PlayerMarker::new(player_index as u8)
                && game_state.get_tile_taken_from_center()
            {
                floor_line.push_str("\u{001b}[32m");
            }
            floor_line.push_str(&format!("{:2} ", -penalty));
            floor_line.push_str("\x1b[0m");
            previous_penalty = *relative_penalty as i16;
        }
        string.push_str(&floor_line);

        let total_penalty = FLOOR_LINE_PENALTY[progress] as i16;
        string.push_str(&format!(" {:5}", -total_penalty));
        if player_index != NUM_PLAYERS - 1 {
            string.push_str(" |  ");
        }
    }

    string.push('\n');

    string
}
