# Ayumu

A small, lightweight, user-oriented query language for search forms.

```sql
-- Search 'Hayao', 'Miyazaki' keywords such that release >= 2000
-- then sort by title then year
Hayao sortby:title year>=2000 Miyazaki sortby:year asc
```

The syntax is designed to be fast, natural, fault tolerant and easy to write on
a search textbox.

Terms are separated by whitespaces and can be either a comparison, a sort-by
instruction, or a keyword (only if unrecognized as a command).

> The symbols were picked based on how easy they are to reach on either a PC
> keyboard or a smartphone.

For example, one use case is to produce a consistent representation that can be
stored in a database as user-created filters, and then compiled into a **safe**
SQL string.

```rust
// Sample from src/tests/sqlite.rs 
// using SQLiteWhere, which is an example of such compiler

// Specify the target columns
let mut sqlite = SQLiteWhere::new(list_string(&["title", "tags", "year"]), true);

// Specify which column to compare orphan keywords with
sqlite
    .match_keywords_with(list_string(&["title", "tags"]))
    .unwrap();

// The parser is very fault-tolerant, meaning it will  
// eat up anything and error out rarely. Most errors will be caught at the compiler level  
// (e.g. bad column name in a well-defined term).
let output = sqlite.convert("Hayao sortby:title year>=2000 Miyazaki sortby:year asc");

debug_assert_eq!(
    output,
    Ok(WhereClause {
        where_clause: "(title LIKE ? OR tags LIKE ?) AND (year >= ?)".to_string(),
        order_by: "title, tags, RANDOM(), year ASC".to_string(),
        bindings: vec![
            (
                "title".to_string(),
                Value::String("%Hayao%Miyazaki%".to_string())
            ),
            (
                "tags".to_string(),
                Value::String("%Hayao%Miyazaki%".to_string())
            ),
            ("year".to_string(), Value::Number(2000.0))
        ]
    })
);
```
