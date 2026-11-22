//! Výuková knihovna „Číslicová technika“ (`docs/knihovny/cislicova-technika.lflib`).
//!
//! Prvky jsou hradlová zapojení běžných úloh číslicové techniky na střední škole
//! (podle osnov a sbírky úloh rc2000 – µLAB, kap. 7): hradla z NAND a NOR,
//! kombinační obvody, sčítačky a odčítačky, klopné obvody, čítače, registry
//! a jednoduché aplikace. Knihovnu generuje tento test pomocí sestavovače schémat;
//! každý prvek se ověří simulací (pravdivostní tabulka, průběh čítání).
//!
//! Po změně prvků se soubor knihovny přegeneruje:
//! `LOGICFLOW_ZAPSAT_KNIHOVNU=1 cargo test -p logicflow-core --test vyukova_knihovna`
//!
//! [en]
//!
//! Teaching library “Digital electronics” (`docs/knihovny/cislicova-technika.lflib`).
//!
//! The items are gate-level circuits of common secondary-school digital electronics
//! tasks (following the curriculum and the rc2000 – µLAB task collection, chapter 7):
//! gates from NAND and NOR, combinational circuits, adders and subtractors,
//! flip-flops, counters, registers and simple applications. This test generates
//! the library with the schematic builder; every item is verified by simulation
//! (truth table, counting sequence).
//!
//! After changing the items, regenerate the library file:
//! `LOGICFLOW_ZAPSAT_KNIHOVNU=1 cargo test -p logicflow-core --test vyukova_knihovna`

mod sestavovac;

use logicflow_core::library::{DEFAULT_LICENSE, Library, LibraryInfo, LibraryItem, Translation};
use logicflow_core::{CompId, FlipFlopKind, GateKind, LogicValue, Simulator, SubcircuitDef};
use sestavovac::{Sestavovac, Sig};

use FlipFlopKind::{D as Dff, Jk, T as Tff};
use GateKind::{And, Nand, Nor, Not, Or, Xnor, Xor};

/// Cesta ke generovanému souboru (vzhledem ke kořeni repozitáře).
///
/// [en] Path to the generated file (relative to the repository root).
const SOUBOR: &str = "docs/knihovny/cislicova-technika.lflib";

/// Uložený soubor knihovny.
///
/// [en] The stored library file.
const ULOZENA: &str = include_str!("../../../docs/knihovny/cislicova-technika.lflib");

/// Datum vytvoření prvků.
///
/// [en] Creation date of the items.
const DATUM: &str = "2026-11-22";

/// Autor knihovny a prvků.
///
/// [en] Author of the library and the items.
const AUTOR: &str = "Mgr. Stanislav Jan Fidrmuc";

/// Popis prvku: identita, názvy, kategorie a popisy česky a anglicky.
///
/// [en] Item description: identity, names, category and descriptions in Czech and English.
struct Popis {
    /// Stabilní identita (UUID).
    ///
    /// [en] Stable identity (UUID).
    uid: &'static str,
    /// Český název.
    ///
    /// [en] Czech name.
    cs: &'static str,
    /// Anglický název.
    ///
    /// [en] English name.
    en: &'static str,
    /// Klíč kategorie.
    ///
    /// [en] Category key.
    kat: &'static str,
    /// Český popis.
    ///
    /// [en] Czech description.
    popis_cs: &'static str,
    /// Anglický popis.
    ///
    /// [en] English description.
    popis_en: &'static str,
}

/// Rozpracovaná knihovna.
///
/// [en] Library being built.
struct Kn {
    /// Knihovna.
    ///
    /// [en] The library.
    lib: Library,
}

impl Kn {
    /// Přidá prvek se zapojením ze sestavovače a vrátí jeho definici (pro vnoření).
    ///
    /// [en] Adds an item with the circuit from the builder and returns its definition (for nesting).
    fn pridej(&mut self, p: Popis, b: Sestavovac) -> SubcircuitDef {
        let mut item = LibraryItem {
            uid: p.uid.to_owned(),
            id: self.lib.next_item_id(),
            name: p.cs.to_owned(),
            category: p.kat.to_owned(),
            description: p.popis_cs.to_owned(),
            translations: [(
                "en".to_owned(),
                Translation {
                    name: p.en.to_owned(),
                    description: p.popis_en.to_owned(),
                },
            )]
            .into(),
            author: AUTOR.to_owned(),
            license: DEFAULT_LICENSE.to_owned(),
            version: 1,
            created: DATUM.to_owned(),
            modified: DATUM.to_owned(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            circuit: b.sestav(),
        };
        let def = item.def();
        (item.inputs, item.outputs) = def.pin_names();
        self.lib.items.push(item);
        def
    }
}

/// Vstupy se zadanými popisky.
///
/// [en] Inputs with the given labels.
fn vstupy(b: &mut Sestavovac, labels: &[&str]) -> Vec<Sig> {
    labels.iter().map(|l| b.vstup(l)).collect()
}

/// Sestaví celou výukovou knihovnu.
///
/// [en] Builds the whole teaching library.
#[allow(clippy::too_many_lines)]
fn knihovna() -> Library {
    let mut info = LibraryInfo::new(
        "22e3b7de-7bdd-429b-8c26-275bf7a1298e".to_owned(),
        "Číslicová technika – výukové prvky",
        AUTOR,
        DATUM,
    );
    info.description = "Hradlová zapojení běžných úloh číslicové techniky: hradla z NAND a NOR, \
        kombinační obvody, sčítačky a odčítačky, klopné obvody, čítače, registry a aplikace. \
        Každý prvek je ověřený simulací."
        .to_owned();
    info.translations.insert(
        "en".to_owned(),
        Translation {
            name: "Digital electronics – teaching elements".to_owned(),
            description: "Gate-level circuits of common digital electronics tasks: gates from \
                NAND and NOR, combinational circuits, adders and subtractors, flip-flops, \
                counters, registers and applications. Every item is verified by simulation."
                .to_owned(),
        },
    );
    let mut k = Kn {
        lib: Library::new(info),
    };

    // --- Hradla z NAND a NOR -------------------------------------------------
    // [en] --- Gates from NAND and NOR ------------------------------------------

    let mut b = Sestavovac::new();
    let a = b.vstup("A");
    let y = b.hradlo(Nand, &[a, a]);
    b.vystup("Y", y);
    k.pridej(
        Popis {
            uid: "54faa239-6297-4d53-a08d-f4304e74bd16",
            cs: "NOT z NAND",
            en: "NOT from NAND",
            kat: "combinational",
            popis_cs: "Invertor z jednoho hradla NAND se spojenými vstupy: Y = ¬(A·A) = ¬A.",
            popis_en: "Inverter from one NAND gate with joined inputs: Y = ¬(A·A) = ¬A.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B"]);
    let n = b.hradlo(Nand, &[v[0], v[1]]);
    let y = b.hradlo(Nand, &[n, n]);
    b.vystup("Y", y);
    k.pridej(
        Popis {
            uid: "1fa1d145-dd94-4edf-8414-e5d2104403ff",
            cs: "AND z NAND",
            en: "AND from NAND",
            kat: "combinational",
            popis_cs: "Součin ze dvou hradel NAND: NAND a za ním NAND zapojený jako invertor. \
                Y = ¬¬(A·B) = A·B.",
            popis_en: "AND from two NAND gates: a NAND followed by a NAND used as an inverter. \
                Y = ¬¬(A·B) = A·B.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B"]);
    let na = b.hradlo(Nand, &[v[0], v[0]]);
    let nb = b.hradlo(Nand, &[v[1], v[1]]);
    let y = b.hradlo(Nand, &[na, nb]);
    b.vystup("Y", y);
    k.pridej(
        Popis {
            uid: "368aab1e-bcc6-4e3f-a363-96d26003b3a1",
            cs: "OR z NAND",
            en: "OR from NAND",
            kat: "combinational",
            popis_cs: "Součet ze tří hradel NAND podle De Morganova zákona: Y = ¬(¬A·¬B) = A + B.",
            popis_en: "OR from three NAND gates by De Morgan's law: Y = ¬(¬A·¬B) = A + B.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B"]);
    let n1 = b.hradlo(Nand, &[v[0], v[1]]);
    let n2 = b.hradlo(Nand, &[v[0], n1]);
    let n3 = b.hradlo(Nand, &[n1, v[1]]);
    let y = b.hradlo(Nand, &[n2, n3]);
    b.vystup("Y", y);
    k.pridej(
        Popis {
            uid: "cd439513-b7dc-442a-8970-0775fa4dc44f",
            cs: "XOR ze čtyř NAND",
            en: "XOR from four NAND",
            kat: "combinational",
            popis_cs: "Nonekvivalence (výlučný součet) ze čtyř hradel NAND – klasické zapojení \
                z obvodu 7400. Y = A ⊕ B.",
            popis_en: "Exclusive OR from four NAND gates – the classic circuit with a 7400. \
                Y = A ⊕ B.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let a = b.vstup("A");
    let y = b.hradlo(Nor, &[a, a]);
    b.vystup("Y", y);
    k.pridej(
        Popis {
            uid: "99d4da6d-d6b0-45bc-b9e0-72e0749fd75b",
            cs: "NOT z NOR",
            en: "NOT from NOR",
            kat: "combinational",
            popis_cs: "Invertor z jednoho hradla NOR se spojenými vstupy: Y = ¬(A + A) = ¬A.",
            popis_en: "Inverter from one NOR gate with joined inputs: Y = ¬(A + A) = ¬A.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B"]);
    let n = b.hradlo(Nor, &[v[0], v[1]]);
    let y = b.hradlo(Nor, &[n, n]);
    b.vystup("Y", y);
    k.pridej(
        Popis {
            uid: "e96fe3fd-2a2c-46a8-813e-2a00675ef62a",
            cs: "OR z NOR",
            en: "OR from NOR",
            kat: "combinational",
            popis_cs: "Součet ze dvou hradel NOR: NOR a za ním NOR zapojený jako invertor. \
                Y = ¬¬(A + B) = A + B.",
            popis_en: "OR from two NOR gates: a NOR followed by a NOR used as an inverter. \
                Y = ¬¬(A + B) = A + B.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B"]);
    let na = b.hradlo(Nor, &[v[0], v[0]]);
    let nb = b.hradlo(Nor, &[v[1], v[1]]);
    let y = b.hradlo(Nor, &[na, nb]);
    b.vystup("Y", y);
    k.pridej(
        Popis {
            uid: "38d166ab-b260-48a5-b108-8856e34d6e88",
            cs: "AND z NOR",
            en: "AND from NOR",
            kat: "combinational",
            popis_cs: "Součin ze tří hradel NOR podle De Morganova zákona: Y = ¬(¬A + ¬B) = A·B.",
            popis_en: "AND from three NOR gates by De Morgan's law: Y = ¬(¬A + ¬B) = A·B.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B"]);
    let n1 = b.hradlo(Nor, &[v[0], v[1]]);
    let n2 = b.hradlo(Nor, &[v[0], n1]);
    let n3 = b.hradlo(Nor, &[n1, v[1]]);
    let y = b.hradlo(Nor, &[n2, n3]);
    b.vystup("Y", y);
    k.pridej(
        Popis {
            uid: "c32bd571-71c9-4411-94e7-746106d815ce",
            cs: "XNOR ze čtyř NOR",
            en: "XNOR from four NOR",
            kat: "combinational",
            popis_cs: "Ekvivalence ze čtyř hradel NOR (stejné zapojení jako XOR ze čtyř NAND, \
                jen s NOR dává negovaný výsledek). Y = ¬(A ⊕ B).",
            popis_en: "Equivalence from four NOR gates (the same circuit as XOR from four NAND; \
                with NOR it gives the negated result). Y = ¬(A ⊕ B).",
        },
        b,
    );

    // --- Kombinační obvody ---------------------------------------------------
    // [en] --- Combinational circuits -------------------------------------------

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B", "C"]);
    let ab = b.hradlo(And, &[v[0], v[1]]);
    let ac = b.hradlo(And, &[v[0], v[2]]);
    let bc = b.hradlo(And, &[v[1], v[2]]);
    let y = b.hradlo(Or, &[ab, ac, bc]);
    b.vystup("Y", y);
    k.pridej(
        Popis {
            uid: "5c1e521f-d9a9-4ca2-a8e4-4e4395e92056",
            cs: "Majoritní obvod 2 ze 3",
            en: "2-out-of-3 majority",
            kat: "combinational",
            popis_cs: "Hlasování: výstup je 1, když jsou v 1 aspoň dva ze tří vstupů. \
                Y = A·B + A·C + B·C. Typická úloha na minimalizaci (Karnaughova mapa).",
            popis_en: "Voting: the output is 1 when at least two of the three inputs are 1. \
                Y = A·B + A·C + B·C. A typical minimisation task (Karnaugh map).",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["D0", "D1", "D2", "D3"]);
    let x1 = b.hradlo(Xor, &[v[0], v[1]]);
    let x2 = b.hradlo(Xor, &[v[2], v[3]]);
    let p = b.hradlo(Xor, &[x1, x2]);
    b.vystup("P", p);
    k.pridej(
        Popis {
            uid: "d512488f-d0bf-4586-b250-87e38d2bc980",
            cs: "Generátor sudé parity 4 bity",
            en: "4-bit even parity generator",
            kat: "combinational",
            popis_cs: "Paritní bit P doplní čtyři datové bity na sudý počet jedniček: \
                P = D0 ⊕ D1 ⊕ D2 ⊕ D3. Stejný obvod s pátým vstupem slouží jako kontrola parity.",
            popis_en: "The parity bit P completes four data bits to an even number of ones: \
                P = D0 ⊕ D1 ⊕ D2 ⊕ D3. The same circuit with a fifth input serves as a parity check.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["B0", "B1", "B2", "B3"]);
    let g0 = b.hradlo(Xor, &[v[0], v[1]]);
    let g1 = b.hradlo(Xor, &[v[1], v[2]]);
    let g2 = b.hradlo(Xor, &[v[2], v[3]]);
    b.vystup("G0", g0);
    b.vystup("G1", g1);
    b.vystup("G2", g2);
    b.vystup("G3", v[3]);
    k.pridej(
        Popis {
            uid: "725e47f5-1e0f-4fda-a3fd-098a58424c85",
            cs: "Převodník binární → Grayův kód 4 bity",
            en: "4-bit binary to Gray code converter",
            kat: "code-converters",
            popis_cs: "Grayův kód: sousední čísla se liší v jediném bitu. G3 = B3, \
                Gi = Bi+1 ⊕ Bi. Úloha 7.12 (kodér Grayova kódu).",
            popis_en: "Gray code: neighbouring numbers differ in a single bit. G3 = B3, \
                Gi = Bi+1 ⊕ Bi. Task 7.12 (Gray code encoder).",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["G0", "G1", "G2", "G3"]);
    let b2 = b.hradlo(Xor, &[v[3], v[2]]);
    let b1 = b.hradlo(Xor, &[b2, v[1]]);
    let b0 = b.hradlo(Xor, &[b1, v[0]]);
    b.vystup("B0", b0);
    b.vystup("B1", b1);
    b.vystup("B2", b2);
    b.vystup("B3", v[3]);
    k.pridej(
        Popis {
            uid: "78a8dee8-3996-44c4-a241-72ac7cf6ccc8",
            cs: "Převodník Grayův kód → binární 4 bity",
            en: "4-bit Gray code to binary converter",
            kat: "code-converters",
            popis_cs: "Zpětný převod: B3 = G3, Bi = Bi+1 ⊕ Gi – každý bit závisí na vyšším, \
                proto řetěz hradel XOR. Úloha 7.12 (dekodér Grayova kódu).",
            popis_en: "The reverse conversion: B3 = G3, Bi = Bi+1 ⊕ Gi – each bit depends on the \
                higher one, hence a chain of XOR gates. Task 7.12 (Gray code decoder).",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["D0", "D1", "S"]);
    let ns = b.hradlo(Not, &[v[2]]);
    let t0 = b.hradlo(And, &[v[0], ns]);
    let t1 = b.hradlo_ve(2, And, &[v[1], v[2]]);
    let y = b.hradlo(Or, &[t0, t1]);
    b.vystup("Y", y);
    k.pridej(
        Popis {
            uid: "5d670fec-36b0-47d4-922f-6dca0398ef2f",
            cs: "Multiplexor 2:1 z hradel",
            en: "2:1 multiplexer from gates",
            kat: "combinational",
            popis_cs: "Výběr jednoho ze dvou datových vstupů adresou S: Y = ¬S·D0 + S·D1.",
            popis_en: "Selects one of two data inputs by the address S: Y = ¬S·D0 + S·D1.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["D0", "D1", "D2", "D3", "S0", "S1"]);
    let n0 = b.hradlo(Not, &[v[4]]);
    let n1 = b.hradlo(Not, &[v[5]]);
    let t0 = b.hradlo(And, &[v[0], n1, n0]);
    let t1 = b.hradlo(And, &[v[1], n1, v[4]]);
    let t2 = b.hradlo(And, &[v[2], v[5], n0]);
    let t3 = b.hradlo_ve(2, And, &[v[3], v[5], v[4]]);
    let y = b.hradlo(Or, &[t0, t1, t2, t3]);
    b.vystup("Y", y);
    k.pridej(
        Popis {
            uid: "1da2ff22-7f24-476d-9550-6c29fee1c622",
            cs: "Multiplexor 4:1 z hradel",
            en: "4:1 multiplexer from gates",
            kat: "combinational",
            popis_cs: "Výběr jednoho ze čtyř datových vstupů adresou S1 S0; každý součin odpovídá \
                jedné adrese (mintermu). Stejnou funkci má obvod 74153.",
            popis_en: "Selects one of four data inputs by the address S1 S0; each product term \
                matches one address (minterm). The 74153 has the same function.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A0", "A1", "EN"]);
    let n0 = b.hradlo(Not, &[v[0]]);
    let n1 = b.hradlo(Not, &[v[1]]);
    let y0 = b.hradlo(And, &[v[2], n1, n0]);
    let y1 = b.hradlo(And, &[v[2], n1, v[0]]);
    let y2 = b.hradlo(And, &[v[2], v[1], n0]);
    let y3 = b.hradlo_ve(2, And, &[v[2], v[1], v[0]]);
    for (i, s) in [y0, y1, y2, y3].into_iter().enumerate() {
        b.vystup(&format!("Y{i}"), s);
    }
    k.pridej(
        Popis {
            uid: "c2e7be2b-bbe6-4df2-b752-9f441db9acf5",
            cs: "Dekodér 2 → 4 z hradel",
            en: "2-to-4 decoder from gates",
            kat: "combinational",
            popis_cs: "Při EN = 1 je v 1 právě ten výstup Yi, jehož číslo je na adrese A1 A0; \
                při EN = 0 jsou všechny výstupy 0. Zapojený jako demultiplexor (EN = data).",
            popis_en: "With EN = 1 exactly the output Yi whose number is on the address A1 A0 is 1; \
                with EN = 0 all outputs are 0. Used as a demultiplexer (EN = data).",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B"]);
    let na = b.hradlo(Not, &[v[0]]);
    let nb = b.hradlo(Not, &[v[1]]);
    let lt = b.hradlo(And, &[na, v[1]]);
    let eq = b.hradlo_ve(2, Xnor, &[v[0], v[1]]);
    let gt = b.hradlo(And, &[v[0], nb]);
    b.vystup("A<B", lt);
    b.vystup("A=B", eq);
    b.vystup("A>B", gt);
    let komp1 = k.pridej(
        Popis {
            uid: "04e6b19c-b70d-4e56-b1fa-3be69bf1aaee",
            cs: "Komparátor 1 bit",
            en: "1-bit comparator",
            kat: "combinational",
            popis_cs: "Porovná dva bity: A<B = ¬A·B, A=B = ¬(A ⊕ B), A>B = A·¬B. Úloha 7.10.",
            popis_en: "Compares two bits: A<B = ¬A·B, A=B = ¬(A ⊕ B), A>B = A·¬B. Task 7.10.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A0", "A1", "B0", "B1"]);
    let c1 = b.podobvod(&komp1, &[v[1], v[3]]);
    let c0 = b.podobvod(&komp1, &[v[0], v[2]]);
    let l0 = b.hradlo(And, &[c1[1], c0[0]]);
    let e = b.hradlo(And, &[c1[1], c0[1]]);
    let g0 = b.hradlo(And, &[c1[1], c0[2]]);
    let lt = b.hradlo(Or, &[c1[0], l0]);
    let gt = b.hradlo(Or, &[c1[2], g0]);
    b.vystup("A<B", lt);
    b.vystup("A=B", e);
    b.vystup("A>B", gt);
    k.pridej(
        Popis {
            uid: "5afe3deb-e03b-445b-9b3e-28f2c893113e",
            cs: "Komparátor 2 bity",
            en: "2-bit comparator",
            kat: "combinational",
            popis_cs: "Porovnání dvoubitových čísel ze dvou komparátorů 1 bit: o výsledku \
                rozhoduje vyšší bit, při rovnosti vyšších bitů nižší bit. \
                A>B = (A1>B1) + (A1=B1)·(A0>B0). Úloha 7.11.",
            popis_en: "Comparing two-bit numbers using two 1-bit comparators: the higher bit \
                decides, if the higher bits are equal the lower bit decides. \
                A>B = (A1>B1) + (A1=B1)·(A0>B0). Task 7.11.",
        },
        b,
    );

    // --- Aritmetika ----------------------------------------------------------
    // [en] --- Arithmetic --------------------------------------------------------

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B"]);
    let s = b.hradlo(Xor, &[v[0], v[1]]);
    let c = b.hradlo(And, &[v[0], v[1]]);
    b.vystup("C", c);
    b.vystup("S", s);
    let polovicni = k.pridej(
        Popis {
            uid: "6d18d9c0-5678-4fe3-8754-cf345c481684",
            cs: "Poloviční sčítačka",
            en: "Half adder",
            kat: "arithmetic",
            popis_cs: "Sečte dva bity: součet S = A ⊕ B, přenos C = A·B. Úloha 7.9.",
            popis_en: "Adds two bits: sum S = A ⊕ B, carry C = A·B. Task 7.9.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B", "Cin"]);
    let h1 = b.podobvod(&polovicni, &[v[0], v[1]]);
    let h2 = b.podobvod(&polovicni, &[h1[1], v[2]]);
    let co = b.hradlo(Or, &[h1[0], h2[0]]);
    b.vystup("Cout", co);
    b.vystup("S", h2[1]);
    let uplna = k.pridej(
        Popis {
            uid: "421820ab-286a-47b3-b52d-169a33bc7385",
            cs: "Úplná sčítačka ze dvou polovičních",
            en: "Full adder from two half adders",
            kat: "arithmetic",
            popis_cs: "Sečte dva bity a přenos z nižšího řádu: první poloviční sčítačka sečte A a B, \
                druhá přičte Cin, přenosy se sloučí hradlem OR. S = A ⊕ B ⊕ Cin. Úloha 7.9.",
            popis_en: "Adds two bits and the carry from the lower position: the first half adder \
                adds A and B, the second adds Cin, the carries are merged by an OR gate. \
                S = A ⊕ B ⊕ Cin. Task 7.9.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let a = vstupy(&mut b, &["A0", "A1", "A2", "A3"]);
    let bb = vstupy(&mut b, &["B0", "B1", "B2", "B3"]);
    let mut carry = b.vstup("Cin");
    let mut sums = Vec::new();
    for i in 0..4 {
        let o = b.podobvod(&uplna, &[a[i], bb[i], carry]);
        carry = o[0];
        sums.push(o[1]);
    }
    b.vystup("Cout", carry);
    for (i, s) in sums.into_iter().enumerate() {
        b.vystup(&format!("S{i}"), s);
    }
    let scitacka4 = k.pridej(
        Popis {
            uid: "76477a84-5097-41e9-aa2f-a2f852a21185",
            cs: "Sčítačka 4 bity",
            en: "4-bit adder",
            kat: "arithmetic",
            popis_cs: "Paralelní sčítačka se sériovým přenosem ze čtyř úplných sčítaček \
                (Cout jednoho řádu → Cin vyššího). Stejnou funkci má obvod 74283.",
            popis_en: "Ripple-carry adder from four full adders (Cout of one position → Cin of the \
                next). The 74283 has the same function.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B"]);
    let d = b.hradlo(Xor, &[v[0], v[1]]);
    let na = b.hradlo(Not, &[v[0]]);
    let bo = b.hradlo(And, &[na, v[1]]);
    b.vystup("Bo", bo);
    b.vystup("D", d);
    k.pridej(
        Popis {
            uid: "36362f91-dfe5-4108-b79c-cfbc5220e9fa",
            cs: "Poloviční odčítačka",
            en: "Half subtractor",
            kat: "arithmetic",
            popis_cs: "Odečte bit B od bitu A: rozdíl D = A ⊕ B, výpůjčka Bo = ¬A·B. Úloha 7.8.",
            popis_en: "Subtracts bit B from bit A: difference D = A ⊕ B, borrow Bo = ¬A·B. Task 7.8.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B", "Bi"]);
    let x = b.hradlo(Xor, &[v[0], v[1]]);
    let na = b.hradlo(Not, &[v[0]]);
    let d = b.hradlo(Xor, &[x, v[2]]);
    let nx = b.hradlo(Not, &[x]);
    let t1 = b.hradlo(And, &[na, v[1]]);
    let t2 = b.hradlo(And, &[nx, v[2]]);
    let bo = b.hradlo(Or, &[t1, t2]);
    b.vystup("Bo", bo);
    b.vystup("D", d);
    k.pridej(
        Popis {
            uid: "d7530ccb-f4be-456f-a642-f83fe19cde16",
            cs: "Úplná odčítačka",
            en: "Full subtractor",
            kat: "arithmetic",
            popis_cs: "Odečte B a výpůjčku Bi z nižšího řádu od A: D = A ⊕ B ⊕ Bi, \
                Bo = ¬A·B + ¬(A ⊕ B)·Bi. Úloha 7.8.",
            popis_en: "Subtracts B and the borrow Bi from the lower position from A: \
                D = A ⊕ B ⊕ Bi, Bo = ¬A·B + ¬(A ⊕ B)·Bi. Task 7.8.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let a = vstupy(&mut b, &["A0", "A1", "A2", "A3"]);
    let bb = vstupy(&mut b, &["B0", "B1", "B2", "B3"]);
    let m = b.vstup("M");
    let bx: Vec<Sig> = bb.iter().map(|&x| b.hradlo(Xor, &[x, m])).collect();
    let mut ins = a.clone();
    ins.extend(&bx);
    ins.push(m);
    let o = b.podobvod(&scitacka4, &ins);
    b.vystup("C", o[0]);
    for i in 0..4 {
        b.vystup(&format!("S{i}"), o[i + 1]);
    }
    k.pridej(
        Popis {
            uid: "26ef336f-0810-41ba-bd69-9c153e1f1219",
            cs: "Sčítačka a odčítačka 4 bity",
            en: "4-bit adder-subtractor",
            kat: "arithmetic",
            popis_cs: "M = 0: S = A + B; M = 1: S = A − B ve dvojkovém doplňku (B se neguje hradly \
                XOR a přičte se 1 přes Cin). Při odčítání C = 1 znamená A ≥ B. Úloha 7.8.",
            popis_en: "M = 0: S = A + B; M = 1: S = A − B in two's complement (B is negated by XOR \
                gates and 1 is added via Cin). When subtracting, C = 1 means A ≥ B. Task 7.8.",
        },
        b,
    );

    // --- Sekvenční obvody ----------------------------------------------------
    // [en] --- Sequential circuits ----------------------------------------------

    let mut b = Sestavovac::new();
    let r = b.vstup("R");
    let s = b.vstup("S");
    let gq = b.hradlo_bez(1, Nor, 2);
    let gqn = b.hradlo_bez(1, Nor, 2);
    let q = b.vystupy(gq)[0];
    let qn = b.vystupy(gqn)[0];
    b.spoj(gq, 0, r);
    b.spoj(gq, 1, qn);
    b.spoj(gqn, 0, q);
    b.spoj(gqn, 1, s);
    b.vystup("Q", q);
    b.vystup("Qn", qn);
    k.pridej(
        Popis {
            uid: "81a2b144-b7af-4a21-9918-e0f7bb483196",
            cs: "Klopný obvod RS z NOR",
            en: "RS flip-flop from NOR",
            kat: "sequential",
            popis_cs: "Dvě hradla NOR se zpětnou vazbou. S = 1 nastaví Q = 1, R = 1 nuluje, \
                S = R = 0 pamatuje. S = R = 1 je zakázaný stav (Q = Qn = 0). \
                Po zapnutí je stav neurčený (X), dokud se obvod nenastaví nebo nevynuluje.",
            popis_en: "Two NOR gates with feedback. S = 1 sets Q = 1, R = 1 resets, S = R = 0 holds. \
                S = R = 1 is the forbidden state (Q = Qn = 0). After power-up the state is \
                undefined (X) until the circuit is set or reset.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let sn = b.vstup("Sn");
    let rn = b.vstup("Rn");
    let gq = b.hradlo_bez(1, Nand, 2);
    let gqn = b.hradlo_bez(1, Nand, 2);
    let q = b.vystupy(gq)[0];
    let qn = b.vystupy(gqn)[0];
    b.spoj(gq, 0, sn);
    b.spoj(gq, 1, qn);
    b.spoj(gqn, 0, q);
    b.spoj(gqn, 1, rn);
    b.vystup("Q", q);
    b.vystup("Qn", qn);
    k.pridej(
        Popis {
            uid: "83f6b87d-dcde-4d65-b227-6be1d9352499",
            cs: "Klopný obvod RS z NAND",
            en: "RS flip-flop from NAND",
            kat: "sequential",
            popis_cs: "Dvě hradla NAND se zpětnou vazbou, vstupy aktivní v nule (Sn, Rn). \
                Sn = 0 nastaví Q = 1, Rn = 0 nuluje, Sn = Rn = 1 pamatuje; Sn = Rn = 0 je zakázaný \
                stav (Q = Qn = 1). Pozor: po zapnutí jsou přepínače v 0, tedy v zakázaném stavu – \
                nejdřív dejte do 1 jen jeden ze vstupů.",
            popis_en: "Two NAND gates with feedback, inputs active low (Sn, Rn). Sn = 0 sets Q = 1, \
                Rn = 0 resets, Sn = Rn = 1 holds; Sn = Rn = 0 is the forbidden state (Q = Qn = 1). \
                Note: after power-up the switches are at 0, i.e. in the forbidden state – first set \
                only one of the inputs to 1.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let d = b.vstup("D");
    let c = b.vstup("C");
    let nd = b.hradlo(Not, &[d]);
    let s1 = b.hradlo(Nand, &[d, c]);
    let r1 = b.hradlo(Nand, &[c, nd]);
    let gq = b.hradlo_bez(3, Nand, 2);
    let gqn = b.hradlo_bez(3, Nand, 2);
    let q = b.vystupy(gq)[0];
    let qn = b.vystupy(gqn)[0];
    b.spoj(gq, 0, s1);
    b.spoj(gq, 1, qn);
    b.spoj(gqn, 0, q);
    b.spoj(gqn, 1, r1);
    b.vystup("Q", q);
    b.vystup("Qn", qn);
    k.pridej(
        Popis {
            uid: "56992124-6ad5-4f23-b346-155e619880fc",
            cs: "Klopný obvod D (úroveň) z NAND",
            en: "D latch from NAND",
            kat: "sequential",
            popis_cs: "Klopný obvod RS z NAND s řídicími hradly: při C = 1 výstup Q sleduje vstup D \
                (obvod je „průhledný“), při C = 0 si pamatuje poslední hodnotu. Invertor vylučuje \
                zakázaný stav RS. Stejnou funkci má obvod 7475.",
            popis_en: "An RS flip-flop from NAND with steering gates: with C = 1 the output Q follows \
                the input D (the latch is “transparent”), with C = 0 it holds the last value. The \
                inverter prevents the forbidden RS state. The 7475 has the same function.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let c = b.vstup("C");
    let one = b.konstanta(true);
    let mut clk = c;
    let mut qs = Vec::new();
    for i in 0..4 {
        let ff = b.klopny(i + 1, Tff, false);
        b.spoj(ff, 0, one);
        b.spoj(ff, 1, clk);
        let o = b.vystupy(ff);
        qs.push(o[0]);
        clk = o[1];
    }
    for (i, q) in qs.into_iter().enumerate() {
        b.vystup(&format!("Q{i}"), q);
    }
    k.pridej(
        Popis {
            uid: "9d62bb08-de3d-465f-93d0-780778ea56c8",
            cs: "Asynchronní čítač 4 bity",
            en: "4-bit asynchronous counter",
            kat: "sequential",
            popis_cs: "Čítač nahoru 0–15 ze čtyř klopných obvodů T (T = 1, každý stupeň dělí \
                dvěma). Každý další stupeň je hodinován výstupem Qn předchozího, takže se stupně \
                překlápějí postupně – v časovém diagramu je vidět zpoždění. Stejnou funkci má 7493.",
            popis_en: "Up counter 0–15 from four T flip-flops (T = 1, each stage divides by two). \
                Each next stage is clocked by the Qn output of the previous one, so the stages \
                toggle one after another – the delay is visible in the timing diagram. The 7493 has \
                the same function.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let c = b.vstup("C");
    let one = b.konstanta(true);
    let ffs: Vec<_> = (0..4).map(|_| b.klopny(1, Jk, false)).collect();
    let q: Vec<Sig> = ffs.iter().map(|&f| b.vystupy(f)[0]).collect();
    let a1 = b.hradlo_ve(2, And, &[q[0], q[1]]);
    let a2 = b.hradlo_ve(2, And, &[q[0], q[1], q[2]]);
    for (&ff, jk) in ffs.iter().zip([one, q[0], a1, a2]) {
        b.spoj(ff, 0, jk);
        b.spoj(ff, 1, c);
        b.spoj(ff, 2, jk);
    }
    for (i, s) in q.iter().enumerate() {
        b.vystup(&format!("Q{i}"), *s);
    }
    k.pridej(
        Popis {
            uid: "9c8a6ed8-19b6-43c2-8dc4-514c06cf2ca1",
            cs: "Synchronní čítač 4 bity",
            en: "4-bit synchronous counter",
            kat: "sequential",
            popis_cs: "Čítač nahoru 0–15 ze čtyř klopných obvodů JK se společnými hodinami. \
                Stupeň se překlopí, když jsou všechny nižší stupně v 1: J = K = Q0·Q1·…",
            popis_en: "Up counter 0–15 from four JK flip-flops with a common clock. A stage \
                toggles when all lower stages are 1: J = K = Q0·Q1·…",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let c = b.vstup("C");
    let one = b.konstanta(true);
    let ffs: Vec<_> = (0..4).map(|_| b.klopny(1, Jk, false)).collect();
    let o: Vec<Vec<Sig>> = ffs.iter().map(|&f| b.vystupy(f)).collect();
    let (q0, q1, q2, q3n) = (o[0][0], o[1][0], o[2][0], o[3][1]);
    let j1 = b.hradlo_ve(2, And, &[q0, q3n]);
    let a01 = b.hradlo_ve(2, And, &[q0, q1]);
    let j3 = b.hradlo_ve(2, And, &[q0, q1, q2]);
    for (ff, (j, kk)) in ffs.iter().zip([(one, one), (j1, q0), (a01, a01), (j3, q0)]) {
        b.spoj(*ff, 0, j);
        b.spoj(*ff, 1, c);
        b.spoj(*ff, 2, kk);
    }
    for (i, oo) in o.iter().enumerate() {
        b.vystup(&format!("Q{i}"), oo[0]);
    }
    k.pridej(
        Popis {
            uid: "2a3b8a37-7bc1-4791-b5ab-ecc06259fc29",
            cs: "Synchronní čítač modulo 10 (BCD)",
            en: "Synchronous mod-10 (BCD) counter",
            kat: "sequential",
            popis_cs: "Dekadický čítač 0–9 z klopných obvodů JK: po 9 (1001) následuje 0. \
                J1 = Q0·¬Q3, K1 = Q0, J2 = K2 = Q0·Q1, J3 = Q0·Q1·Q2, K3 = Q0. \
                Úloha 7.14 (čítač modulo N); stejnou funkci má 7490.",
            popis_en: "Decade counter 0–9 from JK flip-flops: 9 (1001) is followed by 0. \
                J1 = Q0·¬Q3, K1 = Q0, J2 = K2 = Q0·Q1, J3 = Q0·Q1·Q2, K3 = Q0. \
                Task 7.14 (mod-N counter); the 7490 has the same function.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let c = b.vstup("C");
    let u = b.vstup("U");
    let one = b.konstanta(true);
    let ffs: Vec<_> = (0..3).map(|_| b.klopny(1, Tff, false)).collect();
    let un = b.hradlo_ve(1, Not, &[u]);
    let o: Vec<Vec<Sig>> = ffs.iter().map(|&f| b.vystupy(f)).collect();
    let up1 = b.hradlo_ve(2, And, &[u, o[0][0]]);
    let dn1 = b.hradlo_ve(2, And, &[un, o[0][1]]);
    let up2 = b.hradlo_ve(2, And, &[u, o[0][0], o[1][0]]);
    let dn2 = b.hradlo_ve(2, And, &[un, o[0][1], o[1][1]]);
    let t1 = b.hradlo_ve(3, Or, &[up1, dn1]);
    let t2 = b.hradlo_ve(3, Or, &[up2, dn2]);
    for (ff, t) in ffs.iter().zip([one, t1, t2]) {
        b.spoj(*ff, 0, t);
        b.spoj(*ff, 1, c);
    }
    for (i, oo) in o.iter().enumerate() {
        b.vystup(&format!("Q{i}"), oo[0]);
    }
    k.pridej(
        Popis {
            uid: "f6c47598-7bba-480d-a21e-4dcf87e66cb4",
            cs: "Čítač nahoru / dolů 3 bity",
            en: "3-bit up/down counter",
            kat: "sequential",
            popis_cs: "Synchronní čítač z klopných obvodů T: U = 1 čítá nahoru, U = 0 dolů. \
                Stupeň se překlopí, když jsou nižší stupně všechny v 1 (nahoru) nebo všechny v 0 \
                (dolů). Úloha 7.13; obdobnou funkci má 74193.",
            popis_en: "Synchronous counter from T flip-flops: U = 1 counts up, U = 0 down. A stage \
                toggles when the lower stages are all 1 (up) or all 0 (down). Task 7.13; the \
                74193 has a similar function.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let c = b.vstup("C");
    let d = b.vstup("D");
    let mut data = d;
    let mut qs = Vec::new();
    for i in 0..4 {
        let ff = b.klopny(i + 1, Dff, false);
        b.spoj(ff, 0, data);
        b.spoj(ff, 1, c);
        data = b.vystupy(ff)[0];
        qs.push(data);
    }
    for (i, q) in qs.into_iter().enumerate() {
        b.vystup(&format!("Q{i}"), q);
    }
    k.pridej(
        Popis {
            uid: "622b9a50-b32c-40dc-9997-1e679198914d",
            cs: "Posuvný registr 4 bity",
            en: "4-bit shift register",
            kat: "sequential",
            popis_cs: "Čtyři klopné obvody D za sebou se společnými hodinami: každou náběžnou hranou \
                se data posunou o jeden stupeň (D → Q0 → Q1 → Q2 → Q3). Sériový vstup, paralelní \
                výstup – jako 74164.",
            popis_en: "Four D flip-flops in a row with a common clock: on every rising edge the data \
                shift by one stage (D → Q0 → Q1 → Q2 → Q3). Serial in, parallel out – like the 74164.",
        },
        b,
    );

    for (uid, johnson) in [
        ("ff0d3dca-3e5b-4f0a-9206-7ea6add9f4ba", true),
        ("6b88169d-604a-44c5-ada8-0a29ed772234", false),
    ] {
        let mut b = Sestavovac::new();
        let c = b.vstup("C");
        let ffs: Vec<_> = (0..4)
            .map(|i| b.klopny(i + 1, Dff, !johnson && i == 0))
            .collect();
        let o: Vec<Vec<Sig>> = ffs.iter().map(|&f| b.vystupy(f)).collect();
        let back = if johnson { o[3][1] } else { o[3][0] };
        for (i, ff) in ffs.iter().enumerate() {
            let data = if i == 0 { back } else { o[i - 1][0] };
            b.spoj(*ff, 0, data);
            b.spoj(*ff, 1, c);
        }
        for (i, oo) in o.iter().enumerate() {
            b.vystup(&format!("Q{i}"), oo[0]);
        }
        let popis = if johnson {
            Popis {
                uid,
                cs: "Johnsonův čítač 4 bity",
                en: "4-bit Johnson counter",
                kat: "sequential",
                popis_cs: "Posuvný registr, jehož poslední výstup se vrací negovaný (Qn3 → D): \
                    0000 → 0001 → 0011 → 0111 → 1111 → 1110 → … – osm stavů, sousední se liší \
                    v jediném bitu.",
                popis_en: "A shift register whose last output returns negated (Qn3 → D): \
                    0000 → 0001 → 0011 → 0111 → 1111 → 1110 → … – eight states, neighbours differ \
                    in a single bit.",
            }
        } else {
            Popis {
                uid,
                cs: "Kruhový čítač 4 bity",
                en: "4-bit ring counter",
                kat: "sequential",
                popis_cs: "Posuvný registr uzavřený do kruhu (Q3 → D) s jedinou jedničkou, která \
                    obíhá: Q0 → Q1 → Q2 → Q3 → Q0. Po spuštění je Q0 = 1 (počáteční stav klopného \
                    obvodu). Hodí se k postupnému spínání (běžící světlo).",
                popis_en: "A shift register closed into a ring (Q3 → D) with a single one that \
                    circulates: Q0 → Q1 → Q2 → Q3 → Q0. After start Q0 = 1 (the flip-flop's initial \
                    state). Useful for sequential switching (running light).",
            }
        };
        k.pridej(popis, b);
    }

    // --- Aplikace ------------------------------------------------------------
    // [en] --- Applications ------------------------------------------------------

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["D", "O", "P", "Z"]);
    let any = b.hradlo(Or, &[v[0], v[1], v[2]]);
    let r = b.hradlo(Not, &[v[3]]);
    let s = b.hradlo(And, &[v[3], any]);
    let ga = b.hradlo_bez(3, Nor, 2);
    let gn = b.hradlo_bez(3, Nor, 2);
    let q = b.vystupy(ga)[0];
    let qn = b.vystupy(gn)[0];
    b.spoj(ga, 0, r);
    b.spoj(ga, 1, qn);
    b.spoj(gn, 0, q);
    b.spoj(gn, 1, s);
    b.vystup("A", q);
    k.pridej(
        Popis {
            uid: "9097ceb1-ead8-4aad-a890-d9642c2a36ff",
            cs: "Alarm s pamětí",
            en: "Alarm with memory",
            kat: "applications",
            popis_cs: "Zabezpečovací zařízení: je-li zapnuté (Z = 1) a otevřou se dveře (D), okno (O) \
                nebo hlásí čidlo pohybu (P), rozsvítí se alarm A a svítí dál i po zavření dveří, \
                dokud se zařízení nevypne (Z = 0). Paměť tvoří klopný obvod RS z NOR. Úloha 7.6.",
            popis_en: "Security system: when it is armed (Z = 1) and the door (D) or window (O) opens \
                or the motion sensor (P) reports, the alarm A turns on and stays on even after the \
                door closes, until the system is disarmed (Z = 0). The memory is an RS flip-flop \
                from NOR. Task 7.6.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let v = vstupy(&mut b, &["A", "B", "C"]);
    let n: Vec<Sig> = v.iter().map(|&x| b.hradlo(Not, &[x])).collect();
    let zel = b.hradlo_ve(2, And, &[v[0], v[1], v[2]]);
    let ab = b.hradlo(And, &[n[0], n[1]]);
    let ac = b.hradlo(And, &[n[0], n[2]]);
    let bc = b.hradlo(And, &[n[1], n[2]]);
    let cer = b.hradlo(Or, &[ab, ac, bc]);
    let zlu = b.hradlo(Nor, &[zel, cer]);
    b.vystup("L1 zelená", zel);
    b.vystup("L2 žlutá", zlu);
    b.vystup("L3 červená", cer);
    k.pridej(
        Popis {
            uid: "12525482-6598-4c65-9796-66ad71ac1dbd",
            cs: "Indikace funkce ventilátorů",
            en: "Fan status indicator",
            kat: "applications",
            popis_cs: "Tři ventilátory A, B, C (1 = běží). Zelená: běží všechny. Žlutá: jeden \
                stojí. Červená: stojí dva nebo všechny (majoritní funkce negovaných vstupů). \
                Úloha 7.7.",
            popis_en: "Three fans A, B, C (1 = running). Green: all running. Yellow: one stopped. \
                Red: two or all stopped (majority function of the negated inputs). Task 7.7.",
        },
        b,
    );

    let mut b = Sestavovac::new();
    let c = b.vstup("C");
    let one = b.konstanta(true);
    let f0 = b.klopny(1, Tff, false);
    let f1 = b.klopny(1, Tff, false);
    let o0 = b.vystupy(f0);
    let o1 = b.vystupy(f1);
    b.spoj(f0, 0, one);
    b.spoj(f0, 1, c);
    b.spoj(f1, 0, o0[0]);
    b.spoj(f1, 1, c);
    let red = b.hradlo(Not, &[o1[0]]);
    let green = b.hradlo(And, &[o1[0], o0[1]]);
    b.vystup("L1 červená", red);
    b.vystup("L2 oranžová", o0[0]);
    b.vystup("L3 zelená", green);
    k.pridej(
        Popis {
            uid: "2ca55177-224c-4e20-9642-421a553a8480",
            cs: "Semafor",
            en: "Traffic light",
            kat: "applications",
            popis_cs: "Dvoubitový synchronní čítač z klopných obvodů T a dekodér světel: \
                červená → červená + oranžová → zelená → oranžová → červená … \
                Každá náběžná hrana hodin C posune semafor o jednu fázi.",
            popis_en: "A two-bit synchronous counter from T flip-flops and a light decoder: \
                red → red + amber → green → amber → red … Every rising edge of the clock C advances \
                the traffic light by one phase.",
        },
        b,
    );

    k.lib
}

// --- Pomocné funkce testů ----------------------------------------------------
// [en] --- Test helpers ----------------------------------------------------------

/// Simulace jednoho prvku s ovládáním vstupů a čtením výstupů podle popisku.
///
/// [en] Simulation of one item with inputs set and outputs read by label.
struct Zkouska {
    /// Simulátor prvku.
    ///
    /// [en] The item's simulator.
    sim: Simulator,
    /// Vstupy (přepínače) v pořadí rozhraní: popisek a id.
    ///
    /// [en] Inputs (switches) in interface order: label and id.
    ins: Vec<(String, CompId)>,
    /// Výstupy (LED) v pořadí rozhraní: popisek a id.
    ///
    /// [en] Outputs (LEDs) in interface order: label and id.
    outs: Vec<(String, CompId)>,
}

impl Zkouska {
    /// Spustí simulaci prvku s daným českým názvem.
    ///
    /// [en] Starts the simulation of the item with the given Czech name.
    fn new(lib: &Library, name: &str) -> Self {
        let item = lib
            .items
            .iter()
            .find(|i| i.name == name)
            .unwrap_or_else(|| panic!("prvek {name} chybí / item missing"));
        let (ins, outs) = item.circuit.io();
        let pick = |v: Vec<&logicflow_core::Component>| {
            v.into_iter().map(|c| (c.label.clone(), c.id)).collect()
        };
        let mut z = Self {
            sim: Simulator::with_subcircuits(&item.circuit, &lib.defs(), None),
            ins: pick(ins),
            outs: pick(outs),
        };
        z.ustal();
        z
    }

    /// Nechá obvod ustálit.
    ///
    /// [en] Lets the circuit settle.
    fn ustal(&mut self) {
        assert!(self.sim.run(100_000), "obvod se neustálil / did not settle");
    }

    /// Nastaví vstup podle popisku.
    ///
    /// [en] Sets an input by label.
    fn nastav(&mut self, label: &str, on: bool) {
        let id = self
            .ins
            .iter()
            .find(|(l, _)| l == label)
            .expect("vstup / input")
            .1;
        self.sim.set_source(id, LogicValue::from(on));
        self.ustal();
    }

    /// Hodnota výstupu podle popisku.
    ///
    /// [en] Value of an output by label.
    fn vystup(&self, label: &str) -> LogicValue {
        let id = self
            .outs
            .iter()
            .find(|(l, _)| l == label)
            .expect("výstup / output")
            .1;
        self.sim.inputs(id)[0]
    }

    /// Výstup podle popisku jako bool (X nebo Z je chyba testu).
    ///
    /// [en] Output by label as bool (X or Z fails the test).
    fn je(&self, label: &str) -> bool {
        match self.vystup(label) {
            LogicValue::One => true,
            LogicValue::Zero => false,
            v => panic!("výstup {label} je {v:?} / output is {v:?}"),
        }
    }

    /// Číslo z výstupů Q0, Q1, … (Q0 nejnižší bit).
    ///
    /// [en] Number from the outputs Q0, Q1, … (Q0 the lowest bit).
    fn cislo(&self, prefix: &str, bits: usize) -> u32 {
        (0..bits)
            .map(|i| u32::from(self.je(&format!("{prefix}{i}"))) << i)
            .sum()
    }

    /// Jeden hodinový impuls na vstupu C (náběžná a sestupná hrana).
    ///
    /// [en] One clock pulse on input C (rising and falling edge).
    fn takt(&mut self) {
        self.nastav("C", true);
        self.nastav("C", false);
    }
}

/// Ověří kombinační prvek proti funkci pro všechny kombinace vstupů
/// (vstupy a výstupy v pořadí rozhraní).
///
/// [en] Checks a combinational item against a function for all input combinations
/// (inputs and outputs in interface order).
fn kombinacni(lib: &Library, name: &str, f: impl Fn(&[bool]) -> Vec<bool>) {
    let mut z = Zkouska::new(lib, name);
    let n = z.ins.len();
    let labels: Vec<String> = z.ins.iter().map(|(l, _)| l.clone()).collect();
    let outs: Vec<String> = z.outs.iter().map(|(l, _)| l.clone()).collect();
    for combo in 0..1u32 << n {
        let x: Vec<bool> = (0..n).map(|i| combo >> i & 1 == 1).collect();
        for (l, &v) in labels.iter().zip(&x) {
            z.nastav(l, v);
        }
        let got: Vec<bool> = outs.iter().map(|l| z.je(l)).collect();
        assert_eq!(got, f(&x), "{name}: vstupy {labels:?} = {x:?}");
    }
}

/// Číslo z bitů (první nejnižší).
///
/// [en] Number from bits (the first is the lowest).
fn num(bits: &[bool]) -> u32 {
    bits.iter()
        .enumerate()
        .map(|(i, &b)| u32::from(b) << i)
        .sum()
}

/// Bity čísla (první nejnižší).
///
/// [en] Bits of a number (the first is the lowest).
fn bits(n: u32, count: usize) -> Vec<bool> {
    (0..count).map(|i| n >> i & 1 == 1).collect()
}

// --- Testy -------------------------------------------------------------------
// [en] --- Tests -----------------------------------------------------------------

#[test]
#[allow(clippy::disallowed_methods)]
fn soubor_je_aktualni() {
    let lib = knihovna();
    let text = lib.to_json() + "\n";
    // Vygenerovaná knihovna musí projít kontrolou jádra a znovu se načíst beze změny.
    // [en] The generated library must pass the core check and reload unchanged.
    assert_eq!(Library::from_json(&text).unwrap(), lib);
    if std::env::var("LOGICFLOW_ZAPSAT_KNIHOVNU").is_ok() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");
        std::fs::write(format!("{path}{SOUBOR}"), &text).unwrap();
        return;
    }
    let strip = |t: &str| {
        t.lines()
            .filter(|l| !l.trim_start().starts_with("\"app\""))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert!(
        strip(&text) == strip(ULOZENA),
        "{SOUBOR} není aktuální – spusťte LOGICFLOW_ZAPSAT_KNIHOVNU=1 cargo test -p logicflow-core \
         --test vyukova_knihovna / the file is outdated"
    );
}

#[test]
fn hradla_z_nand_a_nor() {
    let lib = knihovna();
    kombinacni(&lib, "NOT z NAND", |x| vec![!x[0]]);
    kombinacni(&lib, "NOT z NOR", |x| vec![!x[0]]);
    kombinacni(&lib, "AND z NAND", |x| vec![x[0] && x[1]]);
    kombinacni(&lib, "AND z NOR", |x| vec![x[0] && x[1]]);
    kombinacni(&lib, "OR z NAND", |x| vec![x[0] || x[1]]);
    kombinacni(&lib, "OR z NOR", |x| vec![x[0] || x[1]]);
    kombinacni(&lib, "XOR ze čtyř NAND", |x| vec![x[0] ^ x[1]]);
    kombinacni(&lib, "XNOR ze čtyř NOR", |x| vec![x[0] == x[1]]);
}

#[test]
fn kombinacni_obvody() {
    let lib = knihovna();
    kombinacni(&lib, "Majoritní obvod 2 ze 3", |x| {
        vec![x.iter().filter(|&&b| b).count() >= 2]
    });
    kombinacni(&lib, "Generátor sudé parity 4 bity", |x| {
        vec![x.iter().filter(|&&b| b).count() % 2 == 1]
    });
    kombinacni(&lib, "Převodník binární → Grayův kód 4 bity", |x| {
        let n = num(x);
        bits(n ^ (n >> 1), 4)
    });
    kombinacni(&lib, "Převodník Grayův kód → binární 4 bity", |x| {
        let g = num(x);
        let mut n = g;
        let mut s = g >> 1;
        while s != 0 {
            n ^= s;
            s >>= 1;
        }
        bits(n, 4)
    });
    kombinacni(&lib, "Multiplexor 2:1 z hradel", |x| {
        vec![if x[2] { x[1] } else { x[0] }]
    });
    kombinacni(&lib, "Multiplexor 4:1 z hradel", |x| {
        vec![x[num(&x[4..6]) as usize]]
    });
    kombinacni(&lib, "Dekodér 2 → 4 z hradel", |x| {
        let a = num(&x[0..2]);
        (0..4).map(|i| x[2] && a == i).collect()
    });
    kombinacni(&lib, "Komparátor 1 bit", |x| {
        let (a, b) = (u32::from(x[0]), u32::from(x[1]));
        vec![a < b, a == b, a > b]
    });
    kombinacni(&lib, "Komparátor 2 bity", |x| {
        let (a, b) = (num(&x[0..2]), num(&x[2..4]));
        vec![a < b, a == b, a > b]
    });
}

#[test]
fn aritmetika() {
    let lib = knihovna();
    kombinacni(&lib, "Poloviční sčítačka", |x| {
        vec![x[0] && x[1], x[0] ^ x[1]]
    });
    kombinacni(&lib, "Úplná sčítačka ze dvou polovičních", |x| {
        let s = x.iter().filter(|&&b| b).count();
        vec![s >= 2, s % 2 == 1]
    });
    kombinacni(&lib, "Sčítačka 4 bity", |x| {
        let s = num(&x[0..4]) + num(&x[4..8]) + u32::from(x[8]);
        let mut out = vec![s >= 16];
        out.extend(bits(s, 4));
        out
    });
    kombinacni(&lib, "Poloviční odčítačka", |x| {
        vec![!x[0] && x[1], x[0] ^ x[1]]
    });
    kombinacni(&lib, "Úplná odčítačka", |x| {
        let d = i32::from(x[0]) - i32::from(x[1]) - i32::from(x[2]);
        vec![d < 0, d.rem_euclid(2) == 1]
    });
    kombinacni(&lib, "Sčítačka a odčítačka 4 bity", |x| {
        let (a, b, m) = (num(&x[0..4]), num(&x[4..8]), x[8]);
        let (c, s) = if m {
            (a >= b, a.wrapping_sub(b) & 15)
        } else {
            (a + b >= 16, (a + b) & 15)
        };
        let mut out = vec![c];
        out.extend(bits(s, 4));
        out
    });
}

#[test]
fn klopne_obvody_rs_a_d() {
    let lib = knihovna();
    let mut z = Zkouska::new(&lib, "Klopný obvod RS z NOR");
    z.nastav("S", true);
    z.nastav("S", false);
    assert!(z.je("Q") && !z.je("Qn"), "nastaveno / set");
    z.nastav("R", true);
    z.nastav("R", false);
    assert!(!z.je("Q") && z.je("Qn"), "vynulováno / reset");

    let mut z = Zkouska::new(&lib, "Klopný obvod RS z NAND");
    assert!(
        z.je("Q") && z.je("Qn"),
        "zakázaný stav po zapnutí / forbidden state"
    );
    z.nastav("Rn", true);
    assert!(z.je("Q") && !z.je("Qn"));
    z.nastav("Sn", true);
    assert!(z.je("Q"), "pamatuje / holds");
    z.nastav("Rn", false);
    z.nastav("Rn", true);
    assert!(!z.je("Q") && z.je("Qn"), "vynulováno / reset");

    let mut z = Zkouska::new(&lib, "Klopný obvod D (úroveň) z NAND");
    for d in [true, false, true] {
        z.nastav("D", d);
        z.nastav("C", true);
        assert_eq!(z.je("Q"), d, "průhledný / transparent");
        z.nastav("C", false);
        z.nastav("D", !d);
        assert_eq!(z.je("Q"), d, "pamatuje / holds");
        assert_eq!(z.je("Qn"), !d);
    }
}

#[test]
fn citace() {
    let lib = knihovna();
    for name in ["Asynchronní čítač 4 bity", "Synchronní čítač 4 bity"] {
        let mut z = Zkouska::new(&lib, name);
        for n in 0..40 {
            assert_eq!(z.cislo("Q", 4), n % 16, "{name}");
            z.takt();
        }
    }
    let mut z = Zkouska::new(&lib, "Synchronní čítač modulo 10 (BCD)");
    for n in 0..30 {
        assert_eq!(z.cislo("Q", 4), n % 10, "BCD");
        z.takt();
    }
    let mut z = Zkouska::new(&lib, "Čítač nahoru / dolů 3 bity");
    z.nastav("U", true);
    for n in 0..10 {
        assert_eq!(z.cislo("Q", 3), n % 8, "nahoru / up");
        z.takt();
    }
    z.nastav("U", false);
    for n in (0..=10u32).rev() {
        assert_eq!(z.cislo("Q", 3), n % 8, "dolů / down");
        z.takt();
    }
}

#[test]
fn registry() {
    let lib = knihovna();
    let mut z = Zkouska::new(&lib, "Posuvný registr 4 bity");
    let data = [true, false, true, true, false, false, true, false];
    let mut expect = 0u32;
    for &d in &data {
        z.nastav("D", d);
        z.takt();
        expect = (expect << 1 | u32::from(d)) & 15;
        // Q0 je poslední vstoupivší bit (nejnižší bit čísla).
        // [en] Q0 is the most recently entered bit (the lowest bit of the number).
        assert_eq!(z.cislo("Q", 4), expect);
    }

    let mut z = Zkouska::new(&lib, "Johnsonův čítač 4 bity");
    let seq = [
        0b0000, 0b0001, 0b0011, 0b0111, 0b1111, 0b1110, 0b1100, 0b1000,
    ];
    for n in 0..16 {
        assert_eq!(z.cislo("Q", 4), seq[n % 8], "Johnson");
        z.takt();
    }

    let mut z = Zkouska::new(&lib, "Kruhový čítač 4 bity");
    for n in 0..12 {
        assert_eq!(z.cislo("Q", 4), 1 << (n % 4), "kruhový / ring");
        z.takt();
    }
}

#[test]
fn aplikace() {
    let lib = knihovna();
    let mut z = Zkouska::new(&lib, "Alarm s pamětí");
    assert!(!z.je("A"), "vypnuto / disarmed");
    z.nastav("D", true);
    assert!(!z.je("A"), "vypnuté zařízení nereaguje / disarmed ignores");
    z.nastav("D", false);
    z.nastav("Z", true);
    assert!(!z.je("A"));
    for vstup in ["D", "O", "P"] {
        z.nastav(vstup, true);
        z.nastav(vstup, false);
        assert!(z.je("A"), "alarm drží / alarm latches ({vstup})");
        z.nastav("Z", false);
        assert!(!z.je("A"), "vypnutí zruší alarm / disarming clears");
        z.nastav("Z", true);
    }

    kombinacni(&lib, "Indikace funkce ventilátorů", |x| {
        let stoji = x.iter().filter(|&&b| !b).count();
        vec![stoji == 0, stoji == 1, stoji >= 2]
    });

    let mut z = Zkouska::new(&lib, "Semafor");
    let faze = [
        (true, false, false),
        (true, true, false),
        (false, false, true),
        (false, true, false),
    ];
    for n in 0..8 {
        let got = (z.je("L1 červená"), z.je("L2 oranžová"), z.je("L3 zelená"));
        assert_eq!(got, faze[n % 4], "fáze / phase {n}");
        z.takt();
    }
}

#[test]
fn rozhrani_prvku() {
    let lib = knihovna();
    // Rozhraní (pořadí vývodů na značce) vybraných prvků.
    // [en] Interface (pin order on the symbol) of selected items.
    let io = |name: &str| {
        let d = lib.items.iter().find(|i| i.name == name).unwrap().def();
        d.pin_names()
    };
    assert_eq!(
        io("Úplná sčítačka ze dvou polovičních").0,
        ["A", "B", "Cin"]
    );
    assert_eq!(io("Komparátor 1 bit").1, ["A<B", "A=B", "A>B"]);
    assert_eq!(io("Semafor").1, ["L1 červená", "L2 oranžová", "L3 zelená"]);
    // Každý prvek má anglický název, kategorii a popis v obou jazycích.
    // [en] Every item has an English name, a category and a description in both languages.
    for item in &lib.items {
        assert!(
            !item.description.is_empty() && !item.category.is_empty(),
            "{}",
            item.name
        );
        let en = &item.translations["en"];
        assert!(
            !en.name.is_empty() && !en.description.is_empty(),
            "{}",
            item.name
        );
    }
}

/// Náhled pro program bez podpory knihoven: s proměnnou `LOGICFLOW_NAHLED=<složka>`
/// zapíše každý prvek jako samostatný projekt `.logicflow` (hlavní obvod = prvek)
/// a katalog `cislicova-technika.logicflow` se všemi prvky jako podobvody a jejich
/// značkami v hlavním obvodu. Soubory se do repozitáře neukládají.
///
/// [en] Preview for a program without library support: with the variable
/// `LOGICFLOW_NAHLED=<folder>` it writes every item as a separate `.logicflow` project
/// (main circuit = the item) and a catalogue `cislicova-technika.logicflow` with all
/// items as subcircuits and their symbols in the main circuit. The files are not
/// stored in the repository.
#[test]
#[allow(clippy::disallowed_methods)]
fn nahled_projekty() {
    use logicflow_core::{Circuit, Point, ProjectFile};
    let Ok(dir) = std::env::var("LOGICFLOW_NAHLED") else {
        return;
    };
    let lib = knihovna();
    let defs = lib.defs();
    for (i, item) in lib.items.iter().enumerate() {
        let deps: Vec<SubcircuitDef> = lib
            .item_with_dependencies(&item.uid)
            .unwrap()
            .into_iter()
            .filter(|d| d.id != item.id)
            .collect();
        let text = ProjectFile::to_json(&item.circuit, &deps);
        std::fs::write(format!("{dir}/{:02}.logicflow", i + 1), text).unwrap();
    }
    // Katalog: značky všech prvků v mřížce po šesti.
    // [en] Catalogue: symbols of all items in a grid of six.
    let mut main = Circuit::default();
    let (mut x, mut y, mut row_h) = (0, 0, 0);
    for (i, d) in defs.iter().enumerate() {
        if i > 0 && i % 6 == 0 {
            x = 0;
            y += row_h + 6;
            row_h = 0;
        }
        let kind = d.kind();
        let (_, y1, _, y2) = kind.body_local();
        let id = main.add_component(kind, Point::new(x, y - y1.floor() as i32));
        main.component_mut(id).unwrap().label = String::new();
        row_h = row_h.max((y2 - y1).ceil() as i32);
        x += 24;
    }
    let text = ProjectFile::to_json(&main, &defs);
    std::fs::write(format!("{dir}/cislicova-technika.logicflow"), text).unwrap();
}
