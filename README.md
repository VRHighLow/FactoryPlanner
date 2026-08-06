# FactoryPlanner

Native factory builder (Linux / Windows / macOS): placeable dual-lane conveyors, power poles, **online multiplayer** (code join — works UK ↔ USA).

## Downloads

GitHub Actions builds **Linux, Windows, and macOS** (Intel + Apple Silicon) on version tags.

- Latest release: https://github.com/VRHighLow/FactoryPlanner/releases/latest
- Download the **`.zip`**, unzip, and run the binary **from inside the folder** (so `assets/` sits next to the exe). Same layout Steam will use later — **no source code** is shipped.

| Platform | Artifact |
|----------|----------|
| Linux x86_64 | `factory_planner-linux-x86_64.zip` |
| Windows x86_64 | `factory_planner-windows-x86_64.zip` |
| macOS Apple Silicon | `factory_planner-macos-aarch64.zip` |
| macOS Intel | `factory_planner-macos-x86_64.zip` |

```
FactoryPlanner/
  factory_planner.exe   # or factory_planner on Linux/macOS
  assets/
    belts/ buildings/ data/ environment/ icons/ items/ …
```

```powershell
# Windows example
Invoke-WebRequest -Uri "https://github.com/VRHighLow/FactoryPlanner/releases/latest/download/factory_planner-windows-x86_64.zip" -OutFile fp.zip
Expand-Archive fp.zip -DestinationPath .
cd FactoryPlanner
.\factory_planner.exe
```

## Run (dev)

```bash
cargo run --release
```

## Gameplay

- **Play → Single Player** or **Multiplayer**
- **Host**: get a 6-digit code. Friends Join with that code only (no IP / port forwarding).
- **Belts**: click an item port, then another (same as power wires). Longer wires = longer travel time. Use a **Splitter** to branch.
- **Power**: wire Solar → Power Pole (orange). Machines need a live pole field.
- Dual lanes on each belt wire.
- **HUD**: floating hotbar (1–9), bottom-right tools (Build / Tech / Map / Nodes). **B** opens Build.

## Multiplayer (online, worldwide)

Gameplay traffic uses **iroh** (P2P + public n0 relays). A short code is only used to exchange an iroh ticket via a public MQTT broker — cursors and world sync do **not** ride MQTT.

1. Host: **Multiplayer → Host Game** → the **6-digit code appears immediately** → you can **Enter World** while P2P finishes in the background. Share the code when status says joinable/online.
2. Friend: **Multiplayer → Join Game** → type the code → **Connect** (can take up to ~1 minute while it finds the host).
3. Host status should show a player connected / world synced. Then place buildings.

Both players must use the **same release version**.

Optional MQTT override (ticket rendezvous only):

```bash
FACTORY_MQTT_HOST=broker.emqx.io FACTORY_MQTT_PORT=1883 cargo run --release
```
