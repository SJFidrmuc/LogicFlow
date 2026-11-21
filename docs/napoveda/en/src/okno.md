# Window and areas

Like in Blender, the window is divided into areas. Every area has a header with an editor type selector that can be switched at any time.

## Top bar

- **Menus** File, Edit, View, Simulation and Help. Every item shows its keyboard shortcut on the right; items that cannot be used right now (for example Undo with no previous edit) are greyed out. Besides Undo and Redo, the Edit menu also offers selecting, rotating, deleting and opening a subcircuit – the same as the keys over the schematic.
- **Workspaces** Design, Simulation, Analysis and Memory in the middle.
- **Language switch** CS / EN on the right.

At the bottom, the **status bar** shows whether the simulation runs, the tick number, the circuit name and the program version.

## Editor types

| Editor | Purpose |
| --- | --- |
| Schematic | drawing and simulating the circuit |
| Timing diagram | inputs and outputs over time |
| Properties | settings of the selected component (label, type, number of inputs, frequency…) |
| Hierarchy | project sheets (main circuit and subcircuits) and the list of components |
| Component library | all components with search; click a component, then the schematic to place it |
| Truth table | table computed by simulating every input combination |
| Memory editor | preview of the future ROM and RAM editor (memories arrive in a later version) |
| Help | this help as an area next to the schematic |

## Adding and closing areas

- On the right of an area header there is the **Area** menu; right-clicking the header opens it too.
- **Split side by side** and **Split below** create a new area with the same editor; switch its type afterwards.
- **Close area** removes it. The last area cannot be closed.
- Drag the border between areas to resize them.
- Drag an area by the empty part of its header. Dropping it onto the middle of another area creates tabs.

## Workspaces

The Design, Simulation, Analysis and Memory tabs are workspaces. Each has its own area layout, but all work with the same circuit:

- **Design** – schematic, hierarchy, component library and properties,
- **Simulation** – schematic with the timing diagram below,
- **Analysis** – schematic with the truth table,
- **Memory** – prepared for memories in a later version.

Ctrl+PageUp and Ctrl+PageDown switch workspaces too. **View → Reset area layout** restores the default. The program remembers the area layout and window size after closing.

> **Tip:** Every schematic editor has its own view. One area can show the whole circuit while another zooms into a detail.
