mod game_manager;
mod logging;
mod shared_state;
mod websocket_api;

#[tokio::main]
async fn main() {
    logging::init_logging("backend_server");
    websocket_api::run_websocket_api().await.unwrap();
}
