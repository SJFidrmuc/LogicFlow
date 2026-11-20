# Drawing schematics

## Adding components

- **Shift+A** over the schematic opens the Add menu with groups: Inputs, Gates, Flip-flops, Function blocks, Outputs and Subcircuits.
- Or click a component in the **Component library** and then in the schematic. The Search field at the top of the library filters by name.
- While placing, **R** rotates by 90°, **Shift+click** places another one, **Esc** or the right button cancels.

## Wires

- A wire starts by dragging from a component pin (highlighted with a ring under the mouse) or from any point of an existing wire. With Ctrl held you can start anywhere.
- Wires bend at right angles. The first mouse movement decides the direction of the first segment.
- Where a wire end lies on another wire, a connection with a dot is made. Wires that merely cross (no dot) are not connected.
- A component pin connects only to a wire end. A wire merely passing over a pin is not connected to it.
- While drawing, an orange ring marks where the wire would also connect besides its start and end. If the opposite bend has fewer such places, the program uses it automatically.
- When a component is moved, attached wires are bent so they touch no other net. If that is impossible, the wire is detached from the pin: the loose end is visible and easy to reconnect.

## Selecting and editing

| Action | Result |
| --- | --- |
| click | select a component or wire |
| Shift+click | add to or remove from the selection |
| drag on empty space | box select |
| drag a component, G | move the selection (wires follow) |
| R | rotate by 90° |
| X, Delete | delete the selection |
| Shift+D | duplicate the selection |
| A, Alt+A | select all, deselect |
| Ctrl+Z, Ctrl+Shift+Z | undo, redo |

## Viewing the schematic

- The **mouse wheel** zooms, the **middle or right button** with dragging pans the view.
- **Home** or the **Fit all** button in the header shows the whole circuit. The current zoom is shown next to it in percent.
- A line with the most important keys is shown at the bottom of the schematic.

## Component properties

The **Properties** area shows the type and settings of the selected component. Depending on the component you can change:

- the **label** (e.g. A, B, CLK, Y1); it also sets the order of signals in the timing diagram and truth table,
- the **gate type** and **number of inputs** (2–8),
- the **constant value** and the clock **frequency**,
- for flip-flops the **type**, **trigger** (level, rising or falling edge) and **initial state**,
- for function blocks the **number of bits** and, for the counter, the **count direction**,
- for displays the **pin side**,
- the **rotation**; the **Delete** button removes the component.

Properties also show **live values**: state, inputs and outputs (e.g. 0011 = 3 for a counter) and the delay in ticks.
