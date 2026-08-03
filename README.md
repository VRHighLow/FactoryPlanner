# FactoryPlanner

Native factory builder (Linux / Windows / macOS): placeable dual-lane conveyors, power poles, **online multiplayer** (code join — works UK ↔ USA).

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

## Multiplayer (online, worldwide)

Gameplay traffic uses **iroh** (P2P + public n0 relays). A short code is only used to exchange an iroh ticket via a public MQTT broker — cursors and world sync do **not** ride MQTT.

1. Host: **Multiplayer → Host Game** → wait until status says ready → copy the **6-digit code** → **Enter World**.
2. Friend: **Multiplayer → Join Game** → type the code → **Connect**.
3. Host status should show a player connected / world synced. Then place buildings.

Both players must use the **same release version**.

Optional MQTT override (ticket rendezvous only):

```bash
FACTORY_MQTT_HOST=broker.emqx.io FACTORY_MQTT_PORT=1883 cargo run --release
```

## Windows download (after GitHub release)

```powershell
Invoke-WebRequest -Uri "https://github.com/VRHighLow/FactoryPlanner/releases/latest/download/factory_planner-windows-x86_64.exe" -OutFile factory_planner.exe
.\factory_planner.exe
```

Or from Releases page: download `factory_planner-windows-x86_64.exe` and double-click.
