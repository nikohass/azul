use crate::websocket_api::{EventType, WebSocketConnection, WebSocketMessage};
use game::{
    GameState, MoveGenerationResult, MoveList, Player, SharedState, TileColor,
    CENTER_FACTORY_INDEX, FLOOR_LINE_PENALTY, NUM_PLAYERS, NUM_TILE_COLORS,
};
use rand::{rngs::SmallRng, SeedableRng};
use std::collections::HashMap;
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref MATCHES: SharedState<HashMap<String, SharedState<Match>>> = SharedState::new(HashMap::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchState {
    NotStarted,
    GameOver,
}

pub struct Match {
    id: String,
    game_state: GameState,
    players: Vec<Box<dyn Player>>,
    state: MatchState,
}

impl Match {
    pub async fn new_with_players(players: Vec<Box<dyn Player>>) -> SharedState<Match> {
        let mut rng = SmallRng::from_entropy();
        let game_state = GameState::new(&mut rng);
        let id = Uuid::new_v4().to_string();
        let game_manager = Self {
            id: id.clone(),
            game_state,
            players,
            state: MatchState::NotStarted,
        };
        let shared_state = SharedState::new(game_manager);
        let mut all_games = MATCHES.lock().await;
        all_games.insert(id.to_string(), shared_state.clone());
        shared_state
    }

    pub async fn get_game(id: &str) -> Option<SharedState<Self>> {
        let all_games = MATCHES.lock().await;
        all_games.get(id).cloned()
    }

    pub async fn get_player_names(&self) -> Vec<String> {
        let mut player_names = Vec::new();
        for player in &self.players {
            player_names.push(player.get_name().to_string());
        }
        player_names
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub async fn start_match(&mut self, websocket: WebSocketConnection) {
        let game_state = &mut self.game_state;
        let mut move_list = MoveList::default();
        let mut rng = SmallRng::from_entropy();
        let mut round = 0;
        loop {
            game_state.check_integrity().unwrap(); // Check the integrity of the game state. If it is not valid, panic and crash the tokio task
            send_game_state_update(game_state, &websocket); // Send the game state to the players
            let mut turn = 0;
            println!("{}", game_state);
            let mut is_game_over;
            loop {
                is_game_over = matches!(
                    game_state.get_possible_moves(&mut move_list, &mut rng),
                    MoveGenerationResult::GameOver
                );
                if is_game_over {
                    // If there are no legal moves we end the game
                    break;
                }

                // Get the move from the current player
                let current_player = game_state.get_current_player();
                let current_player = usize::from(current_player);
                log::info!(
                    "Round {}, turn {}, player {}",
                    round,
                    turn,
                    self.players[current_player].get_name()
                );
                send_game_state_update(game_state, &websocket);
                let move_ = self.players[current_player].get_move(game_state).await;

                // Validate the move
                if !move_list.contains(move_) {
                    // If the move is not legal, panic and crash the tokio task
                    panic!(
                        "Player {} made an illegal move: {:?}",
                        current_player, move_
                    );
                }

                // Apply the move to the game state
                game_state.do_move(move_);
                println!("{}", game_state);

                send_game_state_update(game_state, &websocket);

                // Check integrity of the game state after the move
                game_state.check_integrity().unwrap();

                turn += 1;
            }
            // At the end of the round, evaluate it by counting the points and moving the first player marker
            send_game_state_update(game_state, &websocket);
            println!("{}", game_state);
            if is_game_over {
                self.state = MatchState::GameOver;
                break;
            }
            round += 1;
        }
        send_game_state_update(game_state, &websocket);
    }
}

pub fn send_game_state_update(game_state: &GameState, websocket: &WebSocketConnection) {
    log::info!("Sending game state update to {}", websocket.get_address());
    let json = game_state_to_json(game_state);
    let message = WebSocketMessage {
        event_type: EventType::GameStateUpdate,
        data: json,
    };
    websocket.send_message(message);
}

pub fn game_state_to_json(game_state: &GameState) -> serde_json::Value {
    let mut players = Vec::new();

    let floor_lines = game_state.get_floor_line_progress();
    let scores = game_state.get_scores();
    let walls = game_state.get_walls();
    let pattern_line_occupancy = game_state.get_pattern_lines_occupancy();
    let pattern_line_colors = game_state.get_pattern_lines_colors();
    for player in 0..NUM_PLAYERS {
        let player_json = serde_json::json!({
            "floor_line_progress": floor_lines[player],
            "floor_line_penalty": FLOOR_LINE_PENALTY[floor_lines[player].min(FLOOR_LINE_PENALTY.len() as u8 - 1) as usize],
            "score": scores[player],
            "wall": wall_to_json(&walls[player]),
            "pattern": pattern_lines_to_json(&pattern_line_occupancy[player], &pattern_line_colors[player]),
        });
        players.push(player_json);
    }

    let factories = game_state.get_factories();
    let mut factories_json = Vec::new();
    for (factory_index, factory) in factories.iter().enumerate() {
        let mut factory_json = Vec::new();
        for (color_index, number) in factory.iter().enumerate() {
            let color = TileColor::from(color_index);
            let color: char = color.into();
            factory_json.push(serde_json::json!({
                "color": color,
                "number_of_tiles": number,
            }));
        }
        let is_center = factory_index == CENTER_FACTORY_INDEX;
        factories_json.push(serde_json::json!({
            "is_center": is_center,
            "tiles": factory_json,
        }));
    }

    // Create new json object
    let ret = serde_json::json!({
        "bag": game_state.get_bag(),
        "factories": factories_json,
        "players": players,
        "current_player": usize::from(game_state.get_current_player()),
        "next_round_starting_player": usize::from(game_state.get_next_round_starting_player()),
    });
    ret
}

pub fn wall_to_json(wall: &[u32; NUM_TILE_COLORS]) -> Vec<serde_json::Value> {
    let mut wall_tiles = Vec::new();
    for row in 0..5 {
        for col in 0..5 {
            let mut color = ' ';
            for (c, wall_bitboard) in wall.iter().enumerate() {
                let field = game::field_at(row, col);
                if wall_bitboard & field > 0 {
                    color = TileColor::from(c).into();
                    break;
                }
            }
            if color != ' ' {
                wall_tiles.push(serde_json::json!({
                    "row": row,
                    "col": col,
                    "color": color,
                }));
            }
        }
    }
    wall_tiles
}

pub fn pattern_lines_to_json(
    pattern_line_occupancy: &[u8; 5],
    pattern_line_colors: &[Option<TileColor>; 5],
) -> Vec<serde_json::Value> {
    let mut pattern_lines = Vec::new();
    for i in 0..5 {
        let color = pattern_line_colors[i];
        if let Some(color) = color {
            let color: char = color.into();
            pattern_lines.push(serde_json::json!({
                "patern_line_index": i,
                "color": color,
                "number_of_tiles": pattern_line_occupancy[i],
            }));
        }
    }
    pattern_lines
}
