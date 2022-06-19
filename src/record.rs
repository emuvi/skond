use scrap::{Capturer, Display};

use std::io::BufRead;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct Like {
  display: usize,
  duration: Option<u64>,
  frames_ps: u64,
  destiny: PathBuf,
}

pub fn start(
  display: usize,
  duration: Option<u64>,
  frames_ps: u64,
  destiny: &str,
) -> std::io::Result<()> {
  record(Like {
    display,
    duration,
    frames_ps,
    destiny: destiny.into(),
  })
}

fn record(like: Like) -> std::io::Result<()> {
  let duration = like.duration.map(Duration::from_secs);

  let displays = Display::all().expect("Couldn't get a list of the displays.");
  let display = displays
    .into_iter()
    .nth(like.display)
    .expect(&format!("Display {} not found.", like.display));

  let mut capturer = Capturer::new(display).expect("Couldn't get a capturer.");

  let like = Arc::new(like);
  let frames_saved = Arc::new(AtomicU64::new(0));
  let pause = Arc::new(AtomicBool::new(false));
  let stop = Arc::new(AtomicBool::new(false));

  std::thread::spawn({
    let like = like.clone();
    let frames_saved = frames_saved.clone();
    let pause = pause.clone();
    let stop = stop.clone();
    let stdin = std::io::stdin();
    move || {
      for line in stdin.lock().lines() {
        let command = line.unwrap();
        let command = command.trim();
        if command == "like" {
          println!("{:?}", like);
        } else if command == "saved" {
          println!("{}", frames_saved.load(Ordering::Acquire));
        } else if command == "pause" {
          pause.store(true, Ordering::Release);
          println!("Paused");
        } else if command == "continue" {
          pause.store(false, Ordering::Release);
          println!("Continued");
        } else if command == "stop" {
          stop.store(true, Ordering::Release);
          println!("Stopped");
          break;
        }
      }
    }
  });

  let nanos_time_base = 1_000_000_000 / like.frames_ps;
  let capture_interval = Duration::from_nanos(nanos_time_base);
  let start_time = Instant::now();

  while !stop.load(Ordering::Acquire) {
    if pause.load(Ordering::Acquire) {
      std::thread::sleep(Duration::from_millis(1));
      continue;
    }
    let start_cycle = Instant::now();
    if Some(true) == duration.map(|d| start_time.elapsed() > d) {
      break;
    }
    let mut was_block = false;
    match capturer.frame() {
      Ok(frame) => {
        println!("Got frame in {}", start_time.elapsed().as_millis());
        frames_saved.fetch_add(1, Ordering::AcqRel);
      }
      Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
        was_block = true;
      }
      Err(e) => {
        eprintln!("{}", e);
      }
    }
    if !was_block {
      let cycle_elapsed = start_cycle.elapsed();
      if cycle_elapsed < capture_interval {
        let sleep_duration = capture_interval - cycle_elapsed;
        println!("Sleeping for {}", sleep_duration.as_millis());
        std::thread::sleep(sleep_duration);
      }
    }
  }
  Ok(())
}
