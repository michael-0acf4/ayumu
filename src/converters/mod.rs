pub mod sqlite;
use crate::{ast::Term, parser::parse_query};

#[derive(Debug, Clone, PartialEq)]
pub struct ConvertError<E: From<String>> {
    pub error: E,
    pub start: usize,
    pub end: usize,
}

pub trait Convert<O, E: From<String>> {
    fn convert(&self, query: &str) -> Result<O, ConvertError<E>> {
        let terms = parse_query(query).map_err(|e| ConvertError {
            error: e.into(),
            start: 0,
            end: query.len().saturating_sub(1),
        })?;

        self.convert_terms(&terms)
    }

    fn convert_terms(&self, terms: &[Term]) -> Result<O, ConvertError<E>>;
}

fn propose_closest(items: &[String], name: &str, dist: Option<usize>) -> Option<String> {
    let dist = dist.unwrap_or(3);
    let mut top = None;

    for entry in items {
        if top.is_some() {
            break;
        }
        let edit_dist = strsim::levenshtein(&entry, name);
        if edit_dist <= dist {
            top = Some(entry.to_owned());
        }
    }

    top
}
