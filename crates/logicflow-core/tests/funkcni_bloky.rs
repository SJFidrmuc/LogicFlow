//! Vestavěná knihovna „Funkční bloky – vnitřní zapojení“ (`docs/knihovny/funkcni-bloky.lflib`).
//!
//! Pro každý funkční blok programu (multiplexor, dekodér, kodér, sčítačka,
//! komparátor, převodník BCD na 7 segmentů, čítač, registr, posuvný registr) se
//! všemi hodnotami parametru obsahuje zapojení z hradel se stejně pojmenovanými
//! vývody. Sekvenční bloky jsou z klopných obvodů D řízených hranou, které jsou
//! samy ze šesti hradel NAND (jako 7474), takže jde Tabem rozkliknout každý blok
//! až na hradla. Program ho otevře Tabem na bloku (jen pro čtení).
//!
//! Knihovnu generuje tento test sestavovačem schémat a každý prvek porovná se
//! simulací bloku: kombinační pro všechny kombinace vstupů, sekvenční na dlouhém
//! pseudonáhodném průběhu vstupů (hodiny, nulování, povolení, data).
//!
//! Po změně prvků se soubor knihovny přegeneruje:
//! `LOGICFLOW_ZAPSAT_KNIHOVNU=1 cargo test -p logicflow-core --test funkcni_bloky`
//!
//! [en]
//!
//! Built-in library “Function blocks – internal circuits” (`docs/knihovny/funkcni-bloky.lflib`).
//!
//! For every function block of the program (multiplexer, decoder, encoder, adder,
//! comparator, BCD to 7-segment decoder, counter, register, shift register) with
//! all values of its parameter it contains a gate-level circuit with pins of the
//! same names. Sequential blocks are made of edge-triggered D flip-flops that are
//! themselves made of six NAND gates (like the 7474), so every block can be opened
//! with Tab all the way down to gates. The program opens it with Tab on a block
//! (read-only).
//!
//! This test generates the library with the schematic builder and compares every
//! item with the simulation of the block: combinational ones for all input
//! combinations, sequential ones on a long pseudo-random sequence of inputs
//! (clock, reset, enable, data).
//!
//! After changing the items, regenerate the library file:
//! `LOGICFLOW_ZAPSAT_KNIHOVNU=1 cargo test -p logicflow-core --test funkcni_bloky`

#[allow(dead_code)]
mod sestavovac;

use std::collections::BTreeMap;

use logicflow_core::block::{BLOCK_LIBRARY_UID, SEVEN_SEG_HEX};
use logicflow_core::library::{DEFAULT_LICENSE, Library, LibraryInfo, LibraryItem, Translation};
use logicflow_core::{BlockKind, CompId, GateKind, LogicValue, Simulator, SubcircuitDef};
use sestavovac::{Sestavovac, Sig};

use GateKind::{And, Nand, Nor, Not, Or, Xnor, Xor};

/// Cesta ke generovanému souboru (vzhledem ke kořeni repozitáře).
///
/// [en] Path to the generated file (relative to the repository root).
const SOUBOR: &str = "docs/knihovny/funkcni-bloky.lflib";

/// Uložený soubor knihovny.
///
/// [en] The stored library file.
const ULOZENA: &str = include_str!("../../../docs/knihovny/funkcni-bloky.lflib");

/// Datum vytvoření prvků.
///
/// [en] Creation date of the items.
const DATUM: &str = "2026-11-27";

/// Autor knihovny a prvků.
///
/// [en] Author of the library and the items.
const AUTOR: &str = "Mgr. Stanislav Jan Fidrmuc";

/// Identita pomocného klopného obvodu D s nulováním.
///
/// [en] Identity of the helper D flip-flop with reset.
const UID_D: &str = "2fb61d2d-120f-4578-bc83-37cb2672d765";

/// Identita pomocného klopného obvodu T s nulováním.
///
/// [en] Identity of the helper T flip-flop with reset.
const UID_T: &str = "cf087cb5-9698-4cd9-afb6-7c375249e241";

/// Popis prvku: identita, názvy, kategorie a popisy česky a anglicky.
///
/// [en] Item description: identity, names, category and descriptions in Czech and English.
struct Popis {
    /// Stabilní identita (UUID).
    ///
    /// [en] Stable identity (UUID).
    uid: String,
    /// Český název.
    ///
    /// [en] Czech name.
    cs: String,
    /// Anglický název.
    ///
    /// [en] English name.
    en: String,
    /// Klíč kategorie.
    ///
    /// [en] Category key.
    kat: &'static str,
    /// Český popis.
    ///
    /// [en] Czech description.
    popis_cs: String,
    /// Anglický popis.
    ///
    /// [en] English description.
    popis_en: String,
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
            uid: p.uid,
            id: self.lib.next_item_id(),
            name: p.cs,
            category: p.kat.to_owned(),
            description: p.popis_cs,
            translations: [(
                "en".to_owned(),
                Translation {
                    name: p.en,
                    description: p.popis_en,
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
fn vstupy(b: &mut Sestavovac, prefix: &str, n: usize) -> Vec<Sig> {
    (0..n).map(|i| b.vstup(&format!("{prefix}{i}"))).collect()
}

/// Negace všech signálů.
///
/// [en] Negation of all signals.
fn negace(b: &mut Sestavovac, sigs: &[Sig]) -> Vec<Sig> {
    sigs.iter().map(|&s| b.hradlo(Not, &[s])).collect()
}

/// Součin signálů; jediný signál zůstane bez hradla.
///
/// [en] AND of the signals; a single signal stays without a gate.
fn soucin(b: &mut Sestavovac, sigs: &[Sig]) -> Sig {
    if sigs.len() == 1 {
        sigs[0]
    } else {
        b.hradlo(And, sigs)
    }
}

/// Součet signálů; jediný signál zůstane bez hradla.
///
/// [en] OR of the signals; a single signal stays without a gate.
fn soucet(b: &mut Sestavovac, sigs: &[Sig]) -> Sig {
    if sigs.len() == 1 {
        sigs[0]
    } else {
        b.hradlo(Or, sigs)
    }
}

/// Signály adresy pro minterm `k`: přímý bit, kde je v `k` jednička, jinak negovaný.
///
/// [en] Address signals for minterm `k`: the direct bit where `k` has a one, otherwise the negated one.
fn minterm(k: usize, a: &[Sig], na: &[Sig]) -> Vec<Sig> {
    (0..a.len())
        .map(|i| if k >> i & 1 == 1 { a[i] } else { na[i] })
        .collect()
}

/// „1 bit“, „4bitový“ / „4bitová“ podle rodu.
///
/// [en] “1 bit”, “4bitový” / “4bitová” by grammatical gender (Czech).
fn bitovy(n: u8, zensky: bool) -> String {
    if zensky {
        format!("{n}bitová")
    } else {
        format!("{n}bitový")
    }
}

/// Český a anglický název bloku (stejně jako v Knihovně součástek, čítač se směrem).
///
/// [en] Czech and English name of the block (as in the component library, the counter with its direction).
fn nazev(kind: BlockKind) -> (String, String) {
    match kind {
        BlockKind::Mux { select } => (
            format!("Multiplexor {}:1", 1 << select),
            format!("Multiplexer {}:1", 1 << select),
        ),
        BlockKind::Decoder { bits } => (
            format!("Dekodér (demultiplexor) {bits}:{}", 1 << bits),
            format!("Decoder (demultiplexer) {bits}:{}", 1 << bits),
        ),
        BlockKind::Encoder { bits } => (
            format!("Prioritní kodér {}:{bits}", 1 << bits),
            format!("Priority encoder {}:{bits}", 1 << bits),
        ),
        BlockKind::Adder { bits: 1 } => (
            "Úplná sčítačka (1 bit)".to_owned(),
            "Full adder (1 bit)".to_owned(),
        ),
        BlockKind::Adder { bits } => (
            format!("Sčítačka {}", bitovy(bits, true)),
            format!("Adder {bits}-bit"),
        ),
        BlockKind::Comparator { bits } => (
            format!("Komparátor {}", bitovy(bits, false)),
            format!("Comparator {bits}-bit"),
        ),
        BlockKind::BcdTo7Seg => (
            "Převodník BCD na 7 segmentů".to_owned(),
            "BCD to 7-segment decoder".to_owned(),
        ),
        BlockKind::Counter { bits, down } => (
            format!(
                "Čítač {} {}",
                bitovy(bits, false),
                if down { "dolů" } else { "nahoru" }
            ),
            format!("Counter {bits}-bit {}", if down { "down" } else { "up" }),
        ),
        BlockKind::Register { bits } => (
            format!("Registr {}", bitovy(bits, false)),
            format!("Register {bits}-bit"),
        ),
        BlockKind::ShiftRegister { bits } => (
            format!("Posuvný registr {}", bitovy(bits, false)),
            format!("Shift register {bits}-bit"),
        ),
    }
}

/// Upozornění na rozdíl proti bloku u nepřipojených řídicích vstupů.
///
/// [en] Note on the difference from the block for unconnected control inputs.
fn pozor_na(vstupy: &str, en_vstupy: &str, hodnoty: &str, en_hodnoty: &str) -> (String, String) {
    (
        format!(
            " Na rozdíl od bloku je třeba {vstupy} připojit – blok bere nepřipojený {hodnoty}, \
             zapojení z hradel dostane neznámou hodnotu X."
        ),
        format!(
            " Unlike the block, {en_vstupy} must be connected – the block treats an unconnected \
             {en_hodnoty}, the gate-level circuit gets the unknown value X."
        ),
    )
}

/// Upozornění u sekvenčních bloků česky: po spuštění neznámý stav, dokud nepřijde R.
///
/// [en] Note for sequential blocks in Czech: unknown state after start until R comes.
const SEKV_CS: &str = " Blok začíná ve stavu 0; zapojení z hradel je po spuštění simulace v neznámém \
    stavu X (jako skutečné obvody po zapnutí), dokud ho R = 1 nevynuluje.";

/// Totéž upozornění anglicky.
///
/// [en] The same note in English.
const SEKV_EN: &str = " The block starts in state 0; the gate-level circuit is in the unknown state X \
    after the simulation starts (like real chips after power-up) until R = 1 resets it.";

/// Popis prvku bloku česky a anglicky.
///
/// [en] Description of a block item in Czech and English.
fn popis(kind: BlockKind) -> (String, String) {
    let (mut cs, mut en) = match kind {
        BlockKind::Mux { select } => {
            let m = 1 << select;
            (
                format!(
                    "Vnitřní zapojení bloku MUX: každý datový vstup D0–D{} má hradlo AND, které \
                     ho propustí jen při své adrese na S0–S{} (přímé a negované adresové bity) \
                     a při EN = 1; výstupy AND sčítá hradlo OR. Y = D(S) · EN.",
                    m - 1,
                    select - 1
                ),
                format!(
                    "Internal circuit of the MUX block: every data input D0–D{} has an AND gate \
                     that passes it only at its address on S0–S{} (direct and negated address \
                     bits) and with EN = 1; an OR gate adds the AND outputs. Y = D(S) · EN.",
                    m - 1,
                    select - 1
                ),
            )
        }
        BlockKind::Decoder { bits } => (
            format!(
                "Vnitřní zapojení bloku X/Y: každý výstup Y0–Y{} je hradlo AND ze vstupu EN a \
                 adresových bitů A0–A{} (přímých nebo negovaných), takže svítí jen výstup s číslem \
                 adresy. EN = 0 vypne všechny výstupy (demultiplexor: EN jsou data).",
                (1 << bits) - 1,
                bits - 1
            ),
            format!(
                "Internal circuit of the X/Y block: every output Y0–Y{} is an AND gate of the \
                 EN input and the address bits A0–A{} (direct or negated), so only the output \
                 with the address number is active. EN = 0 switches all outputs off \
                 (demultiplexer: EN is the data).",
                (1 << bits) - 1,
                bits - 1
            ),
        ),
        BlockKind::Encoder { bits } => (
            format!(
                "Vnitřní zapojení bloku HPRI: výstup A{} až A0 je číslo nejvyššího aktivního \
                 vstupu I0–I{}. Bit Aj je součet vstupů, jejichž číslo má j-tý bit 1, přičemž \
                 každý z nich se uplatní jen tehdy, když není aktivní žádný vyšší vstup s j-tým \
                 bitem 0. V = 1, když je aktivní aspoň jeden vstup.",
                bits - 1,
                (1 << bits) - 1
            ),
            format!(
                "Internal circuit of the HPRI block: the outputs A{} to A0 give the number of the \
                 highest active input I0–I{}. Bit Aj is the OR of the inputs whose number has bit j \
                 set, each of them counting only when no higher input with bit j equal to 0 is \
                 active. V = 1 when at least one input is active.",
                bits - 1,
                (1 << bits) - 1
            ),
        ),
        BlockKind::Adder { bits: 1 } => (
            "Vnitřní zapojení bloku Σ pro 1 bit: S0 = A0 ⊕ B0 ⊕ CI, \
             CO = A0·B0 + CI·(A0 ⊕ B0). Ze dvou hradel XOR, dvou AND a jednoho OR."
                .to_owned(),
            "Internal circuit of the Σ block for 1 bit: S0 = A0 ⊕ B0 ⊕ CI, \
             CO = A0·B0 + CI·(A0 ⊕ B0). Made of two XOR gates, two AND gates and one OR gate."
                .to_owned(),
        ),
        BlockKind::Adder { bits } => (
            format!(
                "Vnitřní zapojení bloku Σ: {bits} úplné sčítačky (1 bit) za sebou, přenos CO \
                 každého řádu vede do CI vyššího (sčítačka s postupným přenosem). Tabem jde otevřít \
                 úplnou sčítačku až na hradla."
            ),
            format!(
                "Internal circuit of the Σ block: {bits} full adders (1 bit) in a row, the carry \
                 CO of every position goes to CI of the next one (ripple-carry adder). Tab opens \
                 the full adder down to gates."
            ),
        ),
        BlockKind::Comparator { bits } => (
            format!(
                "Vnitřní zapojení bloku COMP: pro každý bit hradlo XNOR (Ai = Bi) a AND \
                 (Ai · ¬Bi). A>B rozhodne nejvyšší bit, ve kterém se čísla liší (porovnává se od \
                 A{0}/B{0} dolů), A=B je součin všech XNOR a A<B = ¬(A>B + A=B).",
                bits - 1
            ),
            format!(
                "Internal circuit of the COMP block: for every bit an XNOR gate (Ai = Bi) and an \
                 AND gate (Ai · ¬Bi). A>B is decided by the highest bit in which the numbers differ \
                 (compared from A{0}/B{0} downwards), A=B is the AND of all XNORs and \
                 A<B = ¬(A>B + A=B).",
                bits - 1
            ),
        ),
        BlockKind::BcdTo7Seg => (
            "Vnitřní zapojení bloku BCD/7SEG pro číslice 0–F: každý segment a–g je \
             minimalizovaný součet součinů (Quine–McCluskey) ze vstupů D0–D3 a jejich negací; \
             stejné součiny sdílí více segmentů."
                .to_owned(),
            "Internal circuit of the BCD/7SEG block for digits 0–F: every segment a–g is a \
             minimised sum of products (Quine–McCluskey) of the inputs D0–D3 and their \
             negations; identical products are shared by several segments."
                .to_owned(),
        ),
        BlockKind::Counter { bits, down } => {
            let (smer, dir, q) = if down {
                ("dolů", "down", "Qn")
            } else {
                ("nahoru", "up", "Q")
            };
            (
                format!(
                    "Vnitřní zapojení bloku CTR{bits} ({smer}): synchronní čítač z {bits} klopných \
                     obvodů T se společnými hodinami C a nulováním R. Stupeň i se překlopí, když \
                     EN = 1 a všechny nižší stupně mají {q} = 1 (hradla AND s paralelním přenosem). \
                     CO = 1 ve stavu {}. Klopný obvod T je klopný obvod D s hradlem XOR, ten je \
                     ze šesti hradel NAND – Tabem jde až na hradla.",
                    if down { "0" } else { "samých jedniček" }
                ),
                format!(
                    "Internal circuit of the CTR{bits} block ({dir}): a synchronous counter of \
                     {bits} T flip-flops with a common clock C and reset R. Stage i toggles when \
                     EN = 1 and all lower stages have {q} = 1 (AND gates with parallel carry). \
                     CO = 1 in the state {}. The T flip-flop is a D flip-flop with an XOR gate, \
                     which is made of six NAND gates – Tab goes down to gates.",
                    if down { "0" } else { "of all ones" }
                ),
            )
        }
        BlockKind::Register { bits } => (
            format!(
                "Vnitřní zapojení bloku RG{bits}: {bits} klopných obvodů D se společnými hodinami \
                 C a nulováním R; náběžná hrana C zapíše D0–D{} do Q0–Q{}. Klopný obvod D je ze \
                 šesti hradel NAND (jako 7474).",
                bits - 1,
                bits - 1
            ),
            format!(
                "Internal circuit of the RG{bits} block: {bits} D flip-flops with a common clock \
                 C and reset R; the rising edge of C stores D0–D{} into Q0–Q{}. The D flip-flop \
                 is made of six NAND gates (like the 7474).",
                bits - 1,
                bits - 1
            ),
        ),
        BlockKind::ShiftRegister { bits } => (
            format!(
                "Vnitřní zapojení bloku SRG{bits}: {bits} klopných obvodů D za sebou se \
                 společnými hodinami C a nulováním R; při náběžné hraně C se SI zapíše do Q0 a \
                 každý Qi se posune do Qi+1. Klopný obvod D je ze šesti hradel NAND."
            ),
            format!(
                "Internal circuit of the SRG{bits} block: {bits} D flip-flops in a row with a \
                 common clock C and reset R; on the rising edge of C, SI is stored into Q0 and \
                 every Qi moves to Qi+1. The D flip-flop is made of six NAND gates."
            ),
        ),
    };
    let pozn = match kind {
        BlockKind::Mux { .. } | BlockKind::Decoder { .. } => {
            Some(pozor_na("EN", "EN", "EN jako 1", "EN as 1"))
        }
        BlockKind::Counter { .. } => Some(pozor_na(
            "R a EN",
            "R and EN",
            "R jako 0 a EN jako 1",
            "R as 0 and EN as 1",
        )),
        BlockKind::Register { .. } | BlockKind::ShiftRegister { .. } => {
            Some(pozor_na("R", "R", "R jako 0", "R as 0"))
        }
        _ => None,
    };
    if let Some((c, e)) = pozn {
        cs += &c;
        en += &e;
    }
    if kind.is_sequential() {
        cs += SEKV_CS;
        en += SEKV_EN;
    }
    (cs, en)
}

/// Kategorie prvku bloku.
///
/// [en] Category of a block item.
fn kategorie(kind: BlockKind) -> &'static str {
    match kind {
        BlockKind::Mux { .. } | BlockKind::Decoder { .. } | BlockKind::Encoder { .. } => {
            "combinational"
        }
        BlockKind::Adder { .. } | BlockKind::Comparator { .. } => "arithmetic",
        BlockKind::BcdTo7Seg => "code-converters",
        BlockKind::Register { .. } => "memory",
        BlockKind::Counter { .. } | BlockKind::ShiftRegister { .. } => "sequential",
    }
}

/// Přímé implikanty a pokrytí pro funkci 4 proměnných (Quine–McCluskey).
/// Implikant je (hodnota, maska): bity v masce jsou „cokoli“.
///
/// [en] Prime implicants and a cover for a function of 4 variables (Quine–McCluskey).
/// An implicant is (value, mask): bits in the mask are “don't care”.
fn minimalizuj(minterms: &[u32]) -> Vec<(u32, u32)> {
    let mut level: Vec<(u32, u32)> = minterms.iter().map(|&m| (m, 0)).collect();
    let mut primes: Vec<(u32, u32)> = Vec::new();
    while !level.is_empty() {
        let mut used = vec![false; level.len()];
        let mut next: Vec<(u32, u32)> = Vec::new();
        for i in 0..level.len() {
            for j in i + 1..level.len() {
                let ((v1, m1), (v2, m2)) = (level[i], level[j]);
                let diff = v1 ^ v2;
                if m1 == m2 && diff.count_ones() == 1 {
                    used[i] = true;
                    used[j] = true;
                    let t = (v1 & !diff, m1 | diff);
                    if !next.contains(&t) {
                        next.push(t);
                    }
                }
            }
        }
        for (i, &t) in level.iter().enumerate() {
            if !used[i] && !primes.contains(&t) {
                primes.push(t);
            }
        }
        level = next;
    }
    let covers = |(v, m): (u32, u32), x: u32| x & !m == v;
    // Nejdřív podstatné implikanty, pak hladově ten, který pokryje nejvíc zbylých.
    // [en] Essential implicants first, then greedily the one covering most of the rest.
    let mut chosen: Vec<(u32, u32)> = Vec::new();
    for &x in minterms {
        let c: Vec<_> = primes.iter().filter(|&&p| covers(p, x)).collect();
        if c.len() == 1 && !chosen.contains(c[0]) {
            chosen.push(*c[0]);
        }
    }
    loop {
        let left: Vec<u32> = minterms
            .iter()
            .copied()
            .filter(|&x| !chosen.iter().any(|&p| covers(p, x)))
            .collect();
        if left.is_empty() {
            break;
        }
        let best = *primes
            .iter()
            .max_by_key(|&&p| {
                (
                    left.iter().filter(|&&x| covers(p, x)).count(),
                    p.1.count_ones(),
                    std::cmp::Reverse(p),
                )
            })
            .expect("implikant");
        chosen.push(best);
    }
    chosen.sort_by_key(|&(v, m)| (std::cmp::Reverse(m.count_ones()), v, m));
    chosen
}

/// Klopný obvod D řízený náběžnou hranou s asynchronním nulováním ze šesti hradel
/// NAND (zapojení 7474 bez nastavení).
///
/// [en] Rising-edge D flip-flop with asynchronous reset from six NAND gates (the 7474
/// circuit without preset).
fn klopny_d(k: &mut Kn) -> SubcircuitDef {
    let mut b = Sestavovac::new();
    let c = b.vstup("C");
    let d = b.vstup("D");
    let r = b.vstup("R");
    let rn = b.hradlo(Not, &[r]);
    let n1 = b.hradlo_bez(2, Nand, 2);
    let n4 = b.hradlo_bez(2, Nand, 3);
    let n2 = b.hradlo_bez(3, Nand, 3);
    let n3 = b.hradlo_bez(3, Nand, 3);
    let n5 = b.hradlo_bez(4, Nand, 2);
    let n6 = b.hradlo_bez(4, Nand, 3);
    let [s1, s4, s2, s3, q, qn] = [n1, n4, n2, n3, n5, n6].map(|p| b.vystupy(p)[0]);
    for (part, ins) in [
        (n1, vec![s4, s2]),
        (n4, vec![s3, rn, d]),
        (n2, vec![s1, rn, c]),
        (n3, vec![s2, c, s4]),
        (n5, vec![s2, qn]),
        (n6, vec![q, s3, rn]),
    ] {
        for (i, s) in ins.into_iter().enumerate() {
            b.spoj(part, i, s);
        }
    }
    b.vystup("Q", q);
    b.vystup("Qn", qn);
    k.pridej(
        Popis {
            uid: UID_D.to_owned(),
            cs: "Klopný obvod D s nulováním (z NAND)".to_owned(),
            en: "D flip-flop with reset (from NAND)".to_owned(),
            kat: "sequential",
            popis_cs: "Klopný obvod D řízený náběžnou hranou C s asynchronním nulováním R \
                (aktivní v 1) ze šesti hradel NAND – zapojení obvodu 7474. Dvě vstupní paměti \
                RS si při C = 0 připraví D; náběžná hrana C podle něj nastaví nebo vynuluje \
                výstupní paměť a dokud je C = 1, změny D už neprojdou. R = 1 nuluje hned, bez \
                ohledu na hodiny. Po spuštění simulace je stav neznámý (X), dokud nepřijde \
                hrana C nebo R."
                .to_owned(),
            popis_en: "Rising-edge triggered D flip-flop with asynchronous reset R (active \
                high) from six NAND gates – the circuit of the 7474. Two input RS latches prepare \
                D while C = 0; the rising edge of C sets or resets the output latch accordingly \
                and while C = 1 changes of D no longer pass. R = 1 resets at once, regardless of \
                the clock. After the simulation starts the state is unknown (X) until an edge of \
                C or R comes."
                .to_owned(),
        },
        b,
    )
}

/// Klopný obvod T s nulováním: klopný obvod D a hradlo XOR (D = T ⊕ Q).
///
/// [en] T flip-flop with reset: a D flip-flop and an XOR gate (D = T ⊕ Q).
fn klopny_t(k: &mut Kn, dff: &SubcircuitDef) -> SubcircuitDef {
    let mut b = Sestavovac::new();
    let c = b.vstup("C");
    let r = b.vstup("R");
    let t = b.vstup("T");
    let x = b.hradlo_bez(1, Xor, 2);
    let xs = b.vystupy(x)[0];
    let o = b.podobvod(dff, &[c, xs, r]);
    b.spoj(x, 0, t);
    b.spoj(x, 1, o[0]);
    b.vystup("Q", o[0]);
    b.vystup("Qn", o[1]);
    k.pridej(
        Popis {
            uid: UID_T.to_owned(),
            cs: "Klopný obvod T s nulováním (z D)".to_owned(),
            en: "T flip-flop with reset (from D)".to_owned(),
            kat: "sequential",
            popis_cs: "Klopný obvod T řízený náběžnou hranou C s asynchronním nulováním R: \
                klopný obvod D, na jehož vstup vede T ⊕ Q. Při T = 1 se hranou překlopí, při \
                T = 0 drží stav. Tabem jde otevřít klopný obvod D až na hradla NAND."
                .to_owned(),
            popis_en: "Rising-edge triggered T flip-flop with asynchronous reset R: a D \
                flip-flop whose input gets T ⊕ Q. With T = 1 it toggles on the edge, with T = 0 it \
                holds its state. Tab opens the D flip-flop down to NAND gates."
                .to_owned(),
        },
        b,
    )
}

/// Zapojení bloku z hradel (a pomocných prvků).
///
/// [en] Gate-level circuit of the block (and helper items).
fn zapojeni(
    kind: BlockKind,
    dff: &SubcircuitDef,
    tff: &SubcircuitDef,
    fa: Option<&SubcircuitDef>,
) -> Sestavovac {
    let mut b = Sestavovac::new();
    match kind {
        BlockKind::Mux { select } => {
            let s = vstupy(&mut b, "S", select as usize);
            let d = vstupy(&mut b, "D", 1 << select);
            let en = b.vstup("EN");
            let ns = negace(&mut b, &s);
            let terms: Vec<Sig> = (0..1 << select)
                .map(|k| {
                    let mut ins = vec![d[k], en];
                    ins.extend(minterm(k, &s, &ns));
                    b.hradlo(And, &ins)
                })
                .collect();
            let y = soucet(&mut b, &terms);
            b.vystup("Y", y);
        }
        BlockKind::Decoder { bits } => {
            let a = vstupy(&mut b, "A", bits as usize);
            let en = b.vstup("EN");
            let na = negace(&mut b, &a);
            for k in 0..1 << bits {
                let mut ins = vec![en];
                ins.extend(minterm(k, &a, &na));
                let y = b.hradlo(And, &ins);
                b.vystup(&format!("Y{k}"), y);
            }
        }
        BlockKind::Encoder { bits } => {
            let m = 1usize << bits;
            let i = vstupy(&mut b, "I", m);
            // Negace vstupů, které některý člen potřebuje.
            // [en] Negations of the inputs needed by some term.
            let mut ni: BTreeMap<usize, Sig> = BTreeMap::new();
            for j in 0..bits as usize {
                for k in (0..m).filter(|k| k >> j & 1 == 1) {
                    for h in (k + 1..m).filter(|h| h >> j & 1 == 0) {
                        ni.entry(h).or_insert_with(|| b.hradlo(Not, &[i[h]]));
                    }
                }
            }
            for j in 0..bits as usize {
                let terms: Vec<Sig> = (0..m)
                    .filter(|k| k >> j & 1 == 1)
                    .map(|k| {
                        let mut ins = vec![i[k]];
                        ins.extend((k + 1..m).filter(|h| h >> j & 1 == 0).map(|h| ni[&h]));
                        soucin(&mut b, &ins)
                    })
                    .collect();
                let a = soucet(&mut b, &terms);
                b.vystup(&format!("A{j}"), a);
            }
            let v = b.hradlo(Or, &i);
            b.vystup("V", v);
        }
        BlockKind::Adder { bits: 1 } => {
            let a = b.vstup("A0");
            let bb = b.vstup("B0");
            let ci = b.vstup("CI");
            let x = b.hradlo(Xor, &[a, bb]);
            let s = b.hradlo(Xor, &[x, ci]);
            let g = b.hradlo(And, &[a, bb]);
            let p = b.hradlo(And, &[ci, x]);
            let co = b.hradlo(Or, &[g, p]);
            b.vystup("S0", s);
            b.vystup("CO", co);
        }
        BlockKind::Adder { bits } => {
            let fa = fa.expect("úplná sčítačka");
            let a = vstupy(&mut b, "A", bits as usize);
            let bb = vstupy(&mut b, "B", bits as usize);
            let mut c = b.vstup("CI");
            for i in 0..bits as usize {
                // Rozhraní úplné sčítačky: A0, B0, CI → CO, S0.
                // [en] Full adder interface: A0, B0, CI → CO, S0.
                let o = b.podobvod(fa, &[a[i], bb[i], c]);
                b.vystup(&format!("S{i}"), o[1]);
                c = o[0];
            }
            b.vystup("CO", c);
        }
        BlockKind::Comparator { bits } => {
            let n = bits as usize;
            let a = vstupy(&mut b, "A", n);
            let bb = vstupy(&mut b, "B", n);
            let nb = negace(&mut b, &bb);
            let e: Vec<Sig> = (0..n).map(|i| b.hradlo(Xnor, &[a[i], bb[i]])).collect();
            let g: Vec<Sig> = (0..n).map(|i| b.hradlo(And, &[a[i], nb[i]])).collect();
            // A>B: nejvyšší bit, ve kterém je A = 1 a B = 0, a všechny vyšší stejné.
            // [en] A>B: the highest bit where A = 1 and B = 0, with all higher bits equal.
            let terms: Vec<Sig> = (0..n)
                .rev()
                .map(|i| {
                    let mut ins: Vec<Sig> = e[i + 1..].to_vec();
                    ins.push(g[i]);
                    soucin(&mut b, &ins)
                })
                .collect();
            let gt = soucet(&mut b, &terms);
            let eq = soucin(&mut b, &e);
            let lt = b.hradlo(Nor, &[gt, eq]);
            b.vystup("A<B", lt);
            b.vystup("A=B", eq);
            b.vystup("A>B", gt);
        }
        BlockKind::BcdTo7Seg => {
            let d = vstupy(&mut b, "D", 4);
            let nd = negace(&mut b, &d);
            let mut shared: BTreeMap<(u32, u32), Sig> = BTreeMap::new();
            let mut segs = Vec::new();
            for seg in 0..7 {
                let minterms: Vec<u32> = (0..16u32)
                    .filter(|&x| SEVEN_SEG_HEX[x as usize] >> seg & 1 == 1)
                    .collect();
                let terms: Vec<Sig> = minimalizuj(&minterms)
                    .into_iter()
                    .map(|(v, m)| {
                        *shared.entry((v, m)).or_insert_with(|| {
                            let lits: Vec<Sig> = (0..4)
                                .filter(|i| m >> i & 1 == 0)
                                .map(|i| if v >> i & 1 == 1 { d[i] } else { nd[i] })
                                .collect();
                            soucin(&mut b, &lits)
                        })
                    })
                    .collect();
                segs.push(terms);
            }
            for (seg, terms) in segs.iter().enumerate() {
                let y = soucet(&mut b, terms);
                b.vystup(&((b'a' + seg as u8) as char).to_string(), y);
            }
        }
        BlockKind::Counter { bits, down } => {
            let n = bits as usize;
            let c = b.vstup("C");
            let en = b.vstup("EN");
            let r = b.vstup("R");
            let ffs: Vec<_> = (0..n).map(|_| b.podobvod_bez(1, tff)).collect();
            let q: Vec<Sig> = ffs.iter().map(|&f| b.vystupy(f)[0]).collect();
            let qn: Vec<Sig> = ffs.iter().map(|&f| b.vystupy(f)[1]).collect();
            let x = if down { &qn } else { &q };
            for (i, &ff) in ffs.iter().enumerate() {
                // Překlopí se, když EN = 1 a všechny nižší stupně jsou na konci rozsahu.
                // [en] It toggles when EN = 1 and all lower stages are at the end of the range.
                let t = if i == 0 {
                    en
                } else {
                    let mut ins = vec![en];
                    ins.extend(&x[..i]);
                    b.hradlo_ve(2, And, &ins)
                };
                // Rozhraní klopného obvodu T: C, R, T → Q, Qn.
                // [en] T flip-flop interface: C, R, T → Q, Qn.
                b.spoj(ff, 0, c);
                b.spoj(ff, 1, r);
                b.spoj(ff, 2, t);
            }
            let co = b.hradlo_ve(2, And, x);
            for (i, &s) in q.iter().enumerate() {
                b.vystup(&format!("Q{i}"), s);
            }
            b.vystup("CO", co);
        }
        BlockKind::Register { bits } => {
            let d = vstupy(&mut b, "D", bits as usize);
            let r = b.vstup("R");
            let c = b.vstup("C");
            for (i, &di) in d.iter().enumerate() {
                // Rozhraní klopného obvodu D: C, D, R → Q, Qn.
                // [en] D flip-flop interface: C, D, R → Q, Qn.
                let o = b.podobvod(dff, &[c, di, r]);
                b.vystup(&format!("Q{i}"), o[0]);
            }
        }
        BlockKind::ShiftRegister { bits } => {
            let si = b.vstup("SI");
            let r = b.vstup("R");
            let c = b.vstup("C");
            let mut d = si;
            for i in 0..bits as usize {
                let o = b.podobvod(dff, &[c, d, r]);
                b.vystup(&format!("Q{i}"), o[0]);
                d = o[0];
            }
        }
    }
    b
}

/// Sestaví celou knihovnu.
///
/// [en] Builds the whole library.
fn knihovna() -> Library {
    let mut info = LibraryInfo::new(
        BLOCK_LIBRARY_UID.to_owned(),
        "Funkční bloky – vnitřní zapojení",
        AUTOR,
        DATUM,
    );
    info.description = "Vnitřní zapojení všech funkčních bloků programu (multiplexor, dekodér, \
        kodér, sčítačka, komparátor, převodník BCD na 7 segmentů, čítač, registr, posuvný registr) \
        z hradel, se stejně pojmenovanými vývody. Sekvenční bloky jsou z klopných obvodů D ze \
        šesti hradel NAND, takže jde každý blok rozkliknout Tabem až na hradla. Každý prvek je \
        ověřený proti simulaci bloku."
        .to_owned();
    info.translations.insert(
        "en".to_owned(),
        Translation {
            name: "Function blocks – internal circuits".to_owned(),
            description: "Gate-level internal circuits of all function blocks of the program \
                (multiplexer, decoder, encoder, adder, comparator, BCD to 7-segment decoder, \
                counter, register, shift register) with pins of the same names. Sequential \
                blocks are made of D flip-flops from six NAND gates, so every block can be opened \
                with Tab down to gates. Every item is verified against the simulation of the \
                block."
                .to_owned(),
        },
    );
    let mut k = Kn {
        lib: Library::new(info),
    };
    let dff = klopny_d(&mut k);
    let tff = klopny_t(&mut k, &dff);
    let mut fa: Option<SubcircuitDef> = None;
    for kind in BlockKind::variants() {
        let b = zapojeni(kind, &dff, &tff, fa.as_ref());
        let (cs, en) = nazev(kind);
        let (popis_cs, popis_en) = popis(kind);
        let def = k.pridej(
            Popis {
                uid: kind.library_uid(),
                cs,
                en,
                kat: kategorie(kind),
                popis_cs,
                popis_en,
            },
            b,
        );
        if kind == (BlockKind::Adder { bits: 1 }) {
            fa = Some(def);
        }
    }
    k.lib
}

// --- Pomocné funkce testů ----------------------------------------------------
// [en] --- Test helpers ----------------------------------------------------------

/// Simulace prvku a bloku vedle sebe: vstupy prvku podle názvů vývodů bloku.
///
/// [en] Simulation of the item and the block side by side: item inputs by the block's pin names.
struct Srovnani {
    /// Blok.
    ///
    /// [en] The block.
    kind: BlockKind,
    /// Simulátor prvku.
    ///
    /// [en] The item's simulator.
    sim: Simulator,
    /// Id přepínačů prvku v pořadí vstupů bloku.
    ///
    /// [en] Ids of the item's switches in the order of the block's inputs.
    ins: Vec<CompId>,
    /// Id LED prvku v pořadí výstupů bloku.
    ///
    /// [en] Ids of the item's LEDs in the order of the block's outputs.
    outs: Vec<CompId>,
    /// Aktuální hodnoty vstupů (v pořadí bloku).
    ///
    /// [en] Current input values (in block order).
    values: Vec<bool>,
    /// Stav sekvenčního bloku.
    ///
    /// [en] State of a sequential block.
    state: Vec<LogicValue>,
    /// Předchozí hodnota hodin bloku.
    ///
    /// [en] Previous clock value of the block.
    prev_clock: LogicValue,
}

impl Srovnani {
    /// Připraví srovnání bloku s jeho prvkem v knihovně.
    ///
    /// [en] Prepares the comparison of a block with its library item.
    fn new(lib: &Library, kind: BlockKind) -> Self {
        let item = lib.item(&kind.library_uid()).expect("prvek bloku");
        let (ins, outs) = item.circuit.io();
        let find = |list: &[&logicflow_core::Component], name: &str| {
            list.iter()
                .find(|c| c.label == name)
                .unwrap_or_else(|| panic!("{}: vývod {name} chybí / pin missing", item.name))
                .id
        };
        let in_ids = kind.input_names().iter().map(|n| find(&ins, n)).collect();
        let out_ids = kind.output_names().iter().map(|n| find(&outs, n)).collect();
        let mut s = Self {
            kind,
            sim: Simulator::with_subcircuits(&item.circuit, &lib.defs(), None),
            ins: in_ids,
            outs: out_ids,
            values: vec![false; kind.input_names().len()],
            state: vec![LogicValue::Zero; kind.state_bits()],
            prev_clock: LogicValue::X,
        };
        assert!(s.sim.run(100_000));
        if kind.is_sequential() {
            s.krok_bloku();
        }
        s
    }

    /// Nastaví vstup (v pořadí bloku) v prvku i v bloku.
    ///
    /// [en] Sets an input (in block order) in the item and in the block.
    fn nastav(&mut self, i: usize, v: bool) {
        if self.values[i] == v {
            return;
        }
        self.values[i] = v;
        self.sim.set_source(self.ins[i], v.into());
        assert!(
            self.sim.run(100_000),
            "{:?} se neustálil / did not settle",
            self.kind
        );
        if self.kind.is_sequential() {
            self.krok_bloku();
        }
    }

    /// Vstupy bloku jako logické hodnoty.
    ///
    /// [en] Block inputs as logic values.
    fn vstupy_bloku(&self) -> Vec<LogicValue> {
        self.values.iter().map(|&v| v.into()).collect()
    }

    /// Krok sekvenčního bloku po změně vstupu.
    ///
    /// [en] Step of the sequential block after an input change.
    fn krok_bloku(&mut self) {
        let ins = self.vstupy_bloku();
        self.kind.step(&ins, &mut self.state, &mut self.prev_clock);
    }

    /// Výstupy prvku (v pořadí bloku).
    ///
    /// [en] Item outputs (in block order).
    fn vystupy_prvku(&self) -> Vec<LogicValue> {
        self.outs.iter().map(|&id| self.sim.inputs(id)[0]).collect()
    }

    /// Výstupy bloku pro aktuální vstupy a stav.
    ///
    /// [en] Block outputs for the current inputs and state.
    fn vystupy_bloku(&self) -> Vec<LogicValue> {
        if self.kind.is_sequential() {
            self.kind.outputs_from_state(&self.state)
        } else {
            self.kind.eval(&self.vstupy_bloku())
        }
    }

    /// Porovná výstupy prvku a bloku.
    ///
    /// [en] Compares the outputs of the item and the block.
    fn porovnej(&self, kde: &str) {
        assert_eq!(
            self.vystupy_prvku(),
            self.vystupy_bloku(),
            "{:?} {kde}: vstupy {:?} = {:?}",
            self.kind,
            self.kind.input_names(),
            self.values
        );
    }

    /// Index vstupu bloku podle názvu.
    ///
    /// [en] Index of a block input by name.
    fn vstup(&self, name: &str) -> usize {
        self.kind
            .input_names()
            .iter()
            .position(|n| n == name)
            .expect("vstup bloku")
    }
}

/// Jednoduchý generátor pseudonáhodných čísel (opakovatelný).
///
/// [en] A simple pseudo-random number generator (repeatable).
struct Nahoda(u64);

impl Nahoda {
    /// Další číslo menší než `n`.
    ///
    /// [en] Next number below `n`.
    fn pod(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) % n as u64) as usize
    }
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
         --test funkcni_bloky / the file is outdated"
    );
}

#[test]
fn kazdy_blok_ma_prvek_se_stejnymi_vyvody() {
    let lib = knihovna();
    assert_eq!(lib.items.len(), BlockKind::variants().len() + 2);
    for kind in BlockKind::variants() {
        let item = lib.item(&kind.library_uid()).expect("prvek bloku");
        let mut names = kind.input_names();
        names.sort_by(|a, b| logicflow_core::circuit::natural_cmp(a, b));
        assert_eq!(item.inputs, names, "{kind:?}");
        let mut names = kind.output_names();
        names.sort_by(|a, b| logicflow_core::circuit::natural_cmp(a, b));
        assert_eq!(item.outputs, names, "{kind:?}");
        let en = &item.translations["en"];
        assert!(!en.name.is_empty() && !en.description.is_empty());
    }
}

#[test]
fn kombinacni_bloky_jako_blok() {
    let lib = knihovna();
    for kind in BlockKind::variants()
        .into_iter()
        .filter(|k| !k.is_sequential())
    {
        let mut s = Srovnani::new(&lib, kind);
        let n = s.values.len();
        // Grayův kód: mezi kombinacemi se mění jen jeden vstup.
        // [en] Gray code: only one input changes between combinations.
        for combo in 0..1u32 << n {
            let gray = combo ^ (combo >> 1);
            for i in 0..n {
                s.nastav(i, gray >> i & 1 == 1);
            }
            s.porovnej("kombinace / combination");
        }
    }
}

#[test]
fn sekvencni_bloky_jako_blok() {
    let lib = knihovna();
    for kind in BlockKind::variants()
        .into_iter()
        .filter(|k| k.is_sequential())
    {
        let mut s = Srovnani::new(&lib, kind);
        let (r, c) = (s.vstup("R"), s.vstup("C"));
        // EN zapnout (blok ho nepřipojený bere jako 1, tady je připojený).
        // [en] Switch EN on (the block treats it as 1 when unconnected; here it is connected).
        if let Some(en) = kind.input_names().iter().position(|n| n == "EN") {
            s.nastav(en, true);
        }
        // Po spuštění je zapojení z hradel v neznámém stavu – vynulovat.
        // [en] After start the gate-level circuit is in an unknown state – reset it.
        s.nastav(r, true);
        s.porovnej("nulování / reset");
        s.nastav(r, false);
        s.porovnej("po nulování / after reset");
        let mut rnd = Nahoda(u64::from(kind.param().unwrap_or(0)) * 7 + 1);
        let n = s.values.len();
        for krok in 0..600 {
            // Hlavně hodiny, občas jiný vstup, zřídka nulování.
            // [en] Mostly the clock, sometimes another input, rarely the reset.
            let i = match rnd.pod(20) {
                0..=10 => c,
                11 if krok % 3 == 0 => r,
                _ => {
                    let j = rnd.pod(n);
                    if j == r && rnd.pod(4) != 0 { c } else { j }
                }
            };
            let v = !s.values[i];
            s.nastav(i, v);
            s.porovnej(&format!("krok / step {krok}"));
        }
    }
}

#[test]
fn citac_obejde_vsechny_stavy() {
    let lib = knihovna();
    for (bits, down) in [(3, false), (3, true), (8, false)] {
        let kind = BlockKind::Counter { bits, down };
        let mut s = Srovnani::new(&lib, kind);
        let (r, en, c) = (s.vstup("R"), s.vstup("EN"), s.vstup("C"));
        s.nastav(en, true);
        s.nastav(r, true);
        s.nastav(r, false);
        for _ in 0..(1u32 << bits) + 3 {
            s.nastav(c, true);
            s.porovnej("hrana / edge");
            s.nastav(c, false);
        }
    }
}

#[test]
fn klopny_obvod_d_z_nand() {
    let lib = knihovna();
    let item = lib.item(UID_D).unwrap();
    let ff = item.def();
    // Rozkliknutí až na hradla: uvnitř jsou jen hradla, přepínače a LED.
    // [en] Down to gates: inside there are only gates, switches and LEDs.
    assert!(ff.circuit.components.iter().all(|c| !matches!(
        c.kind,
        logicflow_core::ComponentKind::FlipFlop { .. }
            | logicflow_core::ComponentKind::Subcircuit { .. }
            | logicflow_core::ComponentKind::Block { .. }
    )));
    let nand = ff
        .circuit
        .components
        .iter()
        .filter(|c| {
            matches!(
                c.kind,
                logicflow_core::ComponentKind::Gate {
                    gate: GateKind::Nand,
                    ..
                }
            )
        })
        .count();
    assert_eq!(nand, 6);
}

#[test]
fn zadny_prvek_nema_primitivni_klopny_obvod_ani_blok() {
    let lib = knihovna();
    for item in &lib.items {
        assert!(
            item.circuit.components.iter().all(|c| !matches!(
                c.kind,
                logicflow_core::ComponentKind::FlipFlop { .. }
                    | logicflow_core::ComponentKind::Block { .. }
            )),
            "{}",
            item.name
        );
    }
}

#[test]
fn minimalizace_pokryje_presne_funkci() {
    for seg in 0..7 {
        let minterms: Vec<u32> = (0..16u32)
            .filter(|&x| SEVEN_SEG_HEX[x as usize] >> seg & 1 == 1)
            .collect();
        let cover = minimalizuj(&minterms);
        for x in 0..16u32 {
            let on = cover.iter().any(|&(v, m)| x & !m == v);
            assert_eq!(on, minterms.contains(&x), "segment {seg}, číslo {x}");
        }
    }
}

/// Náhled: s proměnnou `LOGICFLOW_NAHLED=<složka>` zapíše každý prvek jako
/// samostatný projekt `.logicflow` (hlavní obvod = prvek). Soubory se do
/// repozitáře neukládají.
///
/// [en] Preview: with the variable `LOGICFLOW_NAHLED=<folder>` it writes every item
/// as a separate `.logicflow` project (main circuit = the item). The files are not
/// stored in the repository.
#[test]
#[allow(clippy::disallowed_methods)]
fn nahled_projekty() {
    use logicflow_core::ProjectFile;
    let Ok(dir) = std::env::var("LOGICFLOW_NAHLED") else {
        return;
    };
    let lib = knihovna();
    for (i, item) in lib.items.iter().enumerate() {
        let deps: Vec<SubcircuitDef> = lib
            .item_with_dependencies(&item.uid)
            .unwrap()
            .into_iter()
            .filter(|d| d.id != item.id)
            .collect();
        let text = ProjectFile::to_json(&item.circuit, &deps);
        std::fs::write(format!("{dir}/b{:02}.logicflow", i + 1), text).unwrap();
    }
}
