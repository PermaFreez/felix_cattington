use std::time::Duration;
use rusqlite::Connection;
use tokio::time::sleep;

use log::info;

use crate::UNLOCK_TIME;

pub async fn unlock_public(filename: String) {
    sleep(Duration::from_secs(UNLOCK_TIME)).await;

    let conn = Connection::open("database.db").unwrap();

    let query = "SELECT tags FROM memes WHERE FileName = ?1";
    let mut stmt = conn.prepare(&query).unwrap();
    for row in stmt.query_map(&[("?1", &filename)], |row| Ok(row.get(0).unwrap())).unwrap() {
        let tags: String = row.unwrap();
        if !tags.is_empty() {
            return;
        }
    }

    let query2 = "SELECT UserId, Memes FROM users;";
    let mut stmt2 = conn.prepare(query2).unwrap();
    let mut memes: (String, String) = (String::new(), String::new());
    for row in stmt2.query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap()))).unwrap() {
        memes = row.unwrap();
        let memes_vec: Vec<String> = serde_json::from_str(&memes.1).unwrap();
        if memes_vec.contains(&filename) {
            break;
        }
    }

    let mut memes_vec: Vec<&str> = serde_json::from_str(&memes.1).unwrap();
    memes_vec.retain(|a| a != &filename.as_str());
    let new_memes: String = serde_json::to_string(&memes_vec).unwrap();

    let query3 = "UPDATE users SET Memes = ?1 WHERE UserId = ?2;";
    conn.execute(&query3, (new_memes, memes.0)).unwrap();

    let query4 = "INSERT INTO upforgrabs (FileName) VALUES (?1);";
    conn.execute(&query4, &[("?1", &filename)]).unwrap();

    info!("A \"{}\" mémet mostantól bárki tagelheti!", &filename);
}