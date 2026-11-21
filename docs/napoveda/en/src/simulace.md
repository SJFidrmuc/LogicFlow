# Simulation

Each gate has a delay of one tick, so a signal travels through the circuit step by step – just like in a real circuit.

## Edit mode and simulation mode

The program works in two modes, similar to DigitalWorks:

- **Edit mode** – you draw and edit the schematic. Logic is recomputed at once (LEDs react), but time stands still: clocks do not tick and the timing diagram does not move. A switch is toggled by a **double-click**, a push button is pressed by the second press of a double-click – a single click only selects the component, so you do not toggle it by accident while moving it.
- **Simulation mode** – started with **F5** (or Simulation → Run simulation, or Space). The schematic is locked: you cannot add, delete, move, rotate or draw wires. Only the controls work – a switch and a button with a single click, panning and zooming, Tab into subcircuits, the timing diagram and the truth table. The clock frequency can be changed in Properties.
- In simulation mode the schematic has an **amber frame** with a **SIMULATION** badge; the status bar shows Edit mode / Simulation running / Simulation paused.
- **Esc** or **Shift+F5** stops the simulation and returns to edit mode. **The circuit state stays** (flip-flops, counters, LEDs and the timing diagram); only Restart simulation (Ctrl+R) clears it.
- If you try to edit in simulation mode, the message "Stop the simulation to edit (Esc)" appears at the top.
- If you want to edit while it runs, switch on **Simulation → Allow editing while simulating** (off by default).

## Wire colours

| Colour | Value | When |
| --- | --- | --- |
| light green | 1 | logic one |
| dark green | 0 | logic zero |
| red | X | undefined: unconnected input, two outputs colliding, uninitialised memory |
| blue | Z | unconnected wire (high impedance) |

## Stepping

- **Space** starts the simulation in edit mode; in simulation mode it pauses or resumes it.
- The **Right arrow** (or Simulation → Step) performs one tick; from edit mode it goes straight to a paused simulation. You can watch a change pass gate by gate.
- If the circuit never settles (e.g. an inverter feeding itself), the warning **The circuit oscillates** appears.
- **Simulation → Restart simulation** (Ctrl+R) starts again from the initial state.
- The status bar shows the mode, whether the simulation runs and the tick number.

> **Tip:** Circuits with memory, such as an SR latch made of NOR gates, start in state X. Only set (S) or reset (R) defines their state. After editing the schematic such a circuit keeps its state.
