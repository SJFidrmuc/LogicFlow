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

- Produces a regular square wave. Set the frequency in Properties (0.2 to 10 Hz).
- The clock runs only while the simulation runs. While paused it advances by a quarter of a second with every step (right arrow).
- Clocks are always at the top of the timing diagram, so you can see on which edge the outputs change.

## Restarting the simulation

**Simulation → Restart simulation** (Ctrl+R) returns all flip-flops to their initial state, sets clocks to 0 and clears the timing diagram.

> **Tip:** Try the examples 0–7 counter from T flip-flops and 4-bit shift register. In the counter each stage is clocked by Q̄ of the previous one – an asynchronous up counter.
