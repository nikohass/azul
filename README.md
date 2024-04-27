# Azul - Rust Implementation

Welcome to the Azul repository, a Rust-based implementation of the popular board game Azul. This university project integrates a Monte Carlo Tree Search (MCTS) algorithm to simulate intelligent AI behavior for playing the game. The implementation is structured into several components, including a backend server for the UI, a test client, a test server for client battles, and a playground for miscellaneous testing.

## Features
- **Azul**: An executable that allows human and AI players to play the game.
- **Backend Server**: The backend for the [UI](https://github.com/Dahmspiegel/Uni-Azul-Frontend)
- **Test Client**: A client designed to be initiated by the test server to enable different versions to compete.
- **Test Server**: Facilitates automated matches between clients, running multiple games in parallel.
- **Playground**: A utility executable for general testing purposes.

## Getting Started

Follow these instructions to set up the project on your local system for development and experimentation.

### Prerequisites

Ensure the following tools are installed on your system before starting:
- Rust programming language (latest stable release)
- Cargo (Rust's package management and compilation tool)

Installation instructions for both are available on the [official Rust website](https://www.rust-lang.org/tools/install).

### Installation
To compile the project and generate executable files, run the following command. You can adjust the number of players by activating specific feature flags during the build.

```bash
cargo build --release
```
For a three-player optimized game, use:
```bash
cargo build --release --features "three_players"
```
For four players:
```bash
cargo build --release --features "four_players"
```
Use the `--release` flag to build in release mode, which enhances performance essential for smooth gameplay.

### Running the Executables

Post compilation, the executables can be found in the `target/release` directory.

1. **Azul**:
    ```bash
    ./target/release/azul
    ```
    Launches the game in a terminal interface for both human and AI players.

2. **Backend Server**:
    ```bash
    ./target/release/backend_server
    ```
   Activates the server allowing the UI to connect and manage games.

3. **Test Server**:
    ```bash
    ./target/release/test_server
    ```
   Initiates automated competitions among different client versions, requiring a configuration file created by `./scripts/automated_test.py`.

4. **Playground**:
    ```bash
    ./target/release/playground
    ```
   Utilize this for assorted testing and development activities.