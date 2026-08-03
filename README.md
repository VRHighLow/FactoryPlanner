# FactoryPlanner

Native desktop factory builder.

## Run

```bash
cargo run
```

## Logistics

- **Nodes** have input ports, output ports, or both (ore = out, smelter = in+out, box = in, splitter = in+2 outs).
- **Belts** are drawn like power wires: click **output port → input port**. Each belt is a Factorio-style **2-lane** conveyor; items travel on left/right lanes.
- **Power** wires are separate (orange): Solar → Power Pole. Poles project a power field.

Transport build menu currently has **Splitter** only — belts themselves are the port-to-port links.
