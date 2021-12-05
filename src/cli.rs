use std::{convert::TryInto, process, str::FromStr};

use clap::{crate_authors, crate_version, App, Arg, SubCommand};
use color_eyre::{eyre::eyre, Report};

use crate::{enums::Effects, keyboard_manager::KeyboardManager, profile::Profile};

pub fn try_cli(manager: &mut KeyboardManager) -> Result<bool, Report> {
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
		.arg(Arg::with_name("save").help("Saves the typed profile").short("p").takes_value(true))
		.subcommand(
			SubCommand::with_name("Load")
				.about("Load an effect from a file")
				.arg(Arg::with_name("path").help("A path to the file").index(1).required(true)),
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

		let input_matches = matches.subcommand_matches(input).unwrap();
		if input != "Load" {
			let effect: Effects = Effects::from_str(input).unwrap();
			let speed = matches.value_of("speed").unwrap_or_default().parse::<u8>().unwrap_or(1);
			let brightness = matches.value_of("brightness").unwrap_or_default().parse::<u8>().unwrap_or(1);

			let rgb_array: [u8; 12] = match effect {
				Effects::Static | Effects::Breath | Effects::LeftSwipe | Effects::RightSwipe | Effects::Fade => {
					let color_array = if let Some(value) = input_matches.value_of("colors") {
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

			if let Some(filename) = matches.value_of("save") {
				let profile = Profile::new(rgb_array, effect, speed, brightness, [false; 5]);
				profile.save(filename).expect("Failed to save.");
			}

			manager.set_effect(effect, &rgb_array, speed, brightness);
		} else if let Some(path_string) = input_matches.value_of("path") {
			match Profile::from_file(path_string.to_string()) {
				Ok(profile) => {
					manager.set_effect(profile.effect, &profile.rgb_array, profile.speed, profile.brightness);
				}
				Err(err) => {
					return Err(eyre!("{} ", err.to_string()));
				}
			}
		}
		Ok(true)
	} else {
		Ok(false)
	}
}
