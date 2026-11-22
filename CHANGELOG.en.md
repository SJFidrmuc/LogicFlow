# LogicFlow changes and roadmap

[Česky](CHANGELOG.md) · **English**

What each version brought and what is planned for the next ones. The newest version is at the top, the roadmap is at the end.

- **Unreleased** – changes already merged into `main` (or prepared in a pull request) that will ship in the next version. Every pull request that changes something visible or important adds a line here, in Czech and in English.
- **When a version is released**, the Unreleased section is renamed to the version number with the date and a new empty one is started. The roadmap is updated to match reality. The procedure is in [CONTRIBUTING.md](CONTRIBUTING.md) → Vydání nové verze (Releasing a new version); `python3 scripts/zmeny.py kontrola` checks it in CI and during the release.

Ready-made programs to download: [Releases](https://github.com/SJFidrmuc/LogicFlow/releases).

## [Unreleased]

### Added
- **VCC supply and GND ground** with symbols as in electrical schematics (logic 1 and 0, no automatic label). Joining VCC to GND is a **power short**: the net turns red and the schematic and status bar show a message, even when the short is inside a subcircuit.
- **Rolling build from the main branch:** the pre-release “Průběžné sestavení (main)” with packages for all platforms is replaced after every change – new features can be tried without waiting for a release. **Version archive** on the web: a table of all releases × platforms with download links.
- **Finer clock settings:** frequency by a slider 0.1–100 Hz (logarithmic, with a field for typing a number), by the period in ms or by quick choices; can be changed while simulating without a phase jump – the clock continues smoothly.
- **Simulation speed ×0.1 to ×10** in the status bar and the Simulation menu; the status bar shows the simulated time.
- **Step to the next clock edge** (Shift+Right arrow) and **step one whole period** to the next rising edge (Ctrl+Right arrow).
- **Clipboard Ctrl+C / X / V**: copying the selected components with the wires between them, pasting at the mouse pointer (the pasted part follows the mouse, a click drops it); works between sheets and between two LogicFlow windows, subcircuits used come along. Copy, Cut and Paste also in the Edit menu and the context menu.
- **Context menu on the right button** depending on the clicked place: on a component Toggle, Frequency, Inputs, Pins, Bits, Count direction, Open subcircuit, label, Rotate, Duplicate, Move, Delete; on a wire Select / Delete the whole net and Delete segment; on an empty place Add component here, Select all, Fit view and simulation controls. Dragging with the right button still pans. Edits are greyed out in simulation mode.
- Right button also in the Hierarchy (Open and Delete subcircuit, Find in schematic) and in the truth table (Copy as text – into Word or a spreadsheet as columns).
- **Edit mode and simulation mode** (as in DigitalWorks): F5 starts the simulation and locks the schematic – only switches, buttons, panning, zooming, Tab and the clock frequency work; Esc or Shift+F5 returns to editing and the circuit state (flip-flops, counters, LEDs, timing diagram) stays. Amber frame with a SIMULATION badge, the mode in the status bar, the message "Stop the simulation to edit (Esc)". Option Simulation → Allow editing while simulating (off by default).
- The **Simulation** workspace switches simulation mode on: switching to it starts the simulation and locks the schematic, leaving it (or Esc) stops it and returns to the previous workspace.
- Menus show the keyboard shortcut of every item and unavailable items are greyed out; the Edit menu now offers Select all, Deselect, Rotate, Delete and Open subcircuit / back, the View menu switches workspaces.
- Unified list of commands in the program – the basis for the context menu and the toolbar.
- Ctrl+Y as a second shortcut for Redo.
- Overview of changes and the roadmap in this file (Czech and English) – checked in CI; release notes are filled in from it automatically.
- Source code comments in Czech and English in all files; checked in CI, and a copy of the sources with comments in one language only can be exported (`scripts/komentare.py`).
- Element library format `.lflib` in the core (loading, saving, checking, names in several languages, categories) – the basis for libraries in version 0.4; format description and a sample library in `docs/knihovny`.
- **Settings file `logicflow.toml`** in the home folder (Windows `%APPDATA%\LogicFlow` – roams with a roaming profile, macOS `~/Library/Application Support/LogicFlow`, Linux `~/.config/logicflow`): readable text with comments, safe writing, an unreadable file is kept as `.bak`. The existing language and symbol settings are taken over. **Edit → Settings…** window: language, symbols, editing while simulating, author name, default licence, personal libraries folder, **shared library folders** (network, OneDrive, club; read-only option), opening the settings folder and restoring defaults. The Documents folder is found via the system known folders (redirection to the network works too).
- **Element libraries in the Component library:** the built-in teaching library “Digital electronics” and personal libraries from the folder `Documents/LogicFlow/Knihovny` (Open folder and Reload buttons), elements by category, search by name and category, the chosen element shows its description, pins, author and licence. An element is inserted as a project subcircuit (a copy) together with the elements it uses – like Ctrl+V at the mouse pointer; names in the interface language. A broken library file is shown as an error, the program does not crash.
- **Teaching library “Digital electronics”** (`docs/knihovny/cislicova-technika.lflib`, built into the program): 36 gate-level circuits of common tasks – gates from NAND and NOR, multiplexers, decoder, comparators, Gray code converters, parity, adders and subtractors, RS and D flip-flops, counters (asynchronous, synchronous, BCD, up/down), shift register, Johnson and ring counters, alarm, fan indicator and traffic light; in Czech and English, every item verified by simulation.

### Fixed
- Ctrl+R restarts the simulation (the shortcut was in the menu and the help but did not work).
- Keys over the schematic react only to the exact combination (Shift+X no longer deletes like X).
- The component library and the memory editor show the current plan for upcoming components (memories in version 0.5, 74xx chips in version 0.7).

### Changed
- **Version plan split:** 0.4 without buses and the School edition, which come in 0.5; the following milestones moved by one version. Upcoming components in the library and the Memory panel show the new versions (memories 0.6, 74xx chips 0.8).
- The author's name is given with the title everywhere – Mgr. Stanislav Jan Fidrmuc (About window, licence, help, libraries, README); a test checks it.
- In edit mode a switch is toggled by a double-click and a push button pressed by the second press of a double-click – a single click only selects the component, so it is not toggled by accident while moving. Clocks stand still in edit mode.
- Space starts the simulation in edit mode; the Right arrow goes from edit mode to a paused simulation and makes a step.
- Preparation for the future web version (iPhone, iPad, Android, Chromebook): the simulation core does not use files or the system clock and CI also compiles it for WebAssembly; time in the application via `web_time`. The program behaves the same.
- Before building, the release workflow checks that the tag matches the program version and that the version is described in the changelog.

## [0.3.1] – 2026-11-21

### Added
- The help is written in Markdown (`docs/napoveda`, Czech and English) and is attached to each release as HTML (ZIP), optionally also published on the web (GitHub Pages).
- Language (Czech / English) and symbol style (European / American) are chosen from the system settings on first start.
- Link to the project website [www.logicflow-sim.cz](https://www.logicflow-sim.cz) in the Help menu and the About window.
- Diagram of the application structure and data flow (`docs/architektura.svg`).

### Fixed
- The clock ticks evenly according to real time (before, it ran about three times slower when idle).

### Changed
- GitHub Actions in CI and the release workflow moved to Node.js 24.

## [0.3.0] – 2026-11-20

No separate GitHub release – the content shipped in version 0.3.1.

### Added
- **Function blocks:** multiplexer, decoder / demultiplexer, priority encoder, adder, comparator, BCD to 7-segment converter, up / down counter, register and shift register with an adjustable number of bits; rectangular symbols according to IEC 60617-12.
- **Displays:** seven-segment display and hex display, choice of the pin side.
- **Subcircuits (macros):** project sheets, interface made of switches and LEDs, Tab into a subcircuit and back, nesting up to depth 16; subcircuits are saved in the circuit file.
- **American symbols** (ANSI/IEEE Std 91), switchable in the View menu; settings and window size are remembered between runs.
- File dialog: New folder button.
- Examples: 4:1 multiplexer, decimal counter with a seven-segment display, full adder made of subcircuits.

### Fixed
- Moving components and drawing wires no longer creates unwanted connections; the wire preview marks the joining points.

## [0.2.0] – 2026-11-19

### Added
- **Sequential logic:** RS flip-flop, level-triggered D latch, edge-triggered D, JK and T flip-flops (rising or falling edge), initial state; clock generator with adjustable frequency.
- Simulation restart (Ctrl+R), clock in the timing diagram, Shift+A menu with categories.
- **Help** (F1) in Czech and English as a window and as an area editor; splitting areas (side by side, one above the other).
- Automatic packages for six platforms on the Releases page (Windows x64/ARM64, macOS ARM/Intel, Linux x64, Raspberry Pi).
- Examples: D flip-flop with a clock, 0–7 counter, shift register.

### Changed
- Negation is drawn with the negation indicator (triangle) according to IEC 60617-12.

## [0.1.0] – 2026-11-18

No GitHub release (source code only).

### Added
- **Schematic drawing:** components on a grid with rotation, wires by dragging, junctions, rectangle selection, moving, duplicating, deleting, undo / redo.
- **Simulation:** event-driven simulator with gate delays, values 0 / 1 / X / Z, stepping and oscillation detection.
- Components: switch, push button, constants, gates with 2–8 inputs, LED.
- Properties panel, hierarchy, timing diagram and truth table from the real circuit.
- Saving and opening `.logicflow` files, own file dialog, prompt about unsaved changes.
- Examples: AND / OR, half adder, RS latch made of NOR gates.

## [0.0.1] – 2026-11-18

Project foundation, not released.

### Added
- Blender-style interface: dark theme, splittable areas, Design / Simulation / Analysis / Memory workspaces.
- Simulation core with values 0 / 1 / X / Z and basic gates, Czech and English.
- CI for Windows, macOS, Linux and Raspberry Pi.

## Roadmap

Approximate – the order and content may change. The detailed plan is kept by the project author.

### 0.4 (2027)
- **Edit mode and simulation mode:** while the simulation runs, the schematic is locked and only the controls work; the circuit state stays after stopping.
- **Context menu** on the right mouse button and **clipboard** (Ctrl+C / X / V).
- Finer setting of the clock frequency and simulation speed, stepping by one edge.
- **VCC and GND** power supply with symbols.
- Author, licence and description stored in the circuit file.
- LED colour and a two-pin LED (anode, cathode), importing a finished circuit as a subcircuit.
- Settings file in the home folder.
- Language packs, in-program help directly from Markdown.
- Light theme (projector, printing), exporting a schematic image as PNG / SVG.
- Update check, splash screen, Support development link.
- **Component libraries:** built-in teaching library, personal libraries and shared folders, exporting a subcircuit to a library.

### 0.5 (2027)
- **Buses 1–64 bits wide:** splitter, tri-state buffer, number input, hex keypad, logic probe, function blocks with bus pins.
- **School edition:** classroom settings with locking, Classroom administration, school libraries (teacher's and class exchange folder).

### 0.6 (2027)
- ROM, RAM and EEPROM memories with a hex editor.
- Toolbar with icons in two forms: compact, and large icons with tabs (touch, projector).
- Subcircuit interface pads (subcircuit inputs and outputs with a name and order, also bus pads) and net labels – connection by name without wires, also for VCC and GND.
- **Matrix circuits:** 4×4 and 3×4 matrix keypad, 5×7 and 8×8 dot-matrix LED display with afterglow (scanning and display multiplexing).

### 0.7 (2027)
- VHDL export.

### 0.8 (2027)
- 74xx series chips (DIP package with VCC and GND pins), PDF manual, sample exercises.
- Two-pin button and switch, pull-up resistor, open-collector outputs.

### 1.0 (September 2027)
- Version for teaching.

### Around version 1.0 and later
- **Web version** for iPhone, iPad, Android and Chromebooks (in the browser, installable on the home screen) with touch control.
- Online circuit library directly in the program.
- Verilog, Karnaugh maps, uploading to an FPGA, exercises with automatic checking.

[Unreleased]: https://github.com/SJFidrmuc/LogicFlow/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/SJFidrmuc/LogicFlow/releases/tag/v0.3.1
[0.3.0]: https://github.com/SJFidrmuc/LogicFlow/compare/v0.2.0...v0.3.1
[0.2.0]: https://github.com/SJFidrmuc/LogicFlow/releases/tag/v0.2.0
[0.1.0]: https://github.com/SJFidrmuc/LogicFlow/commit/2d78684
[0.0.1]: https://github.com/SJFidrmuc/LogicFlow/commit/78db4ae
