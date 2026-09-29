//! Pixel snapping in a real document: at a fractional scale every box lands on the device grid,
//! and at a whole scale nothing moves.

mod support;

use blitz_kit::snap::snap_layout;
use blitz_kit::units::Scale120;

const BOX: &str = "position:absolute;left:11px;top:11px;width:10px;height:10px;\
border:1px solid black";

/// The box's (x, y, width, height, border-left) in device px at `scale`, before and after the
/// snap.
fn measure(scale: f32, snap_to: Scale120) -> ([f32; 5], [f32; 5]) {
    let (mut doc, body) = support::page(100, 100, scale);
    let node = support::div(&mut doc, body, "box", BOX);
    doc.resolve(0.0);
    let device = |doc: &blitz_dom::BaseDocument| {
        let layout = doc.get_node(node).expect("node").final_layout();
        [
            layout.location.x,
            layout.location.y,
            layout.size.width,
            layout.size.height,
            layout.border.left,
        ]
        .map(|logical| logical * scale)
    };
    let before = device(&doc);
    snap_layout(&mut doc, snap_to);
    (before, device(&doc))
}

fn on_grid(device: f32) -> bool {
    (device - device.round()).abs() < 1e-3
}

#[test]
fn at_a_fractional_scale_every_edge_lands_on_a_device_pixel() {
    let (before, after) = measure(1.5, Scale120(180));
    assert!(
        !before.iter().all(|&v| on_grid(v)),
        "the case needs Blitz's rounding off-grid: {before:?}"
    );
    assert!(after.iter().all(|&v| on_grid(v)), "{after:?}");
    assert!(after[4] >= 1.0, "a border never vanishes: {after:?}");
}

#[test]
fn at_a_whole_scale_nothing_moves() {
    let (before, after) = measure(2.0, Scale120(240));
    assert_eq!(before, after);
}
