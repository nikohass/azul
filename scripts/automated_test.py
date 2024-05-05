import os
import subprocess
import numpy as np
import enum
import re
from scipy.stats import norm
from itertools import combinations
import json

class TestStatus(enum.Enum):
    OK = 0
    COMPILE_ERROR = 1
    SERVER_ERROR = 2

def feature(num_players: int):
    if num_players == 3:
        return ["--features", "three_players"]
    elif num_players == 4:
        return ["--features", "four_players"]
    return []

def build_executable(binary: str, num_players: int) -> TestStatus:
    command = ["cargo", "build", "--bin", binary, "--release"] + feature(num_players)
    print(f"Building {binary}... ({' '.join(command)})")
    result = subprocess.run(command, capture_output=True, text=True)

    print("STDOUT:")
    print(result.stdout)
    print("STDERR:")
    print(result.stderr)

    if result.returncode != 0:
        print("Build failed")
        return TestStatus.COMPILE_ERROR
    print("Build successful")
    return TestStatus.OK

def run_test_server(num_players):
    build_executable("test_server", num_players)

    command = ["cargo", "run", "--bin", "test_server", "--release"] + feature(num_players)
    print(f"Starting test server... ({' '.join(command)})")
    # Start the test server and yield each line of output
    with subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1, universal_newlines=True) as proc:
        if proc.stdout:
            for line in proc.stdout:
                yield line

        # Check for errors after the process has finished
        proc.wait()
        if proc.returncode != 0:
            return TestStatus.SERVER_ERROR

class PlayerConfig:
    def __init__(self, executable: str, think_time: int):
        self.executable = executable
        self.think_time = think_time
        self.index = None

    def to_toml(self, index: int) -> str:
        name = ["one", "two", "three", "four"][index]
        self.index = index
        result = f"[player_{name}]\n"
        result += f"executable = \"{self.executable}\"\n"
        result += f"think_time = {self.think_time}\n"
        return result

class GameConfig:
    def __init__(self, players: list[PlayerConfig], num_games: int, num_simulations_games: int, constant_ordering: bool = False, stop_on_significant_difference: bool = True):
        self.players = players
        self.num_games = num_games
        self.num_simulations_games = num_simulations_games
        self.constant_ordering = constant_ordering
        self.stop_on_significant_difference = stop_on_significant_difference
    
    def to_toml(self) -> str:
        result = "[game]\n"
        result += f"num_games = {self.num_games}\n"
        result += f"num_simultaneous_games = {self.num_simulations_games}\n"
        result += f"constant_ordering = {str(self.constant_ordering).lower()}\n"
        result += "verbose = false\n"
        
        for i, player in enumerate(self.players):
            result += player.to_toml(i)

        return result

    def activate(self):
        with open("default_config.toml", "w") as file:
            file.write(self.to_toml())
    
    def __dict__(self):
        return {
            "players": [player.__dict__ for player in self.players],
            "num_games": self.num_games,
            "num_simulations_games": self.num_simulations_games,
            "constant_ordering": self.constant_ordering,
            "stop_on_significant_difference": self.stop_on_significant_difference
        }

class GameResult:
    def __init__(self, scores, wins):
        self.scores = scores
        self.wins = wins

    def __str__(self):
        result = ""
        for i, (score, win) in enumerate(zip(self.scores, self.wins), start=1):
            result += f"Player {i} - Score: {score}, Wins: {win}\n"
        return result

    def __repr__(self):
        return str(self)

def parse_game_results(log_lines):
    player_pattern = re.compile(r"Player (\d) - Average score: ([\d.]+), Wins: (\d+), Draws: \d+, Losses: \d+")

    players_scores = []
    players_wins = []
    for line in log_lines:
        for match in player_pattern.finditer(line):
            player_index = int(match.group(1)) - 1  # Adjust index for 0-based list
            score = float(match.group(2))
            wins = int(match.group(3))
            # Ensure lists are large enough
            while len(players_scores) <= player_index:
                players_scores.append(0)
                players_wins.append(0)
            # Store results
            players_scores[player_index] = score
            players_wins[player_index] = wins

    return GameResult(players_scores, players_wins)

class PlayerStats:
    def __init__(self):
        self.wins = 0
        self.games = 0

    def update(self, wins):
        self.wins = wins
        self.games += 1

def multi_player_proportion_test(players_stats, alpha=0.001, min_games=10):
    n_players = len(players_stats)
    results = []

    # Compare each pair of players
    for i, j in combinations(range(n_players), 2):
        stats1, stats2 = players_stats[i], players_stats[j]
        total1, wins1 = stats1.games, stats1.wins
        total2, wins2 = stats2.games, stats2.wins

        if total1 < min_games or total2 < min_games:
            results.append((0, 1, f"Not enough games between player {i+1} and player {j+1}"))
            continue

        p1 = wins1 / total1
        p2 = wins2 / total2
        pooled_n = total1 + total2
        if pooled_n > 0:
            p_pool = (wins1 + wins2) / pooled_n
            variance_component = p_pool * (1 - p_pool)
            se = np.sqrt(variance_component * (1 / total1 + 1 / total2)) if variance_component > 0 and total1 > 0 and total2 > 0 else 0
        else:
            results.append((0, 1, "Insufficient data to compute statistics"))
            continue

        if se == 0:
            results.append((0, 1, f"No variation between player {i+1} and player {j+1}, standard error is zero"))
            continue

        z = (p1 - p2) / se if se > 0 else float('-inf')
        p_value = 2 * (1 - norm.cdf(abs(z)))

        if p_value < alpha:
            better_player = i+1 if p1 > p2 else j+1
            results.append((z, p_value, f"Player {better_player} is significantly better than player {i+1 if better_player == j+1 else j+1}"))
        else:
            results.append((z, p_value, f"No significant difference between player {i+1} and player {j+1}"))

    return results

def run_hypothesis_tests_for_players(game_stream, num_players: int, min_games: int, alpha=0.001):
    players_stats = [PlayerStats() for _ in range(num_players)]

    for game_result in game_stream:
        # Update player statistics
        for i in range(num_players):
            players_stats[i].update(game_result.wins[i])

        # Conduct the test if all players have played at least min_games
        if all(stats.games >= min_games for stats in players_stats):
            test_results = multi_player_proportion_test(players_stats, alpha=alpha, min_games=min_games)
            for z_stat, p_value, conclusion in test_results:
                print(f"Conclusion: {conclusion} (Z-statistic = {z_stat}, P-value = {p_value})")
                # Just stop testing if a player is significantly better than someone else
                if "significantly better" in conclusion:
                    return

class Test:
    def __init__(self, game_config: GameConfig):
        self.game_config = game_config
        self.result = None

    def run(self):
        self.game_config.activate()
        build_executable("test_client", len(self.game_config.players))
        game_result_stream = self.run_games()
        if self.game_config.stop_on_significant_difference:
            run_hypothesis_tests_for_players(game_result_stream, len(self.game_config.players), self.game_config.num_simulations_games * 2)
        else:
            for _ in game_result_stream:
                None

    def run_games(self):
        num_players = len(game_config.players)
        log_lines = []
        for line in run_test_server(num_players):
            log_lines.append(line)
            results = parse_game_results(log_lines[-4:])
            if len(results.scores) == num_players:
                print(results)
                self.result = results
                yield results
                log_lines = []

        with open("logs/automated_test.log", "a") as file:
            data = {
                "result": self.result.__dict__,
                "game_config": self.game_config.__dict__(),
            }
            file.write(json.dumps(data) + "\n")

if __name__ == "__main__":
    # Change the current working directory to the project root
    os.chdir(os.path.join(os.path.dirname(__file__), ".."))

    game_config = GameConfig([
            PlayerConfig("target/release/test_client.exe", think_time=1000),
            PlayerConfig("clients/2/2.exe", think_time=1000),
        ],
        num_games=300,
        num_simulations_games=10,
        stop_on_significant_difference=True
    )
    test = Test(game_config=game_config)
    test.run()
