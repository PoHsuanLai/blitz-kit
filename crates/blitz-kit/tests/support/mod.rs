//! A bare Blitz document built by hand: no HTML parser, no Dioxus, no host.

#![allow(dead_code)]

use blitz_dom::{
    Attribute, BaseDocument, DocumentConfig, FontContext, LocalName, Namespace, NodeId, QualName,
};
use blitz_traits::shell::{ColorScheme, Viewport};

/// A user-agent sheet: block boxes and no body margin, so a page starts at its (0, 0).
const UA: &str = "html,body,div,main{display:block}html,body{margin:0}";

fn name(local: &str) -> QualName {
    QualName::new(None, Namespace::from(""), LocalName::from(local))
}

/// An empty document `width` x `height` logical px at `scale`, laid out once, and its `<body>`.
pub fn page(width: u32, height: u32, scale: f32) -> (BaseDocument, NodeId) {
    page_with(width, height, scale, None)
}

/// [`page`] resolving fonts through `fonts`.
pub fn page_with(
    width: u32,
    height: u32,
    scale: f32,
    fonts: Option<FontContext>,
) -> (BaseDocument, NodeId) {
    let viewport = Viewport::new(
        (f64::from(width) * f64::from(scale)).round() as u32,
        (f64::from(height) * f64::from(scale)).round() as u32,
        scale,
        ColorScheme::Light,
    );
    let mut doc = BaseDocument::new(DocumentConfig {
        viewport: Some(viewport),
        font_ctx: fonts,
        ..DocumentConfig::default()
    });
    doc.add_user_agent_stylesheet(UA);
    let mut mutr = doc.mutate();
    let html = mutr.create_element(name("html"), vec![]);
    mutr.append_children(mutr.doc.root_node().id, &[html]);
    let body = mutr.create_element(name("body"), vec![]);
    mutr.append_children(html, &[body]);
    drop(mutr);
    doc.resolve(0.0);
    (doc, body)
}

/// Append `<div id=… style=…>` under `parent`.
pub fn div(doc: &mut BaseDocument, parent: NodeId, id: &str, style: &str) -> NodeId {
    let attrs = [("id", id), ("style", style)]
        .map(|(key, value)| Attribute {
            name: name(key),
            value: value.to_owned(),
        })
        .to_vec();
    let mut mutr = doc.mutate();
    let div = mutr.create_element(name("div"), attrs);
    mutr.append_children(parent, &[div]);
    div
}

/// Append a text node under `parent`.
pub fn text(doc: &mut BaseDocument, parent: NodeId, text: &str) -> NodeId {
    let mut mutr = doc.mutate();
    let node = mutr.create_text_node(text);
    mutr.append_children(parent, &[node]);
    node
}

/// The node whose `id` attribute is `id`.
pub fn by_id(doc: &BaseDocument, id: &str) -> NodeId {
    doc.get_element_by_id(id)
        .unwrap_or_else(|| panic!("no #{id} in the page"))
}
