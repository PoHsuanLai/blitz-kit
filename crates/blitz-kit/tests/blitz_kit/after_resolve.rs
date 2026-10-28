//! Readings registered with `AfterResolve` run on the resolved document, publish only when what
//! they read changed, and stop when their registration is dropped.

use crate::support;

use blitz_kit::after_resolve::{AfterResolve, publish};
use std::cell::Cell;
use std::rc::Rc;

/// A reading of the node's laid-out width that publishes it into `seen`.
fn width_into(seen: &Rc<Cell<Option<f32>>>) -> blitz_kit::after_resolve::Read {
    let seen = Rc::clone(seen);
    let mut last = None;
    Box::new(move |doc, node| {
        let now = doc
            .get_node(node)
            .map(|node| node.final_layout().size.width);
        if now == last {
            return None;
        }
        last = now;
        let seen = Rc::clone(&seen);
        Some(Box::new(move || {
            seen.set(now);
            true
        }))
    })
}

#[test]
fn a_reading_publishes_only_what_changed_and_ends_with_its_registration() {
    let (mut doc, body) = support::page(200, 100, 1.0);
    let card = support::div(&mut doc, body, "card", "width:40px;height:10px");
    doc.resolve(0.0);
    let readings = AfterResolve::default();
    let seen = Rc::new(Cell::new(None));
    let registration = readings.watch(card, width_into(&seen));

    assert_eq!(
        publish(readings.read(&doc)),
        1,
        "the first reading publishes"
    );
    assert_eq!(seen.get(), Some(40.0));
    assert!(
        readings.read(&doc).is_empty(),
        "an unchanged reading is quiet"
    );

    support::set_style(&mut doc, card, "width:64px;height:10px");
    doc.resolve(0.0);
    assert_eq!(publish(readings.read(&doc)), 1, "a restyle publishes again");
    assert_eq!(seen.get(), Some(64.0));

    drop(registration);
    assert!(readings.is_empty());
    assert!(readings.read(&doc).is_empty());
}
