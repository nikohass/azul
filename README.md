# Azul - Rust Implementation

Welcome to the Azul repository, a Rust-based implementation of the popular board game Azul. This project also integrates a Monte Carlo Tree Search (MCTS) algorithm to simulate intelligent AI behavior for playing the game. The implementation is structured into several components, including a backend server for the UI, a test client, a test server for client battles, and a playground for miscellaneous testing.

## Features

- **Backend Server**: Host games that allow human players and AI players to play against each other.
- **Test Client**: A client designed to be initiated by the test server to enable different versions to compete.
- **Test Server**: Facilitates automated matches between clients, running multiple games in parallel.
- **Playground**: A utility executable for general testing purposes.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

### Prerequisites

Before you begin, ensure you have the following installed:
- Rust programming language (latest stable version)
- Cargo (Rust's package manager and build tool)

You can install both by following the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).

### Installation

1. Clone the repository to your local machine:
    ```bash
    git clone https://github.com/nikohass/azul.git
    ```
   
2. Navigate into the project directory:
    ```bash
    cd azul
    ```

3. Build the project using Cargo. This command compiles the project and produces the executable files:
    ```bash
    cargo build --release
    ```

   Note: The `--release` flag builds the executables in release mode, optimizing them for performance.

### Running the Executables

After compiling the project, you will find the executables inside `target/release` directory.

1. **Backend Server**:
    ```bash
    ./target/release/backend_server
    ```
   This starts the server for the UI to connect and host games.

2. **Test Server**:
    ```bash
    ./target/release/test_server
    ```
   This starts a session to test different client versions against each other, running multiple games in parallel.

3. **Playground**:
    ```bash
    ./target/release/playground
    ```
   Execute this for miscellaneous testing and development experiments.
