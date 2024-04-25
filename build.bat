@echo off
setlocal enabledelayedexpansion

rem -- Set the path of the clients directories
set "CLIENTS_DIR=%~dp0clients"

rem -- Set the array of client folders to process
set "FOLDERS=2 3 4"

rem -- Initialize max_num to 0
set /a "max_num=0"

rem -- Loop over each client directory to find the absolute maximum number
for %%d in (%FOLDERS%) do (
    if not exist "!CLIENTS_DIR!/%%d" mkdir "!CLIENTS_DIR!/%%d"
    pushd "!CLIENTS_DIR!/%%d"
    for /f "delims=" %%f in ('dir /b /a-d') do (
        set "filename=%%~nf"
        if "!filename!" gtr "!max_num!" (
            set /a "max_num=!filename!"
        )
    )
    popd
)

rem -- Increment the max number by 1
set /a "new_num=max_num + 1"

rem -- Compile and move the executables for each feature setup
for %%d in (%FOLDERS%) do (
    echo Building for clients/%%d with new number !new_num!.exe

    rem -- Adjust features based on folder number
    if %%d EQU 2 (
        cargo build --bin test_client --release
    ) else if %%d EQU 3 (
        cargo build --bin test_client --release --features three_players
    ) else if %%d EQU 4 (
        cargo build --bin test_client --release --features four_players
    )

    rem -- Check if executable exists and move it
    if exist ".\target\release\test_client.exe" (
        move ".\target\release\test_client.exe" "!CLIENTS_DIR!/%%d/!new_num!.exe"
    ) else (
        echo Failed to find the built executable. Please check the build logs.
    )
)

echo All clients updated successfully.
endlocal
