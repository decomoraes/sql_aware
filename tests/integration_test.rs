use sql_aware::sql;

#[test]
fn test_sql() {
    let table_name = "users";
    let name = "John Doe";
    let input = sql!(
        SELECT users.name
        FROM {table_name}
        WHERE deleted IS NULL
        AND name = "{name}"
        AND color = "blue"
    );

    let limit = 10;
    let offset = 20;

    let query = sql!(
        SELECT id, name
        FROM users
        WHERE active = true
        LIMIT {limit}
        OFFSET {offset}
    );

    println!("query: {:#?}\n\n", query);

    let expected = "SELECT users.name FROM users WHERE deleted IS NULL AND name = 'John Doe' AND color = 'blue'";

    println!("{:#?}\n\n", input);

    assert_eq!(expected.to_string(), input.to_string());
}
