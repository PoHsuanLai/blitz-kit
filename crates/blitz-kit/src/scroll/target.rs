//! The smooth step for a consumer that owns its own offset (a canvas, a paged viewer) instead of
//! a Blitz scroller: the same rule as the engine's wheel and key steps (design/11 §11.3.11,
//! §11.5.4) as a value that hands out how far to move since the last frame.
//!
//! [`Target::push`] adds a step to what is still undelivered and restarts from where the offset
//! is now, so detents arriving during the animation accumulate; [`Target::advance`] returns the
//! distance to move at a frame. The consumer applies and clamps the distance itself. Pure.

use crate::scroll::engine::Motion;
use crate::scroll::smooth;
use crate::scroll::time::Elapsed;

/// A step in progress: `total` px to deliver along the curve from `t0`, `delivered` of them so far.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Step {
    total: f64,
    delivered: f64,
    t0: Elapsed,
    dur: f64,
}

/// One axis of a consumer's smooth scrolling.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Target {
    step: Option<Step>,
}

/// Seconds from `t0` to `now`.
fn since(t0: Elapsed, now: Elapsed) -> f64 {
    now.0.as_secs_f64() - t0.0.as_secs_f64()
}

impl Target {
    /// Add a step of `by` px (signed) at `now`: the undelivered rest of the last step plus `by`,
    /// eased from here.
    pub fn push(self, by: f64, now: Elapsed) -> Target {
        let rest = self.step.map_or(0.0, |s| s.total - s.delivered);
        let total = rest + by;
        let dur = smooth::duration(total);
        if dur <= 0.0 {
            return Target { step: None };
        }
        Target {
            step: Some(Step {
                total,
                delivered: 0.0,
                t0: now,
                dur,
            }),
        }
    }

    /// The distance to move at `now` since the last call, and the target after it.
    pub fn advance(self, now: Elapsed) -> (Target, f64) {
        let Some(step) = self.step else {
            return (self, 0.0);
        };
        let p = since(step.t0, now) / step.dur;
        if p >= 1.0 {
            return (Target { step: None }, step.total - step.delivered);
        }
        let position = step.total * smooth::e_out(p);
        let next = Step {
            delivered: position,
            ..step
        };
        (Target { step: Some(next) }, position - step.delivered)
    }

    /// Whether a frame is wanted.
    pub fn motion(&self) -> Motion {
        match self.step {
            Some(_) => Motion::Animating,
            None => Motion::Still,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn at(ms: u64) -> Elapsed {
        Elapsed(Duration::from_millis(ms))
    }

    /// Frames every 8 ms from `from` to `to` (ms), the distances handed out, summed.
    fn run(mut target: Target, from: u64, to: u64) -> (Target, f64) {
        let mut sum = 0.0;
        for ms in (from..=to).step_by(8) {
            let (next, by) = target.advance(at(ms));
            target = next;
            sum += by;
        }
        (target, sum)
    }

    #[test]
    fn one_detent_delivers_60_px_in_60_ms_by_frames() {
        let target = Target::default().push(60.0, at(0));
        let (_, first) = target.advance(at(8));
        assert!(
            first > 0.0 && first < 60.0,
            "an intermediate frame: {first}"
        );
        let (done, sum) = run(target, 8, 80);
        assert!((sum - 60.0).abs() < 1e-9, "{sum}");
        assert_eq!(done.motion(), Motion::Still);
    }

    #[test]
    fn detents_during_the_animation_accumulate_on_the_target() {
        // (name, pushes as (ms, px), frames until, total)
        let cases = [
            (
                "three rapid detents",
                vec![(0, 60.0), (10, 60.0), (20, 60.0)],
                300,
                180.0,
            ),
            ("reversed detent", vec![(0, 60.0), (10, -60.0)], 300, 0.0),
            ("a page", vec![(0, 860.0)], 300, 860.0),
        ];
        for (name, pushes, until, total) in cases {
            let mut target = Target::default();
            let mut sum = 0.0;
            let mut ms = 0;
            for (when, px) in pushes {
                while ms < when {
                    let (next, by) = target.advance(at(ms));
                    target = next;
                    sum += by;
                    ms += 8;
                }
                target = target.push(px, at(when));
            }
            let (target, rest) = run(target, ms, until);
            assert!((sum + rest - total).abs() < 1e-9, "{name}: {}", sum + rest);
            assert_eq!(target.motion(), Motion::Still, "{name}");
        }
    }

    #[test]
    fn a_step_is_capped_at_200_ms() {
        let target = Target::default().push(5000.0, at(0));
        let (_, before) = target.advance(at(199));
        let (done, whole) = target.advance(at(200));
        assert!(before > 4900.0 && before < 5000.0, "{before}");
        assert_eq!(whole, 5000.0);
        assert_eq!(done.motion(), Motion::Still);
    }

    #[test]
    fn nothing_pushed_hands_out_nothing() {
        let (target, by) = Target::default().advance(at(10));
        assert_eq!((target, by), (Target::default(), 0.0));
    }
}
