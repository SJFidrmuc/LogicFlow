# LogicFlow element library format (`.lflib`)

**Format 1 specification** · status: LogicFlow 0.3.1 and unreleased changes (0.4.0) · [Česky](lflib.md) · [Format overview and history](README.md#english) · [JSON Schema](lflib.schema.json)

## 1. Purpose and scope

A library is a file of ready-made subcircuits – **items** – that can be inserted into any project. The program shows them in the Component library under Element libraries: the built-in library, personal libraries (`Documents/LogicFlow/Knihovny`) and libraries in shared folders. A user overview and sample files are in [docs/knihovny](../knihovny/README.md#english).

The `.lflib` format exists **since version 0.4.0**. Item circuits use the `circuit` object from the project format – chapters 6–11 of the [logicflow.en.md](logicflow.en.md) specification apply unchanged and are not repeated here.

Code: `crates/logicflow-core/src/library.rs` (`Library`, `LibraryInfo`, `LibraryItem`, `LibraryError`, `CATEGORIES`).

## 2. Requirement levels and terms

As in [logicflow.en.md](logicflow.en.md), chapter 2 (MUST / SHOULD / MAY, writer, reader, Since). In addition:

- **Item** – one subcircuit in the library with a name, description, author, licence and version.
- **Identity** (`uid`) – a permanent label of the library or an item that does not change on renaming or editing.
- **Number** (`id`) – the item number within the file that instances inside other items of the same library refer to.

## 3. The file as a whole

- **JSON** text in **UTF-8** without a BOM, extension `.lflib`. Other rules as for a project ([logicflow.en.md](logicflow.en.md), chapter 3): integers only, key order and formatting do not matter, LogicFlow indents by 2 spaces.
- A library is told apart from a project by `"type": "logicflow-library"`.
- A reader **must** ignore unknown keys. LogicFlow 0.4 does not keep them when saving a library.

## 4. Root object

| Key | Type | Required | Since | Meaning |
| --- | --- | --- | --- | --- |
| `format` | integer ≥ 1 | yes | 0.4.0 | library format version, now **1** (independent of the project format) |
| `type` | string | yes | 0.4.0 | always `logicflow-library` |
| `app` | string | no | 0.4.0 | program that saved the file, e.g. `"LogicFlow 0.4.0"` – informative only |
| `library` | object | yes | 0.4.0 | library header (chapter 5) |
| `items` | array of objects | no (default `[]`) | 0.4.0 | items (chapter 6) in the order the panel shows them within a category |

**Reading procedure:** (1) the text must be JSON; (2) `type` must be `logicflow-library`, otherwise the file is not a library; (3) if `format` is higher than the reader knows, it **must** refuse the file with an explanation – only then is the rest of the structure read, because a newer format may have a different shape; (4) the checks of chapter 9.

## 5. Header `library`

| Key | Type | Required | Since | Meaning |
| --- | --- | --- | --- | --- |
| `uid` | UUID | yes | 0.4.0 | library identity (chapter 7.1) |
| `name` | string | yes | 0.4.0 | library name in the author's language |
| `description` | string | no | 0.4.0 | library description |
| `translations` | object | no | 0.4.0 | translations of the name and description (chapter 7.3); not written when empty |
| `author` | string | no | 0.4.0 | author or maintainer of the library |
| `license` | string | no | 0.4.0 | library licence (chapter 7.4) |
| `version` | integer ≥ 1 | no (default 1) | 0.4.0 | library version – increased on every save |
| `created` | date | no | 0.4.0 | creation date `YYYY-MM-DD` |
| `modified` | date | no | 0.4.0 | date of the last save `YYYY-MM-DD` |

## 6. Item (`items[]`)

| Key | Type | Required | Since | Meaning |
| --- | --- | --- | --- | --- |
| `uid` | UUID | yes | 0.4.0 | item identity (chapter 7.1) – tells which item a copy in a project came from |
| `id` | integer ≥ 0 | yes | 0.4.0 | item number within the file; instances `{"type": "subcircuit", "id": …}` in other items refer to it |
| `name` | string | yes | 0.4.0 | name on the symbol, in the author's language |
| `category` | string | no | 0.4.0 | built-in category key (chapter 8) or custom text |
| `description` | string | no | 0.4.0 | description – what the item does, formulas, a link to a task |
| `translations` | object | no | 0.4.0 | translations of the name and description (chapter 7.3) |
| `author` | string | no | 0.4.0 | author of the item |
| `license` | string | no | 0.4.0 | item licence (chapter 7.4) |
| `version` | integer ≥ 1 | no (default 1) | 0.4.0 | item version – increased on every change of the circuit or description |
| `created` | date | no | 0.4.0 | creation date |
| `modified` | date | no | 0.4.0 | date of the last change |
| `inputs` | array of strings | no | 0.4.0 | input names – **informative only** (chapter 6.1) |
| `outputs` | array of strings | no | 0.4.0 | output names – **informative only** |
| `circuit` | object | yes | 0.4.0 | the item's circuit – the `circuit` object from [logicflow.en.md](logicflow.en.md), chapter 6 |

### 6.1 Item interface

The interface (the symbol's pins) is determined as for a project subcircuit ([logicflow.en.md](logicflow.en.md), chapter 11): inputs = switches and push buttons, outputs = LEDs inside `circuit`, sorted naturally by label, at most 32 on each side. The `inputs` and `outputs` arrays are a copy of these names for readers of the file and the library website: a writer **should** write them, a reader **must not** use them instead of the circuit (LogicFlow ignores them on loading and recomputes them on saving).

### 6.2 Items using other items

A `subcircuit` instance inside an item refers by its `id` to the **number of another item of the same library**. A library has no `subcircuits` part – the items themselves play that role. A reference to a number that is not in the library is an error; an item **must not** contain itself (even indirectly). When inserting into a project, all items used by the inserted item are inserted as well.

## 7. Common data types

### 7.1 Identity `uid`

- Text in the UUID form `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (8-4-4-4-12 hexadecimal digits). A writer **should** use random version 4 UUIDs in lowercase; a reader compares **case-insensitively**.
- Item `uid`s **must** be unique within the library. A `uid` **must not** change on renaming, editing the circuit or moving to another category. A new item – even one created as a variant by copying another – gets a new `uid`.
- The library `uid` identifies the library as a whole and does not change between versions of the file.

### 7.2 Number `id`

An integer, unique among the items of the library. Numbers need not be consecutive. A new item gets a number one higher than the highest used. An item number should not change, because other items refer to it.

### 7.3 Translations `translations`

An object whose keys are **ISO 639-1** language codes (`en`, `cs`, `sk`, `uk`, `de` …), the values objects with optional keys `name` and `description`:

```json
"translations": { "en": { "name": "1-bit full adder", "description": "Full adder with carry in and carry out." } }
```

| Key | Type | Since | Meaning |
| --- | --- | --- | --- |
| `name` | string | 0.4.0 | translated name; empty = the main one is used |
| `description` | string | 0.4.0 | translated description; empty = the main one is used |

The program shows the text in the interface language if the translation exists and is not empty, otherwise the main `name` / `description`. Empty values are not written.

### 7.4 Licence and dates

- `license` is free text. The program offers `CC BY 4.0`, `CC BY-SA 4.0`, `CC BY-NC 4.0`, `CC0 1.0` and “neuvedeno” / “not specified” (= no licence, like empty text). **The default licence of new libraries and items is `CC BY 4.0`** (as in the online circuit library).
- Dates have the form `YYYY-MM-DD` or are empty.

## 8. Categories

`category` is a built-in category key or custom text (saved as the author wrote it, trimmed). The panel orders built-in categories as in the table and custom ones after them.

| Key | Česky | English | Since |
| --- | --- | --- | --- |
| `combinational` | Kombinační obvody | Combinational circuits | 0.4.0 |
| `arithmetic` | Aritmetika | Arithmetic | 0.4.0 |
| `code-converters` | Převodníky kódů | Code converters | 0.4.0 |
| `sequential` | Sekvenční obvody | Sequential circuits | 0.4.0 |
| `memory` | Paměti | Memories | 0.4.0 |
| `applications` | Aplikace | Applications | 0.4.0 |
| `other` | Ostatní | Other | 0.4.0 |

A reader recognises a built-in category by its key and by its Czech or English name, ignoring letter case and surrounding spaces. A writer **should** write a built-in category as its key (LogicFlow converts `Aritmetika` → `arithmetic`). An unknown key from a newer version is shown as a custom category.

## 9. Checks on loading

A reader **must** refuse the library when:

| Error (`LibraryError`) | LogicFlow message (cs / en) |
| --- | --- |
| the text is not JSON or does not match the structure | Soubor knihovny nejde přečíst: … / The library file cannot be read: … |
| `"type": "logicflow-library"` is missing | Soubor není knihovna LogicFlow. / The file is not a LogicFlow library. |
| `format` is higher than known | Knihovna je z novější verze aplikace (formát N), tato verze umí formát 1. / The library is from a newer version of the application (format N), this version supports format 1. |
| the `uid` of the library or an item is not a UUID | Neplatná identita (uid) „…“. / Invalid identity (uid) “…”. |
| two items have the same `uid` | Dva prvky mají stejnou identitu „…“. / Two items have the same identity “…”. |
| two items have the same `id` | Dva prvky mají stejné číslo N. / Two items have the same number N. |
| an instance refers to a number that is not in the library | Prvek „…“ používá prvek číslo N, který v knihovně není. / Item “…” uses item number N, which is not in the library. |
| an item contains itself | Prvek „…“ obsahuje sám sebe. / Item “…” contains itself. |

A broken library is shown as an error in the panel; the program does not crash and loads the other libraries.

## 10. Rules for writers

When saving a library, LogicFlow:

- writes `format` = 1, `type` = `logicflow-library`, `app` = `"LogicFlow X.Y.Z"`;
- converts built-in categories to keys, recomputes `inputs` / `outputs` from the circuit and the pin counts of instances from the items they refer to;
- increases the library `version` and writes `modified`; for a changed item increases its `version` and writes `modified`;
- creates a new `uid` from 16 random bytes (a version 4 UUID); the application provides the date and randomness, the core does not need them (because of the web version).

## 11. Inserting an item into a project (informative)

An item is inserted into a project as a **copy**, just like Ctrl+V: the core prepares text in the [`.logicflow`](logicflow.en.md) shape with one instance of the item and the definitions of all items it uses (`Library::insertion_text`, names in the interface language), and the project inserts it as from the clipboard. An origin record (library, item `uid` and `version`) on the inserted subcircuit is being prepared (stage K1, step 4) – it will be added as an optional field of the project format with Since according to its release.

## 12. Compatibility with program versions

- LogicFlow supports libraries **since 0.4.0**; older versions do not open `.lflib` files (they have no libraries).
- Item circuits follow the same rules as a project ([logicflow.en.md](logicflow.en.md), chapter 13): the lowest version that loads a library is the highest “Since” of all components in all items – but at least 0.4.0. A library with an unknown component is shown as an error in the panel.
- Within format 1 keys and values are never renamed or removed and every new field has a default value.

## 13. Example

Two items: an inverter made of a NAND gate (number 1) and a buffer made of two inverters (number 2), which uses item 1 twice. The test `formaty.rs` checks that the example loads and that both items compute correctly. A fuller sample is [docs/knihovny/vzor.lflib](../knihovny/vzor.lflib). (The example is identical in both language versions of this specification.)

<!-- example: lflib -->
```json
{
  "format": 1,
  "type": "logicflow-library",
  "app": "LogicFlow 0.4.0",
  "library": {
    "uid": "3f6c2d1a-9b4e-4c7f-8a21-5d0e7b9c4f12",
    "name": "Příklad ze specifikace",
    "description": "Invertor z NAND a opakovač, který ho používá.",
    "translations": { "en": { "name": "Specification example", "description": "An inverter made of NAND and a buffer that uses it." } },
    "author": "Mgr. Stanislav Jan Fidrmuc",
    "license": "CC BY 4.0",
    "version": 1,
    "created": "2026-11-27",
    "modified": "2026-11-27"
  },
  "items": [
    {
      "uid": "b1d7e0c4-2f3a-4e58-9c6d-7a8b9c0d1e2f",
      "id": 1,
      "name": "Invertor z NAND",
      "category": "combinational",
      "description": "Oba vstupy hradla NAND spojené: Y = Ā.",
      "translations": { "en": { "name": "Inverter from NAND", "description": "Both inputs of a NAND gate joined: Y = Ā." } },
      "author": "Mgr. Stanislav Jan Fidrmuc",
      "license": "CC BY 4.0",
      "version": 1,
      "created": "2026-11-27",
      "modified": "2026-11-27",
      "inputs": ["A"],
      "outputs": ["Y"],
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
    },
    {
      "uid": "c2e8f1d5-3a4b-4f69-8d7e-8b9c0d1e2f30",
      "id": 2,
      "name": "Opakovač ze dvou invertorů",
      "category": "combinational",
      "description": "Dva invertory za sebou: Y = A.",
      "translations": { "en": { "name": "Buffer from two inverters", "description": "Two inverters in series: Y = A." } },
      "author": "Mgr. Stanislav Jan Fidrmuc",
      "license": "CC BY 4.0",
      "version": 1,
      "created": "2026-11-27",
      "modified": "2026-11-27",
      "inputs": ["A"],
      "outputs": ["Y"],
      "circuit": {
        "components": [
          { "id": 0, "kind": { "type": "switch", "on": false }, "pos": { "x": 0, "y": 0 }, "rotation": "R0", "label": "A" },
          { "id": 1, "kind": { "type": "subcircuit", "id": 1, "inputs": 1, "outputs": 1 }, "pos": { "x": 8, "y": -1 }, "rotation": "R0", "label": "P1" },
          { "id": 2, "kind": { "type": "subcircuit", "id": 1, "inputs": 1, "outputs": 1 }, "pos": { "x": 18, "y": -1 }, "rotation": "R0", "label": "P2" },
          { "id": 3, "kind": { "type": "led" }, "pos": { "x": 26, "y": 0 }, "rotation": "R0", "label": "Y" }
        ],
        "wires": [
          { "id": 4, "a": { "x": 2, "y": 0 }, "b": { "x": 4, "y": 0 } },
          { "id": 5, "a": { "x": 12, "y": 0 }, "b": { "x": 14, "y": 0 } },
          { "id": 6, "a": { "x": 22, "y": 0 }, "b": { "x": 24, "y": 0 } }
        ],
        "next_id": 7
      }
    }
  ]
}
```
