//! The last pointer position the document was fed.

use blitz_traits::events::{BlitzPointerEvent, MouseEventButton, UiEvent};

/// The last pointer event with a position that the document was fed, as a move to feed again.
#[derive(Debug, Clone, Default)]
pub enum LastMove {
    /// No pointer on the surface, or it left: nothing to feed again.
    #[default]
    Unknown,
    At(BlitzPointerEvent),
}

/// The record after the document is fed `event`. A move is kept as it is; a press or release
/// is kept as a move at its point with the buttons it left held, so the move fed again carries
/// the buttons that are down now.
pub fn remember(last: LastMove, event: &UiEvent) -> LastMove {
    match event {
        UiEvent::PointerMove(moved) => LastMove::At(moved.clone()),
        UiEvent::PointerDown(changed) | UiEvent::PointerUp(changed) => {
            LastMove::At(BlitzPointerEvent {
                button: MouseEventButton::Main,
                ..changed.clone()
            })
        }
        _ => last,
    }
}
