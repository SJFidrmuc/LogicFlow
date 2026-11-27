//! Shoda dokumentace formátů souborů (`docs/formaty`) s kódem.
//!
//! Hlídá, aby specifikace projektu `.logicflow` a knihovny `.lflib` (česky
//! i anglicky) a jejich JSON Schema popisovaly každý druh součástky, bloku,
//! hradla, klopného obvodu, strany vývodů a otočení s verzí „Od verze“, každé
//! pole, které program zapisuje, a rozsahy parametrů – a aby se příklady ze
//! specifikace načetly a správně počítaly.
//!
//! **Nový druh součástky** (nebo bloku, hradla …): přidejte ho do příslušné
//! funkce `*_since` (úplný `match` – bez toho se test nepřeloží), u součástky
//! i do `all_kinds`, a doplňte řádek do `docs/formaty/logicflow.md`,
//! `logicflow.en.md`, schéma a historii v `docs/formaty/README.md`.
//! Historii proti přehledu změn a schémata proti vzorovým souborům kontroluje
//! `python3 scripts/formaty.py kontrola`.
//!
//! [en]
//!
//! Match between the file format documentation (`docs/formaty`) and the code.
//!
//! Checks that the specifications of the `.logicflow` project and the `.lflib`
//! library (Czech and English) and their JSON Schemas describe every kind of
//! component, block, gate, flip-flop, pin side and rotation with a “Since”
//! version, every field the program writes and the parameter ranges – and that
//! the examples from the specification load and compute correctly.
//!
//! **A new component kind** (or block, gate …): add it to the matching `*_since`
//! function (an exhaustive `match` – the test does not compile otherwise), for a
//! component also to `all_kinds`, and add a row to `docs/formaty/logicflow.md`,
//! `logicflow.en.md`, the schema and the history in `docs/formaty/README.md`.
//! `python3 scripts/formaty.py kontrola` checks the history against the changelog
//! and the schemas against the sample files.

use std::collections::BTreeSet;

use logicflow_core::circuit::MAX_SUBCIRCUIT_PINS;
use logicflow_core::library::CATEGORIES;
use logicflow_core::{
    BlockKind, Circuit, CircuitInfo, ComponentKind, FlipFlopKind, GateKind, Library, LogicValue,
    PinSide, Point, ProjectFile, Rotation, Simulator, SubcircuitDef,
};
use serde_json::Value;

/// Text souboru z `docs/formaty`. Soubory se vkládají při překladu (jádro ani jeho
/// testy nepracují se soubory – `clippy.toml`); změna dokumentace test přeloží znovu.
///
/// [en] Text of a file from `docs/formaty`. The files are embedded at compile time
/// (neither the core nor its tests touch files – `clippy.toml`); a documentation
/// change recompiles the test.
fn read(name: &str) -> String {
    match name {
        "logicflow.md" => include_str!("../../../docs/formaty/logicflow.md"),
        "logicflow.en.md" => include_str!("../../../docs/formaty/logicflow.en.md"),
        "lflib.md" => include_str!("../../../docs/formaty/lflib.md"),
        "lflib.en.md" => include_str!("../../../docs/formaty/lflib.en.md"),
        "logicflow.schema.json" => include_str!("../../../docs/formaty/logicflow.schema.json"),
        "lflib.schema.json" => include_str!("../../../docs/formaty/lflib.schema.json"),
        _ => panic!("neznámý soubor / [en] unknown file docs/formaty/{name}"),
    }
    .to_owned()
}

/// Vzorová knihovna z `docs/knihovny` (má vyplněná všechna pole).
///
/// [en] Sample library from `docs/knihovny` (it has all fields filled in).
const SAMPLE_LIBRARY: &str = include_str!("../../../docs/knihovny/vzor.lflib");

/// JSON Schema z `docs/formaty`.
///
/// [en] JSON Schema from `docs/formaty`.
fn load_schema(name: &str) -> Value {
    serde_json::from_str(&read(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// Specifikace projektu česky a anglicky: (název souboru, text).
///
/// [en] Project specification in Czech and English: (file name, text).
fn project_specs() -> Vec<(&'static str, String)> {
    ["logicflow.md", "logicflow.en.md"]
        .into_iter()
        .map(|n| (n, read(n)))
        .collect()
}

/// Specifikace knihovny česky a anglicky: (název souboru, text).
///
/// [en] Library specification in Czech and English: (file name, text).
fn library_specs() -> Vec<(&'static str, String)> {
    ["lflib.md", "lflib.en.md"]
        .into_iter()
        .map(|n| (n, read(n)))
        .collect()
}

/// Má specifikace řádek tabulky, který začíná buňkou `` `cell` `` a obsahuje
/// buňku s verzí `since`?
///
/// [en] Does the specification have a table row starting with the cell
/// `` `cell` `` and containing a cell with the version `since`?
fn has_row(doc: &str, cell: &str, since: &str) -> bool {
    let start = format!("| `{cell}` |");
    let version = format!("| {since} |");
    doc.lines()
        .any(|l| l.starts_with(&start) && l.contains(&version))
}

/// Zmiňuje specifikace klíč `` `key` `` v některé tabulce?
///
/// [en] Does the specification mention the key `` `key` `` in some table?
fn in_table(doc: &str, key: &str) -> bool {
    let key = format!("`{key}`");
    doc.lines().any(|l| l.starts_with('|') && l.contains(&key))
}

/// Ověří řádek ve všech jazycích specifikace.
///
/// [en] Checks the row in all languages of the specification.
fn assert_rows(specs: &[(&str, String)], cell: &str, since: &str) {
    for (name, doc) in specs {
        assert!(
            has_row(doc, cell, since),
            "{name}: chybí řádek `{cell}` s verzí {since} / [en] row missing"
        );
    }
}

/// První verze programu, která druh součástky zná. Při přidání druhu doplňte
/// i [`all_kinds`].
///
/// [en] First program version that knows the component kind. When adding a kind,
/// also extend [`all_kinds`].
fn kind_since(kind: &ComponentKind) -> &'static str {
    match kind {
        ComponentKind::Gate { .. }
        | ComponentKind::Switch { .. }
        | ComponentKind::Button
        | ComponentKind::Constant { .. }
        | ComponentKind::Led => "0.1.0",
        ComponentKind::FlipFlop { .. } | ComponentKind::Clock { .. } => "0.2.0",
        ComponentKind::Block { .. }
        | ComponentKind::SevenSegment { .. }
        | ComponentKind::HexDisplay { .. }
        | ComponentKind::Subcircuit { .. } => "0.3.0",
        ComponentKind::Vcc | ComponentKind::Gnd => "0.4.0",
    }
}

/// Po jednom zástupci každého druhu součástky.
///
/// [en] One representative of every component kind.
fn all_kinds() -> Vec<ComponentKind> {
    vec![
        ComponentKind::gate(GateKind::And),
        ComponentKind::Switch { on: true },
        ComponentKind::Button,
        ComponentKind::Constant { value: true },
        ComponentKind::Vcc,
        ComponentKind::Gnd,
        ComponentKind::Led,
        ComponentKind::flip_flop(FlipFlopKind::Jk),
        ComponentKind::DEFAULT_CLOCK,
        ComponentKind::Block {
            block: BlockKind::ALL[0],
        },
        ComponentKind::SevenSegment { pins: PinSide::Top },
        ComponentKind::HexDisplay {
            pins: PinSide::Left,
        },
        ComponentKind::Subcircuit {
            id: 1,
            inputs: 1,
            outputs: 1,
        },
    ]
}

/// První verze, která zná druh hradla.
///
/// [en] First version that knows the gate kind.
fn gate_since(gate: GateKind) -> &'static str {
    match gate {
        GateKind::Buffer
        | GateKind::Not
        | GateKind::And
        | GateKind::Or
        | GateKind::Nand
        | GateKind::Nor
        | GateKind::Xor
        | GateKind::Xnor => "0.1.0",
    }
}

/// První verze, která zná druh klopného obvodu.
///
/// [en] First version that knows the flip-flop kind.
fn flip_flop_since(ff: FlipFlopKind) -> &'static str {
    match ff {
        FlipFlopKind::Rs
        | FlipFlopKind::DLatch
        | FlipFlopKind::D
        | FlipFlopKind::Jk
        | FlipFlopKind::T => "0.2.0",
    }
}

/// První verze, která zná stranu vývodů zobrazovače.
///
/// [en] First version that knows the display pin side.
fn pins_since(side: PinSide) -> &'static str {
    match side {
        PinSide::Left | PinSide::Right | PinSide::Top | PinSide::Bottom => "0.3.0",
    }
}

/// První verze, která zná otočení.
///
/// [en] First version that knows the rotation.
fn rotation_since(rotation: Rotation) -> &'static str {
    match rotation {
        Rotation::R0 | Rotation::R90 | Rotation::R180 | Rotation::R270 => "0.1.0",
    }
}

/// První verze, která zná funkční blok.
///
/// [en] First version that knows the function block.
fn block_since(block: BlockKind) -> &'static str {
    match block {
        BlockKind::Mux { .. }
        | BlockKind::Decoder { .. }
        | BlockKind::Encoder { .. }
        | BlockKind::Adder { .. }
        | BlockKind::Comparator { .. }
        | BlockKind::BcdTo7Seg
        | BlockKind::Counter { .. }
        | BlockKind::Register { .. }
        | BlockKind::ShiftRegister { .. } => "0.3.0",
    }
}

/// Hodnota jako text JSON bez uvozovek (`"Nand"` → `Nand`).
///
/// [en] Value as JSON text without quotes (`"Nand"` → `Nand`).
fn name<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .unwrap()
        .as_str()
        .expect("hodnota je text / value is a string")
        .to_owned()
}

/// Položky `oneOf` v `$defs.<def>` schématu projektu podle hodnoty `const`
/// klíče `key`: (hodnota, položka).
///
/// [en] `oneOf` entries in `$defs.<def>` of the project schema by the `const`
/// value of the key `key`: (value, entry).
fn one_of(schema: &Value, def: &str, key: &str) -> Vec<(String, Value)> {
    schema["$defs"][def]["oneOf"]
        .as_array()
        .unwrap_or_else(|| panic!("schéma: $defs.{def}.oneOf"))
        .iter()
        .map(|e| {
            let v = e["properties"][key]["const"].as_str().expect("const");
            (v.to_owned(), e.clone())
        })
        .collect()
}

#[test]
fn component_kinds_documented() {
    let specs = project_specs();
    let schema = load_schema("logicflow.schema.json");
    let in_schema = one_of(&schema, "componentKind", "type");
    let mut types = BTreeSet::new();
    for kind in all_kinds() {
        let json = serde_json::to_value(kind).unwrap();
        let ty = json["type"].as_str().unwrap().to_owned();
        let since = kind_since(&kind);
        assert_rows(&specs, &ty, since);
        let entry = in_schema
            .iter()
            .find(|(t, _)| *t == ty)
            .unwrap_or_else(|| panic!("schéma: chybí druh {ty} / [en] kind missing"));
        assert_eq!(entry.1["x-since"], since, "schéma: x-since u {ty}");
        // Ukázková hodnota musí projít i zpět (název ve schématu = název v kódu).
        // [en] The sample value must also round-trip (name in schema = name in code).
        assert_eq!(serde_json::from_value::<ComponentKind>(json).unwrap(), kind);
        types.insert(ty);
    }
    let schema_types: BTreeSet<String> = in_schema.into_iter().map(|(t, _)| t).collect();
    assert_eq!(
        schema_types, types,
        "druhy ve schématu / [en] kinds in the schema"
    );
}

#[test]
fn values_documented() {
    let specs = project_specs();
    let schema = load_schema("logicflow.schema.json");
    // (název výčtu ve schématu, hodnoty z kódu s verzí)
    // [en] (enum name in the schema, values from the code with the version)
    let enums: Vec<(&str, Vec<(String, &str)>)> = vec![
        (
            "gateKind",
            GateKind::ALL
                .iter()
                .map(|&g| (name(&g), gate_since(g)))
                .collect(),
        ),
        (
            "flipFlopKind",
            FlipFlopKind::ALL
                .iter()
                .map(|&f| (name(&f), flip_flop_since(f)))
                .collect(),
        ),
        (
            "pinSide",
            PinSide::ALL
                .iter()
                .map(|&p| (name(&p), pins_since(p)))
                .collect(),
        ),
        (
            "rotation",
            [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270]
                .iter()
                .map(|&r| (name(&r), rotation_since(r)))
                .collect(),
        ),
    ];
    for (def, values) in enums {
        let listed: Vec<String> = schema["$defs"][def]["enum"]
            .as_array()
            .unwrap_or_else(|| panic!("schéma: $defs.{def}.enum"))
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect();
        let code: Vec<String> = values.iter().map(|(v, _)| v.clone()).collect();
        assert_eq!(listed, code, "schéma: $defs.{def}");
        for (value, since) in &values {
            assert_rows(&specs, value, since);
            assert_eq!(
                schema["$defs"][def]["x-since"][value], *since,
                "{def}.{value}"
            );
        }
    }
}

#[test]
fn blocks_documented() {
    let specs = project_specs();
    let schema = load_schema("logicflow.schema.json");
    let in_schema = one_of(&schema, "blockKind", "kind");
    assert_eq!(in_schema.len(), BlockKind::ALL.len(), "bloky ve schématu");
    for block in BlockKind::ALL {
        let json = serde_json::to_value(block).unwrap();
        let kind = json["kind"].as_str().unwrap();
        let since = block_since(block);
        assert_rows(&specs, kind, since);
        let (_, entry) = in_schema
            .iter()
            .find(|(k, _)| k == kind)
            .unwrap_or_else(|| panic!("schéma: chybí blok {kind}"));
        assert_eq!(entry["x-since"], since, "schéma: x-since u bloku {kind}");
        // Hlavní parametr (select / bits) a jeho rozsah v obou jazycích i ve schématu.
        // [en] Main parameter (select / bits) and its range in both languages and the schema.
        let param = json
            .as_object()
            .unwrap()
            .iter()
            .find(|(_, v)| v.is_u64())
            .map(|(k, _)| k.clone());
        if let Some(param) = param {
            let (lo, hi) = block.param_range();
            let text = format!("`{param}` {lo}–{hi}");
            for (file, doc) in &specs {
                assert!(
                    doc.lines()
                        .any(|l| l.starts_with(&format!("| `{kind}` |")) && l.contains(&text)),
                    "{file}: blok {kind} – rozsah {text} / [en] range"
                );
            }
            let p = &entry["properties"][&param];
            assert_eq!(
                (p["minimum"].as_u64(), p["maximum"].as_u64()),
                (Some(lo.into()), Some(hi.into()))
            );
        }
    }
}

#[test]
fn limits_documented() {
    let specs = project_specs();
    let schema = load_schema("logicflow.schema.json");
    let (lo, hi) = (ComponentKind::MIN_INPUTS, ComponentKind::MAX_INPUTS);
    for (file, doc) in &specs {
        assert!(
            doc.contains(&format!("{lo}–{hi}")),
            "{file}: počet vstupů hradla {lo}–{hi}"
        );
        assert!(
            doc.contains(&format!("0–{MAX_SUBCIRCUIT_PINS}")),
            "{file}: nejvýš {MAX_SUBCIRCUIT_PINS} vývodů podobvodu"
        );
        assert!(
            doc.lines().any(|l| l.starts_with("| `format` |")
                && l.contains(&format!("**{}**", ProjectFile::FORMAT))),
            "{file}: verze formátu {}",
            ProjectFile::FORMAT
        );
    }
    assert_eq!(schema["properties"]["format"]["const"], ProjectFile::FORMAT);
    let gate = &one_of(&schema, "componentKind", "type")[0].1;
    assert_eq!(gate["properties"]["inputs"]["maximum"], u64::from(hi));
    assert_eq!(
        gate["else"]["properties"]["inputs"]["minimum"],
        u64::from(lo)
    );
    let sub = one_of(&schema, "componentKind", "type")
        .into_iter()
        .find(|(t, _)| t == "subcircuit")
        .unwrap()
        .1;
    assert_eq!(
        sub["properties"]["inputs"]["maximum"],
        MAX_SUBCIRCUIT_PINS as u64
    );

    let lib_schema = load_schema("lflib.schema.json");
    assert_eq!(lib_schema["properties"]["format"]["const"], Library::FORMAT);
    for (file, doc) in &library_specs() {
        assert!(
            doc.lines().any(|l| l.starts_with("| `format` |")
                && l.contains(&format!("**{}**", Library::FORMAT))),
            "{file}: verze formátu knihovny"
        );
    }
}

#[test]
fn categories_documented() {
    let schema = load_schema("lflib.schema.json");
    let builtin = schema["$defs"]["category"]["x-builtin"]
        .as_object()
        .unwrap();
    assert_eq!(builtin.len(), CATEGORIES.len(), "kategorie ve schématu");
    for c in CATEGORIES {
        let row = format!(
            "| `{}` | {} | {} | {} |",
            c.key,
            c.cs,
            c.en,
            builtin[c.key].as_str().unwrap()
        );
        for (file, doc) in &library_specs() {
            assert!(doc.contains(&row), "{file}: chybí řádek {row}");
        }
    }
}

/// Všechny klíče objektů v JSON (bez kódů jazyků v `translations` a bez
/// obsahu `circuit`, pokud `skip_circuit`).
///
/// [en] All object keys in the JSON (without language codes in `translations`
/// and without the content of `circuit` if `skip_circuit`).
fn keys(value: &Value, skip_circuit: bool, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                out.insert(k.clone());
                if k == "translations" {
                    for t in v.as_object().into_iter().flat_map(|m| m.values()) {
                        keys(t, skip_circuit, out);
                    }
                } else if !(skip_circuit && k == "circuit") {
                    keys(v, skip_circuit, out);
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|v| keys(v, skip_circuit, out)),
        _ => {}
    }
}

/// Názvy všech vlastností ve schématu (klíče objektů `properties`).
///
/// [en] Names of all properties in the schema (keys of the `properties` objects).
fn schema_properties(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                if k == "properties" {
                    out.extend(v.as_object().unwrap().keys().cloned());
                }
                schema_properties(v, out);
            }
        }
        Value::Array(items) => items.iter().for_each(|v| schema_properties(v, out)),
        _ => {}
    }
}

#[test]
fn written_fields_documented() {
    // Projekt se vším, co program umí zapsat.
    // [en] A project with everything the program can write.
    let mut circuit = Circuit::default();
    for (i, kind) in all_kinds().into_iter().enumerate() {
        circuit.add_component(kind, Point::new(10 * i as i32, 0));
    }
    for block in BlockKind::ALL {
        circuit.add_component(ComponentKind::Block { block }, Point::new(0, 20));
    }
    circuit.add_wire(Point::new(0, 0), Point::new(4, 0));
    let info = CircuitInfo {
        title: "t".into(),
        authors: vec!["a".into()],
        description: "d".into(),
        license: "l".into(),
        school: "s".into(),
        created: "2026-01-01".into(),
        modified: "2026-01-01".into(),
        app_version: "0.4.0".into(),
        ..CircuitInfo::default()
    };
    let defs = vec![SubcircuitDef {
        id: 1,
        name: "P".into(),
        circuit: circuit.clone(),
    }];
    let project: Value =
        serde_json::from_str(&ProjectFile::to_json_with_info(&info, &circuit, &defs)).unwrap();
    let mut written = BTreeSet::new();
    keys(&project, false, &mut written);
    let mut in_schema = BTreeSet::new();
    schema_properties(&load_schema("logicflow.schema.json"), &mut in_schema);
    for key in &written {
        for (file, doc) in &project_specs() {
            assert!(
                in_table(doc, key),
                "{file}: pole `{key}` není popsané / [en] field not described"
            );
        }
        assert!(
            in_schema.contains(key),
            "logicflow.schema.json: chybí pole {key}"
        );
    }

    // Knihovna – vzor má vyplněná všechna pole včetně překladů.
    // [en] Library – the sample has all fields filled in, including translations.
    let library: Value =
        serde_json::from_str(&Library::from_json(SAMPLE_LIBRARY).unwrap().to_json()).unwrap();
    let mut written = BTreeSet::new();
    keys(&library, true, &mut written);
    let mut in_schema = BTreeSet::new();
    schema_properties(&load_schema("lflib.schema.json"), &mut in_schema);
    for key in &written {
        for (file, doc) in &library_specs() {
            assert!(
                in_table(doc, key),
                "{file}: pole `{key}` není popsané / [en] field not described"
            );
        }
        assert!(
            in_schema.contains(key),
            "lflib.schema.json: chybí pole {key}"
        );
    }
}

/// JSON příklad ze specifikace podle značky `<!-- příklad: X -->` /
/// `<!-- example: X -->`.
///
/// [en] JSON example from the specification by the marker.
fn example(doc: &str, format: &str) -> String {
    let marker = doc
        .find(&format!("<!-- příklad: {format} -->"))
        .or_else(|| doc.find(&format!("<!-- example: {format} -->")))
        .expect("značka příkladu / example marker");
    let rest = &doc[marker..];
    let start = rest.find("```json\n").expect("```json") + "```json\n".len();
    let end = rest[start..]
        .find("\n```")
        .expect("konec bloku / end of block");
    rest[start..start + end].to_owned()
}

/// Hodnota LED `out` pro hodnoty vstupu `input` 0 a 1 v obvodu s podobvody `defs`.
///
/// [en] Value of the LED `out` for the input `input` values 0 and 1 in a circuit
/// with subcircuits `defs`.
fn response(circuit: &Circuit, defs: &[SubcircuitDef], input: &str, out: &str) -> [LogicValue; 2] {
    let id = |label: &str| {
        circuit
            .components
            .iter()
            .find(|c| c.label == label)
            .unwrap_or_else(|| panic!("součástka {label}"))
            .id
    };
    let mut sim = Simulator::with_subcircuits(circuit, defs, None);
    [false, true].map(|v| {
        sim.set_source(id(input), LogicValue::from(v));
        assert!(sim.run(10_000), "obvod se ustálí / settles");
        sim.inputs(id(out))[0]
    })
}

#[test]
fn examples_work() {
    use LogicValue::{One, Zero};
    let specs = project_specs();
    let text = example(&specs[0].1, "logicflow");
    assert_eq!(
        text,
        example(&specs[1].1, "logicflow"),
        "příklad je v obou jazycích stejný"
    );
    let (circuit, defs) = ProjectFile::from_json(&text).unwrap();
    assert_eq!(response(&circuit, &defs, "A", "Y"), [One, Zero], "Y = Ā");

    let specs = library_specs();
    let text = example(&specs[0].1, "lflib");
    assert_eq!(
        text,
        example(&specs[1].1, "lflib"),
        "příklad je v obou jazycích stejný"
    );
    let lib = Library::from_json(&text).unwrap();
    let defs = lib.defs();
    let expected = [[One, Zero], [Zero, One]];
    for (item, want) in lib.items.iter().zip(expected) {
        assert_eq!(
            response(&item.circuit, &defs, "A", "Y"),
            want,
            "{}",
            item.name
        );
        let (ins, outs) = item.def().pin_names();
        assert_eq!((ins, outs), (item.inputs.clone(), item.outputs.clone()));
    }
}
