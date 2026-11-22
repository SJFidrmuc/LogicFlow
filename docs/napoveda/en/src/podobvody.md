# Subcircuits

A subcircuit is a circuit you insert into another one as a single component – for example a half adder used twice in a full adder. Its inputs are its switches and buttons, its outputs its LEDs. Their labels set the pin order on the symbol (A, B, C…).

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

## Element libraries

Ready-made circuits are in the **Component library** area under **Element libraries**. The elements are sorted into categories (Combinational circuits, Arithmetic, Sequential circuits…) and the search finds them by name and category.

- **The built-in library “Digital electronics – teaching elements”** contains 36 circuits of common tasks: gates from NAND and NOR, multiplexers, a decoder, comparators, Gray code converters, adders and subtractors, flip-flops, counters, registers, an alarm and a traffic light.
- Insert an element like a component: click it, then click in the schematic. It appears at the mouse pointer and a click drops it.
- An inserted element is a **project subcircuit** (a copy). Open it with Tab and edit it; it is saved with the project. The elements it uses (e.g. the full adders in the 4-bit adder) are inserted with it.
- Below the chosen element you see its description, pins, author and licence.
- **Personal libraries** are `.lflib` files in the folder `Documents/LogicFlow/Knihovny`. The **Open folder** button creates and opens it; after copying files there click **Reload**.

> **Coming soon:** exporting a subcircuit into a library, shared library folders and school libraries.

> **Ready-made example:** File → Examples → Full adder from two subcircuits.
