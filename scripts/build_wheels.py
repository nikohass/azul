import os
import re
import sys

INTERPRETER = sys.executable  

for wheel_name, feature in [("azul2", None), ("azul3", "three_players"), ("azul4", "four_players")]:
    print(f"Building wheel for {wheel_name} with feature {feature}")
    # Edit pyproject.toml to change the name of the wheel
    with open("./python_package/pyproject.toml") as f:
        lines = f.readlines()

    # Replace the line that matches name = "..." with name = "..."
    for i, line in enumerate(lines):
        if re.match(r'name = ".+"', line):
            lines[i] = re.sub(r'name = ".+"', f"name = \"{wheel_name}\"", line)
            break

    with open("./python_package/pyproject.toml", "w") as f:
        f.writelines(lines)
        
    # Edit the name of the pymodule in rust
    with open("./python_package/src/lib.rs") as f:
        lines = f.readlines()

    pattern = r'fn (\w+)'
    for i, line in enumerate(lines):
        match = re.search(pattern, line)
        if match:
            lines[i] = "fn " + re.sub(pattern, wheel_name, line)
            break

    with open("./python_package/src/lib.rs", "w") as f:
        f.writelines(lines)


    build_command = f"maturin build --manifest-path ./python_package/Cargo.toml --release --no-default-features --interpreter {INTERPRETER}"
    if feature:
        build_command += f" --features {feature}"
    os.system(build_command)
    os.system(f"pip install ./target/wheels/{wheel_name}-0.1.0-cp310-none-win_amd64.whl --force-reinstall")