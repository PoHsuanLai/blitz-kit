//! What a point hits, lifted to its element, in a real document.

use crate::support;

use blitz_dom::{LocalName, Namespace, QualName};
use blitz_kit::hit::{element_at, element_of, is_content_element};
use blitz_kit::units::PagePoint;

fn point(x: f64, y: f64) -> PagePoint {
    PagePoint { x, y }
}

#[test]
fn a_point_hits_the_element_under_it_and_a_text_node_lifts_to_its_element() {
    let (mut doc, body) = support::page(300, 200, 1.0);
    let label = support::div(&mut doc, body, "label", "width:100px;height:30px");
    let text = support::text(&mut doc, label, "hello");
    doc.resolve(0.0);
    assert_eq!(element_at(&doc, point(5.0, 5.0)), Some(label));
    assert_eq!(element_of(&doc, text), Some(label));
    assert_eq!(element_of(&doc, label), Some(label));
}

#[test]
fn a_transformed_box_is_hit_where_it_paints() {
    let (mut doc, body) = support::page(300, 200, 1.0);
    let wrap = support::div(
        &mut doc,
        body,
        "wrap",
        "width:100px;height:40px;transform:translateX(150px)",
    );
    doc.resolve(0.0);
    assert_eq!(element_at(&doc, point(200.0, 10.0)), Some(wrap));
    assert_ne!(element_at(&doc, point(10.0, 10.0)), Some(wrap));
}

#[test]
fn document_furniture_is_not_content() {
    let (mut doc, body) = support::page(300, 200, 1.0);
    let name = |local: &str| QualName::new(None, Namespace::from(""), LocalName::from(local));
    let mut mutr = doc.mutate();
    let style = mutr.create_element(name("style"), vec![]);
    let section = mutr.create_element(name("section"), vec![]);
    mutr.append_children(body, &[style, section]);
    drop(mutr);
    let data = |id| {
        doc.get_node(id)
            .and_then(|n| n.element_data())
            .expect("element")
    };
    assert!(!is_content_element(data(style)));
    assert!(is_content_element(data(section)));
}
