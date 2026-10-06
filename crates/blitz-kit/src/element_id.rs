//! The `id` attribute of a DOM element, as a typed name.

/// A DOM element's `id` attribute, resolved by the caller after layout
/// (`get_element_by_id(id.as_str())`).
///
/// A string, not a hash: Blitz finds an element only by its `id` attribute string
/// (`BaseDocument::get_element_by_id(&str)`), so anything else would need an interning table to
/// get the string back (resolved by this newtype).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ElementId(String);

impl ElementId {
    /// The element whose `id` attribute is `id`.
    pub fn new(id: impl Into<String>) -> ElementId {
        ElementId(id.into())
    }

    /// The `id` attribute this names.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
