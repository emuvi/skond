use scrap::Display;

mod clip;

fn main() -> std::io::Result<()> {
  let args = clip::parse();
  if args.is_present("displays") {
    return displays();
  }
  let mut display: usize = 0;
  if let Some(screen_arg) = args.value_of("screen") {
    display = screen_arg.parse::<usize>().unwrap();
  }
  let mut duration: Option<usize> = None;
  if let Some(extent_arg) = args.value_of("extent") {
    duration = Some(extent_arg.parse::<usize>().unwrap());
  }
  let mut frames_ps: usize = 30;
  if let Some(frames_ps_arg) = args.value_of("frames_ps") {
    frames_ps = frames_ps_arg.parse::<usize>().unwrap();
  }
  if args.is_present("record") {
    let destiny = args
      .value_of("record")
      .expect("Could not parse the record PATH argument.");
    return start(display, duration, frames_ps, destiny);
  }
  Ok(())
}

fn displays() -> std::io::Result<()> {
  let displays = Display::all()?;
  for (i, display) in displays.into_iter().enumerate() {
    println!("Display {} [{}x{}]", i, display.width(), display.height());
  }
  Ok(())
}

fn start(
  display: usize,
  duration: Option<usize>,
  frames_ps: usize,
  destiny: &str,
) -> std::io::Result<()> {
  Ok(())
}
