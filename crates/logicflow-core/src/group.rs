//! Vytvoření podobvodu z výběru (Ctrl+G).
//!
//! Vybrané součástky se přesunou do nového podobvodu a na jejich místo se postaví
//! jeho značka. Rozhraní vznikne samo ze sítí, které z výběru vedou ven: síť
//! buzená zevnitř je výstup (LED uvnitř), ostatní jsou vstupy (přepínač uvnitř).
//! Ovládací a zobrazovací prvky (přepínače, tlačítka, hodiny, LED, zobrazovače)
//! zůstávají venku – obvod se po zabalení ovládá a čte stejně jako předtím.
//!
//! Zapojení uvnitř zachová vodiče hraničních sítí; přepínač nebo LED rozhraní se
//! postaví do místa, kde síť venku končila na vývodu jiné součástky (tam je síť
//! jistě stejná), a slepé konce vodičů se odstraní. Venku se značka umístí
//! a každý její vývod se připojí krátkým lomeným vodičem; každý pokus se ověří
//! sestavením sítí – vodič nesmí spojit dvě různé sítě. Vývod, který nejde
//! připojit bez takového spojení, zůstane volný a výsledek to hlásí.
//!
//! [en]
//!
//! Creating a subcircuit from the selection (Ctrl+G).
//!
//! The selected components move into a new subcircuit and its symbol takes their
//! place. The interface is created from the nets that lead out of the selection: a
//! net driven from inside is an output (an LED inside), the others are inputs (a
//! switch inside). Control and display elements (switches, push buttons, clocks,
//! LEDs, displays) stay outside – after grouping the circuit is operated and read
//! the same way as before.
//!
//! The circuit inside keeps the wires of the boundary nets; the interface switch or
//! LED is placed where the net ended outside on a pin of another component (the net
//! is surely the same there), and dead-end wires are removed. Outside, the symbol is
//! placed and each of its pins is connected with a short bent wire; every attempt is
//! verified by building the nets – a wire must not join two different nets. A pin
//! that cannot be connected without such a join stays free and the result reports it.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::circuit::{MAX_SUBCIRCUIT_PINS, natural_cmp};
use crate::sim::Netlist;
use crate::{Circuit, CompId, Component, ComponentKind, LedColor, Point, Rotation};

/// Výsledek vytvoření podobvodu z výběru.
///
/// [en] Result of creating a subcircuit from the selection.
#[derive(Clone, Debug)]
pub struct Grouping {
    /// Zapojení nového podobvodu.
    ///
    /// [en] Circuit of the new subcircuit.
    pub inner: Circuit,
    /// Původní obvod, ve kterém výběr nahradila instance podobvodu.
    ///
    /// [en] The original circuit in which an instance of the subcircuit replaced the selection.
    pub outer: Circuit,
    /// Id nové instance ve vnějším obvodu.
    ///
    /// [en] Id of the new instance in the outer circuit.
    pub instance: CompId,
    /// Počet vstupů podobvodu.
    ///
    /// [en] Number of the subcircuit's inputs.
    pub inputs: usize,
    /// Počet výstupů podobvodu.
    ///
    /// [en] Number of the subcircuit's outputs.
    pub outputs: usize,
    /// Kolik vývodů instance nešlo připojit (zůstaly volné).
    ///
    /// [en] How many pins of the instance could not be connected (they stay free).
    pub unconnected: usize,
}

/// Proč z výběru nejde vytvořit podobvod.
///
/// [en] Why no subcircuit can be created from the selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupError {
    /// Ve výběru nejsou žádné součástky kromě ovládacích a zobrazovacích prvků.
    ///
    /// [en] The selection has no components apart from control and display elements.
    Empty,
    /// Rozhraní by mělo víc než 32 vstupů nebo výstupů.
    ///
    /// [en] The interface would have more than 32 inputs or outputs.
    TooManyPins,
}

/// Zůstává součástka při vytváření podobvodu venku? (Ovládací a zobrazovací prvky.)
///
/// [en] Does the component stay outside when creating a subcircuit? (Control and display elements.)
pub fn stays_outside(kind: &ComponentKind) -> bool {
    kind.is_input() || kind.is_display() || matches!(kind, ComponentKind::Clock { .. })
}

/// Vývod součástky v obvodu: index součástky, poloha, je to výstup?
///
/// [en] A component pin in the circuit: component index, position, is it an output?
#[derive(Clone, Copy, Debug)]
struct Pin {
    /// Index součástky v `Circuit::components`.
    ///
    /// [en] Index of the component in `Circuit::components`.
    comp: usize,
    /// Poloha vývodu.
    ///
    /// [en] Position of the pin.
    at: Point,
    /// Výstup (jinak vstup).
    ///
    /// [en] Output (otherwise input).
    output: bool,
}

/// Síť, která vede z výběru ven, a její vývody uvnitř a venku.
///
/// [en] A net leading out of the selection and its pins inside and outside.
#[derive(Clone, Debug)]
struct Boundary {
    /// Číslo sítě v netlistu původního obvodu.
    ///
    /// [en] Net number in the netlist of the original circuit.
    net: usize,
    /// Výstup podobvodu (síť buzená zevnitř), jinak vstup.
    ///
    /// [en] Output of the subcircuit (net driven from inside), otherwise an input.
    output: bool,
    /// Vývody ostatních součástek.
    ///
    /// [en] Pins of the other components.
    outer: Vec<Pin>,
    /// Popisek vývodu podobvodu.
    ///
    /// [en] Label of the subcircuit pin.
    label: String,
}

/// Vytvoří z vybraných součástek podobvod s id `def_id` (viz popis modulu).
///
/// [en] Creates a subcircuit with id `def_id` from the selected components (see the module description).
pub fn group(circuit: &Circuit, selected: &[CompId], def_id: u32) -> Result<Grouping, GroupError> {
    let inside: BTreeSet<CompId> = circuit
        .components
        .iter()
        .filter(|c| selected.contains(&c.id) && !stays_outside(&c.kind))
        .map(|c| c.id)
        .collect();
    if inside.is_empty() {
        return Err(GroupError::Empty);
    }
    let netlist = Netlist::build(circuit);
    let mut boundaries = boundary_nets(circuit, &netlist, &inside);
    label_boundaries(circuit, &mut boundaries);
    let (mut ins, mut outs): (Vec<Boundary>, Vec<Boundary>) =
        boundaries.into_iter().partition(|b| !b.output);
    if ins.len() > MAX_SUBCIRCUIT_PINS || outs.len() > MAX_SUBCIRCUIT_PINS {
        return Err(GroupError::TooManyPins);
    }
    // Pořadí vývodů na značce = přirozené pořadí popisků (jako `Circuit::io`).
    // [en] Pin order on the symbol = natural order of the labels (like `Circuit::io`).
    ins.sort_by(|a, b| natural_cmp(&a.label, &b.label));
    outs.sort_by(|a, b| natural_cmp(&a.label, &b.label));

    let inner = build_inner(circuit, &netlist, &inside, &ins, &outs);
    let (outer, instance, unconnected) =
        build_outer(circuit, &netlist, &inside, &ins, &outs, def_id);
    Ok(Grouping {
        inner,
        outer,
        instance,
        inputs: ins.len(),
        outputs: outs.len(),
        unconnected,
    })
}

/// Všechny vývody součástky s jejich sítěmi: (vývod, síť).
///
/// [en] All pins of a component with their nets: (pin, net).
fn pins_with_nets(circuit: &Circuit, netlist: &Netlist, i: usize) -> Vec<(Pin, usize)> {
    let c = &circuit.components[i];
    let ins = c.input_pins().into_iter().zip(&netlist.input_nets[i]);
    let outs = c.output_pins().into_iter().zip(&netlist.output_nets[i]);
    ins.map(|(at, &n)| {
        (
            Pin {
                comp: i,
                at,
                output: false,
            },
            n,
        )
    })
    .chain(outs.map(|(at, &n)| {
        (
            Pin {
                comp: i,
                at,
                output: true,
            },
            n,
        )
    }))
    .collect()
}

/// Sítě, které spojují vybrané součástky s ostatními.
///
/// [en] Nets that connect the selected components with the others.
fn boundary_nets(circuit: &Circuit, netlist: &Netlist, inside: &BTreeSet<CompId>) -> Vec<Boundary> {
    let mut by_net: BTreeMap<usize, (Vec<Pin>, Vec<Pin>)> = BTreeMap::new();
    for (i, c) in circuit.components.iter().enumerate() {
        let is_inside = inside.contains(&c.id);
        for (pin, net) in pins_with_nets(circuit, netlist, i) {
            let entry = by_net.entry(net).or_default();
            if is_inside {
                entry.0.push(pin);
            } else {
                entry.1.push(pin);
            }
        }
    }
    by_net
        .into_iter()
        .filter(|(_, (i, o))| !i.is_empty() && !o.is_empty())
        .map(|(net, (inner, outer))| Boundary {
            net,
            output: inner.iter().any(|p| p.output),
            outer,
            label: String::new(),
        })
        .collect()
}

/// Popisky vývodů: podle přepínače (vstup) nebo LED (výstup), které síť venku
/// obsluhují, jinak A, B, C… a Y1, Y2…; popisky se neopakují.
///
/// [en] Pin labels: after the switch (input) or LED (output) serving the net outside,
/// otherwise A, B, C… and Y1, Y2…; labels do not repeat.
fn label_boundaries(circuit: &Circuit, boundaries: &mut [Boundary]) {
    let mut taken: BTreeSet<String> = BTreeSet::new();
    // Nejdřív popisky převzaté z ovládacích a zobrazovacích prvků.
    // [en] First the labels taken over from control and display elements.
    for b in boundaries.iter_mut() {
        let label = b
            .outer
            .iter()
            .map(|p| &circuit.components[p.comp])
            .filter(|c| {
                if b.output {
                    matches!(c.kind, ComponentKind::Led { .. })
                } else {
                    c.kind.is_input()
                }
            })
            .map(|c| c.label.trim().to_owned())
            .find(|l| !l.is_empty() && !taken.contains(l));
        if let Some(l) = label {
            taken.insert(l.clone());
            b.label = l;
        }
    }
    for b in boundaries.iter_mut().filter(|b| b.label.is_empty()) {
        let candidates: Box<dyn Iterator<Item = String>> = if b.output {
            Box::new((1..).map(|n| format!("Y{n}")))
        } else {
            Box::new(
                ('A'..='Z')
                    .map(|c| c.to_string())
                    .chain((1..).map(|n| format!("X{n}"))),
            )
        };
        let label = candidates
            .into_iter()
            .find(|l| !taken.contains(l))
            .unwrap_or_default();
        taken.insert(label.clone());
        b.label = label;
    }
}

/// Zapojení podobvodu: vybrané součástky, jejich vnitřní vodiče, vodiče hraničních
/// sítí a přepínač / LED rozhraní pro každou hraniční síť.
///
/// [en] Circuit of the subcircuit: the selected components, their internal wires, the
/// wires of the boundary nets and an interface switch / LED for each boundary net.
fn build_inner(
    circuit: &Circuit,
    netlist: &Netlist,
    inside: &BTreeSet<CompId>,
    ins: &[Boundary],
    outs: &[Boundary],
) -> Circuit {
    let ids: Vec<CompId> = inside.iter().copied().collect();
    let mut inner = circuit.extract(&ids, &[]);
    let boundary_set: BTreeSet<usize> = ins.iter().chain(outs).map(|b| b.net).collect();
    for w in &circuit.wires {
        if boundary_set.contains(&netlist.wire_net[&w.id])
            && !inner.wires.iter().any(|x| x.id == w.id)
        {
            inner.wires.push(*w);
        }
    }
    let mut anchors: BTreeSet<Point> = BTreeSet::new();
    let mut loose: Vec<Point> = Vec::new();
    for b in ins.iter().chain(outs) {
        let anchor = choose_anchor(circuit, b);
        anchors.insert(anchor.at);
        loose.extend(b.outer.iter().map(|p| p.at).filter(|p| *p != anchor.at));
        let original = &circuit.components[anchor.comp];
        // Původní přepínač nebo LED se zkopíruje (stejná poloha a otočení), jinak
        // vznikne nový, jehož vývod leží v bodě kotvy.
        // [en] The original switch or LED is copied (same position and rotation),
        // otherwise a new one is made whose pin lies at the anchor point.
        let (kind, pos, rotation) = if b.output {
            match original.kind {
                ComponentKind::Led { color } if !anchor.output => (
                    ComponentKind::Led { color },
                    original.pos,
                    original.rotation,
                ),
                _ => (
                    ComponentKind::Led {
                        color: LedColor::Red,
                    },
                    anchor.at.offset(2, 0),
                    Rotation::R0,
                ),
            }
        } else if original.kind.is_input() && anchor.output {
            (
                ComponentKind::Switch { on: false },
                original.pos,
                original.rotation,
            )
        } else {
            (
                ComponentKind::Switch { on: false },
                anchor.at.offset(-2, 0),
                Rotation::R0,
            )
        };
        let id = inner.add_component(kind, pos);
        if let Some(c) = inner.component_mut(id) {
            c.rotation = rotation;
            c.label = b.label.clone();
        }
    }
    tidy(&mut inner, &loose);
    inner
}

/// Kotva rozhraní: vývod součástky venku, na jehož místo se postaví přepínač nebo
/// LED uvnitř. Vstup: přednostně výstup přepínače / tlačítka, pak jiný výstup;
/// výstup: přednostně vstup LED, pak jiný vstup.
///
/// [en] Interface anchor: a pin of an outside component in whose place the switch or
/// LED is put inside. Input: preferably the output of a switch / push button, then
/// another output; output: preferably an LED input, then another input.
fn choose_anchor(circuit: &Circuit, b: &Boundary) -> Pin {
    let kind = |p: &Pin| circuit.components[p.comp].kind;
    let rank = |p: &Pin| -> u8 {
        match (b.output, p.output) {
            (false, true) if kind(p).is_input() => 0,
            (false, true) => 1,
            (true, false) if matches!(kind(p), ComponentKind::Led { .. }) => 0,
            (true, false) => 1,
            _ => 2,
        }
    };
    *b.outer
        .iter()
        .min_by_key(|p| (rank(p), p.at))
        .expect("hraniční síť má vývod venku")
}

/// Odstraní slepé vodiče, které začínají v některém z bodů `seeds` (konec vodiče,
/// na kterém nic jiného není), a postupně i další, které tím osiří.
///
/// [en] Removes dead-end wires starting at one of the points `seeds` (a wire end with
/// nothing else on it), and step by step also the others that become orphaned by it.
fn prune_dead_ends(c: &mut Circuit, seeds: &[Point]) {
    let mut queue: Vec<Point> = seeds.to_vec();
    while let Some(p) = queue.pop() {
        let pins = c.components.iter().any(|k| k.pins().contains(&p));
        let touching: Vec<usize> = (0..c.wires.len())
            .filter(|&i| c.wires[i].contains(p))
            .collect();
        if pins || touching.len() != 1 || !c.wires[touching[0]].is_endpoint(p) {
            continue;
        }
        // Vodič s odbočkou „T“ uprostřed (konec jiného vodiče na něm) slepý není.
        // [en] A wire with a “T” branch in its middle (the end of another wire on it) is not a dead end.
        let w = c.wires[touching[0]];
        let branch = c.wires.iter().any(|o| {
            o.id != w.id
                && [o.a, o.b]
                    .iter()
                    .any(|e| w.contains(*e) && !w.is_endpoint(*e))
        });
        if branch {
            // Slepý konec se zkrátí k nejbližší odbočce.
            // [en] The dead end is shortened to the nearest branch.
            let far = if w.a == p { w.b } else { w.a };
            let nearest = c
                .wires
                .iter()
                .filter(|o| o.id != w.id)
                .flat_map(|o| [o.a, o.b])
                .filter(|e| w.contains(*e) && !w.is_endpoint(*e))
                .min_by_key(|e| (e.x - p.x).abs() + (e.y - p.y).abs());
            if let Some(n) = nearest
                && let Some(x) = c.wires.iter_mut().find(|x| x.id == w.id)
            {
                *x = crate::circuit::Wire {
                    id: w.id,
                    a: n,
                    b: far,
                };
            }
            continue;
        }
        c.wires.remove(touching[0]);
        queue.push(if w.a == p { w.b } else { w.a });
    }
}

/// Vnější obvod: bez vybraných součástek a jejich vnitřních vodičů, s instancí
/// podobvodu připojenou k hraničním sítím. Vrací (obvod, id instance, počet
/// nepřipojených vývodů).
///
/// [en] The outer circuit: without the selected components and their internal wires,
/// with an instance of the subcircuit connected to the boundary nets. Returns
/// (circuit, instance id, number of unconnected pins).
fn build_outer(
    circuit: &Circuit,
    netlist: &Netlist,
    inside: &BTreeSet<CompId>,
    ins: &[Boundary],
    outs: &[Boundary],
    def_id: u32,
) -> (Circuit, CompId, usize) {
    let ids: Vec<CompId> = inside.iter().copied().collect();
    let internal = circuit.extract(&ids, &[]);
    let mut outer = circuit.clone();
    outer.components.retain(|c| !inside.contains(&c.id));
    outer
        .wires
        .retain(|w| !internal.wires.iter().any(|x| x.id == w.id));
    // Vodiče, které vedly jen k vybraným součástkám, se zkrátí k odbočce – uvolní místo pro značku.
    // [en] Wires that led only to the selected components are shortened to the branch – making room for the symbol.
    let selected_points: Vec<Point> = circuit
        .components
        .iter()
        .filter(|c| inside.contains(&c.id))
        .flat_map(Component::pins)
        .collect();
    tidy(&mut outer, &selected_points);

    // Body sítí uvnitř výběru, kam vede vodič zvenku – cíle pro připojení.
    // [en] Points of the nets inside the selection where a wire comes from outside – targets to connect to.
    let order: Vec<&Boundary> = ins.iter().chain(outs).collect();
    let selected_pins: Vec<Point> = circuit
        .components
        .iter()
        .filter(|c| inside.contains(&c.id))
        .flat_map(Component::pins)
        .collect();

    // Původní sítě zbylých součástek – nesmí se spojit (kromě připojení instance).
    // [en] Original nets of the remaining components – they must not merge (except connecting the instance).
    let reference = Reference::new(circuit, netlist, inside, &order);

    let kind = ComponentKind::Subcircuit {
        id: def_id,
        inputs: ins.len() as u8,
        outputs: outs.len() as u8,
    };
    let center = centroid(circuit, inside);
    // Přednostně místo, kde tělo značky nepřekrývá vodiče a vývody neleží na vodičích;
    // jinak první místo, kde značka nic nespojí.
    // [en] Preferably a place where the symbol body does not overlap wires and the pins
    // do not lie on wires; otherwise the first place where the symbol joins nothing.
    let mut placed = None;
    let mut fallback = None;
    let mut tried = 0;
    let others: Vec<(f32, f32, f32, f32)> = outer.components.iter().map(Component::body).collect();
    let other_pins: Vec<Point> = outer.components.iter().flat_map(Component::pins).collect();
    for (dx, dy) in spiral(30) {
        let probe = Component {
            id: CompId(u32::MAX),
            kind,
            pos: center.offset(dx, dy),
            rotation: Rotation::R0,
            label: String::new(),
            pressed: false,
        };
        let body = probe.body();
        let pins = probe.pins();
        // Kolem značky (i s vývody) má zůstat volný okraj, aby šly vést vodiče.
        // [en] A free margin should stay around the symbol (pins included) so wires can be led.
        let near = |q: &Point, (a, b, c, d): (f32, f32, f32, f32), m: f32| {
            let (x, y) = (q.x as f32, q.y as f32);
            x > a - m && x < c + m && y > b - m && y < d + m
        };
        let clean = outer
            .wires
            .iter()
            .all(|w| !crosses_body(w.a, w.b, body) && !pins.iter().any(|p| w.contains(*p)))
            && others
                .iter()
                .all(|o| !overlaps(*o, body) && !pins.iter().any(|p| near(p, *o, 1.5)))
            && other_pins.iter().all(|q| !near(q, body, 2.5));
        if !clean && (fallback.is_some() || tried > 40) {
            continue;
        }
        tried += 1;
        let mut trial = outer.clone();
        let id = trial.add_component(kind, probe.pos);
        if reference.check(&trial, id).is_none() {
            continue;
        }
        if clean {
            placed = Some((trial, id));
            break;
        }
        fallback = Some((trial, id));
    }
    let placed = placed.or(fallback);
    let Some((outer, id)) = placed else {
        // Bez volného místa se značka položí doprostřed nepřipojená.
        // [en] Without a free place the symbol is put in the middle unconnected.
        let id = outer.add_component(kind, center);
        return (outer, id, order.len());
    };
    let outer_without_routes = outer.clone();

    // Vývody se připojují postupně; když některý nejde, zkusí se opačné pořadí.
    // [en] The pins are connected one by one; if one fails, the reverse order is tried.
    let forward: Vec<usize> = (0..order.len()).collect();
    let backward: Vec<usize> = (0..order.len()).rev().collect();
    let (mut outer, mut unconnected) = connect_pins(
        circuit,
        netlist,
        &order,
        &selected_pins,
        &reference,
        outer,
        id,
        &forward,
    );
    if unconnected > 0 {
        let base = outer_without_routes.clone();
        let (o2, u2) = connect_pins(
            circuit,
            netlist,
            &order,
            &selected_pins,
            &reference,
            base,
            id,
            &backward,
        );
        if u2 < unconnected {
            outer = o2;
            unconnected = u2;
        }
    }
    tidy(&mut outer, &selected_pins);
    // Konečná kontrola po odstranění slepých vodičů.
    // [en] Final check after removing the dead-end wires.
    if let Some(ok) = reference.check(&outer, id) {
        unconnected = ok.iter().filter(|c| !**c).count();
    }
    (outer, id, unconnected)
}

/// Připojí vývody instance `id` v pořadí `pin_order` k jejich sítím (viz [`routes`]).
/// Vrací upravený obvod a počet vývodů, které připojit nešlo.
///
/// [en] Connects the pins of the instance `id` in the order `pin_order` to their nets
/// (see [`routes`]). Returns the modified circuit and the number of pins that could not be connected.
#[allow(clippy::too_many_arguments)]
fn connect_pins(
    circuit: &Circuit,
    netlist: &Netlist,
    order: &[&Boundary],
    selected_pins: &[Point],
    reference: &Reference,
    mut outer: Circuit,
    id: CompId,
    pin_order: &[usize],
) -> (Circuit, usize) {
    let inst = outer.component(id).expect("instance").clone();
    let pin_points: Vec<Point> = inst
        .input_pins()
        .into_iter()
        .chain(inst.output_pins())
        .collect();
    // Síť každého vodiče (původní podle netlistu, nové podle vývodu, ke kterému vedou).
    // [en] The net of each wire (original ones from the netlist, new ones by the pin they lead to).
    let mut wire_net: HashMap<crate::WireId, usize> = outer
        .wires
        .iter()
        .filter_map(|w| circuit.wire(w.id).map(|o| (w.id, netlist.wire_net[&o.id])))
        .collect();
    // Síť vývodů součástek venku.
    // [en] The net of the pins of the components outside.
    let pin_net: HashMap<Point, usize> = order
        .iter()
        .flat_map(|b| b.outer.iter().map(|q| (q.at, b.net)))
        .collect();
    let mut unconnected = 0;
    for &k in pin_order {
        let b = order[k];
        let p = pin_points[k];
        let dir = if b.output { 1 } else { -1 };
        let net_wires: Vec<crate::circuit::Wire> = outer
            .wires
            .iter()
            .filter(|w| wire_net.get(&w.id) == Some(&b.net))
            .copied()
            .collect();
        // Cíle: vývody sítě venku, konce jejích vodičů a nejbližší bod každého
        // vodiče (připojení odbočkou „T“ doprostřed vodiče).
        // [en] Targets: the net's pins outside, the ends of its wires and the nearest
        // point of each wire (connecting by a “T” branch to the middle of a wire).
        let mut targets: Vec<Point> = b.outer.iter().map(|q| q.at).collect();
        for w in &net_wires {
            targets.extend([w.a, w.b]);
            targets.push(Point::new(
                p.x.clamp(w.a.x.min(w.b.x), w.a.x.max(w.b.x)),
                p.y.clamp(w.a.y.min(w.b.y), w.a.y.max(w.b.y)),
            ));
        }
        targets.sort_by_key(|t| ((t.x - p.x).abs() + (t.y - p.y).abs(), *t));
        targets.dedup();
        let busy: Vec<Point> = outer
            .components
            .iter()
            .flat_map(Component::pins)
            .chain(selected_pins.iter().copied())
            .collect();
        let bodies: Vec<(f32, f32, f32, f32)> =
            outer.components.iter().map(Component::body).collect();
        let mut done = false;
        'targets: for t in targets.into_iter().take(10) {
            // Cíl nesmí ležet na vodiči ani vývodu jiné sítě.
            // [en] The target must not lie on a wire or pin of another net.
            let foreign = outer
                .wires
                .iter()
                .any(|w| w.contains(t) && wire_net.get(&w.id) != Some(&b.net))
                || (busy.contains(&t) && pin_net.get(&t) != Some(&b.net));
            if foreign {
                continue;
            }
            for route in routes(p, t, dir) {
                if !route_clear(&route, &outer, &busy, &bodies) {
                    continue;
                }
                let mut trial = outer.clone();
                let added: Vec<crate::WireId> = route
                    .windows(2)
                    .filter_map(|pair| trial.add_wire(pair[0], pair[1]))
                    .collect();
                // Geometrická kontrola by měla stačit; sítě se přesto ověří.
                // [en] The geometric check should suffice; the nets are verified anyway.
                if let Some(ok) = reference.check(&trial, id)
                    && ok[k]
                {
                    for w in added {
                        wire_net.insert(w, b.net);
                    }
                    outer = trial;
                    done = true;
                    break 'targets;
                }
            }
        }
        // Když žádná jednoduchá trasa nejde, hledá se cesta po mřížce (bludiště).
        // [en] When no simple route works, a path is searched on the grid (a maze).
        if !done {
            let mut targets: Vec<Point> = b.outer.iter().map(|q| q.at).collect();
            for w in &net_wires {
                targets.extend([w.a, w.b]);
            }
            targets.retain(|t| {
                !outer
                    .wires
                    .iter()
                    .any(|w| w.contains(*t) && wire_net.get(&w.id) != Some(&b.net))
                    && !(busy.contains(t) && pin_net.get(t) != Some(&b.net))
            });
            if let Some(route) = maze(p, &targets, &outer, &busy, &bodies) {
                let mut trial = outer.clone();
                let added: Vec<crate::WireId> = route
                    .windows(2)
                    .filter_map(|pair| trial.add_wire(pair[0], pair[1]))
                    .collect();
                if let Some(ok) = reference.check(&trial, id)
                    && ok[k]
                {
                    for w in added {
                        wire_net.insert(w, b.net);
                    }
                    outer = trial;
                    done = true;
                }
            }
        }
        if !done {
            unconnected += 1;
        }
    }
    (outer, unconnected)
}

/// Hledání cesty po mřížce z vývodu `p` do některého z bodů `targets` (Dijkstra,
/// ohyb stojí víc než krok). Cesta nevede přes těla součástek ani cizí vývody,
/// vodiče jen kolmo kříží (bez ohybu na nich a mimo jejich konce). Vrací lomenou
/// čáru (začátek, ohyby, cíl).
///
/// [en] Path search on the grid from pin `p` to one of the points `targets` (Dijkstra,
/// a bend costs more than a step). The path does not run over component bodies or
/// foreign pins and only crosses wires at right angles (no bend on them, away from
/// their ends). Returns a polyline (start, bends, target).
fn maze(
    p: Point,
    targets: &[Point],
    outer: &Circuit,
    busy: &[Point],
    bodies: &[(f32, f32, f32, f32)],
) -> Option<Vec<Point>> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    if targets.is_empty() {
        return None;
    }
    const DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    const TURN: u32 = 6;
    const MARGIN: i32 = 8;
    let xs = outer
        .wires
        .iter()
        .flat_map(|w| [w.a.x, w.b.x])
        .chain(outer.components.iter().map(|c| c.pos.x))
        .chain(targets.iter().map(|t| t.x))
        .chain([p.x]);
    let ys = outer
        .wires
        .iter()
        .flat_map(|w| [w.a.y, w.b.y])
        .chain(outer.components.iter().map(|c| c.pos.y))
        .chain(targets.iter().map(|t| t.y))
        .chain([p.y]);
    let (x0, x1) = (xs.clone().min()? - MARGIN, xs.max()? + MARGIN);
    let (y0, y1) = (ys.clone().min()? - MARGIN, ys.max()? + MARGIN);
    let goal: BTreeSet<Point> = targets.iter().copied().collect();
    let ends: BTreeSet<Point> = outer.wires.iter().flat_map(|w| [w.a, w.b]).collect();
    let in_body = |q: Point| {
        bodies.iter().any(|&(a, b, c, d)| {
            let (x, y) = (q.x as f32, q.y as f32);
            x > a - 0.01 && x < c + 0.01 && y > b - 0.01 && y < d + 0.01
        })
    };
    // Vodiče procházející bodem: (vodorovný, svislý).
    // [en] Wires passing through a point: (horizontal, vertical).
    let wires_at = |q: Point| {
        let mut hv = (0, 0);
        for w in &outer.wires {
            if w.contains(q) {
                if w.a.y == w.b.y {
                    hv.0 += 1;
                } else {
                    hv.1 += 1;
                }
            }
        }
        hv
    };
    let mut best: HashMap<(Point, usize), u32> = HashMap::new();
    let mut prev: HashMap<(Point, usize), (Point, usize)> = HashMap::new();
    let mut heap = BinaryHeap::new();
    for (d, _) in DIRS.iter().enumerate() {
        best.insert((p, d), 0);
        heap.push(Reverse((0u32, p.x, p.y, d)));
    }
    while let Some(Reverse((cost, x, y, d))) = heap.pop() {
        let q = Point::new(x, y);
        if best.get(&(q, d)).is_some_and(|c| *c < cost) {
            continue;
        }
        if q != p && goal.contains(&q) {
            // Zpětná stopa a zjednodušení na ohyby.
            // [en] Trace back and simplify to bends.
            let mut pts = vec![q];
            let mut state = (q, d);
            while let Some(&s) = prev.get(&state) {
                pts.push(s.0);
                state = s;
            }
            pts.reverse();
            pts.dedup();
            let mut poly = vec![pts[0]];
            for i in 1..pts.len() - 1 {
                let (a, b, c) = (pts[i - 1], pts[i], pts[i + 1]);
                if (a.x == b.x) != (b.x == c.x) {
                    poly.push(b);
                }
            }
            poly.push(q);
            return Some(poly);
        }
        // Na vodiči (křížení) se nesmí zatáčet.
        // [en] No turning on a wire (a crossing).
        let on_wire = q != p && wires_at(q) != (0, 0);
        for (nd, (dx, dy)) in DIRS.iter().enumerate() {
            if on_wire && nd != d {
                continue;
            }
            let n = Point::new(x + dx, y + dy);
            if n.x < x0 || n.x > x1 || n.y < y0 || n.y > y1 {
                continue;
            }
            if !goal.contains(&n) {
                if in_body(n) || busy.contains(&n) {
                    continue;
                }
                let (h, v) = wires_at(n);
                if h + v > 0 {
                    // Jen kolmé křížení jednoho vodiče mimo jeho konce.
                    // [en] Only a perpendicular crossing of one wire away from its ends.
                    let horizontal_move = *dy == 0;
                    let ok = h + v == 1
                        && !ends.contains(&n)
                        && ((horizontal_move && v == 1) || (!horizontal_move && h == 1));
                    if !ok {
                        continue;
                    }
                }
            }
            let nc = cost + 1 + if q != p && nd != d { TURN } else { 0 };
            if best.get(&(n, nd)).is_none_or(|c| nc < *c) {
                best.insert((n, nd), nc);
                prev.insert((n, nd), (q, d));
                heap.push(Reverse((nc, n.x, n.y, nd)));
            }
        }
    }
    None
}

/// Lze trasu nakreslit, aniž by se dotkla čehokoli jiného než svého začátku a cíle?
/// Nesmí vést přes vývod ani tělo součástky, překrývat se s vodičem, mít ohyb nebo
/// začátek na vodiči ani procházet koncem vodiče (vznikla by odbočka „T“).
/// Křížení s vodičem v jeho vnitřním bodě je v pořádku.
///
/// [en] Can the route be drawn without touching anything but its start and target? It
/// must not run over a pin or a component body, overlap a wire, have a bend or its
/// start on a wire, or pass through a wire end (a “T” branch would appear). Crossing a
/// wire at its inner point is fine.
fn route_clear(
    route: &[Point],
    outer: &Circuit,
    busy: &[Point],
    bodies: &[(f32, f32, f32, f32)],
) -> bool {
    let (p, t) = (route[0], route[route.len() - 1]);
    let seg = |a: Point, b: Point| crate::circuit::Wire {
        id: crate::WireId(0),
        a,
        b,
    };
    for pair in route.windows(2) {
        let s = seg(pair[0], pair[1]);
        if busy.iter().any(|q| *q != p && *q != t && s.contains(*q))
            || bodies.iter().any(|b| crosses_body(pair[0], pair[1], *b))
        {
            return false;
        }
        for w in &outer.wires {
            // Konec vodiče na trase (kromě cíle) by vytvořil odbočku.
            // [en] A wire end on the route (except the target) would create a branch.
            if [w.a, w.b].iter().any(|e| *e != t && s.contains(*e)) {
                return false;
            }
            // Překryv rovnoběžných úseků.
            // [en] Overlap of parallel segments.
            let horizontal = |x: &crate::circuit::Wire| x.a.y == x.b.y;
            if horizontal(&s) == horizontal(w) {
                let shared = [s.a, s.b].iter().filter(|q| w.contains(**q)).count()
                    + [w.a, w.b].iter().filter(|q| s.contains(**q)).count();
                let same_line = if horizontal(&s) {
                    s.a.y == w.a.y
                } else {
                    s.a.x == w.a.x
                };
                if same_line && shared > 0 && !(shared == 1 && (w.contains(t) || w.contains(p))) {
                    return false;
                }
            }
        }
    }
    // Ohyby a začátek nesmí ležet na vodiči.
    // [en] The bends and the start must not lie on a wire.
    route[..route.len() - 1]
        .iter()
        .all(|v| !outer.wires.iter().any(|w| w.contains(*v)))
}

/// Úklid po přepojení: odstraní slepé vodiče (od bodů `seeds`) a vodiče, které celé
/// leží na jiném vodiči (nekončí-li na vývodu); opakuje se, dokud se něco mění.
///
/// [en] Tidying after reconnecting: removes dead-end wires (from the points `seeds`) and
/// wires lying entirely on another wire (unless they end at a pin); repeated while
/// something changes.
fn tidy(c: &mut Circuit, seeds: &[Point]) {
    let mut seeds = seeds.to_vec();
    loop {
        prune_dead_ends(c, &seeds);
        let pins: Vec<Point> = c.components.iter().flat_map(Component::pins).collect();
        let covered = c.wires.iter().position(|w| {
            !pins.contains(&w.a)
                && !pins.contains(&w.b)
                && c.wires.iter().any(|o| {
                    o.id != w.id
                        && o.contains(w.a)
                        && o.contains(w.b)
                        && (o.len() > w.len() || (o.len() == w.len() && o.id < w.id))
                })
        });
        let Some(i) = covered else { break };
        let w = c.wires.remove(i);
        seeds = vec![w.a, w.b];
    }
}

/// Překrývají se dvě těla součástek?
///
/// [en] Do two component bodies overlap?
fn overlaps(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
    a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3
}

/// Prochází úsečka a–b (vodorovná nebo svislá) vnitřkem těla součástky?
///
/// [en] Does the segment a–b (horizontal or vertical) pass through the inside of a component body?
fn crosses_body(a: Point, b: Point, (x0, y0, x1, y1): (f32, f32, f32, f32)) -> bool {
    let (ax, ay, bx, by) = (a.x as f32, a.y as f32, b.x as f32, b.y as f32);
    let (lx, hx) = (ax.min(bx), ax.max(bx));
    let (ly, hy) = (ay.min(by), ay.max(by));
    // Vedení po okraji těla se počítá také (vodič by splýval s obrysem).
    // [en] Running along the edge of the body counts too (the wire would merge with the outline).
    const E: f32 = 0.01;
    hx > x0 - E && lx < x1 + E && hy > y0 - E && ly < y1 + E
}

/// Těžiště poloh vybraných součástek (na mřížce).
///
/// [en] Centroid of the positions of the selected components (on the grid).
fn centroid(circuit: &Circuit, inside: &BTreeSet<CompId>) -> Point {
    let pts: Vec<Point> = circuit
        .components
        .iter()
        .filter(|c| inside.contains(&c.id))
        .map(|c| c.pos)
        .collect();
    let n = pts.len().max(1) as i32;
    Point::new(
        pts.iter().map(|p| p.x).sum::<i32>() / n,
        pts.iter().map(|p| p.y).sum::<i32>() / n,
    )
}

/// Posuny (dx, dy) od středu po rostoucí vzdálenosti, nejvýš `r` bodů.
///
/// [en] Offsets (dx, dy) from the centre by increasing distance, at most `r` points.
fn spiral(r: i32) -> Vec<(i32, i32)> {
    let mut v: Vec<(i32, i32)> = (-r..=r)
        .flat_map(|dx| (-r..=r).map(move |dy| (dx, dy)))
        .collect();
    v.sort_by_key(|&(dx, dy)| (dx.abs() + dy.abs(), dx.abs(), dy, dx));
    v
}

/// Možné trasy z vývodu `p` do bodu `t` (lomené čáry s nejvýš dvěma ohyby);
/// `dir` je směr od značky (−1 vlevo pro vstupy, +1 vpravo pro výstupy).
///
/// [en] Possible routes from pin `p` to point `t` (polylines with at most two bends);
/// `dir` is the direction away from the symbol (−1 left for inputs, +1 right for outputs).
fn routes(p: Point, t: Point, dir: i32) -> Vec<Vec<Point>> {
    let mut v = vec![
        vec![p, Point::new(t.x, p.y), t],
        vec![p, Point::new(p.x, t.y), t],
    ];
    for d in [dir, -dir] {
        for s in 1..=10 {
            let x = p.x + d * s;
            v.push(vec![p, Point::new(x, p.y), Point::new(x, t.y), t]);
        }
    }
    // Objížďka: od značky, nahoru nebo dolů vedle cíle a k cíli z boku.
    // [en] Detour: away from the symbol, up or down next to the target and into the target from the side.
    for d in [dir, -dir] {
        for s in 1..=8 {
            let x = p.x + d * s;
            for e in [-1, 1, -2, 2, -3, 3, -4, 4] {
                let y = t.y + e;
                v.push(vec![
                    p,
                    Point::new(x, p.y),
                    Point::new(x, y),
                    Point::new(t.x, y),
                    t,
                ]);
            }
        }
    }
    for v in &mut v {
        v.dedup();
    }
    v.retain(|r| r.len() >= 2);
    v
}

/// Sítě původního obvodu, se kterými se porovnává upravený obvod.
///
/// [en] Nets of the original circuit with which the modified circuit is compared.
struct Reference {
    /// Vývody zbylých součástek (id, poloha) a jejich původní síť.
    ///
    /// [en] Pins of the remaining components (id, position) and their original net.
    pins: Vec<(CompId, Point, usize)>,
    /// Původní síť každé hraniční sítě v pořadí vývodů instance.
    ///
    /// [en] The original net of each boundary net in the order of the instance pins.
    order: Vec<usize>,
}

impl Reference {
    /// Zapamatuje si původní sítě vývodů součástek, které zůstanou venku.
    ///
    /// [en] Remembers the original nets of the pins of the components that stay outside.
    fn new(
        circuit: &Circuit,
        netlist: &Netlist,
        inside: &BTreeSet<CompId>,
        order: &[&Boundary],
    ) -> Self {
        let mut pins = Vec::new();
        for (i, c) in circuit.components.iter().enumerate() {
            if inside.contains(&c.id) {
                continue;
            }
            for (pin, net) in pins_with_nets(circuit, netlist, i) {
                pins.push((c.id, pin.at, net));
            }
        }
        Self {
            pins,
            order: order.iter().map(|b| b.net).collect(),
        }
    }

    /// Ověří upravený obvod s instancí `inst`: žádné dvě původně různé sítě se
    /// nesmí spojit a každý vývod instance je buď volný, nebo ve své síti.
    /// Vrací, které vývody instance jsou připojené, nebo `None` při chybném spojení.
    ///
    /// [en] Checks the modified circuit with the instance `inst`: no two originally
    /// different nets may merge, and each instance pin is either free or in its own net.
    /// Returns which instance pins are connected, or `None` on a wrong connection.
    fn check(&self, c: &Circuit, inst: CompId) -> Option<Vec<bool>> {
        let nl = Netlist::build(c);
        let mut net_at: HashMap<(CompId, Point), usize> = HashMap::new();
        let mut inst_nets = Vec::new();
        for (i, comp) in c.components.iter().enumerate() {
            for (pin, net) in pins_with_nets(c, &nl, i) {
                if comp.id == inst {
                    inst_nets.push(net);
                } else {
                    net_at.insert((comp.id, pin.at), net);
                }
            }
        }
        // Nová síť → původní síť; dvě různé původní sítě v jedné nové = zkrat.
        // [en] New net → original net; two different original nets in one new net = a short.
        let mut origin: HashMap<usize, usize> = HashMap::new();
        for (id, at, old) in &self.pins {
            let new = *net_at.get(&(*id, *at))?;
            if *origin.entry(new).or_insert(*old) != *old {
                return None;
            }
        }
        let mut connected = Vec::with_capacity(inst_nets.len());
        for (k, net) in inst_nets.iter().enumerate() {
            match origin.get(net) {
                None => connected.push(false),
                Some(old) if *old == self.order[k] => connected.push(true),
                Some(_) => return None,
            }
        }
        // Dva vývody instance nesmí sdílet volnou síť (spojily by se dvě sítě uvnitř).
        // [en] Two instance pins must not share a free net (two nets inside would merge).
        let unique: BTreeSet<usize> = inst_nets.iter().copied().collect();
        if unique.len() != inst_nets.len() {
            return None;
        }
        Some(connected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GateKind, LogicValue, Simulator, SubcircuitDef};

    /// Hodnoty všech LED pro všechny kombinace přepínačů (podle popisků).
    ///
    /// [en] Values of all LEDs for all combinations of the switches (by label).
    fn truth(c: &Circuit, defs: &[SubcircuitDef]) -> Vec<Vec<LogicValue>> {
        let (ins, outs) = c.io();
        let ins: Vec<CompId> = ins.iter().map(|k| k.id).collect();
        let outs: Vec<CompId> = outs.iter().map(|k| k.id).collect();
        let mut rows = Vec::new();
        for bits in 0..(1u32 << ins.len()) {
            let mut sim = Simulator::with_subcircuits(c, defs, None);
            for (i, id) in ins.iter().enumerate() {
                sim.set_source(*id, (bits >> i & 1 == 1).into());
            }
            assert!(sim.run(100_000));
            rows.push(outs.iter().map(|id| sim.inputs(*id)[0]).collect());
        }
        rows
    }

    /// Poloviční sčítačka: přepínače A, B → XOR (S) a AND (C) → LED S, C.
    /// Vrací obvod a id obou hradel.
    ///
    /// [en] Half adder: switches A, B → XOR (S) and AND (C) → LEDs S, C. Returns the
    /// circuit and the ids of both gates.
    pub(super) fn half_adder() -> (Circuit, Vec<CompId>) {
        let mut c = Circuit::default();
        let a = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        let b = c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 6));
        c.component_mut(a).unwrap().label = "A".into();
        c.component_mut(b).unwrap().label = "B".into();
        // XOR vstupy (7, −1), (7, 1); AND vstupy (7, 7), (7, 9).
        // [en] XOR inputs (7, −1), (7, 1); AND inputs (7, 7), (7, 9).
        let x = c.add_component(ComponentKind::gate(GateKind::Xor), Point::new(10, 0));
        let y = c.add_component(ComponentKind::gate(GateKind::And), Point::new(10, 8));
        let s = c.add_component(ComponentKind::LED, Point::new(16, 0));
        let k = c.add_component(ComponentKind::LED, Point::new(16, 8));
        c.component_mut(s).unwrap().label = "S".into();
        c.component_mut(k).unwrap().label = "C".into();
        // A (2, 0) → XOR (7, −1) a AND (7, 7).
        // [en] A (2, 0) → XOR (7, −1) and AND (7, 7).
        c.add_wire(Point::new(2, 0), Point::new(4, 0));
        c.add_wire(Point::new(4, -1), Point::new(4, 7));
        c.add_wire(Point::new(4, -1), Point::new(7, -1));
        c.add_wire(Point::new(4, 7), Point::new(7, 7));
        // B (2, 6) → XOR (7, 1) a AND (7, 9).
        // [en] B (2, 6) → XOR (7, 1) and AND (7, 9).
        c.add_wire(Point::new(2, 6), Point::new(5, 6));
        c.add_wire(Point::new(5, 1), Point::new(5, 9));
        c.add_wire(Point::new(5, 1), Point::new(7, 1));
        c.add_wire(Point::new(5, 9), Point::new(7, 9));
        // Výstupy (13, 0), (13, 8) → LED (14, 0), (14, 8).
        // [en] Outputs (13, 0), (13, 8) → LEDs (14, 0), (14, 8).
        c.add_wire(Point::new(13, 0), Point::new(14, 0));
        c.add_wire(Point::new(13, 8), Point::new(14, 8));
        (c, vec![x, y])
    }

    #[test]
    fn half_adder_reference_is_correct() {
        use LogicValue::{One, Zero};
        let (c, _) = half_adder();
        // Pořadí výstupů C, S (podle popisku).
        // [en] Output order C, S (by label).
        assert_eq!(
            truth(&c, &[]),
            vec![
                vec![Zero, Zero],
                vec![Zero, One],
                vec![Zero, One],
                vec![One, Zero]
            ]
        );
    }

    /// Zabalí výběr, zkontroluje, že se pravdivostní tabulka nezměnila, a vrátí výsledek.
    ///
    /// [en] Groups the selection, checks that the truth table did not change, and returns the result.
    fn group_same(c: &Circuit, sel: &[CompId]) -> Grouping {
        let g = group(c, sel, 7).unwrap();
        assert_eq!(g.unconnected, 0, "všechny vývody připojené");
        let def = SubcircuitDef {
            id: 7,
            name: "P".into(),
            circuit: g.inner.clone(),
        };
        assert_eq!(truth(&g.outer, &[def]), truth(c, &[]));
        g
    }

    #[test]
    fn gates_become_subcircuit_with_interface() {
        let (c, gates) = half_adder();
        let g = group_same(&c, &gates);
        assert_eq!((g.inputs, g.outputs), (2, 2));
        // Rozhraní převzalo popisky přepínačů a LED.
        // [en] The interface took over the labels of the switches and LEDs.
        let (ins, outs) = g.inner.io();
        let names = |v: Vec<&Component>| v.iter().map(|k| k.label.clone()).collect::<Vec<_>>();
        assert_eq!(names(ins), ["A", "B"]);
        assert_eq!(names(outs), ["C", "S"]);
        // Venku zůstaly přepínače a LED, hradla jsou uvnitř.
        // [en] The switches and LEDs stayed outside, the gates are inside.
        assert_eq!(g.outer.components.len(), 5);
        assert!(g.outer.component(gates[0]).is_none());
        assert_eq!(
            g.inner
                .components
                .iter()
                .filter(|k| matches!(k.kind, ComponentKind::Gate { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn whole_selection_keeps_controls_outside() {
        let (c, _) = half_adder();
        let all: Vec<CompId> = c.components.iter().map(|k| k.id).collect();
        let g = group_same(&c, &all);
        assert_eq!((g.inputs, g.outputs), (2, 2));
        assert_eq!(
            g.outer
                .components
                .iter()
                .filter(|k| k.kind.is_input() || k.kind.is_led())
                .count(),
            4
        );
    }

    #[test]
    fn part_of_a_chain() {
        // A → NOT → NOT → Y; zabalí se jen první NOT (výstup vede do hradla venku).
        // [en] A → NOT → NOT → Y; only the first NOT is grouped (its output goes to a gate outside).
        let mut c = Circuit::default();
        c.add_component(ComponentKind::Switch { on: false }, Point::new(0, 0));
        let n1 = c.add_component(ComponentKind::gate(GateKind::Not), Point::new(6, 0));
        c.add_component(ComponentKind::gate(GateKind::Not), Point::new(14, 0));
        c.add_component(ComponentKind::LED, Point::new(21, 0));
        c.add_wire(Point::new(2, 0), Point::new(3, 0));
        c.add_wire(Point::new(9, 0), Point::new(11, 0));
        c.add_wire(Point::new(17, 0), Point::new(19, 0));
        let g = group_same(&c, &[n1]);
        assert_eq!((g.inputs, g.outputs), (1, 1));
        let (_, outs) = g.inner.io();
        assert_eq!(outs[0].label, "Y1");
    }

    #[test]
    fn nothing_to_group() {
        let (c, _) = half_adder();
        let switches: Vec<CompId> = c
            .components
            .iter()
            .filter(|k| k.kind.is_input())
            .map(|k| k.id)
            .collect();
        assert_eq!(group(&c, &switches, 1).unwrap_err(), GroupError::Empty);
        assert!(stays_outside(&ComponentKind::DEFAULT_CLOCK));
        assert!(!stays_outside(&ComponentKind::Vcc));
    }

    #[test]
    fn dead_ends_are_pruned() {
        // Slepá větev (0, 0)–(4, 0)–(4, 4) vede jen k LED; odbočka (4, 4)–(4, 8)
        // není semínko, proto zůstane. Vodič k vývodu (8, 0) se nechá.
        // [en] The dead branch (0, 0)–(4, 0)–(4, 4) leads only to an LED; the branch
        // (4, 4)–(4, 8) is not a seed, so it stays. The wire to the pin (8, 0) is kept.
        let mut c = Circuit::default();
        c.add_wire(Point::new(0, 0), Point::new(4, 0));
        c.add_wire(Point::new(4, 0), Point::new(4, 4));
        c.add_component(ComponentKind::LED, Point::new(6, 4));
        c.add_wire(Point::new(4, 4), Point::new(4, 8));
        c.add_component(ComponentKind::LED, Point::new(10, 0));
        c.add_wire(Point::new(5, 0), Point::new(8, 0));
        prune_dead_ends(&mut c, &[Point::new(0, 0), Point::new(5, 0)]);
        let left: Vec<(Point, Point)> = c.wires.iter().map(|w| (w.a, w.b)).collect();
        assert_eq!(left, [(Point::new(4, 4), Point::new(4, 8))]);
    }
}
