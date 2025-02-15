use super::{propose_closest, Convert, ConvertError};
use crate::{
    ast::{Operator, Order, SaveRepr, Term, Value},
    parser::WithPos,
};

#[derive(Clone)]
pub struct SQLiteWhere {
    columns: Vec<String>,
    keyword_columns: Vec<String>,
    ignore_case: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereClause {
    pub where_clause: String,
    pub order_by: String,
    pub bindings: Vec<Value>,
}

impl SQLiteWhere {
    pub fn new(columns: Vec<String>, ignore_case: bool) -> Self {
        Self {
            columns,
            keyword_columns: vec![],
            ignore_case,
        }
    }

    pub fn match_keywords_with(&mut self, columns: Vec<String>) -> Result<(), String> {
        for kcol in &columns {
            if !self.columns.contains(kcol) {
                return Err(format!("Invalid column {kcol:?}"));
            }
        }

        self.keyword_columns = columns;
        Ok(())
    }

    pub fn check_column(&self, column: &WithPos<String>) -> Result<(), ConvertError<String>> {
        if !self.ignore_case && self.columns.contains(&column.value) {
            return Ok(());
        } else if self.ignore_case {
            let hit = self
                .columns
                .iter()
                .map(|c| c.to_lowercase())
                .find(|c| c.eq(&column.value.to_lowercase()));

            if hit.is_some() {
                return Ok(());
            }
        }

        Err(ConvertError {
            error: format!(
                "Invalid column {:?}{}",
                column.value,
                propose_closest(&self.columns, &column.value, Some(3))
                    .map(|closest| format!(": did you mean {closest:?}?"))
                    .unwrap_or("".to_string())
            ),
            start: column.start,
            end: column.end,
        })
    }
}

impl Convert<WhereClause, String> for SQLiteWhere {
    fn convert_terms(&self, terms: &[Term]) -> Result<WhereClause, ConvertError<String>> {
        let mut keywords = vec![];
        let mut normal_terms = vec![];
        let mut ord_terms = vec![];
        let mut nomral_bindings = vec![];

        for term in terms {
            match term {
                Term::Keyword { keyword } => {
                    keywords.push(keyword.value.clone());
                }
                Term::Operation {
                    column,
                    operator,
                    value,
                } => {
                    self.check_column(column)?;
                    let col_repr = match self.ignore_case {
                        true => column.value.clone(),
                        false => format!("{:?}", column.value),
                    };
                    let op_repr = match operator.value {
                        Operator::Contains => "LIKE".to_string(),
                        Operator::NotContains => "NOT LIKE".to_string(),
                        _ => operator.save_repr(),
                    };

                    normal_terms.push(format!("{col_repr} {op_repr} ?"));
                    nomral_bindings.push(value.value.clone());
                }
                Term::SortBy { column, order } => {
                    self.check_column(column)?;
                    let col_repr = match self.ignore_case {
                        true => column.value.clone(),
                        false => format!("{:?}", column.value),
                    };

                    if let Some(order) = order {
                        ord_terms.push(match &order.value {
                            Order::ASC => format!("{col_repr} ASC"),
                            Order::DESC => format!("{col_repr} DESC"),
                            Order::RANDOM => format!("{col_repr}, RANDOM()"),
                        });
                    } else {
                        ord_terms.push(col_repr);
                    }
                }
            }
        }

        let mut keyword_bindings = vec![];
        let mut keyword_terms = vec![];
        for kcol in &self.keyword_columns {
            let col_repr = match self.ignore_case {
                true => kcol.clone(),
                false => format!("{kcol:?}"),
            };

            keyword_terms.push(format!("{col_repr} LIKE ?"));
            keyword_bindings.push(Value::String(format!("%{}%", keywords.join("%"))));
        }

        // merge keyword + normal terms (bindings order matters)
        let mut bindings = vec![];
        let mut where_clause = vec![];

        if !keyword_terms.is_empty() {
            where_clause.push(format!("({})", keyword_terms.join(" OR ")));
            bindings.extend(keyword_bindings);
        }
        if !normal_terms.is_empty() {
            where_clause.push(format!("({})", normal_terms.join(" AND ")));
            bindings.extend(nomral_bindings);
        }

        Ok(WhereClause {
            where_clause: where_clause.join(" AND "),
            order_by: ord_terms.join(", "),
            bindings,
        })
    }
}
