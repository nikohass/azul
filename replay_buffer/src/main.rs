use replay_buffer::{api, BUFFER, DEFAULT_BUFFER_SIZE};
use tokio::net::TcpListener;

lazy_static::lazy_static!(
    pub static ref DATABASE_DIRECTORY: String = format!("db/{}/", game::NUM_PLAYERS);
);

#[tokio::main]
async fn main() {
    shared::logging::init_logging("replay_buffer");

    let url = "127.0.0.1";
    let mut port: u16 = 3044;
    let mut buffer_size: usize = DEFAULT_BUFFER_SIZE;

    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        match args[i].as_str() {
            "--port" | "-p" => {
                port = args
                    .get(i + 1)
                    .and_then(|s| s.parse::<u16>().ok())
                    .expect("Expected a valid port number for the API");
            }
            "--buffer-size" | "-b" => {
                buffer_size = args
                    .get(i + 1)
                    .and_then(|s| s.parse::<usize>().ok())
                    .expect("Expected a valid buffer size");
            }
            _ => {}
        }
    }

    log::debug!("buffer_size: {}", buffer_size);
    log::debug!("port: {}", port);

    BUFFER.lock().await.set_max_size(buffer_size);

    let addr: std::net::SocketAddr = format!("{}:{}", url, port)
        .parse::<std::net::SocketAddr>()
        .unwrap();
    let ip_bytes = match addr.ip() {
        std::net::IpAddr::V4(ip4) => ip4.octets(),
        _ => panic!("IPv6 is not supported"),
    };

    if TcpListener::bind(format!("{}:{}", url, port))
        .await
        .is_err()
    {
        log::error!(
            "Another instance is already running on REST API port {}. Exiting.",
            port
        );
        std::process::exit(1);
    }

    // Create the database directory if it doesn't exist
    std::fs::create_dir_all(&*DATABASE_DIRECTORY).unwrap();

    let server_task = api::run_rest_api(addr);

    let persist_task = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            let buffer = BUFFER.lock().await;
            buffer.store(&DATABASE_DIRECTORY);
        }
    });

    let ctrl_c_handler = handle_shutdown();

    log::info!(
        "Replay buffer server started on {}:{}",
        ip_bytes
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<String>>()
            .join("."),
        port
    );

    tokio::select! {
        _ = server_task => {}
        _ = persist_task => {}
        _ = ctrl_c_handler => {
            log::debug!("Exited due to Ctrl+C");
        }
    }
}

async fn ctrl_c_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C signals");
}

async fn handle_shutdown() {
    let ctrl_c_handling = ctrl_c_signal();

    tokio::select! {
        _ = ctrl_c_handling => {
            log::debug!("Ctrl+C received, exiting")
        },
    }

    log::debug!("Backing up data...");

    let buffer = BUFFER.lock().await;
    buffer.store(&DATABASE_DIRECTORY);
    // TODO: Backup

    log::info!("Backup complete, exiting.");
    std::process::exit(0);
}
