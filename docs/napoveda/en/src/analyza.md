# Analysis

## Timing diagram

The timing diagram records all switches, buttons and LEDs over the last 20 seconds. Signals are sorted by label, clocks are always at the top. While paused, every step moves the diagram by a quarter of a second. It is most convenient in the **Simulation** workspace.

## Truth table

The table is computed by simulating every input combination, for at most 8 inputs (256 rows). Inputs are switches and buttons, outputs are LEDs. The highlighted row matches the current switch settings. You find it in the **Analysis** workspace. Right-click the table → **Copy as text** copies it to the clipboard; pasted into Word or a spreadsheet it becomes columns.

> **Note:** for circuits with memory (flip-flops, counters) the table may not make sense, because the output also depends on the previous state. Use the timing diagram for them.

> **Teaching tip:** Students draw a circuit from the assignment and check with the truth table that it implements the required function.
