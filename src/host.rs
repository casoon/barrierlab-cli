//! Host über `a11y-dom::Arena`: Struktur und Semantik (Rolle, Name über
//! `accname`). Darstellung bedient dieser Host noch nicht.

use a11y_dom::{
    Arena, ArenaBuilder, ArenaNode, Backdrop, ComputedStyle, Document, Node, NodeKind, Rect,
    Rendering, Semantics, ancestors, elements,
};
use accname::IdIndex;
use html5_parser::NodeKind as H;
use style_cascade::{Cascade, Environment};
use stylesheet_parse::Stylesheet;

use crate::fetch::Inline;

pub fn arena(html: &str) -> Arena {
    let doc = html5_parser::parse(html).document;
    let root = doc
        .children(doc.root())
        .find(|&c| matches!(doc.node(c).kind, H::Element { .. }))
        .expect("document without root element");
    copy(&doc, root, Arena::builder()).build()
}

fn copy(doc: &html5_parser::Document, id: html5_parser::NodeId, b: ArenaBuilder) -> ArenaBuilder {
    match &doc.node(id).kind {
        H::Element {
            name, attributes, ..
        } => {
            let mut b = b.open(name);
            for a in attributes {
                b = b.attr(&a.name, &a.value);
            }
            for k in doc.children(id) {
                b = copy(doc, k, b);
            }
            b.close()
        }
        H::DocumentFragment => doc.children(id).fold(b, |b, k| copy(doc, k, b)),
        H::Text { content } => b.text(content),
        _ => b,
    }
}

pub struct Host<'a> {
    doc: &'a Arena,
    ids: IdIndex<'a, ArenaNode<'a>>,
    cascade: Cascade,
}

/// `<style>`-Texte und `<link rel=stylesheet>` in Dokumentreihenfolge.
pub fn style_sources(doc: &Arena) -> Vec<Inline> {
    elements(doc)
        .filter_map(|n| {
            if n.is_element("style") {
                let text = n
                    .children()
                    .filter(|k| k.kind() == NodeKind::Text)
                    .map(|k| k.text())
                    .collect();
                Some(Inline::Style(text))
            } else if n.is_element("link")
                && n.attr("rel").is_some_and(|r| {
                    r.split_whitespace()
                        .any(|t| t.eq_ignore_ascii_case("stylesheet"))
                })
            {
                n.attr("href").map(|h| Inline::Link(h.to_string()))
            } else {
                None
            }
        })
        .collect()
}

impl<'a> Host<'a> {
    pub fn new(doc: &'a Arena, sheets: &[Stylesheet]) -> Self {
        Host {
            ids: IdIndex::build(doc.root()),
            cascade: Cascade::new(sheets, Environment::default()),
            doc,
        }
    }
}

impl Document for Host<'_> {
    type N<'n>
        = ArenaNode<'n>
    where
        Self: 'n;

    fn root(&self) -> Self::N<'_> {
        self.doc.root()
    }
}

impl Semantics for Host<'_> {
    fn role<'n>(&'n self, node: Self::N<'n>) -> Option<String> {
        accname::role(node).map(str::to_string)
    }

    fn accessible_name<'n>(&'n self, node: Self::N<'n>) -> Option<String> {
        accname::name(node, &self.ids)
    }

    fn is_ignored<'n>(&'n self, node: Self::N<'n>) -> bool {
        std::iter::once(node)
            .chain(ancestors(node))
            .any(|n| n.attr("aria-hidden") == Some("true") || n.has_attr("inert"))
    }
}

impl Rendering for Host<'_> {
    /// Berechnete Stile aus der Kaskade über die Stylesheets der Seite.
    fn computed_style<'n>(&'n self, node: Self::N<'n>) -> Option<ComputedStyle> {
        (node.kind() == NodeKind::Element).then(|| self.cascade.computed_style(node))
    }

    /// Ohne Layout gibt es keine Kästen.
    fn bounds<'n>(&'n self, _node: Self::N<'n>) -> Option<Rect> {
        None
    }

    /// Verläufe: Leuchtdichte der Stufen, wo kein einzelner Hintergrund gilt.
    fn sampled_backdrop<'n>(&'n self, node: Self::N<'n>) -> Option<Backdrop> {
        self.cascade
            .backdrop_luminances(node)
            .map(|luminance| Backdrop { luminance })
    }

    /// Dargestellt ist, was weder selbst noch über einen Vorfahren
    /// `display: none` hat und nicht `visibility: hidden` ist.
    fn is_rendered<'n>(&'n self, node: Self::N<'n>) -> bool {
        let verborgen = |n: ArenaNode<'_>| {
            let s = self.cascade.computed_style(n);
            s.display.as_deref() == Some("none")
        };
        let sichtbar = self
            .cascade
            .computed_style(node)
            .visibility
            .is_none_or(|v| v != "hidden" && v != "collapse");
        sichtbar && !std::iter::once(node).chain(ancestors(node)).any(verborgen)
    }
}
