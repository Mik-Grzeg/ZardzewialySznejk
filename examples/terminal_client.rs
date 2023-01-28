use reqwest::blocking::Client;
// use std::time::{Duration, Instant};
// use std::thread::sleep;
// use std::thread;


const URL: &str = "http://localhost:8080/snake";

// fn main() {
//     let client = Client::new();

//     let interval = Duration::from_secs_f32(1.0/30.0);
//     let mut next_time = Instant::now() + interval;

    // let handle = thread::spawn(|| loop {
//         let resp = client.get(URL).send().unwrap().text().unwrap();
//         sleep(next_time - Instant::now());
//         next_time += interval;

//         print!("{}[2J", 27 as char);
//         println!("{}", resp)
//     });
// }

use std::io;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use std::time;

use termion;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::event::Key::Char;


fn main() {
    // Set terminal to raw mode to allow reading stdin one key at a time
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    // Use asynchronous stdin
    let mut stdin = termion::async_stdin().keys();

    let client = Client::new();


    let (tx, rx): (mpsc::Sender<&str>, mpsc::Receiver<&str>) = mpsc::channel();
    let post_client = client.clone();
    let handle = thread::spawn(move || {
        while let Ok(dir) = rx.recv() {
            post_client.post(format!("{URL}/{}", dir)).send().unwrap();
        }
    });


    loop {
        let mut game_text = client.get(URL).send().unwrap().text().unwrap();
        game_text = game_text.replace("\n", "\r\n");

        write!(
            stdout,
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            game_text,
        ).unwrap();
        stdout.lock().flush().unwrap();

        // Read input (if any)
        let input = stdin.next();

        // If a key was pressed
        if let Some(Ok(key)) = input {
            if let Some(dir) = match key {
                Char('q') => break,
                Char('l') => {
                    Some("right")
                    // stdout.lock().flush().unwrap();
                },
                Char('k') => {
                    Some("up")
                    // stdout.lock().flush().unwrap();
                },
                Char('j') => {
                    Some("down")
                },
                Char('h') => {
                    Some("left")
                    // stdout.lock().flush().unwrap();
                },
                _ => None
            } {
                tx.send(dir).unwrap();
            }
        }
        thread::sleep(time::Duration::from_millis(50));
    }
    drop(tx);

    handle.join().unwrap();
}
