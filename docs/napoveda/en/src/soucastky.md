# Components

Components are grouped in the Shift+A menu and in the Component library. This chapter covers inputs, gates and outputs; flip-flops and function blocks have their own chapters.

| Component | Symbol | Function |
| --- | --- | --- |
| Switch | square with 0 or 1 | input 0 or 1; toggled by a click in simulation, by a double-click in edit mode |
| Push button | – | input, 1 only while pressed; in edit mode pressed by the second press of a double-click |
| Constant 0, Constant 1 | – | fixed logic value |
| Supply VCC | line with a bar and the text VCC | logic 1 with a power symbol |
| Ground GND | ground symbol (three shortening lines) | logic 0 with a ground symbol |
| Clock generator | square wave | regular clock signal 0.1–100 Hz (see [Sequential circuits](sekvencni-obvody.md)) |
| BUF | [1] | buffer |
| NOT | [1] with polarity indicator | inverter |
| AND | [&] | 1 when all inputs are 1 |
| OR | [≥1] | 1 when at least one input is 1 |
| NAND, NOR | [&], [≥1] with polarity indicator | negated AND, negated OR |
| XOR | [=1] | 1 for an odd number of ones |
| XNOR | [=1] with polarity indicator | negated XOR |
| LED | circle | output, lit at 1; shows a letter for X or Z |
| LED (anode, cathode) | circle with a diode symbol | lit at A = 1 and K = 0 (see below) |

## VCC and GND supply

The **VCC** and **GND** symbols give a fixed 1 and 0 just like the constants, but they look like the supply in electrical schematics. Connect each symbol to the pin that needs it; there is no need to run supply wires across the whole schematic. The symbols get no automatic label and rotate with the R key like other components.

When a net is connected to VCC (or Constant 1) and to GND (or Constant 0) at the same time, it is a **power short**. The value of the net is X, its wires are highlighted in red, and the schematic and the status bar show the message “Power short”. The message also appears when the short is inside a subcircuit.

## LED colour and two-pin LED

In Properties or in the context menu (right button → **Colour**) you choose the LED colour: red, green, yellow, blue or white. The colour is appearance only – it does not change the simulation. It is handy for a traffic light or to tell outputs apart. An unlit LED has a dark shade of its colour, so the colour is recognisable even when it is off.

The **Pins** option switches between an LED with one pin (an input, as before) and an **LED with two pins – anode A** (left) **and cathode K** (right). Such an LED lights only at A = 1 and K = 0, i.e. when “current flows” from the anode to the cathode – just like a real diode. With X or Z on any pin it stays dark. You can also add it directly from Shift+A → Outputs → **LED (anode, cathode)**. It is useful for matrix circuits, where an LED lights up by a combination of a row and a column.

A two-pin LED is **not a circuit output**: it is not included in the truth table or in the pins of a subcircuit. You can add it to the timing diagram – it shows whether it is lit. A file with a two-pin LED opens in LogicFlow 0.4 and later; older versions do not show the colour (the LED is red).

Gates have 2 to 8 inputs, set in Properties. The seven-segment and hex displays are described in [Function blocks and displays](funkcni-bloky.md).

## Standard symbols

Gate symbols follow IEC 60617-12 (rectangular symbols with &, ≥1, =1, 1). Negation is shown by the **polarity indicator**, a small right triangle on the pin.

## American symbols

**View → Symbols** switches to American distinctive-shape symbols (ANSI/IEEE Std 91): AND as a “D”, OR as a shield, negation as a bubble. The choice is stored on your computer only, not in the circuit file; a circuit opened elsewhere is shown according to the settings there.

> **Coming soon:** ROM, RAM and EEPROM memories and 74xx chips. The Component library already lists them as upcoming items.
