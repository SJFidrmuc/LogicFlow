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
| Ctrl+C, Ctrl+X | copy, cut the selection to the clipboard |
| Ctrl+V | paste from the clipboard at the mouse pointer |
| A, Alt+A | select all, deselect |
| Ctrl+Z, Ctrl+Shift+Z | undo, redo |

### Clipboard

- **Ctrl+C** copies the selected components together with the wires connecting them (wires to unselected components are not copied). **Ctrl+X** also deletes the selection.
- **Ctrl+V** pastes the clipboard at the mouse pointer; the pasted part follows the mouse right away and a click drops it (Esc leaves it where it was pasted).
- Copying and pasting works between sheets (main circuit, subcircuits) and between two running LogicFlow windows. Subcircuits used come along; identical ones are not added again, on a name clash the new subcircuit gets a number, e.g. "Adder (2)".
- Labels (A, B, Y1…) stay if the target circuit does not have them yet; otherwise they get new ones.
- The clipboard holds the circuit as text in the .logicflow file format – the whole content of a file opened in a text editor can be pasted the same way.

## Context menu (right button)

**Clicking the right button** (without dragging) opens a menu depending on where you click. Dragging with the right button still pans the view, and during an action in progress (moving, drawing a wire, placing) the right button cancels it.

| Where you click | What the menu offers |
| --- | --- |
| component | by kind: Toggle (switch), Frequency (clock), Inputs (gate), Pins (display), Colour and Pins (LED), Bits and Count direction (function block), Open subcircuit; then the label, Rotate, Duplicate, Move, Copy, Cut, Delete, Create subcircuit from selection |
| wire | Select the whole net, Delete wire segment, Delete the whole net |
| several selected items | Rotate, Duplicate, Move, Copy, Cut, Delete, Deselect, Create subcircuit from selection |
| empty place | Add component here (placed at the clicked point), Paste, Select all, Fit view, Back one level (in a subcircuit), simulation controls |

- A component or wire that is not selected gets selected first by the right-click.
- Every item shows its keyboard shortcut on the right – the menu teaches the shortcuts too.
- In simulation mode edits are greyed out; Toggle, clock Frequency, Open subcircuit, selection and the simulation controls work.
- The right button also works in the **Hierarchy** (on a sheet: Open, Delete subcircuit; on a component: Find in schematic, Toggle, Rotate, Delete) and in the **truth table** (Copy as text).

## Viewing the schematic

- The **mouse wheel** zooms, the **middle or right button** with dragging pans the view (a plain right-click opens the context menu).
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
