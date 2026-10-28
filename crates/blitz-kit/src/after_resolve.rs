//! Readings a host takes from its document after every resolve, for whoever registered them.
//!
//! A component that sizes itself from the document (a custom property's resolved value, say)
//! cannot read the document while it renders: the host holds it then. It registers a reading
//! here instead, for its node; the host calls [`AfterResolve::read`] right after each resolve,
//! with the document unborrowed by anyone else, and runs the publishes it gets back where the
//! reader's state lives (for Dioxus, inside the runtime). A reading answers a publish only when
//! what it read changed, so a frame whose values held still publishes nothing.
//!
//! The registry is shared: the host keeps one per document and hands clones to whoever
//! registers. Dropping a [`Registration`] ends its reading.
//!
//! ```
//! use blitz_kit::after_resolve::AfterResolve;
//!
//! let readings = AfterResolve::default();
//! assert!(readings.is_empty());
//! ```

use blitz_dom::{BaseDocument, NodeId};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// A reading of one node, given the resolved document. It keeps what it last read and answers
/// with the publish to run only when that changed.
pub type Read = Box<dyn FnMut(&BaseDocument, NodeId) -> Option<Publish>>;

/// Handing a changed reading to its owner: `false` once the owner is gone.
pub type Publish = Box<dyn FnOnce() -> bool>;

struct Reading {
    id: u64,
    node: NodeId,
    read: Read,
}

#[derive(Default)]
struct Book {
    next: u64,
    readings: Vec<Reading>,
}

/// One document's registered readings. Clones share them.
#[derive(Clone, Default)]
pub struct AfterResolve(Rc<RefCell<Book>>);

impl std::fmt::Debug for AfterResolve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AfterResolve")
            .field("readings", &self.0.borrow().readings.len())
            .finish()
    }
}

impl AfterResolve {
    /// Run `read` on `node` after every resolve until the registration is dropped.
    #[must_use = "the reading ends when the registration is dropped"]
    pub fn watch(&self, node: NodeId, read: Read) -> Registration {
        let mut book = self.0.borrow_mut();
        let id = book.next;
        book.next += 1;
        book.readings.push(Reading { id, node, read });
        Registration {
            book: Rc::downgrade(&self.0),
            id,
        }
    }

    /// Take every reading from the resolved `doc`; the publishes of the ones that changed, to run
    /// where their owners live. Call it after each resolve, with nothing else borrowing `doc`.
    pub fn read(&self, doc: &BaseDocument) -> Vec<Publish> {
        self.0
            .borrow_mut()
            .readings
            .iter_mut()
            .filter_map(|reading| (reading.read)(doc, reading.node))
            .collect()
    }

    /// Whether nothing is registered (a host may skip the read then).
    pub fn is_empty(&self) -> bool {
        self.0.borrow().readings.is_empty()
    }
}

/// A registered reading; dropping it ends the reading.
#[derive(Debug)]
pub struct Registration {
    book: Weak<RefCell<Book>>,
    id: u64,
}

impl Drop for Registration {
    fn drop(&mut self) {
        if let Some(book) = self.book.upgrade() {
            book.borrow_mut()
                .readings
                .retain(|reading| reading.id != self.id);
        }
    }
}

/// Run `publishes` and count the ones whose owner was still there (a host redraws when any was).
pub fn publish(publishes: Vec<Publish>) -> usize {
    publishes
        .into_iter()
        .map(|publish| publish())
        .filter(|lived| *lived)
        .count()
}
