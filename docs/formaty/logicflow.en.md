# LogicFlow project format (`.logicflow`)

**Format 1 specification** · status: LogicFlow 0.3.1 and unreleased changes (0.4.0) · [Česky](logicflow.md) · [Format overview and history](README.md#english) · [JSON Schema](logicflow.schema.json)

## 1. Purpose and scope

A `.logicflow` file holds one **project**: the main circuit (a schematic sheet), the definitions of all subcircuits it uses and optional circuit information (author, licence…). The **clipboard** text (Ctrl+C / Ctrl+V) and the text used to insert a library item into a project have the same shape. The **`circuit`** object (chapters 6–10) is shared with the element library [`.lflib`](lflib.en.md).

Code: `crates/logicflow-core/src/circuit.rs` (`ProjectFile`, `Circuit`, `Component`, `ComponentKind`, `Wire`, `SubcircuitDef`, `CircuitInfo`), `block.rs` (`BlockKind`), `gate.rs` (`GateKind`).

## 2. Requirement levels and terms

- **MUST / MUST NOT** – mandatory; a file breaking the rule is not valid. **SHOULD / SHOULD NOT** – recommended; a deviation needs a reason. **MAY** – optional.
- A **writer** is a program creating the file (LogicFlow, a script, a converter). A **reader** is a program loading it.
- The **Since** column gives the first LogicFlow version that knows the field or value. Older versions cannot work with it – see chapter 13.

## 3. The file as a whole

- **JSON** text (RFC 8259) in **UTF-8** without a BOM. Extension `.logicflow`.
- The root is an object (chapter 4). Key order and formatting carry no meaning; LogicFlow writes keys in the order of this specification with 2-space indentation so that files compare well in Git.
- Numbers are integers; they **must not** have a fraction or an exponent.
- A reader **must ignore** keys it does not know (for files from newer versions). A writer **must not** add its own keys outside the `info` part.

## 4. Root object

| Key | Type | Required | Since | Meaning |
| --- | --- | --- | --- | --- |
| `format` | integer ≥ 1 | yes | 0.1.0 | format version, now **1** |
| `app` | string | yes | 0.1.0 | program that saved the file, e.g. `"LogicFlow 0.4.0"` – informative only |
| `info` | object | no | 0.4.0 | circuit information (chapter 5); not written when empty |
| `circuit` | object | yes | 0.1.0 | main circuit (chapter 6) |
| `subcircuits` | array | no (default `[]`) | 0.3.0 | subcircuit definitions (chapter 11); not written when empty |

**Reading procedure:** a reader first checks `format`. If it is higher than it knows, it **must** refuse the file with a message that it comes from a newer program version (LogicFlow: “The file is from a newer version of the application (format N), this version supports format 1.”). A file with `"type": "logicflow-library"` is a library, not a project (see [lflib.en.md](lflib.en.md)).

## 5. Circuit information `info` (since 0.4.0)

All fields are optional and empty ones are not written. The keys match the `info.toml` file of the online circuit library.

| Key | Type | Since | Meaning |
| --- | --- | --- | --- |
| `title` | string | 0.4.0 | circuit title |
| `authors` | array of strings | 0.4.0 | authors; the first is the original author, the others are co-authors |
| `description` | string | 0.4.0 | what the circuit does, what it is for |
| `license` | string | 0.4.0 | licence, e.g. `CC BY 4.0`, `CC BY-SA 4.0`, `CC BY-NC 4.0`, `CC0 1.0`; “neuvedeno” / “not specified” or empty = no licence |
| `school` | string | 0.4.0 | school, class or club |
| `created` | date `YYYY-MM-DD` | 0.4.0 | creation date |
| `modified` | date `YYYY-MM-DD` | 0.4.0 | date of the last change (written on every save) |
| `app_version` | string | 0.4.0 | program version the circuit was created in, e.g. `"0.4.0"` |

A writer that only edits the file **must** keep other keys in `info` (from a newer version or from the online library) unchanged. LogicFlow does so since 0.4.0; versions 0.1–0.3 drop the whole `info` part when saving.

## 6. Circuit `circuit`

One schematic sheet. The same object is used for the project `circuit`, in every subcircuit definition and in every library item.

| Key | Type | Required | Since | Meaning |
| --- | --- | --- | --- | --- |
| `components` | array of objects | yes | 0.1.0 | components (chapter 8) |
| `wires` | array of objects | yes | 0.1.0 | wires (chapter 10) |
| `next_id` | integer ≥ 0 | no (default 0) | 0.1.0 | next free number for a new component or wire |

**Identifiers:** the components and wires of one circuit share one number sequence – `id` **must** be unique within the circuit among both components and wires (0 to 2³²−1). Numbers need not be consecutive. A reader continues numbering from `max(next_id, highest id + 1)`; a writer **should** write a `next_id` greater than all used `id`s.

## 7. Coordinates and rotation

- Points lie on an integer **grid**; the y axis points **down** (as on screen):

| Key | Type | Required | Since | Meaning |
| --- | --- | --- | --- | --- |
| `x` | integer −2³¹ to 2³¹−1 | yes | 0.1.0 | horizontal coordinate, grows to the right |
| `y` | integer −2³¹ to 2³¹−1 | yes | 0.1.0 | vertical coordinate, grows downwards |

- Every component has its **origin** at the point `pos`. Pins lie at fixed local coordinates (chapter 9.7), which are rotated by `rotation` and moved by `pos`:

| `rotation` value | Since | Rotation (clockwise) | Local point (x, y) → world |
| --- | --- | --- | --- |
| `R0` | 0.1.0 | none | (pos.x + x, pos.y + y) |
| `R90` | 0.1.0 | by 90° | (pos.x − y, pos.y + x) |
| `R180` | 0.1.0 | by 180° | (pos.x − x, pos.y − y) |
| `R270` | 0.1.0 | by 270° | (pos.x + y, pos.y − x) |

The digit on displays always stays upright; the pin side of a display is set by `pins`.

## 8. Component

| Key | Type | Required | Since | Meaning |
| --- | --- | --- | --- | --- |
| `id` | integer | yes | 0.1.0 | identifier (chapter 6) |
| `kind` | object | yes | 0.1.0 | kind of component and its settings (chapter 9) |
| `pos` | point | yes | 0.1.0 | position of the symbol's origin |
| `rotation` | string | no (default `R0`) | 0.1.0 | rotation (chapter 7) |
| `label` | string | no (default `""`) | 0.1.0 | label (A, B, Y1…); for switches, buttons and LEDs in a subcircuit it forms the **pin names** (chapter 11) |

The pressed state of a push button is not saved. The state of a switch (`on`) is saved.

## 9. Component kinds `kind`

The `kind` object always has the key **`type`** and further parameters depending on it.

| `type` | Parameters | Inputs | Outputs | Since | Component |
| --- | --- | --- | --- | --- | --- |
| `gate` | `gate`, `inputs` | 1–8 | 1 | 0.1.0 | logic gate (9.1) |
| `switch` | `on` (bool) | 0 | 1 | 0.1.0 | switch – holds 0 or 1 |
| `button` | – | 0 | 1 | 0.1.0 | push button – 1 only while pressed |
| `constant` | `value` (bool) | 0 | 1 | 0.1.0 | constant 0 / 1 |
| `led` | `color` (since 0.4.0) | 1 | 0 | 0.1.0 | light-emitting diode (9.3) |
| `flip_flop` | `ff`, `falling`, `initial` | 2–3 | 2 | 0.2.0 | flip-flop (9.4) |
| `clock` | `period_ms` | 0 | 1 | 0.2.0 | clock generator |
| `block` | `block` (object) | per block | per block | 0.3.0 | function block (9.5) |
| `seven_segment` | `pins` | 8 | 0 | 0.3.0 | seven-segment display (9.3) |
| `hex_display` | `pins` | 4 | 0 | 0.3.0 | hexadecimal display (9.3) |
| `subcircuit` | `id`, `inputs`, `outputs` | 0–32 | 0–32 | 0.3.0 | subcircuit instance (9.6) |
| `vcc` | – | 0 | 1 | 0.4.0 | supply – logic 1 with a power symbol |
| `gnd` | – | 0 | 1 | 0.4.0 | ground – logic 0 with a ground symbol |
| `led_ak` | `color` | 2 | 0 | 0.4.0 | light-emitting diode with two pins – anode and cathode (9.3) |

A reader that does not know a `type` **must** refuse the file (it must not silently skip the component – that would change the circuit).

### 9.1 Gates (`gate`)

`{"type": "gate", "gate": "Nand", "inputs": 2}` – `inputs` is the number of inputs: for `Buffer` and `Not` it **must** be 1, for the others 2–8. The `gate` value starts with a capital letter:

| `gate` | Since | Function |
| --- | --- | --- |
| `Buffer` | 0.1.0 | buffer Y = A |
| `Not` | 0.1.0 | negation Y = Ā |
| `And` | 0.1.0 | logical product |
| `Or` | 0.1.0 | logical sum |
| `Nand` | 0.1.0 | negated product |
| `Nor` | 0.1.0 | negated sum |
| `Xor` | 0.1.0 | odd parity of the inputs (exclusive or) |
| `Xnor` | 0.1.0 | even parity of the inputs |

### 9.2 Signal sources

- `switch` – `on`: `true` = output 1. A **circuit input** (a pin in a subcircuit).
- `button` – 1 only while pressed during simulation, otherwise 0. A **circuit input**.
- `constant` – `value`: `true` = 1.
- `vcc` / `gnd` (since 0.4.0) – logic 1 / 0 with an electrical symbol, no automatic label. Joining sources with different fixed levels (VCC, GND, constants) is a **power short**.
- `clock` – `period_ms`: period in milliseconds, integer ≥ 1; LogicFlow offers 10–10,000 ms (0.1–100 Hz) and clamps values outside the range when edited. A clock outputs 0 when the simulation starts.

### 9.3 Displays

- `led` – a **circuit output** (a pin in a subcircuit); lights at 1 on its input.
- `led_ak` (since 0.4.0) – a light-emitting diode with inputs **A** (anode) and **K** (cathode); it lights at A = 1 and K = 0 and stays dark with X or Z on any pin. It is **not** a circuit output (subcircuit interface, truth table). It is a separate kind, not a parameter of `led`, so that older versions do not open it as a one-pin LED.

The key `color` (optional, default `red`) of `led` and `led_ak` sets the colour of the light; it does not affect the simulation. LogicFlow does not write the default `red`. Older versions skip the key on `led` (the LED is then red), so they open the file:

| `color` | Since | Colour |
| --- | --- | --- |
| `red` | 0.4.0 | red |
| `green` | 0.4.0 | green |
| `yellow` | 0.4.0 | yellow |
| `blue` | 0.4.0 | blue |
| `white` | 0.4.0 | white |

- `seven_segment` – inputs **a, b, c, d, e, f, g, dp** in this order.
- `hex_display` – inputs **D0–D3** (D0 the lowest bit); shows 0–F.

The key `pins` (optional, default `left`) sets the side with the pins of `seven_segment` and `hex_display`:

| `pins` | Since | Pins |
| --- | --- | --- |
| `left` | 0.3.0 | on the left, top to bottom |
| `right` | 0.3.0 | on the right, top to bottom |
| `top` | 0.3.0 | on the top, left to right |
| `bottom` | 0.3.0 | on the bottom, left to right |

### 9.4 Flip-flops (`flip_flop`)

`{"type": "flip_flop", "ff": "jk", "falling": false, "initial": false}`

| `ff` | Since | Inputs (from the top) | Triggering |
| --- | --- | --- | --- |
| `rs` | 0.2.0 | S, R | asynchronous |
| `d_latch` | 0.2.0 | D, C | level – “transparent” while C = 1 |
| `d` | 0.2.0 | D, C | edge of C |
| `jk` | 0.2.0 | J, C, K | edge of C |
| `t` | 0.2.0 | T, C | edge of C |

The outputs are always **Q** (top) and **Q̄** (bottom). `falling` (optional, default `false`): `true` = triggered by the falling edge, otherwise the rising one; not used for `rs` and `d_latch`. `initial` (optional, default `false`): the state of Q when the simulation starts.

### 9.5 Function blocks (`block`)

`{"type": "block", "block": {"kind": "counter", "bits": 4, "down": false}}` – the nested `block` object has the key **`kind`** and a parameter. The parameter **must** be within the given range. Bits are counted from the lowest one (index 0 at the top).

| `kind` | Parameters (range) | Inputs from the top | Outputs from the top | Since |
| --- | --- | --- | --- | --- |
| `mux` | `select` 1–3 | S0…S(s−1), D0…D(2ˢ−1), EN | Y | 0.3.0 |
| `decoder` | `bits` 1–3 | A0…A(n−1), EN | Y0…Y(2ⁿ−1) | 0.3.0 |
| `encoder` | `bits` 2–3 | I0…I(2ⁿ−1) | A0…A(n−1), V | 0.3.0 |
| `adder` | `bits` 1–4 | A0…A(n−1), B0…B(n−1), CI | S0…S(n−1), CO | 0.3.0 |
| `comparator` | `bits` 1–4 | A0…A(n−1), B0…B(n−1) | A<B, A=B, A>B | 0.3.0 |
| `bcd_to7_seg` | – | D0…D3 | a…g | 0.3.0 |
| `counter` | `bits` 2–8, `down` (bool) | R, EN, C | Q0…Q(n−1), CO | 0.3.0 |
| `register` | `bits` 2–8 | D0…D(n−1), R, C | Q0…Q(n−1) | 0.3.0 |
| `shift_register` | `bits` 2–8 | SI, R, C | Q0…Q(n−1) | 0.3.0 |

In sequential blocks (`counter`, `register`, `shift_register`) the clock input C is the last one. An unconnected R input behaves as 0, an unconnected EN as 1.

### 9.6 Subcircuit instance (`subcircuit`)

`{"type": "subcircuit", "id": 1, "inputs": 3, "outputs": 2}` – `id` refers to a definition in `subcircuits` (in a library to the item with the same `id`, see [lflib.en.md](lflib.en.md)). `inputs` and `outputs` are a copy of the definition's interface (0–32) so that the symbol can be drawn without the definition. A writer **must** write the counts according to the definition; a reader **should** recompute them from the definition (LogicFlow does so on saving). For the pin order see chapter 11.

### 9.7 Pin positions

Local coordinates for `R0` (rotation in chapter 7). Pins are counted from the top (for `top`/`bottom` from the left) in the order of the tables above; *k* = 0, 1, 2…

| Component | Inputs | Outputs |
| --- | --- | --- |
| `gate` with *n* inputs | (−3, −(*n*−1) + 2*k*) | (3, 0) |
| `switch`, `button`, `constant` | – | (2, 0) |
| `vcc` | – | (0, 1) |
| `gnd` | – | (0, −1) |
| `led` | (−2, 0) | – |
| `led_ak` | A (−2, 0), K (2, 0) | – |
| `flip_flop` `jk` | (−3, −2), (−3, 0), (−3, 2) | Q (3, −2), Q̄ (3, 2) |
| `flip_flop` others | (−3, −1), (−3, 1) | Q (3, −1), Q̄ (3, 1) |
| `clock` | – | (3, 0) |
| `block`, `subcircuit` (rectangle), *m* = max(inputs, outputs, 1) | (−4, 2 − *m* + 2*k*) | (4, 2 − *m* + 2*k*) |
| `seven_segment` `left` / `right` | (∓4, −7 + 2*k*) | – |
| `seven_segment` `top` / `bottom` | (−7 + 2*k*, ∓7) | – |
| `hex_display` `left` / `right` | (∓4, −3 + 2*k*) | – |
| `hex_display` `top` / `bottom` | (−3 + 2*k*, ∓5) | – |

(For `∓` the upper sign applies to `left` / `top`.)

## 10. Wires and connections

| Key | Type | Required | Since | Meaning |
| --- | --- | --- | --- | --- |
| `id` | integer | yes | 0.1.0 | identifier (chapter 6) |
| `a` | point | yes | 0.1.0 | first end |
| `b` | point | yes | 0.1.0 | second end |

- A wire **must** be horizontal (`a.y = b.y`) or vertical (`a.x = b.x`); a bent wire is written as several segments. A writer **should not** write zero-length wires.
- Wires connect **at their end points**. A wire end lying inside another wire connects to it too (a **“T” junction**). Wires that only **cross** are **not** connected.
- A component **pin** connects to a wire whose end lies on it (or which passes through it at a point where another wire ends). A wire merely passing over a pin does not connect to it.
- Pins of different components at the same point are connected even without a wire.
- A group of connected wires and pins is one **net**. An unconnected input has the value Z (high impedance), which gates read as X.

## 11. Subcircuits `subcircuits` (since 0.3.0)

| Key | Type | Required | Since | Meaning |
| --- | --- | --- | --- | --- |
| `id` | integer | yes | 0.3.0 | definition number – instances refer to it; unique in the file |
| `name` | string | yes | 0.3.0 | name on the symbol and in the Hierarchy |
| `circuit` | object | yes | 0.3.0 | the subcircuit's circuit (chapter 6) |

- The **interface** of a subcircuit is formed by components inside it: **inputs** = `switch` and `button`, **outputs** = `led`. Each group is sorted **naturally** by `label`: the label is split before its first digit into text and a number; the text is compared (by Unicode), then the number, finally the whole label – so `A2` < `A10` < `B`. The first one is the top pin of the symbol. At most 32 inputs and 32 outputs apply; further ones are not connected.
- Pin labels **should** be unique and non-empty.
- An instance refers only to definitions in the same file. A subcircuit **must not** contain itself (even indirectly). Nesting is simulated to a depth of **16**; deeper instances stay unconnected.

## 12. Rules for writers

When saving, LogicFlow:

- writes `format` = the current format version and `app` = `"LogicFlow X.Y.Z"`;
- recomputes `inputs` / `outputs` of subcircuit instances from the definitions;
- in the `info` part writes `modified` (and for a new circuit `created` and `app_version`), omits empty values and keeps unknown keys;
- omits an empty `info` and empty `subcircuits`.

Another writer **must** follow the required fields and ranges of chapters 4–11; it **should** follow the remaining points.

## 13. Compatibility with program versions

| File contains | LogicFlow 0.1 | 0.2 | 0.3 | 0.4+ |
| --- | --- | --- | --- | --- |
| gates, switches, push buttons, constants, LEDs | yes | yes | yes | yes |
| flip-flops, clocks | refuses | yes | yes | yes |
| function blocks, displays, subcircuits | refuses | refuses | yes | yes |
| `vcc`, `gnd`, `led_ak` | refuses | refuses | refuses | yes |
| `color` on `led` | opens, the LED is red, drops the colour on saving | same | same | yes |
| the `info` part | opens, drops it on saving | same | same | yes |
| `format` higher than 1 | refuses | refuses | refuses | refuses |

- The **lowest version** that opens a file = the highest “Since” of all `type`, `gate`, `ff`, block `kind` and `pins` values used in the file (`color` does not count – older versions skip it).
- A new component or value in future versions does not increase `format`, but an older version refuses such a file (see chapter 9). Older versions skip new optional fields.
- Within format 1 keys and values are never renamed or removed and every new field has a default value – so a newer version always opens a file from any older version.

## 14. Example

An inverter made of a NAND gate as a subcircuit: switch A → subcircuit “Invertor z NAND” → LED Y. Inside the subcircuit both NAND inputs are joined by a “T” junction and connected to switch A. The test `formaty.rs` checks that the example loads and that Y = Ā. (The example is identical in both language versions of this specification.)

<!-- example: logicflow -->
```json
{
  "format": 1,
  "app": "LogicFlow 0.4.0",
  "info": {
    "title": "Invertor z NAND",
    "authors": ["Mgr. Stanislav Jan Fidrmuc"],
    "license": "CC BY 4.0",
    "created": "2026-11-27",
    "modified": "2026-11-27",
    "app_version": "0.4.0"
  },
  "circuit": {
    "components": [
      { "id": 0, "kind": { "type": "switch", "on": false }, "pos": { "x": 0, "y": 0 }, "rotation": "R0", "label": "A" },
      { "id": 1, "kind": { "type": "subcircuit", "id": 1, "inputs": 1, "outputs": 1 }, "pos": { "x": 8, "y": -1 }, "rotation": "R0", "label": "P1" },
      { "id": 2, "kind": { "type": "led" }, "pos": { "x": 16, "y": 0 }, "rotation": "R0", "label": "Y" }
    ],
    "wires": [
      { "id": 3, "a": { "x": 2, "y": 0 }, "b": { "x": 4, "y": 0 } },
      { "id": 4, "a": { "x": 12, "y": 0 }, "b": { "x": 14, "y": 0 } }
    ],
    "next_id": 5
  },
  "subcircuits": [
    {
      "id": 1,
      "name": "Invertor z NAND",
      "circuit": {
        "components": [
          { "id": 0, "kind": { "type": "switch", "on": false }, "pos": { "x": 0, "y": 0 }, "rotation": "R0", "label": "A" },
          { "id": 1, "kind": { "type": "gate", "gate": "Nand", "inputs": 2 }, "pos": { "x": 8, "y": 0 }, "rotation": "R0", "label": "G1" },
          { "id": 2, "kind": { "type": "led" }, "pos": { "x": 14, "y": 0 }, "rotation": "R0", "label": "Y" }
        ],
        "wires": [
          { "id": 3, "a": { "x": 2, "y": 0 }, "b": { "x": 4, "y": 0 } },
          { "id": 4, "a": { "x": 4, "y": -1 }, "b": { "x": 4, "y": 1 } },
          { "id": 5, "a": { "x": 4, "y": -1 }, "b": { "x": 5, "y": -1 } },
          { "id": 6, "a": { "x": 4, "y": 1 }, "b": { "x": 5, "y": 1 } },
          { "id": 7, "a": { "x": 11, "y": 0 }, "b": { "x": 12, "y": 0 } }
        ],
        "next_id": 8
      }
    }
  ]
}
```
