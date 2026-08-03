# FactoryPlanner

Native factory builder (Linux / Windows / macOS): placeable dual-lane conveyors, power poles, LAN multiplayer.

## Run (dev)

```bash
cargo run --release
```

## Gameplay

- **Play → Single Player** or **Multiplayer**
- **Host**: get a 6-digit code + `IP:7788`. Friends Join with that address + code.
- Place **Conveyors** (Transport) end-to-end so ports nearly touch — items travel along each segment; **longer belts = longer travel time**.
- Dual lanes on each conveyor (Factorio-style).
- Power: wire Solar → Power Pole (orange). Machines need a live pole field.

## Multiplayer (LAN)

1. Host: Multiplayer → Host Game → Enter World (leave the window open; port **7788**).
2. Join: Multiplayer → Join Game → enter `HOST_IP:7788` and the code → Connect.
3. You should see each other's cursors and placement ghosts; places/removes sync.

Firewall: allow inbound TCP **7788** on the host.

## Windows download (after GitHub release)

```powershell
# Replace VERSION and USER/REPO if different
Invoke-WebRequest -Uri "https://github.com/VRHighLow/FactoryPlanner/releases/latest/download/factory_planner-windows-x86_64.exe" -OutFile factory_planner.exe
.\factory_planner.exe
```

Or from Releases page: download `factory_planner-windows-x86_64.exe` and double-click.
