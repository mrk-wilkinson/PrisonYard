use Justice::Inmate;
use Justice::actions::c2_actions;
use rusqlite::{Connection, Result, params};
use std::time::SystemTime;

pub fn create_db() {
    let conn = Connection::open("prisoninmates.db");
    match conn {
        Ok(conn) => {
           match conn.execute(
                "CREATE TABLE IF NOT EXISTS inmates (
                    rowid INTEGER PRIMARY KEY,
                    os TEXT,
                    hostname TEXT,
                    ip TEXT,
                    pid INTEGER,
                    last_checkin INTEGER,
                    pending_instruct TEXT,
                    pending_instruct_type TEXT,
                    request_actions TEXT,
                    completed_actions TEXT
                )",
                [],
            ) {
                Ok(_) => {
                    {}
                }
                Err(_) => {
                    panic!("Error creating table");
                }
            }
        },
        Err(_) => {
            panic!("Error creating database, make sure you can write a file to the current directory, or modify the path");
        }
    }
}

pub fn implant_exists(implant_id: u32) -> Result<Inmate, &'static str> {
    let inmates = get_all_inmates();
    for inmate in inmates {
        if inmate.rowid == implant_id {
            return Ok(inmate);
        }
    }
    return Err("Does not exist")
}

pub fn update_database(inmate: Inmate) {
    let conn = Connection::open("prisoninmates.db").unwrap();
    let _ = conn.execute(
        "UPDATE inmates SET os = ?1, hostname = ?2, ip = ?3, pid = ?4, last_checkin = ?5, pending_instruct = ?6, pending_instruct_type = ?7, request_actions = ?8, completed_actions = ?9 WHERE rowid = ?10",
        params![inmate.os, inmate.hostname, inmate.ip, inmate.pid, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), inmate.pending_instruct, inmate.pending_instruct_type.to_string(), serde_json::to_string(&inmate.request_actions).unwrap(), serde_json::to_string(&inmate.completed_actions).unwrap(), inmate.rowid]
    );
}
pub fn insert_inmate(inmate: Inmate) {
    let conn = Connection::open("prisoninmates.db").unwrap();
    let _ = conn.execute(
        "INSERT INTO inmates (rowid, os, hostname, ip, pid, last_checkin, pending_instruct, pending_instruct_type, request_actions, completed_actions) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![inmate.rowid, inmate.os, inmate.hostname, inmate.ip, inmate.pid, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), inmate.pending_instruct, inmate.pending_instruct_type.to_string(), serde_json::to_string(&inmate.request_actions).unwrap(), serde_json::to_string(&inmate.completed_actions).unwrap()]
    );
}
pub fn get_all_inmates() -> Vec<Inmate> {
    let conn = Connection::open("prisoninmates.db").unwrap();
    let mut stmt = conn.prepare("SELECT * FROM inmates").unwrap();
    let mut inmate_iter = stmt.query_map([], |row| {
        let instr_type: String = row.get(7)?;
        let request_act: String = row.get(8)?;
        let completed_act: String = row.get(9)?;

        Ok(Inmate {
            rowid: row.get(0)?,
            os: row.get(1)?,
            hostname: row.get(2)?,
            ip: row.get(3)?,
            pid: row.get(4)?,
            last_checkin: row.get(5)?,
            pending_instruct: row.get(6)?,
            pending_instruct_type: instr_type.parse::<c2_actions>().unwrap(),
            request_actions: serde_json::from_str(&request_act).unwrap(),
            completed_actions: serde_json::from_str(&completed_act).unwrap(),
        })
    }).unwrap();
    let mut all_inmates: Vec<Inmate> = Vec::new();
    for inmate in inmate_iter {
        match inmate {
            Ok(inmate_unwrapped) => {
                all_inmates.push(inmate_unwrapped);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    return all_inmates;
}