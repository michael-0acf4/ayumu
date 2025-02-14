use crate::parser::WithPos;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    ASC,
    DESC,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f32),
    String(String),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < 10e-6,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {} // Note: enforce reflexivity: x == x always true

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    Eq,          // "="
    Neq,         // "!="
    Gt,          // ">"
    Gte,         // ">="
    Lt,          // "<"
    Lte,         // "<="
    Contains,    // "~"
    NotContains, // "!~"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Keyword {
        keyword: WithPos<String>,
    },
    Operation {
        column: WithPos<String>,
        operator: WithPos<Operator>,
        value: WithPos<Value>,
    },
    SortBy {
        column: WithPos<String>,
        order: Option<WithPos<Order>>,
    },
}

pub trait SaveRepr {
    fn save_repr(&self) -> String;
}

impl SaveRepr for Vec<Term> {
    fn save_repr(&self) -> String {
        self.iter()
            .map(|term| term.save_repr())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl SaveRepr for Term {
    fn save_repr(&self) -> String {
        match self {
            Term::Keyword { keyword } => keyword.value.to_owned(),
            Term::Operation {
                column,
                operator,
                value,
            } => format!(
                "{} {} {}",
                column.value,
                operator.save_repr(),
                value.save_repr()
            ),
            Term::SortBy { column, order } => format!(
                "sortby:{}{}",
                column.value,
                order
                    .clone()
                    .map(|o| format!(",{}", o.save_repr()))
                    .unwrap_or("".to_string())
            ),
        }
    }
}

impl SaveRepr for Operator {
    fn save_repr(&self) -> String {
        match self {
            Operator::Eq => "=",
            Operator::Neq => "!=",
            Operator::Gt => ">",
            Operator::Gte => ">=",
            Operator::Lt => "<",
            Operator::Lte => "<=",
            Operator::Contains => "~",
            Operator::NotContains => "!~",
        }
        .to_owned()
    }
}

impl SaveRepr for Value {
    fn save_repr(&self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::String(s) => format!("{s:?}"),
        }
    }
}

impl SaveRepr for Order {
    fn save_repr(&self) -> String {
        match self {
            Order::ASC => "asc".to_string(),
            Order::DESC => "desc".to_string(),
        }
    }
}

impl<T: SaveRepr> SaveRepr for WithPos<T> {
    fn save_repr(&self) -> String {
        self.value.save_repr()
    }
}
