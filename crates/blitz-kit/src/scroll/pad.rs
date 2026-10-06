//! A touchpad gesture from a host's pointer-scroll events (design/11 §11.3.2, §11.3.9,
//! §11.3.13): the first pan is Began once its axis locks, the host's stop is Ended. A host maps
//! its own device events (Wayland axis frames, winit's `MouseWheel`) to a [`PointerScroll`]; the
//! machine here is the same for all of them. Pure.

use crate::scroll::config::NaturalScroll;
use crate::scroll::engine::Kinetic;
use crate::scroll::geom::{Dir, ScrollAxis};
use crate::scroll::latch::AxisLock;

/// What one pointer-scroll event means for scrolling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerScroll {
    /// Wheel detents on `axis` (signed, 1.0 per click; positive toward the maximum).
    Detents { axis: ScrollAxis, steps: f64 },
    /// A continuous pan in offset px.
    Pan { dx: f64, dy: f64, kinetic: Kinetic },
    /// The fingers lifted.
    Stop,
    /// Nothing that scrolls.
    Nothing,
}

/// A touchpad gesture's phase, from its pans and stop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PadEvent {
    /// The axis locked: latch toward `dir`, then apply `delta`.
    Began {
        axis: ScrollAxis,
        dir: Dir,
        delta: f64,
        kinetic: Kinetic,
    },
    Changed {
        delta: f64,
    },
    Ended,
}

/// A touchpad gesture (the first pan is Began, a stop is Ended; locked to the larger axis at
/// 10 px).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pad {
    Idle,
    Moving { lock: AxisLock, kinetic: Kinetic },
}

impl Pad {
    /// Feed one classified event.
    pub fn feed(self, input: PointerScroll) -> (Pad, Option<PadEvent>) {
        match (self, input) {
            (Pad::Idle, PointerScroll::Pan { dx, dy, kinetic }) => Pad::Moving {
                lock: AxisLock::START,
                kinetic,
            }
            .feed(PointerScroll::Pan { dx, dy, kinetic }),
            (Pad::Moving { lock, kinetic }, PointerScroll::Pan { dx, dy, .. }) => match lock {
                AxisLock::Locked(axis) => {
                    let delta = match axis {
                        ScrollAxis::X => dx,
                        ScrollAxis::Y => dy,
                    };
                    (self, (delta != 0.0).then_some(PadEvent::Changed { delta }))
                }
                AxisLock::Free { .. } => {
                    let (lock, pending) = lock.feed(dx, dy);
                    let next = Pad::Moving { lock, kinetic };
                    let event = match (lock, pending) {
                        (AxisLock::Locked(axis), Some(delta)) => {
                            Dir::of(delta).map(|dir| PadEvent::Began {
                                axis,
                                dir,
                                delta,
                                kinetic,
                            })
                        }
                        _ => None,
                    };
                    (next, event)
                }
            },
            (Pad::Moving { lock, .. }, PointerScroll::Stop) => {
                let event = matches!(lock, AxisLock::Locked(_)).then_some(PadEvent::Ended);
                (Pad::Idle, event)
            }
            (state, _) => (state, None),
        }
    }
}

/// A finger-direction delta as an offset delta: with `Natural` the content follows the finger,
/// so a finger moving toward the maximum's side moves the offset the other way.
pub fn natural(finger: f64, setting: NaturalScroll) -> f64 {
    match setting {
        NaturalScroll::Natural => -finger,
        NaturalScroll::Traditional => finger,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_touchpad_stroke_is_began_changed_ended() {
        let pan = |dx, dy| PointerScroll::Pan {
            dx,
            dy,
            kinetic: Kinetic::Momentum,
        };
        let (pad, e) = Pad::Idle.feed(pan(0.0, 4.0));
        assert_eq!(e, None);
        let (pad, e) = pad.feed(pan(1.0, 7.0));
        assert_eq!(
            e,
            Some(PadEvent::Began {
                axis: ScrollAxis::Y,
                dir: Dir::Pos,
                delta: 11.0,
                kinetic: Kinetic::Momentum
            })
        );
        let (pad, e) = pad.feed(pan(30.0, 5.0));
        assert_eq!(
            e,
            Some(PadEvent::Changed { delta: 5.0 }),
            "locked: x ignored"
        );
        let (pad, e) = pad.feed(PointerScroll::Stop);
        assert_eq!((pad, e), (Pad::Idle, Some(PadEvent::Ended)));
        let (pad, e) = Pad::Idle.feed(pan(0.0, 2.0)).0.feed(PointerScroll::Stop);
        assert_eq!(
            (pad, e),
            (Pad::Idle, None),
            "a stroke that never locked ends silently"
        );
    }

    #[test]
    fn natural_scrolling_of_11_8_item_15() {
        // A finger moving toward the user (+y) decreases the offset with Natural.
        assert_eq!(natural(10.0, NaturalScroll::Natural), -10.0);
        assert_eq!(natural(10.0, NaturalScroll::Traditional), 10.0);
    }
}
