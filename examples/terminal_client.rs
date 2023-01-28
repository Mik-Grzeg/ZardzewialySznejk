use std::io;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use std::time;

use reqwest::blocking::Client;
use termion::event::Key::Char;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

const URL: &str = "http://localhost:8080/snake";

fn main() {
    let mut stdout = io::stdout().into_raw_mode().unwrap();

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
        game_text = game_text.replace('\n', "\r\n");

        write!(
            stdout,
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            game_text,
        )
        .unwrap();
        stdout.lock().flush().unwrap();

        // Read input (if any)
        let input = stdin.next();

        // If a key was pressed
        if let Some(Ok(key)) = input {
            if let Some(dir) = match key {
                Char('q') => break,
                Char('l') => Some("right"),
                Char('k') => Some("up"),
                Char('j') => Some("down"),
                Char('h') => Some("left"),
                _ => None,
            } {
                tx.send(dir).unwrap();
            }
        }
        thread::sleep(time::Duration::from_millis(50));
    }
    drop(tx);

    handle.join().unwrap();
}
