## SQLdumpRust

### これはなんですか
- SQLdumpRustは、Go版の SQLbackup の代替品です
- written in Rust
- テーブルをテキストのSQL形式で書き出します
- output SQL, stdout

### ビルド
- `cargo build`
  - oracle instant client libraryのインストールが必要

### 制約
- サポートしているoracle typeは以下の通りです
    - `NVARCHAR2, VARCHAR2, NVARCHAR`
    - `NUMBER`
    - `DATE`
    - `BLOB`
    - その他の型はサポートしていないので自分で追加するべき

### 起動オプション
- `SQLdumpRust --dbenv <環境変数> / --ocistring <connect string> [--drop] [--tables table1,table2,..]`
- `--dbenv <環境変数>` あるいは `--ocistring <接続文字列>` dbへの接続方法を指定
- `--drop`  `DROP TABLE `を追加する
- `--tables <table1,table2,...>` dumpするテーブルを指定

