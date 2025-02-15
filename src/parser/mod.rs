use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0},
    combinator::{map, map_res, opt},
    multi::many0,
    number::streaming::float,
    sequence::{delimited, preceded},
    IResult,
};
use nom_locate::LocatedSpan;
use string::parse_string;

use crate::ast::{Operator, Order, Term, Value};

mod string;
type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithPos<T> {
    pub value: T,
    pub start: usize,
    pub end: usize,
}

impl<T> WithPos<T> {
    pub fn transfer<P>(&self, p: P) -> WithPos<P> {
        WithPos {
            value: p,
            start: self.start,
            end: self.end,
        }
    }
}

fn with_position_mut<'a, O>(
    mut parser: impl FnMut(Span<'a>) -> IResult<Span, O>,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, WithPos<O>> {
    move |input: Span<'a>| {
        let start = input.location_offset();
        let (next_input, value) = parser(input)?;
        let end = next_input.location_offset();
        Ok((next_input, WithPos { value, start, end }))
    }
}

/// Contiguous string without spaces or any of <>=:~, in between
fn parse_token(input: Span) -> IResult<Span, WithPos<String>> {
    map(
        with_position_mut(take_while1(|c: char| {
            !c.is_whitespace() && !"<>=:~,".contains(c)
        })),
        |s| s.transfer(s.value.to_string()),
    )(input)
}

fn parse_operator(input: Span) -> IResult<Span, WithPos<Operator>> {
    let op = alt((
        tag("="),
        tag("!="),
        tag("~"),
        tag("!~"),
        tag(">="),
        tag(">"),
        tag("<="),
        tag("<"),
    ));

    map_res(with_position_mut(op), |op| match &op.value.to_string() {
        s if s.eq("=") => Ok(op.transfer(Operator::Eq)),
        s if s.eq("!=") => Ok(op.transfer(Operator::Neq)),
        s if s.eq("~") => Ok(op.transfer(Operator::Contains)),
        s if s.eq("!~") => Ok(op.transfer(Operator::NotContains)),
        s if s.eq(">=") => Ok(op.transfer(Operator::Gte)),
        s if s.eq(">") => Ok(op.transfer(Operator::Gt)),
        s if s.eq("<=") => Ok(op.transfer(Operator::Lte)),
        s if s.eq("<") => Ok(op.transfer(Operator::Lt)),
        _ => Err("Not an operator".to_string()),
    })(input)
}

fn parse_float_value(input: Span) -> IResult<Span, WithPos<Value>> {
    map(with_position_mut(float), |n| {
        n.transfer(Value::Number(n.value))
    })(input)
}

fn parse_string_value(input: Span) -> IResult<Span, WithPos<Value>> {
    let string_expr = map(with_position_mut(parse_string), |s| {
        s.transfer(Value::String(s.value.clone()))
    });
    let just_token = map(parse_token, |t| t.transfer(Value::String(t.value.clone())));
    alt((string_expr, just_token))(input)
}

fn parse_term(input: Span) -> IResult<Span, Term> {
    let (next_input, column) = preceded(multispace0, parse_token)(input)?;
    let (next_input, operator) = preceded(multispace0, parse_operator)(next_input)?;
    let (next_input, value) =
        preceded(multispace0, alt((parse_float_value, parse_string_value)))(next_input)?;

    Ok((
        next_input,
        Term::Operation {
            column,
            operator,
            value,
        },
    ))
}

fn parse_sort_by(input: Span) -> IResult<Span, Term> {
    let (next_input, _c) = map_res(preceded(multispace0, parse_token), |c| {
        if c.value.to_lowercase().eq("sortby") {
            Ok(c)
        } else {
            Err(format!("Not a sortby command: {}", c.value))
        }
    })(input)?;
    let (next_input, _op) = preceded(multispace0, char(':'))(next_input)?;
    let (maybe_order_input, column) = preceded(multispace0, parse_token)(next_input)?;

    let order_parser = preceded(char(','), preceded(multispace0, parse_token));
    let (next_input, order) = map(preceded(multispace0, opt(order_parser)), |asc| match asc {
        Some(order) => {
            let lc_value = order.value.to_lowercase();
            if "asc".eq(&lc_value) {
                Some(order.transfer(Order::ASC))
            } else if "desc".eq(&lc_value) {
                Some(order.transfer(Order::DESC))
            } else if "rand".eq(&lc_value) {
                Some(order.transfer(Order::RANDOM))
            } else {
                None
            }
        }
        None => None,
    })(maybe_order_input)?;

    Ok((
        match order {
            Some(_) => next_input,
            None => maybe_order_input, // backtrack
        },
        Term::SortBy { column, order },
    ))
}

fn parse_query_with_remainder(input: Span) -> IResult<Span, Vec<Term>> {
    let term = alt((
        parse_term,
        parse_sort_by,
        map(parse_token, |t| Term::Keyword { keyword: t }),
    ));

    many0(delimited(multispace0, term, multispace0))(input)
}

pub fn parse_query(input: &str) -> Result<Vec<Term>, String> {
    let (loc_reminder, mut terms) =
        parse_query_with_remainder(input.into()).map_err(|e| e.to_string())?;

    let reminder = loc_reminder.trim();
    if !reminder.is_empty() {
        let start = loc_reminder.location_offset();
        terms.push(Term::Keyword {
            keyword: WithPos {
                value: reminder.to_owned(),
                start,
                end: start + reminder.len(),
            },
        });
    }

    Ok(terms)
}
