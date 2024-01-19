use chrono::Utc;
use reqwest;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::Serialize;
use rocket::{get, launch, routes, State};
use serde_json::{Map, Value};
use std::env;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Serialize)]
struct Res {
    date: String,
    status: Value,
    capacities: Value,
    stats: Value,
    jobs: Value,
}

enum WorkerMessage {
    Status(Value),
    Capacities(Value),
    Stats(Value),
    // Jobs(Value),
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
        thread::spawn(move || status_worker(tx_clone));
    }

    {
        let tx_clone = tx.clone();
        thread::spawn(move || capacities_worker(tx_clone));
    }

    {
        let tx_clone = tx.clone();
        thread::spawn(move || stats_worker(tx_clone));
    }

    {
        // let tx_clone = tx.clone();
        // thread::spawn(move || jobs_worker(tx_clone));
    }

    rocket::build()
        .manage(shared_status)
        .mount("/", routes![index, healthz])
}

//
// Worker
//

fn status_worker(tx: Sender<WorkerMessage>) {
    println!("Status worker started");

    let api_url = env::var("API_URL").expect("API_URL not set");
    let url = format!("{}/landing/v2/status?n=100", api_url);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));

        let response = reqwest::blocking::get(&url).unwrap().text();
        if response.is_err() {
            println!("Status worker error: {}", response.err().unwrap());
            continue;
        }


        let res_value = serde_json::from_str(&response.unwrap());
        if res_value.is_err() {
            println!("Status worker error: {}", res_value.err().unwrap());
            continue;
        }

        let send_res = tx.send(WorkerMessage::Status(res_value.unwrap()));
        if send_res.is_err() {
            println!("Status worker error: {}", send_res.err().unwrap());
            continue;
        }
    }
}

fn capacities_worker(tx: Sender<WorkerMessage>) {
    println!("Capacities worker started");

    let api_url = env::var("API_URL").expect("API_URL not set");
    let url = format!("{}/landing/v2/status?n=1", api_url);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));

        let response = reqwest::blocking::get(&url).unwrap().text();
        if response.is_err() {
            println!("Capacities worker error: {}", response.err().unwrap());
            continue;
        }

        let res_value = serde_json::from_str(&response.unwrap());
        if res_value.is_err() {
            println!("Capacities worker error: {}", res_value.err().unwrap());
            continue;
        }

        let send_res = tx.send(WorkerMessage::Capacities(res_value.unwrap()));
        if send_res.is_err() {
            println!("Capacities worker error: {}", send_res.err().unwrap());
            continue;
        }
    }
}

fn stats_worker(tx: Sender<WorkerMessage>) {
    println!("Stats worker started");

    let api_url = env::var("API_URL").expect("API_URL not set");
    let url = format!("{}/landing/v2/stats?n=1", api_url);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));

        let response = reqwest::blocking::get(&url).unwrap().text();
        if response.is_err() {
            println!("Stats worker error: {}", response.err().unwrap());
            continue;
        }

        let res_value = serde_json::from_str(&response.unwrap());
        if res_value.is_err() {
            println!("Stats worker error: {}", res_value.err().unwrap());
            continue;
        }

        res = tx.send(WorkerMessage::Stats(res_value.unwrap()));
        if send_res.is_err() {
            println!("Stats worker error: {}", send_res.err().unwrap());
            continue;
        }
    }
}

// fn jobs_worker(tx: Sender<WorkerMessage>) {
//     let api_url = env::var("API_URL").expect("API_URL not set");
//     let url = format!("{}/deploy/v1/jobs?n=1&sortBy=createdAt&sortOrder=-1", api_url);

//     loop {
//         let response = reqwest::blocking::get(&url).unwrap().text().unwrap();
//         let res_value = serde_json::from_str(&response).unwrap();
//         tx.send(WorkerMessage::Jobs(res_value)).unwrap();
//     }
// }

fn update_res_thread(rx: Receiver<WorkerMessage>, res: Arc<Mutex<Res>>) {
    for received in rx {
        let mut res = res.lock().unwrap();
        match received {
            WorkerMessage::Status(status) => res.status = status,
            WorkerMessage::Capacities(capacities) => res.capacities = capacities,
            WorkerMessage::Stats(stats) => res.stats = stats,
            // WorkerMessage::Jobs(jobs) => res.jobs = jobs,
        }
    }
}
