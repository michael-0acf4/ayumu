# Ayumu

A small, lightweight, user-oriented query language for search forms.

```sql
-- Search 'Hayao', 'Miyazaki' keywords such that release >= 2000
-- then sort by title then release
Hayao sortby:title release>=2000 Miyazaki sortby:release,asc
```

The syntax is designed to be fast, natural, fault tolerant and easy to write on
a search textbox.

Terms are separated by whitespaces and can be either a comparison, a sort-by
instruction, or a keyword (only if unrecognized as a command).

> The symbols were picked based on how easy they are to reach on either a PC
> keyboard or a smartphone.

The idea is to produce a non-string based representation (except values) that
can be **safely** compiled into a SQL query without having to fear injections.

```sql
SELECT * FROM Movies
WHERE
(
    title LIKE $1
    OR author LIKE $1
)
AND release >= 2000
ORDER BY title ASC, release DESC

-- $1: can be bound to '%Hayao%Miyazaki%'
```
