# Getting started

LogicFlow is a logic circuit simulator for teaching digital electronics. Draw a circuit from components and wires and watch it work right away: wires change colour with their logic value and LEDs light up.

## Starting the program

There is nothing to install. Download the package for your system from [www.logicflow-sim.cz](https://www.logicflow-sim.cz), extract it and run `logicflow` (`logicflow.exe` on Windows). On first start, language and symbol style follow your system; both can be changed in the View menu.

## Your first circuit in a minute

We will draw an AND gate with two switches and an LED. The LED lights only when both switches are on.

1. **File → New** (Ctrl+N) opens an empty schematic.
2. Move the mouse over the schematic and press **Shift+A**. Choose **Switch** and click in the schematic. Repeat – you get inputs A and B.
3. **Shift+A → AND [&]** and place the gate to the right of the switches. Then **Shift+A → LED** and place it to the right of the gate.
4. Hover the output of switch A (the point on its right side gets a ring), press the left button and drag to a gate input. Release – a wire is created. Connect B to the second input and the gate output to the LED the same way.
5. Press **F5** (starts the simulation) and click the switches. A wire carrying logic 1 is light green and the LED lights only when both inputs are 1. **Esc** takes you back to editing.
6. Save the circuit with **Ctrl+S**.

## What next

- Switch to the **Analysis** workspace and look at the truth table the program computed.
- Open a ready-made circuit from **File → Examples**. The SR latch built from NOR gates shows nicely how a circuit remembers its state.
- Press **F1** to open this help in the program; it can also be shown as an area next to the schematic.
