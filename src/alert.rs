use std::thread;
use std::collections::HashMap;
use reqwest::blocking::Client;

pub fn alert(msg: &str) {
    let msg = msg.to_string();
    thread::spawn(|| {
        let client = Client::new();
        let url = "https://hooks.slack.com/services/xxxxxxxxxxx/xxxxxxxxxxx/xxxxxxxxxxxxxxxxxxxxxxxx";
        let mut data = HashMap::new();
        data.insert("text", msg);
        client.post(url)
              .json(&data)
              .header("Content-type", "application/json")
              .send();
        });
}
