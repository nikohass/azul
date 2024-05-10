use game::{GameState, Move, Player};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct Client {
    pub path: String,
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub stdout: Arc<Mutex<ChildStdout>>,
    pub child: Arc<Mutex<Child>>,
    pub verbose: bool,
}

impl Client {
    pub fn from_path(path: &str, verbose: bool) -> Self {
        let mut process = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        Self {
            path: path.to_string(),
            stdin: Arc::new(Mutex::new(process.stdin.take().unwrap())),
            stdout: Arc::new(Mutex::new(process.stdout.take().unwrap())),
            child: Arc::new(Mutex::new(process)),
            verbose,
        }
    }

    // Method to check if the child process has panicked
    pub fn did_panic(&self) -> bool {
        let mut child = self.child.lock().unwrap();
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    println!("Client {} exited with: {}", self.path, status);
                }
                !status.success()
            }
            Ok(None) => false,
            Err(_) => true,
        }
    }
}

#[async_trait::async_trait]
impl Player for Client {
    fn get_name(&self) -> &str {
        &self.path
    }

    fn get_move(&mut self, game_state: &GameState) -> Move {
        if self.did_panic() {
            panic!("Client panicked");
        }
        let mut msg = format!("get_move {}", game_state.serialize_string());
        msg.push('\n');
        self.stdin
            .lock()
            .unwrap()
            .write_all(msg.as_bytes())
            .unwrap();
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
                if self.verbose {
                    println!("{}: {}", self.path, line);
                }
            }
            line.truncate(0);
        }
        line.pop();
        Move::deserialize_string(&line)
    }

    fn notify_move(&mut self, new_game_state: &GameState, move_: Move) {
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

    fn set_time(&mut self, time: u64) {
        let mut msg = format!("time {}", time);
        msg.push('\n');
        self.stdin
            .lock()
            .unwrap()
            .write_all(msg.as_bytes())
            .unwrap();
    }

    fn set_pondering(&mut self, pondering: bool) {
        let mut msg = format!("pondering {}", pondering);
        msg.push('\n');
        self.stdin
            .lock()
            .unwrap()
            .write_all(msg.as_bytes())
            .unwrap();
    }
}
