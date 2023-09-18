pub mod error;
use rusqlite::{params, Connection};

use crate::client::Client;

use self::error::DBError;

pub fn setup_db() -> Result<(), DBError> {
    let conn = Connection::open("users.db").map_err(DBError::Connection)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS clients (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            addr TEXT NOT NULL
        )",
        params![],
    )
    .map_err(DBError::Setup)?;
    Ok(())
}

pub fn register_client(conn: &Connection, client: &Client) -> Result<(), DBError> {
    // conn.execute(
    //     "INSERT INTO clients (id, name, addr) VALUES (?1, ?2, ?3)",
    //     params![client.id.to_string(), client.name, client.addr.to_string()],
    // )
    //.map_err(DBError::Query)?;
    Ok(())
}
