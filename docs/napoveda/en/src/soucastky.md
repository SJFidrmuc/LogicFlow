# Components

Components are grouped in the Shift+A menu and in the Component library. This chapter covers inputs, gates and outputs; flip-flops and function blocks have their own chapters.

| Component | Symbol | Function |
| --- | --- | --- |
| Switch | square with 0 or 1 | input, click to toggle between 0 and 1 |
| Push button | – | input, 1 only while pressed |
| Constant 0, Constant 1 | – | fixed logic value |
| Clock generator | square wave | regular clock signal 0.2–10 Hz (see [Sequential circuits](sekvencni-obvody.md)) |
| BUF | [1] | buffer |
| NOT | [1] with polarity indicator | inverter |
| AND | [&] | 1 when all inputs are 1 |
| OR | [≥1] | 1 when at least one input is 1 |
| NAND, NOR | [&], [≥1] with polarity indicator | negated AND, negated OR |
| XOR | [=1] | 1 for an odd number of ones |
| XNOR | [=1] with polarity indicator | negated XOR |
| LED | circle | output, lit at 1; shows a letter for X or Z |

Gates have 2 to 8 inputs, set in Properties. The seven-segment and hex displays are described in [Function blocks and displays](funkcni-bloky.md).

## Standard symbols

Gate symbols follow IEC 60617-12 (rectangular symbols with &, ≥1, =1, 1). Negation is shown by the **polarity indicator**, a small right triangle on the pin.

## American symbols

**View → Symbols** switches to American distinctive-shape symbols (ANSI/IEEE Std 91): AND as a “D”, OR as a shield, negation as a bubble. The choice is stored on your computer only, not in the circuit file; a circuit opened elsewhere is shown according to the settings there.

> **Coming soon:** ROM, RAM and EEPROM memories and 74xx chips. The Component library already lists them as upcoming items.
