//! Acceleration (the pointer-speed curves libinput and macOS apply, here on scrolling). A wheel
//! click is a click while the wheel turns at a reading pace, and each click carries further the
//! faster the wheel spins, so a flick of the wheel crosses a long page without a hundred turns:
//! the rate is detents per second, smoothed over the burst, and a pause ends the burst. Fingers
//! are tracked 1:1 whatever their speed (the content follows them), and a fast lift carries the
//! glide further instead ([`fling_gain`]). Pure.

use crate::scroll::time::Elapsed;

/// A gap this long (seconds) ends a burst: the next event is a first one.
const BURST_END_S: f64 = 0.25;
/// Events closer than this (seconds) count as this far apart, so a coalesced pair does not
/// read as an infinite rate.
const MIN_GAP_S: f64 = 0.004;
/// How much of the smoothed rate each event replaces.
const SMOOTHING: f64 = 0.4;

/// Up to this many detents a second the wheel is read, not spun: gain 1.
const READING_RATE: f64 = 5.0;
/// Gain added per detent a second above `READING_RATE`.
const GAIN_PER_RATE: f64 = 0.25;
/// A lift slower than this (px/s) glides as it is.
const FLING_FLOOR: f64 = 1000.0;
/// Gain added per px/s of a lift above `FLING_FLOOR`.
const FLING_PER_SPEED: f64 = 1.0 / 3000.0;

/// How much further a lift at `speed` px/s (a magnitude) glides than its own speed says: 1 up to
/// 1000 px/s, then one more for each 3000 px/s above, never over `max` (`max <= 1` is off).
pub fn fling_gain(speed: f64, max: f64) -> f64 {
    (1.0 + (speed - FLING_FLOOR) * FLING_PER_SPEED).clamp(1.0, max.max(1.0))
}

/// The state of a wheel's burst: when it last turned and how fast, smoothed.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Accel {
    last: Option<Elapsed>,
    rate: f64,
}

impl Accel {
    /// A turn of `steps` detents (a magnitude; a high-resolution wheel reports fractions) at
    /// `now`, and the factor to scale it by: 1 at a reading pace, a quarter more for each
    /// detent a second above 5, never over `max` (`max <= 1` is off).
    pub fn feed(self, steps: f64, now: Elapsed, max: f64) -> (Accel, f64) {
        let rate = match self.last {
            Some(last) => {
                let gap = now.0.as_secs_f64() - last.0.as_secs_f64();
                if gap > BURST_END_S {
                    0.0
                } else {
                    let instant = steps.abs() / gap.max(MIN_GAP_S);
                    self.rate * (1.0 - SMOOTHING) + instant * SMOOTHING
                }
            }
            None => 0.0,
        };
        let accel = Accel {
            last: Some(now),
            rate,
        };
        let gain = (1.0 + (rate - READING_RATE) * GAIN_PER_RATE).clamp(1.0, max.max(1.0));
        (accel, gain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const MAX: f64 = 5.0;

    fn at(ms: u64) -> Elapsed {
        Elapsed(Duration::from_millis(ms))
    }

    /// The gains of `count` clicks of `steps` each, `gap_ms` apart.
    fn burst(count: u64, steps: f64, gap_ms: u64) -> Vec<f64> {
        let mut accel = Accel::default();
        (0..count)
            .map(|n| {
                let (next, gain) = accel.feed(steps, at(n * gap_ms), MAX);
                accel = next;
                gain
            })
            .collect()
    }

    #[test]
    fn a_first_click_and_a_reading_pace_are_not_accelerated() {
        assert_eq!(burst(1, 1.0, 0), vec![1.0]);
        // 4 clicks a second.
        assert!(burst(8, 1.0, 250).iter().all(|g| *g == 1.0));
        // 5 a second is the edge.
        assert!(burst(8, 1.0, 200).iter().all(|g| *g == 1.0));
    }

    #[test]
    fn a_spun_wheel_gains_with_its_speed_up_to_the_cap() {
        // name, clicks a second (gap ms), expected gain at the end of 30 clicks (low, high)
        let cases = [
            ("10 a second", 100, 2.0, 2.4),
            ("20 a second", 50, 4.0, 5.0),
            ("a spin of 100 a second", 10, 5.0, 5.0),
        ];
        for (name, gap, low, high) in cases {
            let last = *burst(30, 1.0, gap).last().unwrap();
            assert!(last >= low && last <= high, "{name}: {last}");
        }
    }

    #[test]
    fn the_gain_builds_over_a_burst_and_never_falls_inside_it() {
        let gains = burst(20, 1.0, 50);
        assert!(gains.windows(2).all(|w| w[1] >= w[0] - 1e-9), "{gains:?}");
        assert!(gains[0] == 1.0 && gains[19] > gains[2], "{gains:?}");
    }

    #[test]
    fn a_pause_ends_the_burst() {
        let mut accel = Accel::default();
        for n in 0..20 {
            accel = accel.feed(1.0, at(n * 40), MAX).0;
        }
        let (_, gain) = accel.feed(1.0, at(19 * 40 + 300), MAX);
        assert_eq!(gain, 1.0);
    }

    #[test]
    fn a_high_resolution_wheel_of_eighths_reads_the_same_as_whole_clicks() {
        // 1/8 of a detent every 12.5 ms is 10 detents a second, however it is cut.
        let mut accel = Accel::default();
        let mut gain = 0.0;
        for n in 0..80 {
            let (next, g) = accel.feed(0.125, Elapsed(Duration::from_micros(n * 12_500)), MAX);
            accel = next;
            gain = g;
        }
        let whole = *burst(30, 1.0, 100).last().unwrap();
        assert!((gain - whole).abs() < 0.2, "{gain} vs {whole}");
    }

    #[test]
    fn a_cap_of_one_turns_it_off() {
        let mut accel = Accel::default();
        let mut gain = 0.0;
        for n in 0..30 {
            let (next, g) = accel.feed(1.0, at(n * 20), 1.0);
            accel = next;
            gain = g;
        }
        assert_eq!(gain, 1.0);
    }

    #[test]
    fn coalesced_events_do_not_read_as_infinite() {
        let (accel, _) = Accel::default().feed(1.0, at(0), MAX);
        let (_, gain) = accel.feed(1.0, at(0), MAX);
        assert!(gain <= MAX);
    }

    #[test]
    fn a_fast_lift_glides_further_and_a_slow_one_as_it_is() {
        // name, lift speed px/s, gain
        let cases = [
            ("slow", 400.0, 1.0),
            ("the floor", 1000.0, 1.0),
            ("brisk", 2500.0, 1.5),
            ("fast", 4000.0, 2.0),
            ("a throw", 9000.0, 2.0),
        ];
        for (name, speed, want) in cases {
            let got = fling_gain(speed, 2.0);
            assert!((got - want).abs() < 1e-9, "{name}: {got}");
        }
        assert_eq!(fling_gain(9000.0, 1.0), 1.0, "a cap of one is off");
    }
}
