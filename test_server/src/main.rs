use game::{GameState, Move, PlayerTrait};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct Client {
    pub path: String,
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub stdout: Arc<Mutex<ChildStdout>>,
    pub child: Arc<Mutex<Child>>,
    pub time: u64,
}

impl Client {
    pub fn from_path(path: String, time: u64) -> Self {
        let mut process = Command::new(path.clone())
            // .args(&["--time", &time.to_string()])
            // .args(&["--test", "true"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        Self {
            path,
            stdin: Arc::new(Mutex::new(process.stdin.take().unwrap())),
            stdout: Arc::new(Mutex::new(process.stdout.take().unwrap())),
            child: Arc::new(Mutex::new(process)),
            time,
        }
    }

    // Method to check if the child process has panicked
    pub fn did_panic(&self) -> bool {
        let mut child = self.child.lock().unwrap();
        match child.try_wait() {
            Ok(Some(status)) => !status.success(),
            Ok(None) => false,
            Err(_) => true,
        }
    }
}

#[async_trait::async_trait]
impl PlayerTrait for Client {
    fn name(&self) -> &str {
        &self.path
    }

    async fn get_move(&mut self, state: GameState) -> Move {
        let name = self.name();
        if self.did_panic() {
            panic!("Client panicked");
        }
        let mut msg = format!("get_move {}", state.serialize_string());
        msg.push('\n');
        self.stdin
            .lock()
            .unwrap()
            .write_all(msg.as_bytes())
            .unwrap();
        let start_time = Instant::now();
        let mut stdout = self.stdout.lock().unwrap();
        let mut read = BufReader::new(&mut *stdout);
        let mut line = String::new();
        loop {
            if self.did_panic() {
                panic!("Client panicked");
            }
            read.read_line(&mut line).unwrap();
            if !line.is_empty() && line.contains("move_response ") {
                line = line[14..].to_string();
                break;
            }
            if !line.is_empty() {
                line.pop();
                println!("{}: {}", name, line);
            }
            line.truncate(0);
            let elapsed = start_time.elapsed().as_millis();
            if elapsed > self.time as u128 + 2500 {
                println!("warning: Client {} hard-timeout: {}ms", self.path, elapsed);
            }
        }
        let elapsed = start_time.elapsed().as_millis();
        if elapsed as u64 > 1990 {
            println!("warning: Client {} soft-timeout: {}ms", self.path, elapsed);
        }
        line.pop();
        Move::deserialize_string(&line)
    }

    async fn notify_move(&mut self, new_game_state: &GameState, move_: Move) {
        let mut msg = format!(
            "notify_move {} {}",
            new_game_state.serialize_string(),
            move_.serialize_string()
        );
        msg.push('\n');
        self.stdin
            .lock()
            .unwrap()
            .write_all(msg.as_bytes())
            .unwrap();
    }
}

#[tokio::main]
async fn main() {
    let client_one_path = "./target/release/test_client.exe".to_string();
    let time = 30_000;
    let mut client_one = Client::from_path(client_one_path, time);
    let mut game_state = GameState::default();
    // game_state.fill_factories();
    // let mut move_list = MoveList::new();
    // game_state.get_possible_moves(&mut move_list);
    let move_ = client_one.get_move(game_state).await;
    println!("move_response {}", move_);
}
