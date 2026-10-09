//! design/11 §11.8 acceptance, the pure part: synthetic streams at 120 Hz through
//! `scroll::engine::step` (the simulated scroller is `sim`).

use super::sim::{FRAME, PAGE, Sim};
use crate::scroll::config::{ScrollRubberBand, ScrollScalar, ScrollSettings};
use crate::scroll::engine::{
    self, Engine, Flight, Kinetic, Motion, NotifyPhase, Physics, ScrollIn,
};
use crate::scroll::geom::{Dir, Elastic, Px, ScrollAxis, Scroller};
use crate::scroll::keys::LINE_PX;
use crate::scroll::latch::{Candidate, latch};
use crate::scroll::rubber::AtEdge;

#[test]
fn item_1_a_2000_px_s_flick_glides_501_px_and_ends_at_976_ms() {
    let mut sim = Sim::new(1000.0, 100_000.0, Elastic::Rigid);
    sim.began(Dir::Pos, AtEdge::Inside);
    sim.stroke(2000.0, 100);
    sim.ended();
    let (released, at) = (sim.offset(), sim.t);
    let v0 = sim.flight().v0;
    assert!((v0 - 2000.0).abs() < 1.0, "release velocity {v0}");
    sim.run(3.0);
    let travel = sim.offset() - released;
    assert!((476.0..=527.0).contains(&travel), "travel {travel}");
    let ended = sim.when(NotifyPhase::MomentumEnded).expect("MomentumEnded") - at;
    assert!((ended - 0.976).abs() <= FRAME, "MomentumEnded at {ended}");
}

#[test]
fn item_2_a_pause_before_the_lift_glides_nowhere() {
    let mut sim = Sim::new(1000.0, 100_000.0, Elastic::Rigid);
    sim.began(Dir::Pos, AtEdge::Inside);
    sim.stroke(2000.0, 100);
    sim.t += 0.060;
    sim.ended();
    assert_eq!(sim.engine, Engine::Idle);
    assert_eq!(sim.when(NotifyPhase::MomentumBegan), None);
}

/// A 2000 px/s glide, then a second 2000 px/s flick begun when it is at 1000 px/s; the second
/// begins with a `MayBegin` and a frame first, or straight away as a touchpad's does.
fn boosted(dir: Dir, may_begin: bool) -> f64 {
    let mut sim = Sim::new(1000.0, 100_000.0, Elastic::Rigid);
    sim.began(Dir::Pos, AtEdge::Inside);
    sim.stroke(2000.0, 100);
    sim.ended();
    let t_1000 = (2000f64.powf(0.3) - 1000f64.powf(0.3)) / 9.0;
    sim.t += t_1000;
    if may_begin {
        sim.feed(ScrollIn::Frame);
        sim.feed(ScrollIn::MayBegin);
    }
    sim.began(dir, AtEdge::Inside);
    sim.stroke(dir.sign() * 2000.0, 100);
    sim.ended();
    sim.flight().v0
}

#[test]
fn item_3_a_second_flick_boosts_only_in_the_same_direction() {
    // dir, MayBegin first, expected v0, tolerance
    let cases = [
        ("same direction", Dir::Pos, true, 3000.0, 60.0),
        (
            "same direction, a touchpad's began during the glide",
            Dir::Pos,
            false,
            3000.0,
            60.0,
        ),
        ("opposite", Dir::Neg, true, -2000.0, 40.0),
    ];
    for (name, dir, may_begin, want, tolerance) in cases {
        let got = boosted(dir, may_begin);
        assert!((got - want).abs() <= tolerance, "{name}: {got}");
    }
}

#[test]
fn item_4_touch_stops_the_glide_where_it_is() {
    let mut sim = Sim::new(1000.0, 100_000.0, Elastic::Rigid);
    sim.began(Dir::Pos, AtEdge::Inside);
    sim.stroke(2000.0, 100);
    sim.ended();
    sim.run(0.2);
    let before = sim.offset();
    sim.feed(ScrollIn::MayBegin);
    for _ in 0..2 {
        sim.t += FRAME;
        sim.feed(ScrollIn::Frame);
        assert_eq!(sim.offset(), before);
    }
    assert_eq!(engine::motion(&sim.engine), Motion::Still);
}

/// At the top, a gesture begun inside at 50 px pulls `over` px past the edge.
fn stretch(over: f64, model: ScrollRubberBand) -> f64 {
    let scroll = ScrollSettings {
        rubber_band: model,
        ..ScrollSettings::default()
    };
    let mut sim = Sim::with(50.0, 5000.0, Elastic::Elastic, scroll);
    sim.began(Dir::Neg, AtEdge::Inside);
    sim.pull(-50.0 - over);
    -sim.offset()
}

#[test]
fn item_5_the_stretch_under_the_fingers() {
    let cases = [
        ("bounded 100", 100.0, ScrollRubberBand::Bounded, 52.1),
        ("bounded 1000", 1000.0, ScrollRubberBand::Bounded, 354.8),
        ("linear 100", 100.0, ScrollRubberBand::Linear, 5.0),
        ("linear 1000", 1000.0, ScrollRubberBand::Linear, 50.0),
    ];
    for (name, over, model, want) in cases {
        let s = stretch(over, model);
        assert!((s - want).abs() <= 1.0, "{name}: {s}");
    }
    // Began at the edge: the first 10 px do not stretch.
    let mut sim = Sim::new(0.0, 5000.0, Elastic::Elastic);
    let chain = [Candidate {
        scroller: Scroller::Viewport,
        geom: sim.geom,
    }];
    let latched = latch(&chain, ScrollAxis::Y, Dir::Neg).expect("the elastic page");
    assert_eq!(latched.began, AtEdge::AtEdge);
    sim.feed(ScrollIn::Began {
        latched,
        dir: Dir::Neg,
        kinetic: Kinetic::Momentum,
    });
    sim.pull(-10.0);
    assert_eq!(sim.offset(), 0.0);
    sim.pull(-100.0);
    assert!((sim.offset() + 52.1).abs() <= 1.0, "{}", sim.offset());
}

#[test]
fn item_6_snap_back_from_100_px() {
    let mut sim = Sim::new(50.0, 5000.0, Elastic::Elastic);
    sim.began(Dir::Neg, AtEdge::Inside);
    // Pull until the stretch is exactly 100 px (O = 100 d / (0.55 (d - 100)) = 202.02).
    let over = 100.0 * 1000.0 / (0.55 * 900.0);
    sim.pull(-50.0 - over);
    assert!((sim.offset() + 100.0).abs() < 1e-6, "{}", sim.offset());
    sim.t += 0.060; // held still: v0 = 0
    sim.ended();
    let trace = sim.run(1.0);
    let settled = trace
        .iter()
        .find(|(_, x)| *x == 0.0)
        .expect("back at the edge")
        .0;
    assert!(settled <= 0.400, "at the edge at {settled}");
    let small = trace
        .iter()
        .find(|(_, x)| x.abs() < 1.0)
        .expect("under 1 px")
        .0;
    assert!((small - 0.368).abs() <= FRAME, "under 1 px at {small}");
    assert!(trace.windows(2).all(|w| w[1].1 >= w[0].1), "monotonic");
    assert_eq!(sim.offset(), 0.0, "exactly the edge");
}

#[test]
fn item_7_a_glide_into_the_edge_bounces_18_px() {
    let mut sim = Sim::new(5000.0, 5000.0, Elastic::Elastic);
    sim.engine = Engine::Momentum(Flight {
        latch: PAGE,
        v0: 2000.0,
        x0: 5000.0,
        t0: sim.now(),
    });
    let trace = sim.run(2.0);
    let (peak_t, peak) = trace
        .iter()
        .map(|(t, x)| (*t, x - 5000.0))
        .fold((0.0, 0.0), |best, p| if p.1 > best.1 { p } else { best });
    assert!((peak - 18.3).abs() <= 1.0, "peak {peak}");
    assert!((peak_t - 0.080).abs() <= FRAME, "peak at {peak_t}");
    let settled = sim.when(NotifyPhase::MomentumEnded).expect("settles");
    assert!(settled <= 0.460, "settled at {settled}");
    assert_eq!(sim.offset(), 5000.0);
}

#[test]
fn item_7_a_real_flick_meets_the_edge_at_its_glide_speed() {
    // Released at 3000 px/s with just enough room to slow to 2000 px/s by the edge.
    let glide = Physics::from_settings(&ScrollSettings::default()).glide;
    let t_hit = (3000f64.powf(0.3) - 2000f64.powf(0.3)) / 9.0;
    let room = glide.travel(3000.0, t_hit);
    let mut sim = Sim::new(1000.0, 1000.0 + 300.0 + room, Elastic::Elastic);
    sim.began(Dir::Pos, AtEdge::Inside);
    sim.stroke(3000.0, 100);
    sim.ended();
    let v0 = sim.flight().v0;
    assert!((v0 - 3000.0).abs() < 1.0, "{v0}");
    let max = sim.geom.max.0;
    let trace = sim.run(3.0);
    let peak = trace.iter().map(|(_, x)| x - max).fold(0.0, f64::max);
    assert!((peak - 18.3).abs() <= 1.0, "peak {peak}");
    assert_eq!(sim.offset(), max, "settles at the edge");
}

#[test]
fn item_8_a_rigid_scroller_never_stretches() {
    let mut sim = Sim::new(0.0, 800.0, Elastic::Rigid);
    sim.began(Dir::Neg, AtEdge::AtEdge);
    for _ in 0..50 {
        sim.pull(-10.0);
        assert_eq!(sim.offset(), 0.0);
    }
    sim.ended();
    sim.run(1.0);
    assert_eq!(sim.offset(), 0.0);
}

#[test]
fn item_11_a_held_arrow_ramps_then_springs() {
    let mut sim = Sim::new(0.0, 100_000.0, Elastic::Rigid);
    sim.feed(ScrollIn::Hold {
        latch: PAGE,
        step: Px(LINE_PX),
    });
    let trace = sim.run(1.0);
    let speed = |i: usize| (trace[i].1 - trace[i - 1].1) / FRAME;
    let at_ramp = trace
        .iter()
        .position(|(t, _)| *t >= 0.2 - 1e-9)
        .expect("0.2 s");
    let v = speed(at_ramp + 1);
    assert!((v - 1000.0).abs() < 1.0, "1000 px/s after 0.2 s: {v}");
    assert!(speed(at_ramp - 5) < 1000.0 - 100.0, "still ramping before");
    // Release after 1 s: target = x + 1000 x 0.1.
    let x = sim.offset();
    sim.feed(ScrollIn::Release);
    let target = x + 100.0;
    let trace = sim.run(2.0);
    for (t, got) in &trace {
        let want = target - 100.0 * (-10.0 * t).exp() * (8.660_254 * t).cos();
        if (got - target).abs() > 0.5 {
            assert!((got - want).abs() <= 1.0, "at {t}: {got} vs {want}");
        }
    }
    let (settled, _) = *trace.last().expect("frames");
    assert!(settled <= 0.600, "settled (engine idle) at {settled}");
    assert_eq!(sim.engine, Engine::Idle);
    assert_eq!(sim.offset(), target);
    assert!(
        trace.iter().any(|(_, x)| *x > target),
        "it overshoots, as a spring does"
    );
}

#[test]
fn a_wheel_detent_at_the_edge_never_stretches() {
    let mut edge = Sim::new(4990.0, 5000.0, Elastic::Elastic);
    edge.feed(ScrollIn::Step {
        latch: PAGE,
        by: Px(60.0),
    });
    let trace = edge.run(1.0);
    assert!(trace.iter().all(|(_, x)| *x <= 5000.0), "{trace:?}");
    assert_eq!(edge.offset(), 5000.0);
}

#[test]
fn item_14_the_engine_is_still_once_settled() {
    let mut sim = Sim::new(1000.0, 100_000.0, Elastic::Rigid);
    sim.began(Dir::Pos, AtEdge::Inside);
    sim.stroke(2000.0, 100);
    sim.ended();
    sim.run(3.0);
    assert_eq!(engine::motion(&sim.engine), Motion::Still);
    let settled = sim.offset();
    for _ in 0..600 {
        sim.t += FRAME;
        sim.feed(ScrollIn::Frame);
    }
    assert_eq!(sim.offset(), settled, "5 s of frames move nothing");
}

#[test]
fn a_fast_lift_glides_further_than_its_speed_says_and_a_slow_one_does_not() {
    // name, stroke speed px/s, travel as a multiple of the unaccelerated glide
    let cases = [("slow", 800.0, 1.0), ("fast", 4000.0, 2.0)];
    for (name, speed, times) in cases {
        let travel = |scroll: ScrollSettings| {
            let mut sim = Sim::with(1000.0, 100_000.0, Elastic::Rigid, scroll);
            sim.began(Dir::Pos, AtEdge::Inside);
            sim.stroke(speed, 100);
            sim.ended();
            let released = sim.offset();
            sim.run(5.0);
            sim.offset() - released
        };
        let plain = travel(ScrollSettings {
            fling_accel_max: ScrollScalar(1.0),
            ..ScrollSettings::default()
        });
        let gained = travel(ScrollSettings::default());
        // A glide's travel grows faster than its speed (v^1.3), so at least `times` over.
        assert!(
            gained >= plain * times * 0.99,
            "{name}: {gained} vs {plain}"
        );
        if times == 1.0 {
            assert!((gained - plain).abs() < 1.0, "{name}: {gained} vs {plain}");
        }
    }
}
