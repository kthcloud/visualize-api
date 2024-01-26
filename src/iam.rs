use reqwest;
use serde_json::Value;
use std::collections::HashMap;
use std::env;

pub fn get_oidc_token() -> Result<String, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok(); // Load .env file

    let client_id = env::var("oidc_resource").expect("Environment variable oidc_resource not set");
    let client_secret = env::var("oidc_secret").expect("Environment variable oidc_secret not set");
    let token_url = env::var("oidc_auth_server_url")
        .expect("Environment variable oidc_auth_server_url not set");
    let username = env::var("username").expect("Environment variable oidc_username not set");
    let password = env::var("password").expect("Environment variable oidc_password not set");

    let mut data = HashMap::new();
    data.insert("client_id", client_id);
    data.insert("client_secret", client_secret);
    data.insert("grant_type", String::from("password"));
    data.insert("username", username.to_string());
    data.insert("password", password.to_string());

    let client = reqwest::blocking::Client::new();
    let res = client.post(&token_url).form(&data).send()?;

    if res.status().is_success() {
        let response_data: Value = res.json()?;
        let token: String = response_data["access_token"].as_str().unwrap().to_string();
        return Ok(token);
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Error getting token: {:?}", res.text()?),
        )))
    }
}
