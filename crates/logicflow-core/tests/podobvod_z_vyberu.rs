//! Vytvoření podobvodu z výběru na zapojeních výukové knihovny: pravdivostní
//! tabulka se zabalením nesmí změnit a všechny vývody se mají připojit.
//!
//! [en] Creating a subcircuit from the selection on the circuits of the teaching
//! library: grouping must not change the truth table and all pins should get connected.

use logicflow_core::group::group;
use logicflow_core::{
    Circuit, CompId, ComponentKind, Library, LogicValue, Simulator, SubcircuitDef,
};

/// Výuková knihovna „Číslicová technika“.
///
/// [en] The teaching library “Číslicová technika”.
const LIBRARY: &str = include_str!("../../../docs/knihovny/cislicova-technika.lflib");

/// Hodnoty LED pro všechny kombinace vstupů.
///
/// [en] LED values for all input combinations.
fn truth(c: &Circuit, defs: &[SubcircuitDef]) -> Vec<Vec<LogicValue>> {
    let (ins, outs) = c.io();
    let ins: Vec<CompId> = ins.iter().map(|k| k.id).collect();
    let outs: Vec<CompId> = outs.iter().map(|k| k.id).collect();
    (0..(1u32 << ins.len()))
        .map(|bits| {
            let mut sim = Simulator::with_subcircuits(c, defs, None);
            for (i, id) in ins.iter().enumerate() {
                sim.set_source(*id, (bits >> i & 1 == 1).into());
            }
            sim.run(200_000);
            outs.iter().map(|id| sim.inputs(*id)[0]).collect()
        })
        .collect()
}

/// Je zapojení kombinační (bez klopných obvodů, hodin a sekvenčních bloků)?
///
/// [en] Is the circuit combinational (no flip-flops, clocks or sequential blocks)?
fn combinational(c: &Circuit, defs: &[SubcircuitDef]) -> bool {
    c.components.iter().all(|k| match k.kind {
        ComponentKind::FlipFlop { .. } | ComponentKind::Clock { .. } => false,
        ComponentKind::Block { block } => !block.is_sequential(),
        ComponentKind::Subcircuit { id, .. } => defs
            .iter()
            .find(|d| d.id == id)
            .is_some_and(|d| combinational(&d.circuit, defs)),
        _ => true,
    })
}

#[test]
fn grouping_keeps_truth_tables() {
    let lib = Library::from_json(LIBRARY).unwrap();
    let defs = lib.defs();
    let new_id = defs.iter().map(|d| d.id).max().unwrap() + 1;
    let mut checked = 0;
    let mut free = 0;
    for d in &defs {
        let c = &d.circuit;
        if !combinational(c, &defs) || c.io().0.len() > 6 {
            continue;
        }
        let before = truth(c, &defs);
        let movable: Vec<CompId> = c
            .components
            .iter()
            .filter(|k| !logicflow_core::group::stays_outside(&k.kind))
            .map(|k| k.id)
            .collect();
        // Celé zapojení a každá druhá součástka (víc sítí přes hranici výběru).
        // [en] The whole circuit and every second component (more nets across the selection boundary).
        let half: Vec<CompId> = movable.iter().copied().step_by(2).collect();
        for sel in [movable.clone(), half] {
            let g = group(c, &sel, new_id).unwrap();
            let mut all = defs.clone();
            all.push(SubcircuitDef {
                id: new_id,
                name: "P".into(),
                circuit: g.inner.clone(),
            });
            if g.unconnected == 0 {
                assert_eq!(truth(&g.outer, &all), before, "{}", d.name);
            } else {
                free += g.unconnected;
                eprintln!("{}: {} nepřipojených vývodů", d.name, g.unconnected);
            }
            // Uvnitř je zapojení samo o sobě funkční: stejný počet vývodů jako rozhraní.
            // [en] The circuit inside works on its own: as many pins as the interface.
            let (i, o) = g.inner.io();
            assert_eq!((i.len(), o.len()), (g.inputs, g.outputs), "{}", d.name);
            checked += 1;
        }
    }
    eprintln!("ověřeno {checked} zabalení, nepřipojeno {free}");
    assert!(checked >= 20);
    assert_eq!(free, 0, "všechny vývody se mají připojit");
}
