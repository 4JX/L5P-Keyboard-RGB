use std::{convert::TryInto, process, str::FromStr};

use clap::{crate_authors, crate_version, App, Arg, SubCommand};

use crate::{enums::Effects, keyboard_manager::KeyboardManager};

pub fn try_cli(manager: &mut KeyboardManager) {
	let matches = App::new("Legion Keyboard Control")
		.version(crate_version!())
		.author(crate_authors!())
		.arg(
			Arg::with_name("brightness")
				.help("The brightness of the effect")
				.takes_value(true)
				.short("b")
				.possible_values(&["1", "2"])
				.default_value("1"),
		)
		.arg(
			Arg::with_name("speed")
				.help("The speed of the effect")
				.takes_value(true)
				.short("s")
				.possible_values(&["1", "2", "3", "4"])
				.default_value("1"),
		)
		.subcommand(
			SubCommand::with_name("Static").about("Static effect").arg(
				Arg::with_name("colors")
					.help("List of 4 RGB triplets. Example: 255,0,0,255,255,0,0,0,255,255,128,0")
					.index(1)
					.required(true),
			),
		)
		.subcommand(
			SubCommand::with_name("Breath").about("Breath effect").arg(
				Arg::with_name("colors")
					.help("List of 4 RGB triplets. Example: 255,0,0,255,255,0,0,0,255,255,128,0")
					.index(1)
					.required(true),
			),
		)
		.subcommand(SubCommand::with_name("Smooth").about("Smooth effect"))
		.subcommand(SubCommand::with_name("LeftWave").about("Left Wave effect"))
		.subcommand(SubCommand::with_name("RightWave").about("Right Wave effect"))
		.subcommand(SubCommand::with_name("Lightning").about("Lightning effect"))
		.subcommand(SubCommand::with_name("AmbientLight").about("AmbientLight effect"))
		.subcommand(SubCommand::with_name("SmoothLeftWave").about("SmoothLeftWave effect"))
		.subcommand(SubCommand::with_name("SmoothRightWave").about("SmoothRightWave effect"))
		.subcommand(
			SubCommand::with_name("LeftSwipe").about("Swipe effect").arg(
				Arg::with_name("colors")
					.help("List of 4 RGB triplets. Example: 255,0,0,255,255,0,0,0,255,255,128,0")
					.index(1)
					.required(true),
			),
		)
		.subcommand(
			SubCommand::with_name("RightSwipe").about("Swipe effect").arg(
				Arg::with_name("colors")
					.help("List of 4 RGB triplets. Example: 255,0,0,255,255,0,0,0,255,255,128,0")
					.index(1)
					.required(true),
			),
		)
		.subcommand(SubCommand::with_name("Disco").about("Disco effect"))
		.subcommand(SubCommand::with_name("Christmas").about("Christmas effect"))
		.subcommand(
			SubCommand::with_name("Fade").about("Fade effect").arg(
				Arg::with_name("colors")
					.help("List of 4 RGB triplets. Example: 255,0,0,255,255,0,0,0,255,255,128,0")
					.index(1)
					.required(true),
			),
		)
		.subcommand(SubCommand::with_name("Temperature").about("Temperature effect"))
		.get_matches();

	if let Some(input) = matches.subcommand_name() {
		fn parse_bytes_arg(arg: &str) -> Result<Vec<u8>, <u8 as FromStr>::Err> {
			arg.split(',').map(str::parse::<u8>).collect()
		}

		let effect: Effects = Effects::from_str(input).unwrap();
		let speed = matches.value_of("speed").unwrap_or_default().parse::<u8>().unwrap_or(1);
		let brightness = matches.value_of("brightness").unwrap_or_default().parse::<u8>().unwrap_or(1);

		let matches = matches.subcommand_matches(input).unwrap();

		let color_array: [u8; 12] = match effect {
			Effects::Static | Effects::Breath | Effects::LeftSwipe | Effects::RightSwipe | Effects::Fade => {
				let color_array = if let Some(value) = matches.value_of("colors") {
					parse_bytes_arg(value)
						.expect("Invalid input, please check you used the correct format for the colors")
						.try_into()
						.expect("Invalid input, please check you used the correct format for the colors")
				} else {
					println!("This effect requires specifying the colors to use.");
					process::exit(0);
				};
				color_array
			}
			_ => [0; 12],
		};

		manager.set_effect(effect, &color_array, speed, brightness);
	}
}
