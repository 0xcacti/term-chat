pub mod error;
use rusqlite::{params, Connection};

use crate::client::Client;

use self::error::DBError;

pub fn setup_db() -> Result<(), DBError> {
    let conn = Connection::open("chat_app.db").map_err(DBError::Connection)?;

    // Set up tables
    create_user_table(&conn)?;
    create_sessions_table(&conn)?;

    Ok(())
}

fn create_user_table(conn: &Connection) -> Result<(), DBError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            uuid TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL
        )",
        params![],
    )
    .map_err(DBError::Setup)?;
    Ok(())
}

fn check_user_exists(conn: &Connection, username: &str) -> Result<(), DBError> {
    // let uuid: Option<String> =

    match conn.query_row(
        "SELECT uuid FROM users WHERE username = ?1",
        params![username],
        |row| row.get::<_, String>(0),
    ) {
        Ok(_) => Err(DBError::UserExists),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(()),
        Err(e) => Err(DBError::Query(e)),
    }
}

fn create_sessions_table(conn: &Connection) -> Result<(), DBError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            session_id INTEGER PRIMARY KEY,
            user_uuid TEXT,
            addr TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(user_uuid) REFERENCES users(uuid)
        )",
        params![],
    )
    .map_err(DBError::Setup)?;
    Ok(())
}

pub fn register_client(conn: &Connection, client: &Client) -> Result<(), DBError> {
    // match check_user_exists(conn, &client.name) {
    //     Ok(_) => (),
    //     Err(e) => return Err(e),
    // }
    // conn.execute(
    //     "INSERT INTO clients (id, name, addr) VALUES (?1, ?2, ?3)",
    //     params![client.id.to_string(), client.name, client.addr.to_string()],
    // )
    //.map_err(DBError::Query)?;
    Ok(())
}
