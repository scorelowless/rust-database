# Rust implementation of SQL-like database

The task was to implement a simple database in rust, allowing basic creation/deletion of tables and records as well as basic query operations like SELECT (WHERE), ORDER_BY, LIMIT.

The interaction is handled through the CLI and allows to read/save executed commands to/from a file.

The database handles i64 and String keys. For other values, i64, f64, String and bool are allowed.

The app is also equipped with error handling, communicating errors to the user in a meaningful way.

The app also contains a manually written parser for parsing the commands.