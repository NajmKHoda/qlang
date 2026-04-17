# Query Expressions

## Syntax

```rs
// Immediate query
"query" "{" Query "}"

// Parameterized query
"query" "(" [QName ":" Typename{"," QName ":" Typename}] ")"
"{" Query "}"
```

## Explanation

Queries are expressions, not standalone top-level declarations. QLang supports *immediate queries* (which execute immediately) and *parameterized queries* (which are stored to be called later). Parameterized queries use prepared statements under the hood for optimal performance.

## Examples

```ql
datasource data;

table User from data {
    name: str
}

failable function get_all_users() -> void {
    let user_iter: iter<User> = query { select all from User };
    let users: User[] = user_iter.collect();

    let insertUser = query(_name: str) {
        insert { name: _name }
        into User
    };

    for parent in users {
        let son_name = "son of " + parent.name;
        prints(parent.name);
        prints(son_name);
        insertUser(son_name);
    }
}
```

Expected output:
```text
Umar
son of Umar
John Doe
son of John Doe
Dwayne Johnson
son of Dwayne Johnson
```

## See Also

- [Select Query](../queries/select.md)
- [Insert Query](../queries/insert.md)
- [Lone Expression Statements](../statements/expression-statements.md)
