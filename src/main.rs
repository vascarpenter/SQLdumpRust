use std::sync::{Arc, Mutex};
use r2d2_oracle::OracleConnectionManager;
use r2d2_oracle::r2d2::PooledConnection;
use structopt::StructOpt;

// Rust版 SQLbackup = SQLdumpRust

#[derive(Debug, StructOpt)]
#[structopt(name = "SQLdumpRust", about = "SQLdumpRust: Oracle table dump utility")]
pub struct SQLdumpRust {
    /// specify table names separated by ','
    #[structopt(short, long)]
    tables: Option<String>,

    /// drop table
    #[structopt(short, long)]
    drop: bool,

    /// oci connect string eg. admin/pass@//123.45.67.89/XEPDB1
    #[structopt(short, long, required_unless = "dbenv")]
    ocistring: Option<String>,

    /// environment variable name which contains oci connect string
    #[structopt(long)]
    dbenv: Option<String>,
}

// global variables
#[derive(Debug, Default)]
pub struct AppState {
    drop: bool,
}

fn table_analysis(tablename: &String,
                  conn: &PooledConnection<OracleConnectionManager>,
                 globals: Arc<Mutex<AppState>>,
)
{
    let table = tablename.to_string();
    let mut ddl = "".to_string();

    let sql = format!("select dbms_metadata.get_ddl('TABLE','{}') from dual", table);
    let rows = conn.query_as::<String>(sql.as_str(), &[]).expect("fail query");
    for row_result in rows {
        let ddls = row_result.unwrap();
        ddl.push_str(&ddls);
        ddl.push_str("\n");
    }
    ddl.push_str(";\n");

    // DDL の中に INDEX PKがあるものがあり、"ALTER TABLE "ADMIN"."XXX" ADD PRIMARY KEY" の前にセミコロンがない→つける

    ddl = ddl.replace(" \nALTER TABLE ",";\nALTER TABLE ");
    ddl = ddl.replace(" \n  CREATE UNIQUE INDEX",";\nCREATE UNIQUE INDEX");

    if globals.lock().unwrap().drop == true {
        println!("DROP TABLE {};",table);
    }
    println!("{}",ddl);

    let sql = format!("select * from {}", table);
    let rows = conn.query(sql.as_str(),&[]).expect("query err");
    let column_info = rows.column_info().to_owned(); // loop内で使用されるため cloneしておく必要がある

    println!("SET DEFINE OFF;");

    for row_result in rows {

        let row = row_result.unwrap();
        print!("Insert Into {} (",table);

        let sqlv = row.sql_values().to_owned();
        for (colidx,val) in sqlv.iter().enumerate() {
            if colidx > 0 {
                print!(",");
            }
            print!(r##""{}""##,column_info[colidx].name());
        }
        print!(") VALUES (");
        for (colidx,val) in sqlv.iter().enumerate() {
            if colidx > 0 {
                print!(",");
            }
            let oratype = column_info[colidx].oracle_type().to_string();
            if let Ok(nullflag) = val.is_null() {
                if nullflag {
                    print!("NULL");
                } else if oratype.starts_with("NVARCHAR2")
                    || oratype.starts_with("VARCHAR2")
                    || oratype.starts_with("NVARCHAR") {
                    print!("'{}'", val.to_string());
                } else if oratype.starts_with("NUMBER") {
                    print!("{}", val.to_string());
                } else if oratype.starts_with("DATE") {
                    print!("TO_DATE('{}','YYYY-MM-DD HH24:MI:SS')", val.to_string());
                } else if oratype.starts_with("BLOB") {
                    if let Ok(data) = val.get::<Vec<u8>>() {
                        let result = data.iter().map(|n| format!("{:02X}", n)).collect::<String>();
                        print!("HEXTORAW('{}')", result);
                    }
                } else {
                    print!("'not supported:{}'", oratype);
                }
            }
        }
        println!(");");
    }

}

// oracle形式の connection string を分解して、username,password,connect stringの形式にする

fn divide_ocistring(ocistring: String)-> Vec<String>
{
    let mut v = Vec::new();
    let atmarksep: Vec<&str> = ocistring.split("@").collect();
    let userpass = atmarksep[0];
    let slashsep: Vec<&str> = userpass.split("/").collect();
    v.push(slashsep[0].to_string());
    v.push(slashsep[1].to_string());
    v.push(atmarksep[1].to_string());
    return v;
}

fn main() {
    let options = SQLdumpRust::from_args();
    // global data, fetched from each workers
    let data = Arc::new(Mutex::new(AppState {
        drop: options.drop,
    }));

    let mut ocistring: String = options.ocistring.unwrap_or(String::from(""));

    if let Some(dbe) = options.dbenv {
        match std::env::var(&dbe) {
            Ok(v) =>
                ocistring = v,
            Err(_) =>
                panic!("Error get env var {}", dbe.to_string()),
        }
    }
    if ocistring == "" {
        panic!("--ocistring <oracle db connect string> or --dbenv <ENV name which holds oracle db connection string> needed");
    }

    let vec = divide_ocistring(ocistring.to_string());
    let manager = OracleConnectionManager::new(
        &vec[0],
        &vec[1],
        &vec[2]);

    let pool = r2d2::Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("Failed to create pool; exceeds max connection?");

    let conn = pool.get().expect("cannot get from pool");

    if let Some(t) = options.tables {
        let tablenames: Vec<&str> = t.split(",").collect();
        for table in tablenames {
            table_analysis(&table.to_string(), &conn, data.clone());
        }
    }
    else {
        // select all tables
        let sql = "select table_name from user_tables";
        let rows = conn.query_as::<String>(sql, &[]).expect("fail query");
        for row_result in rows {
            let tablename = row_result.unwrap();
            table_analysis(&tablename, &conn, data.clone());
        }
    }
}
