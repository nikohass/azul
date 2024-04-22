import os
import subprocess
import numpy as np
import enum
import re
from scipy.stats import norm

class TestStatus(enum.Enum):
    OK = 0
    COMPILE_ERROR = 1
    SERVER_ERROR = 2

def build_test_client() -> TestStatus:
    result = subprocess.run(["cargo", "build", "--bin", "test_client", "--release"], capture_output=True, text=True)
    if result.returncode != 0:
        return TestStatus.COMPILE_ERROR
    return TestStatus.OK

def run_test_server():
    # Start the test server and yield each line of output
    with subprocess.Popen(["cargo", "run", "--bin", "test_server", "--release"], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1, universal_newlines=True) as proc:
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
    def __init__(self, players: list[PlayerConfig], num_games: int, num_simulations_games: int, constant_ordering: bool = False):
        self.players = players
        self.num_games = num_games
        self.num_simulations_games = num_simulations_games
        self.constant_ordering = constant_ordering
    
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

def two_sample_proportion_test(wins1, total1, wins2, total2, alpha=0.001, min_games=10):
    if total1 < min_games or total2 < min_games:
        return 0, 1, "Not enough games to make a decision"

    p1 = wins1 / total1 if total1 > 0 else 0
    p2 = wins2 / total2 if total2 > 0 else 0

    pooled_n = total1 + total2
    if pooled_n > 0:
        p_pool = (wins1 + wins2) / pooled_n
        variance_component = p_pool * (1 - p_pool)
        if variance_component <= 0 or total1 <= 0 or total2 <= 0:
            return 0, 1, "Invalid computation conditions"
        se = np.sqrt(variance_component * (1 / total1 + 1 / total2))
    else:
        return 0, 1, "Insufficient data to compute statistics"

    if se == 0:
        return 0, 1, "No variation between samples, standard error is zero"

    z = (p1 - p2) / se if se > 0 else float('-inf')
    p_value = 2 * (1 - norm.cdf(abs(z)))

    if p_value < alpha:
        if p1 > p2:
            return z, p_value, "Player 1 is significantly better than Player 2"
        else:
            return z, p_value, "Player 2 is significantly better than Player 1"
    return z, p_value, "No significant difference detected"

def run_hypothesis_tests(game_stream, min_games: int, alpha=0.001):
    stats1 = PlayerStats()
    stats2 = PlayerStats()

    for game_result in game_stream:
        # Update player statistics
        stats1.update(game_result.wins[0])
        stats2.update(game_result.wins[1])

        # Conduct the test if both players have played at least one game
        if stats1.games > 0 and stats2.games > 0:
            z_stat, p_value, conclusion = two_sample_proportion_test(stats1.wins, stats1.games, stats2.wins, stats2.games, min_games=min_games, alpha=alpha)
            print(f"Game {stats1.games}: Z-statistic = {z_stat}, P-value = {p_value}, Conclusion: {conclusion}")

            if "significantly better" in conclusion:
                break

class Test:
    def __init__(self, game_config: GameConfig):
        self.game_config = game_config
    
    def run(self):
        self.game_config.activate()
        build_test_client()
        game_result_stream = self.run_games()
        run_hypothesis_tests(game_result_stream, self.game_config.num_simulations_games * 2)
    
    def run_games(self):
        num_players = len(game_config.players)
        log_lines = []
        for line in run_test_server():
            log_lines.append(line)
            results = parse_game_results(log_lines[-4:])
            if len(results.scores) == num_players:
                print(results)
                yield results
                log_lines = []

if __name__ == "__main__":
    # Change the current working directory to the project root
    os.chdir(os.path.join(os.path.dirname(__file__), ".."))

    game_config = GameConfig([
        PlayerConfig("target/release/test_client.exe", 2000),
        PlayerConfig("target/release/test_client.exe", 20_000),
    ], num_games=250, num_simulations_games=10)
    test = Test(game_config=game_config)
    test.run()
