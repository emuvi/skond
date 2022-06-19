use clap::{Arg, ArgMatches, Command};

pub fn parse() -> ArgMatches {
  Command::new("Skond")
		.version(clap::crate_version!())
		.about("SkMotion (Screen Motion) is a command program for desktop that records the frames of a screen only when there is motion on it.")
		.author("Éverton M. Vieira <emuvi@outlook.com.br>")
		.arg(
			Arg::new("displays")
				.short('d')
				.long("displays")
				.takes_value(false)
				.required(false)
				.help("Prints a list of all connected displays."),
		)
		.arg(
			Arg::new("screen")
				.short('s')
				.long("screen")
				.value_name("INDEX")
				.takes_value(true)
				.required(false)
				.help("The index of the display to be recorded."),
		)
		.arg(
			Arg::new("extent")
				.short('e')
				.long("extent")
				.value_name("SECONDS")
				.takes_value(true)
				.required(false)
				.help("For how many seconds it will be recorded."),
		)
		.arg(
			Arg::new("frames_ps")
				.short('f')
				.long("frames_ps")
				.value_name("FRAMES_PER_SECOND")
				.takes_value(true)
				.required(false)
				.help("How many frames per second to record."),
		)
		.arg(
			Arg::new("record")
				.short('r')
				.long("record")
				.value_name("PATH")
				.takes_value(true)
				.required(false)
				.help("Records the frames of a display on the PATH."),
		)
		.get_matches()
}
