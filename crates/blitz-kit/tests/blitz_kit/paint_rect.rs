//! An element's painted rectangle in a real document: its layout box carried through its own
//! and its ancestors' CSS transforms, at whole and fractional scales.

use crate::support;

use blitz_kit::paint_rect::painted_rect;
use blitz_kit::units::Bounds;

/// A 100 x 40 card inside a 200 x 100 wrapper, the wrapper's `transform` set to `wrapper`.
fn card_under(wrapper: &str, scale: f32) -> Option<Bounds> {
    let (mut doc, body) = support::page(400, 300, scale);
    let outer = support::div(
        &mut doc,
        body,
        "wrap",
        &format!("width:200px;height:100px;transform:{wrapper}"),
    );
    let card = support::div(&mut doc, outer, "card", "width:100px;height:40px");
    doc.resolve(0.0);
    painted_rect(&doc, card)
}

const fn at(x: f64, y: f64, width: f64, height: f64) -> Bounds {
    Bounds {
        x,
        y,
        width,
        height,
    }
}

#[test]
fn a_parents_transform_moves_the_childs_painted_rect() {
    let cases: &[(&str, &str, f32, Bounds)] = &[
        ("none", "none", 1.0, at(0.0, 0.0, 100.0, 40.0)),
        (
            "translate",
            "translateX(60px)",
            1.0,
            at(60.0, 0.0, 100.0, 40.0),
        ),
        (
            "translate at 1.5x is still CSS px",
            "translateX(60px)",
            1.5,
            at(60.0, 0.0, 100.0, 40.0),
        ),
        (
            "scale about the wrapper's centre",
            "scale(0.5)",
            1.0,
            at(50.0, 25.0, 50.0, 20.0),
        ),
    ];
    for &(name, transform, scale, want) in cases {
        let got = card_under(transform, scale).expect("laid out");
        let near = |a: f64, b: f64| (a - b).abs() < 0.01;
        assert!(
            near(got.x, want.x)
                && near(got.y, want.y)
                && near(got.width, want.width)
                && near(got.height, want.height),
            "{name}: {got:?} != {want:?}"
        );
    }
}
