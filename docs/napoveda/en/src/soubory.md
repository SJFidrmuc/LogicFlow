# Files, examples and settings

## Saving and opening

- **Ctrl+S** saves the circuit, **Ctrl+Shift+S** saves under a new name, **Ctrl+O** opens, **Ctrl+N** creates a new circuit.
- Circuits are stored in **.logicflow** files including all subcircuits. They are readable text (JSON), easy to hand in, e-mail or keep in Git. The same file opens on Windows, macOS, Linux and Raspberry Pi.
- The file browser has buttons for the home folder and Documents; you can type any folder into the path line. The **+ New folder** button creates a subfolder (e.g. for a class or an assignment) and opens it.
- Before closing the program or opening another circuit, the program asks about unsaved changes.
- A file can also be opened from the command line: `logicflow circuit.logicflow`

## Examples

**File → Examples** offers 9 ready-made circuits. An example opens as a new circuit, so you can change it freely and save it under your own name.

| Example | Shows |
| --- | --- |
| AND and OR gates | basic gates and the truth table |
| Half adder | a combinational circuit from XOR and AND gates |
| SR latch from NOR gates | memory built from gates and the initial state X |
| D flip-flop with a clock | storing a value on the clock edge |
| 0–7 counter from T flip-flops | an asynchronous counter and the timing diagram |
| 4-bit shift register | shifting data through a chain of D flip-flops |
| 4:1 multiplexer | the MUX function block |
| Decade counter with a seven-segment display | counter, BCD/7SEG decoder and reset at 10 |
| Full adder from two subcircuits | subcircuits and nesting |

## Language and symbols

- Switch the **language** with the CS / EN buttons in the top right corner or in **View → Language**. The change applies immediately, without a restart.
- Choose the **symbol style** (European IEC 60617-12 or American ANSI/IEEE) in **View → Symbols**.
- On first start both follow your system: Czech in Czech and Slovak environments, English otherwise; American symbols in a US environment. The program remembers your manual choice.

## Help menu

- **Help** (F1) opens this help in the program.
- **Project website** opens www.logicflow-sim.cz.
- **About** shows the program version, licence and links to the website and source code. Please state the version when reporting a bug.
