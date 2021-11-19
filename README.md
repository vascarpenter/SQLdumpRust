## SQLdumpRust

### What is it?
- SQLdumpRust is utility for dump/back up oracle database tables
- written in Rust
- output SQL, stdout

### How to build
- `cargo build`
  - you must install oracle instant client library

### Limitations
- supported  oracle types are below:
  - `NVARCHAR2, VARCHAR2, NVARCHAR`
  - `NUMBER, FLOAT`
  - `DATE`
  - `BLOB`
  - other types? you can add it..

### Launch options
- `SQLdumpRust --dbenv <env var name> ` or `--ocistring <connect string> [--drop] [--tables table1,table2,..]`
- `--dbenv <environment variable name that holds connection string>`
  -  or `--ocistring <connect string>` specify connection settings
- `--drop`  `DROP TABLE ` before create table
- `--tables <table1,table2,...>` specify table name, if not specified, dump all tables
