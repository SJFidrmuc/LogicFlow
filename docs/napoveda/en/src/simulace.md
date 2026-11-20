# Simulation

The simulation runs continuously, even while you draw. Each gate has a delay of one tick, so a signal travels through the circuit step by step – just like in a real circuit.

## Wire colours

| Colour | Value | When |
| --- | --- | --- |
| light green | 1 | logic one |
| dark green | 0 | logic zero |
| red | X | undefined: unconnected input, two outputs colliding, uninitialised memory |
| blue | Z | unconnected wire (high impedance) |

## Stepping

- **Space** pauses or resumes the simulation.
- While paused, the **right arrow** (or Simulation → Step) performs one tick. You can watch a change pass gate by gate.
- If the circuit never settles (e.g. an inverter feeding itself), the warning **The circuit oscillates** appears.
- **Simulation → Restart simulation** (Ctrl+R) starts again from the initial state.
- The status bar shows whether the simulation runs and the tick number.

> **Tip:** Circuits with memory, such as an SR latch made of NOR gates, start in state X. Only set (S) or reset (R) defines their state. After editing the schematic such a circuit keeps its state.
