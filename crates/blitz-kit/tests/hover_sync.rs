//! An element that arrives under a resting pointer: Blitz's `refresh_hover` moves the hover at
//! the end of `resolve` without dispatching, and `hover::repair` fixes it, so the arriving
//! element gets its enter and the covered one its leave, once.
//!
//! A real `BaseDocument`, driven the way a host drives it: feed a move, resolve, repair. The
//! recording document dispatches into a log where a Dioxus document would dispatch into
//! components.

mod support;

use std::cell::RefCell;
use std::rc::Rc;

use blitz_dom::{BaseDocument, DocGuard, DocGuardMut, Document, EventDriver, EventHandler, NodeId};
use blitz_kit::hover::{LastMove, Repaired, Shift, remember, repair};
use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, DomEvent, DomEventData, EventState, MouseEventButton,
    MouseEventButtons, PointerCoords, PointerDetails, UiEvent,
};
use keyboard_types::Modifiers;

type Log = Rc<RefCell<Vec<String>>>;

/// A document that logs `<id> enter` and `<id> leave` for every element with an id.
struct Recording {
    base: BaseDocument,
    log: Log,
}

struct Logger(Log);

impl EventHandler for Logger {
    fn handle_event(
        &mut self,
        _chain: &[NodeId],
        event: &mut DomEvent,
        doc: &mut dyn Document,
        _state: &mut EventState,
    ) {
        let verb = match event.data {
            DomEventData::PointerEnter(_) => "enter",
            DomEventData::PointerLeave(_) => "leave",
            _ => return,
        };
        let inner = doc.inner();
        let id = inner
            .get_node(event.target)
            .and_then(|node| node.element_data())
            .and_then(|data| data.id.as_ref().map(|id| id.to_string()));
        if let Some(id) = id {
            self.0.borrow_mut().push(format!("{id} {verb}"));
        }
    }
}

impl Document for Recording {
    fn inner(&self) -> DocGuard<'_> {
        self.base.inner()
    }

    fn inner_mut(&mut self) -> DocGuardMut<'_> {
        self.base.inner_mut()
    }

    fn handle_ui_event(&mut self, event: UiEvent) {
        EventDriver::new(&mut self.base, Logger(self.log.clone())).handle_ui_event(event);
    }
}

fn move_to(x: f32, y: f32) -> UiEvent {
    UiEvent::PointerMove(BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons: MouseEventButtons::None,
        mods: Modifiers::empty(),
        details: PointerDetails::default(),
        element: Default::default(),
        active_pointers: Default::default(),
    })
}

/// The host's side of the document: the pad always, the card when asked, the last move fed.
struct Surface {
    doc: Recording,
    last: LastMove,
    body: NodeId,
    card: Option<NodeId>,
    log: Log,
}

fn surface() -> Surface {
    let (mut base, body) = support::page(300, 200, 1.0);
    support::div(&mut base, body, "pad", "width:300px;height:200px");
    let log = Log::default();
    Surface {
        doc: Recording {
            base,
            log: log.clone(),
        },
        body,
        last: LastMove::Unknown,
        card: None,
        log,
    }
}

impl Surface {
    fn hovered(&self) -> Option<NodeId> {
        let inner = self.doc.inner();
        blitz_kit::hit::element_of(&inner, inner.get_hover_node_id()?)
    }

    fn feed(&mut self, event: UiEvent) {
        self.last = remember(std::mem::take(&mut self.last), &event);
        self.doc.handle_ui_event(event);
    }

    fn take(&self) -> Vec<String> {
        std::mem::take(&mut self.log.borrow_mut())
    }

    /// One resolve as `SurfaceDocument::resolve` runs it: the hover before, the layout, the
    /// hover after, the repair.
    fn resolve(&mut self) -> (Repaired, Shift<NodeId>) {
        let before = self.hovered();
        self.doc.inner_mut().resolve(0.0);
        let shift = Shift {
            before,
            after: self.hovered(),
        };
        (repair(&mut self.doc, &self.last, shift), shift)
    }

    /// The card is 40..160 x 40..100, on top of the pad.
    fn show_card(&mut self) {
        let style = "position:absolute;left:40px;top:40px;width:120px;height:60px";
        self.card = Some(support::div(&mut self.doc.base, self.body, "card", style));
    }

    fn hide_card(&mut self) {
        if let Some(card) = self.card.take() {
            self.doc.base.mutate().remove_node(card);
        }
    }
}

/// The card's rect is 40..160 x 40..100; the pointer rests at (100, 70), inside it.
#[test]
fn a_card_arriving_under_a_resting_pointer_is_entered_once_and_the_pad_left() {
    let mut s = surface();
    s.doc.inner_mut().resolve(0.0);
    s.feed(move_to(100.0, 70.0));
    assert_eq!(s.take(), ["pad enter"]);

    s.show_card();
    let (repaired, shift) = s.resolve();
    assert_eq!(repaired, Repaired::Yes);
    assert_ne!(shift.before, shift.after);
    assert_eq!(s.take(), ["pad leave", "card enter"], "at the resolve");

    // The nudge that, before the repair, was the first the driver heard of the card: now
    // there is nothing left to dispatch.
    s.feed(move_to(101.0, 70.0));
    let (repaired, shift) = s.resolve();
    assert_eq!(repaired, Repaired::No);
    assert_eq!(shift.before, shift.after);
    assert_eq!(s.take(), Vec::<String>::new(), "after a nudge");

    // And it leaves normally.
    s.feed(move_to(250.0, 150.0));
    assert_eq!(s.take(), ["card leave", "pad enter"]);
}

/// The card going away under the pointer: the pad is entered again (a removed element gets no
/// leave, as in a browser).
#[test]
fn the_card_removed_under_a_resting_pointer_gives_the_pad_back() {
    let mut s = surface();
    s.doc.inner_mut().resolve(0.0);
    s.feed(move_to(100.0, 70.0));
    s.show_card();
    s.resolve();
    s.take();

    s.hide_card();
    s.resolve();
    assert_eq!(s.take(), ["pad enter"]);
}

/// With no pointer on the surface there is nothing to repair: the card arrives unannounced.
#[test]
fn without_a_pointer_the_resolve_dispatches_nothing() {
    let mut s = surface();
    s.doc.inner_mut().resolve(0.0);
    s.feed(move_to(100.0, 70.0));
    s.doc.inner_mut().clear_hover();
    s.last = LastMove::Unknown;
    s.take();

    s.show_card();
    let (repaired, _) = s.resolve();
    assert_eq!(repaired, Repaired::No);
    assert_eq!(s.take(), Vec::<String>::new());
}
