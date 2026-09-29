//! Snapping a laid-out document to the device pixel grid (FINDINGS "Pixel snapping").
//!
//! Blitz lays out in logical pixels and then rounds every box to whole *logical* pixels
//! (`taffy::round_layout`, called at the end of `BaseDocument::resolve_layout`). At a whole
//! scale that is the device grid too. At 1.25, 1.5 or 1.75 it is not: a box at logical y 11 starts
//! at device y 16.5, and a border that stylo had already snapped to one device pixel (0.667
//! logical px at 1.5) is rounded back up to a whole logical pixel, 1.5 device pixels. Either
//! way the line paints one full row and one half row.
//!
//! [`snap_layout`] redoes that rounding on the device grid: the same cumulative rounding
//! taffy does (so neighbours still abut exactly), in units of one device pixel, with border
//! widths rounded on their own to whole device pixels and never below one. It then re-derives
//! what Blitz computed from the logical rounding (each node's transform, whose percentages read
//! the box size, and its scrollable overflow), and rounds a pure translation to whole device
//! pixels, so a `translateX(-50%)` of an odd width does not undo it. Hit testing and rect reads
//! use the same final layout, so they agree with the picture.
//!
//! It must run after every `resolve` and before painting, because every resolve re-runs taffy's
//! rounding. A consumer runs it after each `resolve` (a host's pre-paint hook, a headless
//! harness). At a whole scale it does
//! nothing: Blitz's rounding is already right there, and every picture stays as it was.

mod grid;
mod place;
mod settle;

#[cfg(test)]
mod tests;

use blitz_dom::BaseDocument;

use crate::units::Scale120;
use grid::DeviceGrid;

/// Snap every box of `doc` to the device pixel grid at `scale`; nothing at a whole scale.
pub fn snap_layout(doc: &mut BaseDocument, scale: Scale120) {
    if scale.is_whole() {
        return;
    }
    let grid = DeviceGrid(scale.factor());
    let root = doc.root_element().id;
    place::place(doc, root, grid::Origin::default(), grid);
    settle::settle(doc, root, grid);
}
