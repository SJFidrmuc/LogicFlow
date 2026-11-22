# Sequential circuits

Sequential circuits remember their state: the output depends not only on the inputs but also on what happened before. LogicFlow has ready-made flip-flops (Shift+A → Flip-flops) and a clock generator (Shift+A → Inputs).

## Flip-flops

| Flip-flop | Behaviour |
| --- | --- |
| RS | S = 1 sets Q = 1, R = 1 resets Q = 0, S = R = 0 holds, S = R = 1 is forbidden (X) |
| D (level) | while C = 1, Q follows D; with C = 0 it holds |
| D | stores D on the clock edge |
| JK | on the edge: J = 1 sets, K = 1 resets, J = K = 1 toggles, J = K = 0 holds |
| T | on the edge: T = 1 toggles, T = 0 holds |

Every flip-flop has outputs Q and Q̄ (negated). An edge-triggered input is marked with a triangle inside the outline; a polarity indicator on its pin means the falling edge. In Properties you can choose the type, rising or falling edge and the initial state (0 or 1).

## Clock generator

- Produces a regular square wave. Set the frequency in **Properties**: quick choices 0.2–10 Hz, a **slider 0.1–100 Hz** (the field next to it accepts a typed number) or the **period** in milliseconds. Quick choices are also in the context menu (right button → Frequency).
- The frequency can be changed while simulating too. The clock continues smoothly without a jump: the level and the elapsed part of the half period stay.
- The clock runs only in simulation mode (F5); in edit mode it stands still. Stepping:
  - **Right arrow** – one tick (time moves by a quarter of a second),
  - **Shift+Right arrow** – to the next clock edge (half a period),
  - **Ctrl+Right arrow** – a whole period, to the next rising edge (for a counter = one number).
- **Simulation speed** (×0.1 to ×10) is set in the status bar or in Simulation → Simulation speed. ×0.1 slows down for explaining, ×10 speeds up long counters. The timing diagram and the "time" in the status bar show simulated time. After the program starts the speed is always ×1.
- Clocks are always at the top of the timing diagram, so you can see on which edge the outputs change.

## Restarting the simulation

**Simulation → Restart simulation** (Ctrl+R) returns all flip-flops to their initial state, sets clocks to 0 and clears the timing diagram.

> **Tip:** Try the examples 0–7 counter from T flip-flops and 4-bit shift register. In the counter each stage is clocked by Q̄ of the previous one – an asynchronous up counter.
