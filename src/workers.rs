use serde_json::Value;
use std::env;
use std::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::iam;

pub enum Message {
    Status(Value),
    Capacities(Value),
    Stats(Value),
    Jobs(Value),
}

fn get_api_url() -> String {
    env::var("api_url").expect("Environment variable api_url not set")
}

pub fn status(tx: Sender<Message>) {
    println!("Status worker started");

    let api_url = get_api_url();
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

        let send_res = tx.send(Message::Status(res_value.unwrap()));
        if send_res.is_err() {
            println!("Status worker error: {}", send_res.err().unwrap());
            continue;
        }
    }
}

pub fn capacities(tx: Sender<Message>) {
    println!("Capacities worker started");

    let api_url = get_api_url();
    let url = format!("{}/landing/v2/capacities?n=1", api_url);

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

        let send_res = tx.send(Message::Capacities(res_value.unwrap()));
        if send_res.is_err() {
            println!("Capacities worker error: {}", send_res.err().unwrap());
            continue;
        }
    }
}

pub fn stats(tx: Sender<Message>) {
    println!("Stats worker started");

    let api_url = get_api_url();
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

        let send_res = tx.send(Message::Stats(res_value.unwrap()));
        if send_res.is_err() {
            println!("Stats worker error: {}", send_res.err().unwrap());
            continue;
        }
    }
}

pub fn jobs(tx: Sender<Message>) {
    let args = "pageSize=10&sortBy=createdAt&sortOrder=-1&excludeType=repairDeployment&excludeType=repairVm&excludeType=repairSm&excludeStatus=failed&excludeStatus=terminated&excludeStatus=pending";

    let api_url = get_api_url();
    let url = format!("{}/deploy/v1/jobs?{}", api_url, args);

    // use a variable "token" and a "fetchedAt" timestamp
    // if the token is not set, fetch it from the API
    // if the token is set, check if the timestamp is older than 1 hour
    // if the timestamp is older than 1 hour, fetch a new token

    let mut token = String::new();
    let mut fetched_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(300));

        // Check if token is set and if fetched_at is older than 1 hour
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if token.is_empty() || current_time - fetched_at > 3600 {
            // Fetch new token from the API
            match iam::get_oidc_token() {
                Ok(new_token) => {
                    token = new_token;
                    fetched_at = current_time;
                }
                Err(e) => {
                    println!("Error fetching token: {}", e);
                    continue;
                }
            }
        }

        let client = reqwest::blocking::Client::new();
        let response = client.get(&url).bearer_auth(token.clone()).send();

        if response.is_err() {
            println!(
                "Jobs worker error, no response: {}",
                response.err().unwrap()
            );
            continue;
        }

        let res = response.unwrap();
        if res.status().is_client_error() {
            println!("Jobs worker error, client error: {}", res.status());
            continue;
        }

        let json_res = res.json::<Value>();
        if json_res.is_err() {
            println!(
                "Jobs worker error, could not parse response : {}",
                json_res.err().unwrap()
            );
            continue;
        }

        tx.send(Message::Jobs(json_res.unwrap())).unwrap();
    }
}
