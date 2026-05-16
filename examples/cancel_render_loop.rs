use std::thread;
use std::time::Duration;

use tryx::cancel::{Cancel, CancelToken};

fn render_loop(token: CancelToken, max_frames: u64) -> Cancel<u64> {
    let mut frames = 0;

    for _ in 0..max_frames {
        token.check()?;
        frames += 1;
        thread::sleep(Duration::from_millis(1));
    }

    Cancel::Done(frames)
}

fn main() {
    let (token, handle) = CancelToken::new();

    let worker = thread::spawn(move || render_loop(token, 1_000));

    thread::sleep(Duration::from_millis(10));
    handle.cancel();

    match worker.join() {
        Ok(Cancel::Done(frames)) => println!("rendered {frames} frames"),
        Ok(Cancel::Cancelled) => println!("cancelled"),
        Err(_) => eprintln!("render thread panicked"),
    }
}
