# LogicFlow changes and roadmap

[Česky](CHANGELOG.md) · **English**

What each version brought and what is planned for the next ones. The newest version is at the top, the roadmap is at the end.

- **Unreleased** – changes already merged into `main` (or prepared in a pull request) that will ship in the next version. Every pull request that changes something visible or important adds a line here, in Czech and in English.
- **When a version is released**, the Unreleased section is renamed to the version number with the date and a new empty one is started. The roadmap is updated to match reality. The procedure is in [CONTRIBUTING.md](CONTRIBUTING.md) → Vydání nové verze (Releasing a new version); `python3 scripts/zmeny.py kontrola` checks it in CI and during the release.

Ready-made programs to download: [Releases](https://github.com/SJFidrmuc/LogicFlow/releases).

## [Unreleased]

### Added
- Overview of changes and the roadmap in this file (Czech and English) – checked in CI; release notes are filled in from it automatically.
- Source code comments in Czech and English in all files; checked in CI, and a copy of the sources with comments in one language only can be exported (`scripts/komentare.py`).

### Changed
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
- LED colour, subcircuit interface pads, importing a finished circuit as a subcircuit.
- Settings file in the home folder, **School edition** (classroom settings, Classroom administration).
- Language packs, in-program help directly from Markdown.
- Light theme (projector, printing), exporting a schematic image as PNG / SVG.
- Update check, splash screen, Support development link.
- Block libraries (personal, shared folder, school).
- **Buses 1–64 bits wide:** splitter, tri-state buffer, hex keypad, logic probe.

### 0.5 (2027)
- ROM, RAM and EEPROM memories with a hex editor.
- Toolbar with icons in two forms: compact, and large icons with tabs (touch, projector).

### 0.6 (2027)
- VHDL export.

### 0.7 (2027)
- 74xx series chips (DIP package with VCC and GND pins), PDF manual, sample exercises.

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
