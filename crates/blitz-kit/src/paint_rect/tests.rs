use std::f64::consts::FRAC_PI_2;

use super::*;
use crate::units::{Bounds, PagePoint};

const fn bounds(x: f64, y: f64, width: f64, height: f64) -> Bounds {
    Bounds {
        x,
        y,
        width,
        height,
    }
}

const fn at(x: f64, y: f64, transform: Affine2) -> Placed {
    Placed {
        origin: PagePoint { x, y },
        transform,
    }
}

fn close(left: Bounds, right: Bounds) -> bool {
    let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
    near(left.x, right.x)
        && near(left.y, right.y)
        && near(left.width, right.width)
        && near(left.height, right.height)
}

/// Every case: a 100 x 40 card at (10, 20) unless the case says otherwise.
#[test]
fn a_rect_paints_through_every_transform_on_its_chain() {
    let card = bounds(10.0, 20.0, 100.0, 40.0);
    let cases: Vec<(&str, Bounds, Vec<Placed>, Bounds)> = vec![
        ("no transform", card, vec![], card),
        (
            "identity",
            card,
            vec![at(10.0, 20.0, Affine2::IDENTITY)],
            card,
        ),
        (
            "own translateX(60px)",
            card,
            vec![at(10.0, 20.0, Affine2::translate(60.0, 0.0))],
            bounds(70.0, 20.0, 100.0, 40.0),
        ),
        (
            "a parent's translateX(60px) (sill's swipe)",
            bounds(0.0, 0.0, 100.0, 40.0),
            vec![at(0.0, 0.0, Affine2::translate(60.0, 0.0))],
            bounds(60.0, 0.0, 100.0, 40.0),
        ),
        (
            "own and a parent's translations add",
            card,
            vec![
                at(10.0, 20.0, Affine2::translate(5.0, -3.0)),
                at(0.0, 0.0, Affine2::translate(-20.0, 7.0)),
            ],
            bounds(-5.0, 24.0, 100.0, 40.0),
        ),
        (
            "own scale(0.5) about its top-left",
            card,
            vec![at(10.0, 20.0, Affine2::scale(0.5, 0.5))],
            bounds(10.0, 20.0, 50.0, 20.0),
        ),
        (
            "own scale(0.5) about its centre (transform-origin folded in)",
            card,
            vec![at(
                10.0,
                20.0,
                // translate(50, 20) * scale(0.5) * translate(-50, -20)
                Affine2 {
                    e: 25.0,
                    f: 10.0,
                    ..Affine2::scale(0.5, 0.5)
                },
            )],
            bounds(35.0, 30.0, 50.0, 20.0),
        ),
        (
            "a parent's scale(2) about the parent's origin moves the child too",
            card,
            vec![at(0.0, 0.0, Affine2::scale(2.0, 2.0))],
            bounds(20.0, 40.0, 200.0, 80.0),
        ),
        (
            "rotate(90deg) about the top-left: the turned box's bounding box",
            card,
            vec![at(10.0, 20.0, Affine2::rotate(FRAC_PI_2))],
            bounds(-30.0, 20.0, 40.0, 100.0),
        ),
        (
            "an exit: translateX(110%) leaves the surface",
            card,
            vec![at(10.0, 20.0, Affine2::translate(110.0, 0.0))],
            bounds(120.0, 20.0, 100.0, 40.0),
        ),
        (
            "an exit: scale(0) collapses to its origin",
            card,
            vec![at(10.0, 20.0, Affine2::scale(0.0, 0.0))],
            bounds(10.0, 20.0, 0.0, 0.0),
        ),
    ];
    for (name, rect, chain, expected) in cases {
        let got = painted_bounds(rect, &chain);
        assert!(close(got, expected), "{name}: {got:?} != {expected:?}");
    }
}

#[test]
fn a_device_transform_comes_back_in_css_px() {
    let cases: &[(&str, [f64; 6], f64, Affine2)] = &[
        (
            "1x",
            [1.0, 0.0, 0.0, 1.0, 60.0, 0.0],
            1.0,
            Affine2::translate(60.0, 0.0),
        ),
        (
            "1.5x: translation divided, linear kept",
            [2.0, 0.0, 0.0, 2.0, 90.0, 30.0],
            1.5,
            Affine2 {
                e: 60.0,
                f: 20.0,
                ..Affine2::scale(2.0, 2.0)
            },
        ),
        (
            "a zero scale is read as 1",
            [1.0, 0.0, 0.0, 1.0, 7.0, 0.0],
            0.0,
            Affine2::translate(7.0, 0.0),
        ),
    ];
    for &(name, coeffs, scale, expected) in cases {
        assert_eq!(Affine2::from_device(coeffs, scale), expected, "{name}");
    }
}
