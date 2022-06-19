use chrono::{DateTime, Local};
use repng;
use scrap::{Capturer, Display};

use std::fs::File;
use std::io::BufRead;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
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
  let (w, h) = (capturer.width(), capturer.height());

  let like = Arc::new(like);
  let frames_shots = Arc::new(AtomicU64::new(0));
  let frames_saved = Arc::new(AtomicU64::new(0));
  let pause = Arc::new(AtomicBool::new(false));
  let stop = Arc::new(AtomicBool::new(false));

  std::thread::spawn({
    let like = like.clone();
    let frames_shots = frames_shots.clone();
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
        } else if command == "shots" {
          println!("{}", frames_shots.load(Ordering::Acquire));
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
          println!("Stopping");
          break;
        }
      }
    }
  });

  let nanos_time_base = 1_000_000_000 / like.frames_ps;
  let capture_interval = Duration::from_nanos(nanos_time_base);
  let start_time = Instant::now();
  println!("Started");

  let to_save_pool: Arc<Mutex<Vec<(DateTime<Local>, Vec<u8>)>>> =
    Arc::new(Mutex::new(Vec::new()));

  let saving = std::thread::spawn({
    let to_save_pool = to_save_pool.clone();
    let stop = stop.clone();
    let mut flipped = Vec::with_capacity(w * h * 4);
    move || loop {
      let (time, frame) = {
        let mut to_save_pool = to_save_pool.lock().unwrap();
        if to_save_pool.is_empty() {
          if stop.load(Ordering::Acquire) {
            break;
          }
          std::thread::sleep(std::time::Duration::from_millis(1));
          continue;
        } else {
          to_save_pool.pop().unwrap()
        }
      };
      flipped.clear();
      let stride = frame.len() / h;
      for y in 0..h {
        for x in 0..w {
          let i = y * stride + 4 * x;
          flipped.extend_from_slice(&[frame[i + 2], frame[i + 1], frame[i], 255]);
        }
      }
      let file_name = format!("{}.png", time.format("%Y-%m-%d-%H-%M-%S-%3f"));
      repng::encode(
        File::create(file_name).unwrap(),
        w as u32,
        h as u32,
        &flipped,
      )
      .unwrap();
      frames_saved.fetch_add(1, Ordering::AcqRel);
      println!("Saved: {}", frames_saved.load(Ordering::Acquire));
    }
  });

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
        let time = Local::now();
        let to_save = Vec::from(&frame[..]);
        let mut to_save_pool = to_save_pool.lock().unwrap();
        to_save_pool.push((time, to_save));
        frames_shots.fetch_add(1, Ordering::AcqRel);
        println!("Shots: {}", frames_shots.load(Ordering::Acquire));
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
        std::thread::sleep(sleep_duration);
      }
    }
  }
  saving.join().unwrap();
  println!("Finished");
  Ok(())
}
