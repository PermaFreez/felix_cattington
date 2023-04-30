use std::env;
use rusqlite::Connection;

pub fn add_ownership(user_id: &String, filename: &String) {
    let conn = Connection::open(env::var("DATABASE").unwrap()).unwrap();

    let query = "SELECT Memes FROM users WHERE UserId = ?1";
    let mut stmt = conn.prepare(&query).unwrap();

    let mut memes_vec: Vec<String> = Vec::new();
    let mut did_run = false;
    for row in stmt.query_map(&[("?1", &user_id)], |row| Ok(row.get(0).unwrap())).unwrap() {
        let memes: String = row.unwrap();

        memes_vec = serde_json::from_str(&memes).unwrap();
        memes_vec.push(filename.clone());
        did_run = true;
    }

    let mut query2 = "INSERT INTO users (UserId, Memes) VALUES (?2, ?1)";

    if did_run {
        query2 = "UPDATE users SET Memes = ?1 WHERE UserId = ?2";
    } else {
        memes_vec.push(filename.to_string());
    }    

    let memes_new: String = serde_json::to_string(&memes_vec).unwrap();
    conn.execute(&query2, (&memes_new, &user_id)).unwrap();
}