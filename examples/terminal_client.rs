use reqwest::blocking::Client;
use std::time::{Duration, Instant};
use std::thread::sleep;


const URL: &str = "http://localhost:8080/snake";

fn main() {
    let client = Client::new();

    let interval = Duration::from_secs(2);
    let mut next_time = Instant::now() + interval;

    loop {
        let resp = client.get(URL).send().unwrap().text().unwrap();
        sleep(next_time - Instant::now());
        next_time += interval;

        print!("{}[2J", 27 as char);
        println!("{}", resp)
    }
}
