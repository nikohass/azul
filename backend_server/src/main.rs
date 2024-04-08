use game::init_logging;

mod game_manager;
mod human_player;
mod websocket_api;

#[tokio::main]
async fn main() {
    init_logging("backend_server");
    websocket_api::run_websocket_api().await.unwrap();
}
