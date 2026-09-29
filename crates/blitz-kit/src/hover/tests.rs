use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, BlitzWheelDelta, BlitzWheelEvent, MouseEventButton,
    MouseEventButtons, PointerCoords, PointerDetails, UiEvent,
};
use keyboard_types::Modifiers;

use super::*;
use crate::units::{Bounds, PagePoint};

const AT: PagePoint = PagePoint { x: 50.0, y: 20.0 };

fn coords(at: PagePoint) -> PointerCoords {
    let (x, y) = (at.x as f32, at.y as f32);
    PointerCoords {
        page_x: x,
        page_y: y,
        screen_x: x,
        screen_y: y,
        client_x: x,
        client_y: y,
    }
}

/// A mouse pointer event at `at`.
fn pointer(
    at: PagePoint,
    button: MouseEventButton,
    buttons: MouseEventButtons,
) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: coords(at),
        button,
        buttons,
        mods: Modifiers::empty(),
        details: PointerDetails::default(),
        element: Default::default(),
        active_pointers: Default::default(),
    }
}

fn move_event(at: PagePoint, held: MouseEventButtons) -> UiEvent {
    UiEvent::PointerMove(pointer(at, MouseEventButton::Main, held))
}

fn known() -> LastMove {
    remember(LastMove::Unknown, &move_event(AT, MouseEventButtons::None))
}

/// A 100 x 40 box at (10, 10) whose element is 1, covered by element 2 everywhere in its
/// left `covered_to` px; outside the box is element 0.
fn world(covered_to: f64) -> impl Fn(PagePoint) -> Option<u32> {
    move |p: PagePoint| {
        let inside = (10.0..=110.0).contains(&p.x) && (10.0..=50.0).contains(&p.y);
        match (inside, p.x < 10.0 + covered_to) {
            (false, _) => Some(0),
            (true, true) => Some(2),
            (true, false) => Some(1),
        }
    }
}

const BOX: Bounds = Bounds {
    x: 10.0,
    y: 10.0,
    width: 100.0,
    height: 40.0,
};

#[test]
fn a_changed_hover_under_a_known_pointer_restores_the_old_element_or_clears() {
    type Case = (
        &'static str,
        Option<u32>,
        Option<u32>,
        f64,
        Option<Bounds>,
        HoverSync,
    );
    let cases: [Case; 7] = [
        (
            "unchanged",
            Some(1),
            Some(1),
            0.0,
            Some(BOX),
            HoverSync::Unchanged,
        ),
        (
            "nothing before",
            None,
            Some(2),
            0.0,
            Some(BOX),
            HoverSync::Clear,
        ),
        (
            "uncovered: the centre",
            Some(1),
            Some(3),
            0.0,
            Some(BOX),
            HoverSync::Restore(PagePoint { x: 60.0, y: 30.0 }),
        ),
        (
            "left part covered: the top-right corner",
            Some(1),
            Some(2),
            70.0,
            Some(BOX),
            HoverSync::Restore(PagePoint { x: 109.0, y: 11.0 }),
        ),
        (
            "covered everywhere",
            Some(1),
            Some(2),
            200.0,
            Some(BOX),
            HoverSync::Clear,
        ),
        (
            "gone from layout",
            Some(1),
            Some(2),
            0.0,
            None,
            HoverSync::Clear,
        ),
        (
            "to nothing",
            Some(1),
            None,
            0.0,
            Some(BOX),
            HoverSync::Restore(PagePoint { x: 60.0, y: 30.0 }),
        ),
    ];
    for (name, before, after, covered_to, bounds, want) in cases {
        let got = decide(
            Shift { before, after },
            &known(),
            |_| bounds,
            world(covered_to),
        );
        assert_eq!(got, want, "{name}");
    }
}

#[test]
fn without_a_move_to_feed_again_nothing_is_touched() {
    let got = decide(
        Shift {
            before: Some(1u32),
            after: Some(2),
        },
        &LastMove::Unknown,
        |_| panic!("no bounds asked"),
        |_| panic!("no hit asked"),
    );
    assert_eq!(got, HoverSync::Unchanged);
}

#[test]
fn probes_are_the_centre_then_the_corners_one_px_inside() {
    let cases = [
        (
            "a card",
            BOX,
            [
                (60.0, 30.0),
                (11.0, 11.0),
                (109.0, 11.0),
                (11.0, 49.0),
                (109.0, 49.0),
            ],
        ),
        (
            "a 1 px line collapses onto its middle",
            Bounds {
                x: 0.0,
                y: 5.0,
                width: 1.0,
                height: 10.0,
            },
            [
                (0.5, 10.0),
                (0.5, 6.0),
                (0.5, 6.0),
                (0.5, 14.0),
                (0.5, 14.0),
            ],
        ),
    ];
    for (name, bounds, want) in cases {
        let got = probe_points(bounds).map(|p| (p.x, p.y));
        assert_eq!(got, want, "{name}");
    }
}

fn at(last: &LastMove) -> Option<(f32, f32, MouseEventButtons, MouseEventButton)> {
    match last {
        LastMove::Unknown => None,
        LastMove::At(e) => Some((e.client_x(), e.client_y(), e.buttons, e.button)),
    }
}

#[test]
fn moves_presses_and_releases_are_remembered_as_moves_and_other_events_are_not() {
    let mods = Modifiers::empty();
    let secondary = MouseEventButton::Secondary;
    let down = UiEvent::PointerDown(pointer(AT, secondary, MouseEventButtons::Secondary));
    let up = UiEvent::PointerUp(pointer(AT, secondary, MouseEventButtons::None));
    let wheel = UiEvent::Wheel(BlitzWheelEvent {
        delta: BlitzWheelDelta::Lines(0.0, 1.0),
        coords: coords(PagePoint { x: 0.0, y: 0.0 }),
        buttons: MouseEventButtons::None,
        mods,
        element: Default::default(),
    });
    let held = MouseEventButtons::Secondary;
    let main = MouseEventButton::Main;
    let cases = [
        (
            "a move",
            LastMove::Unknown,
            move_event(AT, held),
            Some((50.0, 20.0, held, main)),
        ),
        (
            "a press",
            LastMove::Unknown,
            down,
            Some((50.0, 20.0, held, main)),
        ),
        (
            "a release",
            known(),
            up,
            Some((50.0, 20.0, MouseEventButtons::None, main)),
        ),
        (
            "a wheel keeps the record",
            known(),
            wheel.clone(),
            Some((50.0, 20.0, MouseEventButtons::None, main)),
        ),
        ("a wheel with no record", LastMove::Unknown, wheel, None),
    ];
    for (name, last, event, want) in cases {
        assert_eq!(at(&remember(last, &event)), want, "{name}");
    }
}
