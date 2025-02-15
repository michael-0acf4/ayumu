use crate::{
    ast::{Operator, SaveRepr, Term, Value},
    parser::{self, WithPos},
};

#[test]
pub fn simple() {
    debug_assert_eq!(parser::parse_query(r#""#), Ok(vec![]));

    debug_assert_eq!(
        parser::parse_query(r#" keyword  "#),
        Ok(vec![Term::Keyword {
            keyword: WithPos {
                value: "keyword".to_string(),
                start: 1,
                end: 8
            }
        }])
    );

    debug_assert_eq!(
        parser::parse_query(r#"  _column1234 !~ "\"Hello\" world" "#),
        Ok(vec![Term::Operation {
            column: WithPos {
                value: "_column1234".to_string(),
                start: 2,
                end: 13
            },
            operator: WithPos {
                value: Operator::NotContains,
                start: 14,
                end: 16
            },
            value: WithPos {
                value: Value::String("\"Hello\" world".to_string()),
                start: 17,
                end: 34
            }
        }])
    );
}

#[test]
pub fn simple_repr() {
    assert_eq!(
        parser::parse_query("").map(|ts| ts.save_repr()),
        Ok("".to_string())
    );

    assert_eq!(
        parser::parse_query(r#" stars >= 5"#).map(|v| v.save_repr()),
        Ok("stars >= 5".to_string())
    );

    assert_eq!(
        parser::parse_query(
            r#" hello
        world ,, one two"#
        )
        .map(|ts| ts.save_repr()),
        Ok("hello world ,, one two".to_string())
    );

    assert_eq!(
        parser::parse_query(r#" _column1234 !~ "\"Hello\" world" "#).map(|ts| ts.save_repr()),
        Ok("_column1234 !~ \"\\\"Hello\\\" world\"".to_string())
    );
}

#[test]
pub fn query_samples() {
    assert_eq!(
        parser::parse_query(r#"A  simple sequence of keywords"#).map(|ts| ts.save_repr()),
        Ok("A simple sequence of keywords".to_string())
    );

    assert_eq!(
        parser::parse_query(
            r#"Hayao sortby:title title ~ "%one two%" release >= 2000  Miyazaki sortby: release  , desc"#
        )
        .map(|ts| ts.save_repr()),
        Ok("Hayao sortby:title title ~ \"%one two%\" release >= 2000 Miyazaki sortby:release,desc".to_string())
    );

    assert_eq!(
        parser::parse_query(
            r#"
            title  ~ "%猫%
            物語%ep%"
            sortby : foo
            stars > 5
            sortby : tag,asc
            stars <= 5
            sortby:stars
        "#
        )
        .map(|ts| ts.save_repr()),
        Ok("title ~ \"%猫%\\n            物語%ep%\" sortby:foo stars > 5 sortby:tag,asc stars <= 5 sortby:stars".to_string())
    );
}

#[test]
pub fn sortby_expansion() {
    assert_eq!(
        parser::parse_query(r#" sOrtBy:foo , asc"#).map(|ts| ts.save_repr()),
        Ok("sortby:foo,asc".to_string())
    );
    assert_eq!(
        parser::parse_query(r#" sOrtBy : foo , keyword"#).map(|ts| ts.save_repr()),
        Ok("sortby:foo , keyword".to_string())
    );
}
