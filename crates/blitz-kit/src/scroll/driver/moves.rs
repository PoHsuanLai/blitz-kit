//! What a driver call did to the document.

use crate::scroll::geom::Latch;

/// Whether a call moved any offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Moved {
    Moved,
    Still,
}

/// Whether the engine took an input for its own, or left it to the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Claim {
    Took,
    Left,
}

/// The offsets a call wrote, in order, and whether the engine took the input.
#[derive(Debug, Clone, PartialEq)]
pub struct Moves {
    written: Vec<Latch>,
    claim: Claim,
}

impl Moves {
    /// Nothing written, the input left alone.
    pub fn none() -> Moves {
        Moves {
            written: Vec::new(),
            claim: Claim::Left,
        }
    }

    /// These writes, the input left alone.
    pub(super) fn wrote(written: Vec<Latch>) -> Moves {
        Moves {
            written,
            claim: Claim::Left,
        }
    }

    /// The scrollers whose offsets were written, once per write.
    pub fn written(&self) -> &[Latch] {
        &self.written
    }

    /// Whether any offset was written.
    pub fn moved(&self) -> Moved {
        match self.written.is_empty() {
            true => Moved::Still,
            false => Moved::Moved,
        }
    }

    /// Whether the engine took the input.
    pub fn claim(&self) -> Claim {
        self.claim
    }

    /// These moves, marked as the engine's own.
    pub(super) fn took(self) -> Moves {
        Moves {
            claim: Claim::Took,
            ..self
        }
    }

    /// These moves followed by `next`'s; the engine took the input if either did.
    pub fn then(mut self, next: Moves) -> Moves {
        self.written.extend(next.written);
        Moves {
            written: self.written,
            claim: match (self.claim, next.claim) {
                (Claim::Left, Claim::Left) => Claim::Left,
                _ => Claim::Took,
            },
        }
    }
}

/// Whether a key reached the engine or belongs to the document.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyUse {
    Scrolled(Moves),
    Passed,
}

/// Whether a scroll key is a fresh press or a repeat of one held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyRepeat {
    First,
    Repeated,
}
