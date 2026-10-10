# Ashes & Bones

A turn-based tactics game where **Humans** and **Undead** clash on a grid. Every attack opens a duel: a short minigame, different for each class, decides the damage, and both players play their own at the same time.

<img width="1190" height="763" alt="image" src="https://github.com/user-attachments/assets/e21ab6ef-1edb-4bbd-81d7-cc40fa35dc37" />

## Features

- **Two factions, twelve classes**: Soldier, Cavalry, Assassin, Longbowman, Mage, Priest against Wraith, Blood Knight, Banshee, Ghoul, Skeleton, Necromancer
- **Duels decided by skill**: each class has its own minigame, attacker and defender play theirs at the same time
- **Positioning matters**: flanking, guarding and range change the odds of every duel
- **Solo** against an AI, or **online** with a friend on the same local network
- **Automatic updates**: the game updates itself when a new version is released

<img width="1184" height="751" alt="image" src="https://github.com/user-attachments/assets/93b5d087-3328-448b-b1eb-fc8059e31b8e" />

## Download

Grab the zip for your system from the [latest release](https://github.com/lennyblk/ashes-bones/releases/latest):

| System | File |
|---|---|
| Windows | `ashes-bones-x86_64-pc-windows-msvc.zip` |
| macOS (Apple Silicon) | `ashes-bones-aarch64-apple-darwin.zip` |
| Linux | `ashes-bones-x86_64-unknown-linux-gnu.zip` |

Unzip it anywhere and launch `ashes-bones` (`ashes-bones.exe` on Windows). Keep the `assets` folder next to the game.

**Windows**: if SmartScreen shows "Windows protected your PC", click **More info** then **Run anyway**.

**macOS**: the game isn't signed by Apple. The first time, right-click `ashes-bones` and choose **Open**, or run:

```sh
xattr -dr com.apple.quarantine path/to/the/unzipped/folder
```

## Updates

At launch the game checks for a newer release and installs it in the background. The title screen tells you when it's done: restart the game to play the new version.

## Playing online

Online games work on a **local network** (same Wi-Fi or same box).

1. One player opens **Multiplayer → Host game**, picks a game name, a player name and a faction.
2. The other opens **Multiplayer**, selects the game in the list and clicks **Join game**.

Both players need the **same version** of the game. If the game doesn't show up in the list:

- **macOS**: allow the game (or your terminal) in *System Settings → Privacy & Security → Local Network*, then restart the Mac if it still doesn't work.
- **Linux** with a firewall: open the ports used by the game.
  ```sh
  sudo firewall-cmd --add-port=6666/tcp --add-port=6667/udp
  ```
- **Windows**: allow the game when the firewall asks, on private networks.
- Some routers don't share broadcasts between their 2.4 GHz and 5 GHz Wi-Fi: put both computers on the same one.

<img width="1193" height="760" alt="image" src="https://github.com/user-attachments/assets/b9028c1e-ee8f-4e72-a5d5-47b2406f5ca1" />

## Controls

| Action | Input |
|---|---|
| Select a unit, move, attack, heal | Left click |
| Cancel an action | `B` |
| Pause menu | `Esc` |
| Map speed x1 / x2 | `F` |

The in-game **Guide** (title screen) explains the duels and every minigame.

## Building from source

You need [Rust](https://rustup.rs) and CMake.

```sh
git clone https://github.com/lennyblk/ashes-bones.git
cd ashes-bones
cargo run --release
```

On Linux, raylib also needs the X11 / OpenGL development packages (on Fedora: `sudo dnf install cmake clang libX11-devel libXrandr-devel libXi-devel libXcursor-devel libXinerama-devel mesa-libGL-devel alsa-lib-devel`).

<img width="1193" height="760" alt="image" src="https://github.com/user-attachments/assets/8afd8777-209c-4e36-815c-62cf28a76f7e" />


<!-- IMAGE : vue de fin de partie / victoire -->
