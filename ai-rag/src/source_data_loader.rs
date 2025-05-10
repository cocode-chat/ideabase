use database::core::DBConn;

pub async  fn load_data_by(db: &DBConn, table: &str, column: &str, limit: i32, offset: i32) {
    let sql = format!("select {column} FROM `{table}` LIMIT {limit} OFFSET {offset};");
    let data = db.query_list(&sql, vec![]).await;
}