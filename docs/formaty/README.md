# Formáty souborů LogicFlow

**Česky** · [English](#english)

Tato složka je **závazný popis souborů**, které LogicFlow čte a zapisuje. Slouží autorům dalších programů (web online knihovny, převodníky, export do VHDL), učitelům, kteří soubory generují skriptem, i vývojářům LogicFlow.

| Soubor | Přípona | Specifikace | JSON Schema |
| --- | --- | --- | --- |
| Projekt (obvod se všemi podobvody); stejný tvar má i schránka | `.logicflow` | [logicflow.md](logicflow.md) · [English](logicflow.en.md) | [logicflow.schema.json](logicflow.schema.json) |
| Knihovna prvků | `.lflib` | [lflib.md](lflib.md) · [English](lflib.en.md) | [lflib.schema.json](lflib.schema.json) |

Uživatelský přehled knihoven a vzorové knihovny jsou v [docs/knihovny](../knihovny/README.md), soubor s nastavením `logicflow.toml` popisuje nápověda (kapitola Soubory, příklady a nastavení).

**Stav dokumentace:** odpovídá LogicFlow **0.3.1** a nevydaným změnám ve větvi `main` (připravovaná verze 0.4.0). Aktuální verze formátu: `.logicflow` **1**, `.lflib` **1**.

## Verze formátu a verze programu

- Pole `format` v souboru je **verze formátu**. Zvyšuje se jen při **nekompatibilní** změně – kdyby starší program soubor přečetl, ale pochopil ho jinak. Program soubor s vyšší verzí formátu odmítne s vysvětlující hláškou.
- Nové součástky a nová nepovinná pole formát **nemění**. Proto je u každé součástky a pole ve specifikaci sloupec **Od verze** – první verze programu, která je umí.
- **Kterou verzí soubor otevřu:** nejnižší verze programu, která soubor otevře, je nejvyšší „Od verze“ ze všech součástek, které soubor používá. Starší program soubor s neznámou součástkou **odmítne otevřít** (chyba při čtení). Neznámé **pole** naopak přeskočí – soubor otevře, ale při uložení pole zahodí (výjimka: neznámé klíče v části `info` se od 0.4.0 zachovají).

## Historie formátů

| Verze programu | `.logicflow` | `.lflib` | Co přibylo |
| --- | --- | --- | --- |
| 0.0.1 | – | – | bez ukládání souborů |
| 0.1.0 | 1 | – | soubor projektu: hradla Buffer, NOT, AND, OR, NAND, NOR, XOR, XNOR (2–8 vstupů), přepínač, tlačítko, konstanta, LED; vodiče, otočení, popisky |
| 0.2.0 | 1 | – | klopné obvody RS, D (úroveň), D, JK, T (`falling`, `initial`); hodiny (`period_ms`) |
| 0.3.0 | 1 | – | funkční bloky (`block`: MUX, dekodér, kodér, sčítačka, komparátor, BCD/7SEG, čítač, registr, posuvný registr), sedmisegmentovka a hex displej (`pins`), podobvody (`subcircuit`, část `subcircuits`); vyšlo jako 0.3.1 |
| 0.3.1 | 1 | – | beze změny formátu |
| 0.4.0 – nevydáno | 1 | 1 | napájení `vcc` a zem `gnd`; barva LED (`color`) a LED se dvěma vývody `led_ak`; informace o obvodu (část `info`, neznámé klíče se zachovají); knihovna prvků `.lflib` |

Verze **0.3.0** nevyšla samostatně, její obsah vyšel ve 0.3.1 – v praxi tedy „od 0.3.0“ znamená „od 0.3.1“.

## Údržba (pro vývojáře)

Dokumentace formátů je součástí každé změny, která formát dotkne (viz [CONTRIBUTING.md](../../CONTRIBUTING.md) → Formáty souborů):

1. **Nová součástka, blok, druh hradla nebo pole:** řádek ve specifikaci **česky i anglicky** se sloupcem Od verze = připravovaná verze, doplnit JSON Schema a řádek historie výše (u verze označené „nevydáno“).
2. **Nekompatibilní změna** (přejmenování nebo odebrání klíče či hodnoty, jiný význam existující hodnoty): zvýšit `FORMAT` v kódu, popsat v historii a ve specifikaci, jak program čte starší formát. Uvnitř jedné verze formátu se klíče ani hodnoty **nikdy nepřejmenovávají ani neodebírají** a nová pole musí mít výchozí hodnotu.
3. **Vydání verze:** u řádku historie odstranit „nevydáno“, aktualizovat **Stav dokumentace** a sloupce Od verze nechat (verze už sedí).

Hlídají to dvě kontroly v CI:

- **Test `crates/logicflow-core/tests/formaty.rs`** – každý druh součástky, bloku, hradla, klopného obvodu, strany vývodů a otočení, každé pole, které program zapisuje, i rozsahy parametrů musí být popsané v obou jazycích i ve schématu, s verzí Od verze; příklady ve specifikaci se musí načíst a správně simulovat. Nový druh součástky bez záznamu v testu se ani nepřeloží (úplný `match`).
- **Skript `python3 scripts/formaty.py kontrola`** – schémata jsou platná, vzorové knihovny, obvody nápovědy a příklady ze specifikace jim odpovídají; historie výše má řádek pro každou verzi z přehledu změn, nevydané verze jsou označené, Stav dokumentace odpovídá verzi v `Cargo.toml` a všechny hodnoty Od verze jsou v historii. Potřebuje `pip install jsonschema`.

---

## English

This folder is the **normative description of the files** LogicFlow reads and writes – for authors of other programs (online library website, converters, VHDL export), teachers generating files by script, and LogicFlow developers.

| File | Extension | Specification | JSON Schema |
| --- | --- | --- | --- |
| Project (a circuit with all its subcircuits); the clipboard has the same shape | `.logicflow` | [logicflow.en.md](logicflow.en.md) · [česky](logicflow.md) | [logicflow.schema.json](logicflow.schema.json) |
| Element library | `.lflib` | [lflib.en.md](lflib.en.md) · [česky](lflib.md) | [lflib.schema.json](lflib.schema.json) |

**Documentation status:** matches LogicFlow **0.3.1** and the unreleased changes on the `main` branch (upcoming version 0.4.0). Current format version: `.logicflow` **1**, `.lflib` **1**.

### Format version and program version

- The `format` field is the **format version**. It is increased only on an **incompatible** change – one an older program would read but misunderstand. A file with a higher format version is refused with an explanatory message.
- New components and new optional fields do **not** change the format. That is why every component and field in the specification has a **Since** column – the first program version that supports it.
- **Which version opens my file:** the lowest program version that opens a file is the highest “Since” of all components the file uses. An older program **refuses to open** a file with an unknown component (read error). An unknown **field** is skipped – the file opens, but the field is dropped on saving (exception: unknown keys in the `info` part are kept since 0.4.0).

### Format history

| Program version | `.logicflow` | `.lflib` | Added |
| --- | --- | --- | --- |
| 0.0.1 | – | – | no file saving |
| 0.1.0 | 1 | – | project file: gates Buffer, NOT, AND, OR, NAND, NOR, XOR, XNOR (2–8 inputs), switch, push button, constant, LED; wires, rotation, labels |
| 0.2.0 | 1 | – | flip-flops RS, D latch, D, JK, T (`falling`, `initial`); clock (`period_ms`) |
| 0.3.0 | 1 | – | function blocks (`block`: MUX, decoder, encoder, adder, comparator, BCD/7SEG, counter, register, shift register), seven-segment and hex display (`pins`), subcircuits (`subcircuit`, the `subcircuits` part); released as 0.3.1 |
| 0.3.1 | 1 | – | no format change |
| 0.4.0 – unreleased | 1 | 1 | supply `vcc` and ground `gnd`; LED colour (`color`) and the two-pin LED `led_ak`; circuit information (the `info` part, unknown keys kept); element library `.lflib` |

Version **0.3.0** was not released on its own; its content shipped in 0.3.1, so “since 0.3.0” means “since 0.3.1” in practice.

### Maintenance (for developers)

The format documentation is part of every change that touches a format (see [CONTRIBUTING.md](../../CONTRIBUTING.md) → Formáty souborů / File formats):

1. **New component, block, gate kind or field:** a row in the specification **in Czech and English** with Since = the upcoming version, the JSON Schema and the history row above (in the version marked “unreleased”).
2. **Incompatible change** (renaming or removing a key or value, a new meaning of an existing value): increase `FORMAT` in the code, describe it in the history and in the specification, including how the program reads the older format. Within one format version keys and values are **never renamed or removed**, and new fields must have a default value.
3. **Release:** remove “unreleased” from the history row and update the **documentation status**; the Since columns stay (the version already fits).

Two CI checks enforce this: the test `crates/logicflow-core/tests/formaty.rs` (every component, block, gate, flip-flop, pin side and rotation kind, every field the program writes and parameter ranges must be described in both languages and in the schema with a Since version; the examples in the specification must load and simulate correctly; a new component kind without an entry in the test does not even compile) and the script `python3 scripts/formaty.py kontrola` (valid schemas; sample libraries, help circuits and specification examples match them; the history has a row for every version in the changelog, unreleased versions are marked, the documentation status matches `Cargo.toml` and every Since value is in the history; needs `pip install jsonschema`).
