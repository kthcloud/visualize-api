use chrono::Utc;
use dotenv::dotenv;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::Serialize;
use rocket::{get, launch, routes, State};
use rocket_cors::CorsOptions;
use serde_json::{Map, Value};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

mod iam;
mod workers;

#[derive(Serialize)]
struct Res {
    date: String,
    status: Value,
    capacities: Value,
    stats: Value,
    jobs: Value,
}

#[get("/")]
async fn index(shared_data: &State<Arc<Mutex<Res>>>) -> String {
    // return data as a json string
    let mut res = shared_data.lock().unwrap();

    res.date = Utc::now().to_rfc3339();

    serde_json::to_string(&*res).unwrap()
}

#[get("/healthz")]
fn healthz() -> status::Custom<&'static str> {
    status::Custom(Status::Ok, "OK")
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let (tx, rx) = mpsc::channel();
    let shared_status = Arc::new(Mutex::new(Res {
        date: String::from(Utc::now().to_rfc3339()),
        status: Value::Object(Map::new()),
        capacities: Value::Object(Map::new()),
        jobs: Value::Object(Map::new()),
        stats: Value::Object(Map::new()),
    }));

    let status_for_thread = Arc::clone(&shared_status);
    thread::spawn(move || update_res_thread(rx, status_for_thread));

    // Spawn worker threads
    {
        let tx_clone = tx.clone();
        thread::spawn(move || workers::status(tx_clone));
    }

    {
        let tx_clone = tx.clone();
        thread::spawn(move || workers::capacities(tx_clone));
    }

    {
        let tx_clone = tx.clone();
        thread::spawn(move || workers::stats(tx_clone));
    }

    {
        let tx_clone = tx.clone();
        thread::spawn(move || workers::jobs(tx_clone));
    }

    let cors = CorsOptions::default()
        .to_cors()
        .expect("CORS configuration error");

    rocket::build()
        .attach(cors)
        .manage(shared_status)
        .mount("/", routes![index, healthz])
}

fn update_res_thread(rx: Receiver<workers::Message>, res: Arc<Mutex<Res>>) {
    for received in rx {
        let mut res = res.lock().unwrap();
        match received {
            workers::Message::Status(status) => res.status = status,
            workers::Message::Capacities(capacities) => res.capacities = capacities,
            workers::Message::Stats(stats) => res.stats = stats,
            workers::Message::Jobs(jobs) => res.jobs = jobs,
        }
    }
}
