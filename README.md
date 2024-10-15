
# `sql_aware`

`sql_aware` is a Rust procedural macro designed to make SQL queries feel more integrated and native within Rust code. The macro enables you to write SQL queries directly in Rust while allowing for Rust expressions to be embedded seamlessly within the SQL syntax.

## Objective

The goal of `sql_aware` is to bring a more natural SQL writing experience to Rust developers by making Rust "SQL aware." This means that, instead of relying on external query builders or complex syntax, you can write SQL queries just like you would in a SQL environment, but with the added benefit of Rust’s safety and type system.

## Features

- Write SQL queries naturally within Rust.
- Embed Rust expressions into SQL queries using `{}`.
- Compile-time SQL syntax checking.
- Parametrized queries to prevent SQL injection (ongoing improvements).
- A familiar SQL-like syntax while leveraging the power of Rust.

## Usage

Using `sql!` is simple: write your SQL query as you would normally, and insert Rust expressions into the query using curly braces (`{}`).

### Example

Here's how you can dynamically generate a SQL query with interpolated values from Rust:

```rust
use sql_aware::sql;

fn main() {
    let table_name = "users";
    let name = "John Doe";
    let input = sql!(
        SELECT users.name
        FROM {table_name}
        WHERE deleted IS NULL
        AND name = "{name}"
        AND color = "blue"
    );

    let expected = "SELECT users.name FROM users WHERE deleted IS NULL AND name = 'John Doe' AND color = 'blue'";

    assert_eq!(input.query, expected);
    println!("Query: {}", input.query);
    println!("Parameters: {:?}", input.params);
}
```

The resulting SQL query will be:

```sql
SELECT users.name FROM users WHERE deleted IS NULL AND name = 'John Doe' AND color = 'blue'
```

### Interpolating Expressions

The `sql!` macro allows for seamless interpolation of Rust expressions into SQL queries. For instance:

```rust
let limit = 10;
let offset = 20;

let query = sql!(
    SELECT id, name
    FROM users
    WHERE active = true
    LIMIT {limit}
    OFFSET {offset}
);
```

The resulting query:

```sql
SELECT id, name FROM users WHERE active = true LIMIT 10 OFFSET 20
```

### Compile-Time SQL Syntax Validation

`sql_aware` uses compile-time checks to validate the syntax of your SQL queries. This ensures that your queries are syntactically correct before running them:

```rust
let query = sql!(
    SELECT name
    FROM WHERE users  -- Syntax error caught at compile time!
);
```

### String Literals

String literals are automatically escaped and integrated into the query:

```rust
let query = sql!(
    SELECT name
    FROM products
    WHERE description = "Best product!"
);
```

Generates the SQL query:

```sql
SELECT name FROM products WHERE description = 'Best product!'
```

## How It Works

The macro converts Rust expressions inside `{}` into SQL placeholders (or inline values in the case of literals). This ensures that string values are properly formatted as part of the SQL query, while expressions are directly injected.

The macro also ensures that SQL strings are properly formatted for your database driver.

## Limitations

- **SQL Injection Prevention**: The macro currently supports safe interpolation through string conversions, but additional improvements are being developed to further enhance SQL injection prevention.
- **Handling Single Quotes**: The use of single quotes (`'`) within the `sql!` macro has not been fully implemented due to limitations in Rust’s handling of strings as characters. Currently, double quotes (`"`) are used in the SQL syntax, which are later converted to single quotes (`'`) in the final query. Contributions and suggestions for improving this behavior are welcome.
- **Complex Queries**: Some advanced SQL queries might require custom handling, but the macro is designed to handle most common SQL constructs smoothly.
- **Driver Compatibility**: The current version ensures that SQL queries generated are valid for most standard SQL drivers. Ensure that your database driver supports the expected query format.

## Contribution

We welcome contributions! If you'd like to add new features, improve the library, or report issues, feel free to open a pull request or issue on GitHub.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.
