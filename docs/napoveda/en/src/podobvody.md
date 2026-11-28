# Subcircuits

A subcircuit is a circuit you insert into another one as a single component – for example a half adder used twice in a full adder. Its inputs are its switches and buttons, its outputs its (one-pin) LEDs. Their labels set the pin order on the symbol (A, B, C…).

## How to

1. In the **Hierarchy** area click **+ New subcircuit**. An empty sheet opens and the schematic header shows *Main circuit / Subcircuit 1*.
2. Draw the circuit with inputs (switches) and outputs (LEDs) and test it, truth table included.
3. Name it in the Hierarchy using the **Name** field.
4. Press **Tab** to return to the main circuit. Insert the subcircuit from **Shift+A → Subcircuits** or from the Component library.

## Working with subcircuits

- **Tab** over a selected subcircuit opens it, another Tab goes back (like node groups in Blender). You can also switch sheets by clicking in the Hierarchy.
- Changes apply to every instance. A new input or output adds a pin to the symbol.
- Subcircuits may contain other subcircuits, but not themselves.
- A subcircuit in use cannot be deleted. Remove its instances first.
- Feed clocks in through a subcircuit input; a clock generator inside a subcircuit does not run.
- Gate delays are kept inside subcircuits and every instance has its own state (e.g. its own flip-flops).
- The whole project including all subcircuits is saved in a single file.

## Subcircuit from the selection

A part of the schematic you have already drawn can be packed into a subcircuit right in place:

1. Select the gates and blocks that should be inside (by dragging the mouse or by clicking with Shift). You may select the switches and LEDs too.
2. **Ctrl+G** (or right button → **Create subcircuit from selection**, or the Edit menu).

- The selected components move into a new subcircuit “Subcircuit 1” (rename it in the Hierarchy) and its symbol takes their place.
- **The inputs and outputs are created automatically** from the wires leading out of the selection: what is fed into the selection is an input, what comes out of it is an output. The symbol is connected to these wires right away, so the circuit computes the same as before.
- **Switches, push buttons, clocks, LEDs and displays stay outside** even if they are selected – you operate and read the circuit the same way. The subcircuit pins take their labels (A, B, Y1…); the others are named A, B, C… and Y1, Y2…
- Inside the subcircuit, the interface switches and LEDs sit where the wires led out – the circuit looks as before.
- If a pin cannot be connected without touching another wire, the program leaves it free and tells you. Connect it by hand then.
- **Ctrl+Z** returns the schematic to the state before grouping.

## A finished circuit as a subcircuit

A circuit you already have saved in a `.logicflow` file (for example a tested half adder or an exercise from the online circuit library) can be inserted into the project as a subcircuit:

1. **File → Import as subcircuit…** (**Ctrl+I**) and choose the file. The preview shows the circuit's title, author and licence.
2. The subcircuit symbol appears at the mouse pointer – click to place it (Esc cancels placing; the subcircuit stays in the project).

- The subcircuit gets the **circuit title** from its properties (File → Circuit properties), otherwise the file name. If the project already has a subcircuit with that name, a number is added, e.g. “Adder (2)”.
- The inputs are the switches and buttons of the imported circuit, the outputs its LEDs – as with any subcircuit.
- A **copy** is imported: later changes to the file do not affect the project and editing the subcircuit does not change the file.
- Subcircuits used by the imported circuit come with it. Identical subcircuits the project already has are reused.
- Importing is not possible in simulation mode – stop the simulation first (Esc).

## Element libraries

Ready-made circuits are in the **Component library** area under **Element libraries**. The elements are sorted into categories (Combinational circuits, Arithmetic, Sequential circuits…) and the search finds them by name and category.

- **The built-in library “Digital electronics – teaching elements”** contains 36 circuits of common tasks: gates from NAND and NOR, multiplexers, a decoder, comparators, Gray code converters, adders and subtractors, flip-flops, counters, registers, an alarm and a traffic light.
- **The built-in library “Function blocks – internal circuits”** contains gate-level circuits of all function blocks (see the next part).
- Insert an element like a component: click it, then click in the schematic. It appears at the mouse pointer and a click drops it.
- An inserted element is a **project subcircuit** (a copy). Open it with Tab and edit it; it is saved with the project. The elements it uses (e.g. the full adders in the 4-bit adder) are inserted with it.
- Below the chosen element you see its description, pins, author and licence.
- **Personal libraries** are `.lflib` files in the folder `Documents/LogicFlow/Knihovny`. The **Open folder** button creates and opens it; after copying files there click **Reload**. Choose a different folder in **Edit → Settings…** (button **Library folders…**).
- **Shared folders** (network, OneDrive, club) are added in **Edit → Settings… → Libraries**; their libraries are labelled **Shared folder**.

> **Coming soon:** exporting a subcircuit into a library and school libraries.

> **Ready-made example:** File → Examples → Full adder from two subcircuits.

## Internal circuits of function blocks

Every function block (multiplexer, decoder, encoder, adder, comparator, BCD to 7-segment decoder, counter, register, shift register) can be opened all the way down to gates.

- Select a block in the schematic and press **Tab** (or right-click → **Open internal circuit**). Its gate-level circuit with pins of the same names opens.
- The circuit is **read-only**: a badge with the block's name is at the top left, inputs can be switched and simulated, but the schematic cannot be changed. The project stays unchanged.
- Sequential blocks are made of T and D flip-flops. **Tab** on a flip-flop goes deeper – the D flip-flop is made of six NAND gates (the circuit of the 7474).
- **Tab** on an empty spot goes one level up, from the top level back to the project. The **Back to the project** button in the Hierarchy or the **Main circuit** link in the schematic header goes straight to the project.
- The **Insert into the project as a subcircuit** button in the Hierarchy copies the circuit into the project as an editable subcircuit (inserted at the mouse pointer like an item from a library).
- The gate-level circuit differs from the block in two things: unconnected **EN** and **R** inputs have the value X (the block treats EN as 1 and R as 0), and a sequential circuit is in the unknown state X after start until **R = 1** resets it (the block starts in state 0).
- All the circuits are also in the library panel in the library **Function blocks – internal circuits**.
