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

Uses **WebRTC peer-to-peer** (Matchbox signaling). After connect, game traffic goes directly between players — not through a public MQTT broker.

1. Host: **Multiplayer → Host Game** → share the **6-digit code** → **Enter World**.
2. Friend: **Multiplayer → Join Game** → enter code → **Connect** (wait for “Peer online”).
3. Cursors and buildings sync over a direct link.

Optional override:

```bash
FACTORY_SIGNALING=wss://match-0-9.helsing.studio cargo run --release
```

> Both players must use the same game version. First peer connection can take a few seconds (NAT hole-punch).

## Windows download (after GitHub release)

```powershell
Invoke-WebRequest -Uri "https://github.com/VRHighLow/FactoryPlanner/releases/latest/download/factory_planner-windows-x86_64.exe" -OutFile factory_planner.exe
.\factory_planner.exe
```

Or from Releases page: download `factory_planner-windows-x86_64.exe` and double-click.
