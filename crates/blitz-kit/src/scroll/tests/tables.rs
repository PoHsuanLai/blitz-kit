//! design/11's numbers as tables, each row computed from the design's formulas by hand: the
//! wheel's detents, the glide's decay, the rubber band's stretch and snap-back, the keys' steps.
//! Every row names its case, and a failure names the row.

use super::sim::{FRAME, PAGE, Sim};
use crate::scroll::config::{ScrollRubberBand, ScrollSettings};
use crate::scroll::engine::{Physics, ScrollAnimate, ScrollIn};
use crate::scroll::geom::{Elastic, Px};
use crate::scroll::keys;
use crate::scroll::rubber::AtEdge;

/// A burst of detents: `(ms, px)` pushes, the distance moved, and the latest time it is there.
type Burst<'a> = (&'a str, &'a [(u32, f64)], f64, f64);

/// A detent, px.
const DETENT: f64 = 60.0;

/// Whether `got` is within `tolerance` of `want`.
fn near(got: f64, want: f64, tolerance: f64) -> bool {
    (got - want).abs() <= tolerance
}

/// Push each `(ms, px)` step in order, rendering a frame every 10 ms between them as a window
/// does, then run until the engine stops: the final offset and the time it was reached, seconds
/// from the first push.
fn steps(pushes: &[(u32, f64)]) -> (f64, f64) {
    let mut sim = Sim::new(1000.0, 100_000.0, Elastic::Elastic);
    let mut at = 0;
    for (ms, by) in pushes {
        while at < *ms {
            sim.t += 0.010;
            sim.feed(ScrollIn::Frame);
            at += 10;
        }
        sim.feed(ScrollIn::Step {
            latch: PAGE,
            by: Px(*by),
        });
    }
    let trace = sim.run(2.0);
    let (end, _) = *trace.last().expect("an animation ran");
    (sim.offset() - 1000.0, f64::from(at) / 1000.0 + end)
}

#[test]
fn the_wheels_detents_accumulate_and_reverse() {
    // name, steps as (ms, px), distance moved, latest time it is there (s, from the first push)
    let cases: &[Burst] = &[
        ("one detent", &[(0, DETENT)], 60.0, 0.060 + FRAME),
        (
            "three rapid detents",
            &[(0, DETENT), (10, DETENT), (20, DETENT)],
            180.0,
            0.200 + FRAME,
        ),
        (
            "five detents within 50 ms",
            &[
                (0, DETENT),
                (10, DETENT),
                (20, DETENT),
                (30, DETENT),
                (40, DETENT),
            ],
            300.0,
            0.250 + FRAME,
        ),
        (
            "a reversed detent brings it back",
            &[(0, DETENT), (30, -DETENT)],
            0.0,
            0.100 + FRAME,
        ),
        (
            "a reversed detent after two",
            &[(0, DETENT), (10, DETENT), (30, -DETENT)],
            60.0,
            0.150 + FRAME,
        ),
        (
            "a slow wheel animates each detent alone",
            &[(0, DETENT), (400, DETENT)],
            120.0,
            0.460 + FRAME,
        ),
    ];
    for (name, pushes, distance, by) in cases {
        let (moved, done) = steps(pushes);
        assert!(near(moved, *distance, 1e-9), "{name}: moved {moved}");
        assert!(done <= *by, "{name}: done at {done}, wanted by {by}");
    }
}

#[test]
fn a_detent_arrives_over_frames_and_not_at_once() {
    let mut sim = Sim::new(0.0, 5000.0, Elastic::Rigid);
    sim.feed(ScrollIn::Step {
        latch: PAGE,
        by: Px(DETENT),
    });
    let trace = sim.run(1.0);
    let before_the_end = &trace[..trace.len() - 1];
    assert!(
        before_the_end.len() >= 5,
        "60 ms is at least five frames at 120 Hz: {trace:?}"
    );
    assert!(
        before_the_end.iter().all(|(_, x)| *x < DETENT),
        "short of 60 px until the last frame"
    );
    assert!(
        before_the_end.windows(2).all(|pair| pair[1].1 > pair[0].1),
        "and moving each frame"
    );
}

#[test]
fn the_glides_speed_decays_along_the_drag_curve() {
    let glide = Physics::from_settings(&ScrollSettings::default()).glide;
    // name, v0 px/s, t s, speed px/s: (v0^0.3 - 9 t)^(10/3) worked by hand
    let cases = [
        ("at release", 2000.0, 0.0, 2000.0),
        ("a quarter second", 2000.0, 0.25, 836.605),
        ("half a second", 2000.0, 0.5, 256.21),
        ("three quarters", 2000.0, 0.75, 40.224),
        ("nearly stopped", 2000.0, 0.9, 5.629),
        ("a slower flick", 1000.0, 0.4, 133.678),
        ("a faster flick", 5000.0, 0.5, 1192.142),
        ("past its end", 2000.0, 1.5, 0.0),
    ];
    for (name, v0, t, want) in cases {
        let got = glide.speed(v0, t);
        assert!(near(got, want, 0.01 * want.max(1.0)), "{name}: {got}");
    }
}

#[test]
fn the_bands_stretch_follows_the_bounded_curve() {
    let bounded = Physics::from_settings(&ScrollSettings::default()).band;
    let linear = Physics::from_settings(&ScrollSettings {
        rubber_band: ScrollRubberBand::Linear,
        ..ScrollSettings::default()
    })
    .band;
    let off = Physics::from_settings(&ScrollSettings {
        rubber_band: ScrollRubberBand::Off,
        ..ScrollSettings::default()
    })
    .band;
    // name, band, overscroll px, viewport px, began, stretch px
    let cases = [
        ("none", bounded, 0.0, 1000.0, AtEdge::Inside, 0.0),
        ("a little", bounded, 10.0, 1000.0, AtEdge::Inside, 5.47),
        ("100 px", bounded, 100.0, 1000.0, AtEdge::Inside, 52.13),
        ("300 px", bounded, 300.0, 1000.0, AtEdge::Inside, 141.63),
        (
            "a full view",
            bounded,
            1000.0,
            1000.0,
            AtEdge::Inside,
            354.84,
        ),
        ("a short view", bounded, 100.0, 500.0, AtEdge::Inside, 49.55),
        (
            "the first 10 px at the edge",
            bounded,
            10.0,
            1000.0,
            AtEdge::AtEdge,
            0.0,
        ),
        (
            "linear, 1000 px",
            linear,
            1000.0,
            1000.0,
            AtEdge::Inside,
            50.0,
        ),
        ("linear, 100 px", linear, 100.0, 1000.0, AtEdge::Inside, 5.0),
        ("off", off, 1000.0, 1000.0, AtEdge::Inside, 0.0),
    ];
    for (name, band, over, view, began, want) in cases {
        let got = band.stretch(over, view, began);
        assert!(near(got, want, 0.01), "{name}: {got}");
    }
}

#[test]
fn the_stretch_springs_back_exponentially() {
    let snap = Physics::from_settings(&ScrollSettings::default()).snap;
    // name, stretch at release px, speed outward px/s, t s, stretch px: (s0 + 0.31 v0 t) e^(-12.5 t)
    let cases = [
        ("at release", 100.0, 0.0, 0.0, 100.0),
        ("50 ms", 100.0, 0.0, 0.05, 53.53),
        ("100 ms", 100.0, 0.0, 0.1, 28.65),
        ("200 ms", 100.0, 0.0, 0.2, 8.21),
        ("300 ms", 100.0, 0.0, 0.3, 2.35),
        ("an impact at 2000 px/s, 40 ms in", 0.0, 2000.0, 0.04, 15.04),
        ("its peak, 80 ms in", 0.0, 2000.0, 0.08, 18.25),
        ("and 200 ms in", 0.0, 2000.0, 0.2, 10.18),
    ];
    for (name, s0, v0, t, want) in cases {
        let got = snap.stretch(s0, v0, t);
        assert!(near(got, want, 0.01), "{name}: {got}");
    }
}

#[test]
fn a_page_key_moves_by_the_page_rule_in_up_to_200_ms() {
    // name, viewport px, distance px: max(0.8 v, v - 40)
    let cases = [
        ("a tiny view keeps the ratio", 50.0, 40.0),
        ("a small view", 100.0, 80.0),
        ("the ratio and the overlap meet at 200", 200.0, 160.0),
        ("a tall view keeps 40 px", 900.0, 860.0),
        ("a full screen", 1080.0, 1040.0),
    ];
    for (name, view, distance) in cases {
        assert_eq!(keys::page(view), distance, "{name}: the rule");
        let mut sim = Sim::new(0.0, 50_000.0, Elastic::Rigid);
        sim.geom.viewport = Px(view);
        sim.feed(ScrollIn::Step {
            latch: PAGE,
            by: Px(keys::page(view)),
        });
        let trace = sim.run(1.0);
        let (done, _) = *trace
            .iter()
            .find(|(_, x)| *x == distance)
            .unwrap_or_else(|| panic!("{name}: never reaches {distance}: {trace:?}"));
        let by = (distance / 1000.0).min(0.2) + FRAME;
        assert!(done <= by, "{name}: done at {done}, wanted by {by}");
    }
}

#[test]
fn the_keys_steps_are_lines_pages_and_edges() {
    // name, scroller length px, the step, distance px from the start at 1000, time bound s
    let cases = [
        (
            "an arrow is a line",
            5000.0,
            ScrollIn::Step {
                latch: PAGE,
                by: Px(keys::LINE_PX),
            },
            40.0,
            0.040 + FRAME,
        ),
        (
            "an arrow up",
            5000.0,
            ScrollIn::Step {
                latch: PAGE,
                by: Px(-keys::LINE_PX),
            },
            -40.0,
            0.040 + FRAME,
        ),
        (
            "Home",
            5000.0,
            ScrollIn::Jump {
                latch: PAGE,
                to: Px(0.0),
                animate: ScrollAnimate::Smooth,
            },
            -1000.0,
            0.200 + FRAME,
        ),
        (
            "End",
            5000.0,
            ScrollIn::Jump {
                latch: PAGE,
                to: Px(5000.0),
                animate: ScrollAnimate::Smooth,
            },
            4000.0,
            0.200 + FRAME,
        ),
        (
            "End from far away",
            50_000.0,
            ScrollIn::Jump {
                latch: PAGE,
                to: Px(50_000.0),
                animate: ScrollAnimate::Smooth,
            },
            49_000.0,
            0.200 + FRAME,
        ),
    ];
    for (name, max, step, distance, by) in cases {
        let mut sim = Sim::new(1000.0, max, Elastic::Rigid);
        sim.feed(step);
        let trace = sim.run(1.0);
        let (done, _) = *trace.last().expect("an animation ran");
        assert!(near(sim.offset() - 1000.0, distance, 1e-9), "{name}");
        assert!(done <= by, "{name}: done at {done}, wanted by {by}");
    }
}
