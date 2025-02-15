use crate::{
    ast::Value,
    converters::{
        sqlite::{SQLiteWhere, WhereClause},
        Convert,
    },
    tests::list_string,
};

#[test]
pub fn empty_query() {
    let mut sqlite = SQLiteWhere::new(list_string(&["title", "tags", "year"]), true);
    assert_eq!(
        sqlite.match_keywords_with(list_string(&["tagsB"])),
        Err("Invalid column \"tagsB\"".to_string())
    );

    sqlite.match_keywords_with(list_string(&[])).unwrap();

    debug_assert_eq!(
        sqlite.convert("should be ignored as there are no columns for match keywords"),
        Ok(WhereClause {
            where_clause: "".to_string(),
            order_by: "".to_string(),
            bindings: vec![]
        })
    );

    debug_assert_eq!(
        sqlite.convert("sortby : year , desc sortby :tags"),
        Ok(WhereClause {
            where_clause: "".to_string(),
            order_by: "year DESC, tags".to_string(),
            bindings: vec![]
        })
    );
}

#[test]
pub fn simple_keyword() {
    let mut sqlite = SQLiteWhere::new(list_string(&["title", "tags", "year"]), true);
    sqlite
        .match_keywords_with(list_string(&["title", "tags"]))
        .unwrap();

    debug_assert_eq!(
        sqlite.convert("Some title !~ BadTitle Keyword sortby:tags, desc"),
        Ok(WhereClause {
            where_clause: "(title LIKE ? OR tags LIKE ?) AND (title NOT LIKE ?)".to_string(),
            order_by: "tags DESC".to_string(),
            bindings: vec![
                Value::String("%Some%Keyword%".to_string()),
                Value::String("%Some%Keyword%".to_string()),
                Value::String("BadTitle".to_string())
            ]
        })
    );

    debug_assert_eq!(
        sqlite.convert("Hayao sortby:title sortby:tags ,rand year>=2000 Miyazaki sortby:year,asc"),
        Ok(WhereClause {
            where_clause: "(title LIKE ? OR tags LIKE ?) AND (year >= ?)".to_string(),
            order_by: "title, tags, RANDOM(), year ASC".to_string(),
            bindings: vec![
                Value::String("%Hayao%Miyazaki%".to_string()),
                Value::String("%Hayao%Miyazaki%".to_string()),
                Value::Number(2000.0)
            ]
        })
    );
}
