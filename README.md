# Kovaaks Discord RPC

A Discord Rich Presence client for Kovaak's FPS Aim Trainer. Shows your current scenario, scores, and session stats on your Discord profile.

No AI was used for the Icon and .png of the Icon believe it or not, I'm just a magician on paint.exe!

If theres any issues DM me on Discord "salzi" is the tag, also "This application is not affiliated with the official Kovaaks team." just saying, also if pc blows up or kovaak's lets ur account implode thats not on me. 
Using this app is fully on your own risk, I have no responsibility of it working and or not getting you into trouble.

## Preview on Discord

### If a Playlist is being played
<img width="995" height="801" alt="image" src="https://github.com/user-attachments/assets/77d06221-a89e-47f9-b887-d43047d51142" />

### If a single Scenario is being played
<img width="995" height="787" alt="image" src="https://github.com/user-attachments/assets/19289b38-4cc8-470b-9f96-88869003de00" />


## Features

- Displays current scenario name on Discord
- Shows local and session high scores
- Syncs with Kovaak's online leaderboards
- Minimizes to system tray
- Auto-starts with Windows (optional)
- Lightweight native application

## Requirements

- Windows 10/11 (64-bit)
- [Rust](https://rustup.rs/) (for building from source)
- Discord desktop app

## Installation

### Option 1: Download Release

Download the latest release from the [Releases](../../releases) page and run `KovaaksDiscordRPC-InstallerWin64.exe` for an installer for windows,
and for a portable version there is `KovaaksDiscordRPC-Portable.exe` that you can just run itself, which I hope works as I just realised I didn't even test that but it should I think.

### Option 2: Build from Source

1. Clone the repository:
```bash
git clone https://github.com/SalziCantAim/Improved-Kovaaks-Discord-RPC.git
cd Improved-Kovaaks-Discord-RPC
```

2. Build the release version:
```bash
cargo build --release
```

3. The executable will be at `target/release/kovaaks-rpc.exe`

4. (Optional) Copy the `assets` folder next to the executable for the application icon.

## Usage

1. Make sure Discord is running
2. Launch `kovaaks-rpc.exe`
3. The app will appear in your system tray
4. Start Kovaaks and your Discord status will update automatically

## Main Menu

This is what the Main Menu looks like (idc that its not centered you can fix it bcs I will not touch rust egui again.)
<img width="849" height="566" alt="image" src="https://github.com/user-attachments/assets/75917ed6-3c1b-4908-944e-028505bbb8cc" />

## Settings Menu

This is the Settings Menu
<img width="839" height="1067" alt="image" src="https://github.com/user-attachments/assets/08f61506-96e6-4f9c-ac74-438d1181defa" />


### Settings

- **Start in Tray**: Launch minimized to system tray (the little "up arrow" next to your clock, and ethernet / wifi symbol)
- **Start with Windows**: Automatically run when Windows starts
- **Open Manually**: Only start RPC when you click the button (don't auto-detect Kovaak's)
- **Start minimized to system tray**: I dont think this works but you can test it yourself mb if not

### Relevant Settings

- **Installation Path**: Auto-detected, or manually set your Kovaaks installation folder, if it doesnt automatically find it go to your steam library rightclick kovaaks > Manage > Browse Local Files > Copy folder path > Click browse in the app > Paste folder path > Select "FPSAimTrainer"
- **Online Only Scenarios**: Only show scenarios that exist on the online leaderboard (If you want to keep private scenarios hidden)
- **Show online scenario highscores**: Currently you can have it first ONLY update the local highscores meaning it will only show the highscores that you got after the last time you reset Kovaaks / reset your PC, this setting will (if you input your kovaaks webappname and click on "Sync Now") make the Discord RPC only show online Highscores (not for every Scenario but thats not on me I can't fix that)
- **Scan Local Stats / Sync Online Scores**: If the shown score on Discord is wrong press either Scan Local Stats if you only want the local highscores to be shown, and Sync Online Scores (+ your kvk name) if you want your online highscores to be shown, if this doesn't work either Kovaaks problem that I can't fix (specifically for older Online highscores), or DM me for local ones


### Tray Menu

Right-click the tray icon for options:
- **Show Window**: Open the main window
- **Start/Stop RPC**: Manually control the Discord connection
- **Exit**: Close the application

## Building

### Prerequisites

- Rust 1.70 or later
- Windows SDK (for building on Windows)

### Debug Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

## AI Disclamer

I did use AI for this README bcs I'm not a word person, and also for structuring and debugging this code bcs I'm a pretty bad coder so yeah if it looks robotic you now know why, also rust fucking sucks.

## License

MIT






