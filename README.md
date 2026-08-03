# FactoryPlanner

Native factory builder (Linux / Windows / macOS): placeable dual-lane conveyors, power poles, **online multiplayer** (code join — works UK ↔ USA).

## Run (dev)

```bash
cargo run --release
```

## Gameplay

- **Play → Single Player** or **Multiplayer**
- **Host**: get a 6-digit code. Friends Join with that code only (no IP / port forwarding).
- Place **Conveyors** (Transport) end-to-end so ports nearly touch — items travel along each segment; **longer belts = longer travel time**.
- Dual lanes on each conveyor (Factorio-style).
- Power: wire Solar → Power Pole (orange). Machines need a live pole field.

## Multiplayer (online, worldwide)

Both players dial out to a public MQTT relay (`broker.emqx.io`). No LAN, no firewall holes on either side.

1. Host: **Multiplayer → Host Game** → wait until status says online → copy the **6-digit code** → **Enter World**.
2. Friend (anywhere): **Multiplayer → Join Game** → type the code → **Connect**.
3. You should see each other's cursors and placements sync.

Optional override (advanced):

```bash
FACTORY_MQTT_HOST=broker.emqx.io FACTORY_MQTT_PORT=1883 cargo run --release
```

> Prototype relay: messages go through a public broker. Fine for friends testing; not a hardened production server.

## Windows download (after GitHub release)

```powershell
Invoke-WebRequest -Uri "https://github.com/VRHighLow/FactoryPlanner/releases/latest/download/factory_planner-windows-x86_64.exe" -OutFile factory_planner.exe
.\factory_planner.exe
```

Or from Releases page: download `factory_planner-windows-x86_64.exe` and double-click.
