# Files, examples and settings

## Saving and opening

- **Ctrl+S** saves the circuit, **Ctrl+Shift+S** saves under a new name, **Ctrl+O** opens, **Ctrl+N** creates a new circuit.
- Circuits are stored in **.logicflow** files including all subcircuits. They are readable text (JSON), easy to hand in, e-mail or keep in Git. The same file opens on Windows, macOS, Linux and Raspberry Pi.
- The file browser has buttons for the home folder and Documents; you can type any folder into the path line. The **+ New folder** button creates a subfolder (e.g. for a class or an assignment) and opens it.
- Before closing the program or opening another circuit, the program asks about unsaved changes.
- A file can also be opened from the command line: `logicflow circuit.logicflow`

## Which program version opens a file

A `.logicflow` file opens in every LogicFlow version that knows **all components** used in it. A newer version opens a file from any older version.

| The circuit contains | Opens in LogicFlow |
| --- | --- |
| gates, switches, push buttons, constants, LEDs | 0.1 and later |
| flip-flops, clock generator | 0.2 and later |
| function blocks, seven-segment display, hex display, subcircuits | 0.3 and later |
| VCC supply and GND ground | 0.4 and later |
| circuit properties (author, licence…) | older versions open it too, but before 0.4 they drop them on saving |

An older version **refuses to open** a file with an unknown component and shows an error – nothing gets damaged. If you hand in or send a circuit to someone with an older version, use only components their version knows. Element libraries (`.lflib`) are supported since version 0.4.

The exact description of both formats – for authors of scripts and other programs – is in the project repository in the folder [docs/formaty](https://github.com/SJFidrmuc/LogicFlow/tree/main/docs/formaty).

## Circuit properties – author and licence

The **File → Circuit properties…** window (or a click on the information at the top of the Hierarchy panel) stores information about the circuit in its file:

- **Title**, **authors**, **description** (what the circuit does, what it is for), **licence** and optionally **school / class**.
- **A new circuit** (Ctrl+N) gets the author and licence from the settings (see below) and today's creation date. Every save records the date of the last modification.
- For someone else's circuit, the **Add me as co-author** button appends your name to the list – the original author stays first.
- Changes take effect with **OK**, **Cancel** discards them. Then save the circuit.
- The title, authors and licence are shown at the top of the **Hierarchy** panel (the description on hover) and in the open dialog after choosing a file – before you open it.
- The **online circuit library** accepts circuits with a title, author, description and the **CC BY 4.0** licence filled in.

> **Tip:** Version 0.3 does not know the circuit information – if you save the file in it, the information is lost.

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
- On first start both follow your system: Czech in Czech and Slovak environments, English otherwise; American symbols in a US environment. The program remembers your manual choice in the settings file.

## Settings

The **Edit → Settings…** window gathers the personal settings. The program keeps them in the settings file.

- **General:** language, symbol style and allowing edits while simulating (the same options as in the menus).
- **Author and licence:** your name and the default licence (CC BY 4.0). They are used for elements exported to a library and in circuit information.
- **Libraries:** the personal libraries folder (default `Documents/LogicFlow/Knihovny`) and **shared folders** – folders on the network, on OneDrive or of a club whose libraries appear in the Component library. For a shared folder, **read-only** means the program never writes into it. An unavailable folder (network disconnected) is shown as a warning in the library.
- **The settings file** is readable text `logicflow.toml`: on Windows in `%APPDATA%\LogicFlow` (with a roaming profile it follows you to other computers), on macOS in `~/Library/Application Support/LogicFlow`, on Linux and Raspberry Pi in `~/.config/logicflow`. The **Open settings folder** button shows it in the file manager.
- The file can be edited by hand. If the program cannot read it, it keeps it as `logicflow.toml.bak` and uses the default settings.
- **Restore default settings…** (with confirmation) sets language and symbols from the system again and clears the author name, licence and library folders.

## Help menu

- **Help** (F1) opens this help in the program.
- **Project website** opens www.logicflow-sim.cz.
- **About** shows the program version, licence and links to the website and source code. Please state the version when reporting a bug.
