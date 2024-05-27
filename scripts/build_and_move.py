import os
import subprocess
import shutil

def build_and_move(player_version, feature_flag=None):
    # Build command
    build_command = ["cargo", "build", "--release", "--bin", "test_client"]
    if feature_flag:
        build_command.extend(["--features", feature_flag])

    # Execute build command
    subprocess.run(build_command, check=True)

    # Determine source file name
    exe_extension = ".exe" if os.name == "nt" else ""
    source_file = os.path.join("target", "release", f"test_client{exe_extension}")

    # Create target directory if it doesn't exist
    target_dir = os.path.join("clients", str(player_version))
    os.makedirs(target_dir, exist_ok=True)

    # Determine the next executable number
    existing_files = os.listdir(target_dir)
    max_version = 0
    for file in existing_files:
        if file.endswith(exe_extension):
            try:
                version = int(file.split(".")[0])
                if version > max_version:
                    max_version = version
            except ValueError:
                continue

    next_version = max_version + 1
    target_file = os.path.join(target_dir, f"{next_version}{exe_extension}")

    # Move the built executable to the target directory
    shutil.move(source_file, target_file)
    print(f"Moved {source_file} to {target_file}")

if __name__ == "__main__":
    # Build for 2 players
    build_and_move(2)

    # Build for 3 players with the three_players feature
    build_and_move(3, "three_players")

    # Build for 4 players with the four_players feature
    build_and_move(4, "four_players")
