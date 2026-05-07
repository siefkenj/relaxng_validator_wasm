// Duplicated and adapted from relaxng-rust/relaxng-validator/src/lib.rs (private code).
// We cannot modify the submodule, so we re-implement the attribute-head traversal
// here using the fully-public `relaxng_model::model` types.

use relaxng_model::model::{NameClass, Pattern};
use std::collections::HashSet;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// describe_nameclass — duplicated from the submodule's free function
// ---------------------------------------------------------------------------

const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

fn format_namespace(namespace_uri: &str, desc: &mut String) {
    if namespace_uri == XML_NS {
        desc.push_str("xml:");
    } else if !namespace_uri.is_empty() {
        desc.push('{');
        desc.push_str(namespace_uri);
        desc.push('}');
    }
}

fn describe_nameclass(nc: &NameClass, desc: &mut String) {
    match nc {
        NameClass::Named { namespace_uri, name } => {
            format_namespace(namespace_uri, desc);
            desc.push_str(name);
        }
        NameClass::NsName {
            namespace_uri,
            except,
        } => {
            if namespace_uri == XML_NS {
                desc.push_str("xml:*");
            } else {
                desc.push_str(namespace_uri);
                desc.push_str(":*");
            }
            if let Some(except) = except {
                desc.push('-');
                describe_nameclass(except, desc);
            }
        }
        NameClass::AnyName { except } => {
            desc.push('*');
            if let Some(except) = except {
                desc.push('-');
                describe_nameclass(except, desc);
            }
        }
        NameClass::Alt { a, b } => {
            describe_nameclass(a, desc);
            desc.push('|');
            describe_nameclass(b, desc);
        }
    }
}

fn describe_nameclass_str(nc: &NameClass) -> String {
    let mut s = String::new();
    describe_nameclass(nc, &mut s);
    s
}

// ---------------------------------------------------------------------------
// Nameclass matching (local name only — pragmatic approximation for error msgs)
// ---------------------------------------------------------------------------

fn nameclass_matches_local(nc: &NameClass, local: &str) -> bool {
    match nc {
        NameClass::Named { name, .. } => name == local,
        NameClass::NsName { except, .. } => except
            .as_ref()
            .map(|e| !nameclass_matches_local(e, local))
            .unwrap_or(true),
        NameClass::AnyName { except } => except
            .as_ref()
            .map(|e| !nameclass_matches_local(e, local))
            .unwrap_or(true),
        NameClass::Alt { a, b } => {
            nameclass_matches_local(a, local) || nameclass_matches_local(b, local)
        }
    }
}

// ---------------------------------------------------------------------------
// collect_attrs — gather all Attribute nameclass strings reachable from a
// Pattern without descending into nested Element patterns.
// `visited` is used for cycle detection on Ref nodes.
// ---------------------------------------------------------------------------

fn collect_attrs(p: &Pattern, visited: &mut HashSet<*const ()>, result: &mut Vec<String>) {
    match p {
        Pattern::Attribute(nc, ..) => {
            // Skip wildcard nameclasses (AnyName / NsName) — they represent
            // foreign-content catch-alls and are not useful as diagnostics.
            if !matches!(nc, NameClass::AnyName { .. } | NameClass::NsName { .. }) {
                result.push(describe_nameclass_str(nc));
            }
        }

        Pattern::Choice(pats, ..)
        | Pattern::Group(pats, ..)
        | Pattern::Interleave(pats, ..) => {
            for q in pats {
                collect_attrs(q, visited, result);
            }
        }

        Pattern::OneOrMore(inner, ..)
        | Pattern::ZeroOrMore(inner, ..)
        | Pattern::Optional(inner, ..)
        | Pattern::Mixed(inner, ..) => {
            collect_attrs(inner, visited, result);
        }

        Pattern::Ref(_, _, patref) => {
            let ptr = Rc::as_ptr(&patref.0) as *const ();
            if !visited.insert(ptr) {
                return;
            }
            if let Some(dr) = patref.0.borrow().as_ref() {
                collect_attrs(dr.pattern(), visited, result);
            }
        }

        // Do not descend into child Element patterns — those are child elements,
        // not attributes of the current element.
        Pattern::Element(..)
        | Pattern::Empty(..)
        | Pattern::Text(..)
        | Pattern::NotAllowed(..)
        | Pattern::DatatypeName { .. }
        | Pattern::DatatypeValue { .. }
        | Pattern::List(..) => {}
    }
}

// ---------------------------------------------------------------------------
// find_expected_attrs — public entry point
//
// Walks the schema Pattern tree looking for an Element whose nameclass matches
// `el_local` (by local name only). Returns all Attribute nameclass strings
// found in that element's content pattern. Results from multiple matching
// elements (different contexts) are merged.
// ---------------------------------------------------------------------------

pub fn find_expected_attrs(
    p: &Pattern,
    el_local: &str,
    visited: &mut HashSet<*const ()>,
) -> Vec<String> {
    match p {
        Pattern::Element(nc, content, ..) => {
            if nameclass_matches_local(nc, el_local) {
                let mut result = vec![];
                collect_attrs(content, &mut HashSet::new(), &mut result);
                result
            } else {
                // The target element may be nested inside this element's content.
                find_expected_attrs(content, el_local, visited)
            }
        }

        Pattern::Choice(pats, ..)
        | Pattern::Group(pats, ..)
        | Pattern::Interleave(pats, ..) => pats
            .iter()
            .flat_map(|q| find_expected_attrs(q, el_local, visited))
            .collect(),

        Pattern::OneOrMore(inner, ..)
        | Pattern::ZeroOrMore(inner, ..)
        | Pattern::Optional(inner, ..)
        | Pattern::Mixed(inner, ..) => find_expected_attrs(inner, el_local, visited),

        Pattern::Ref(_, _, patref) => {
            let ptr = Rc::as_ptr(&patref.0) as *const ();
            if !visited.insert(ptr) {
                return vec![];
            }
            patref
                .0
                .borrow()
                .as_ref()
                .map(|dr| find_expected_attrs(dr.pattern(), el_local, visited))
                .unwrap_or_default()
        }

        _ => vec![],
    }
}
