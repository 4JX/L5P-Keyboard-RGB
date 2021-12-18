use std::{convert::TryInto, env, process, str::FromStr};

use clap::{crate_authors, crate_name, crate_version, App, AppSettings, Arg, SubCommand};
use color_eyre::{eyre::eyre, Help, Report};
use single_instance::SingleInstance;

use crate::{
	custom_effect::CustomEffect,
	enums::{Direction, Effects},
	keyboard_manager::KeyboardManager,
	profile::Profile,
};

pub fn try_cli() -> Result<(), Report> {
	let matches = App::new("Legion Keyboard Control")
		.setting(AppSettings::ColoredHelp)
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
		.arg(
			Arg::with_name("direction")
				.help("The direction of the effect (If applicable)")
				.takes_value(true)
				.short("d")
				.possible_values(&["Left", "Right"]),
		)
		.arg(Arg::with_name("save").help("Saves the typed profile").short("p").takes_value(true))
		.subcommand(
			SubCommand::with_name("LoadProfile")
				.about("Load a profile from a file")
				.arg(Arg::with_name("path").help("A path to the file").index(1).required(true)),
		)
		.subcommand(
			SubCommand::with_name("LoadEffect")
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
		.subcommand(SubCommand::with_name("Wave").about("Wave effect"))
		.subcommand(SubCommand::with_name("Lightning").about("Lightning effect"))
		.subcommand(SubCommand::with_name("AmbientLight").about("AmbientLight effect"))
		.subcommand(SubCommand::with_name("SmoothWave").about("SmoothWave effect"))
		.subcommand(
			SubCommand::with_name("Swipe").about("Swipe effect").arg(
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
		let instance = SingleInstance::new(crate_name!()).unwrap();
		assert!(instance.is_single(), "Another instance of the program is already running, please close it before starting a new one.");

		let mut manager = KeyboardManager::new().unwrap();

		fn parse_bytes_arg(arg: &str) -> Result<Vec<u8>, <u8 as FromStr>::Err> {
			arg.split(',').map(str::parse::<u8>).collect()
		}

		let input_matches = matches.subcommand_matches(input).unwrap();

		match input {
			"LoadProfile" => {
				if let Some(path_string) = input_matches.value_of("path") {
					match Profile::from_file(path_string.to_string()) {
						Ok(profile) => {
							manager.set_effect(profile.effect, profile.direction, &profile.rgb_array, profile.speed, profile.brightness);
						}
						Err(err) => {
							return Err(eyre!("{} ", err.to_string()).suggestion("Make sure you are using a valid profile."));
						}
					}
				}
			}
			"LoadEffect" => {
				if let Some(path_string) = input_matches.value_of("path") {
					match CustomEffect::from_file(path_string.to_string()) {
						Ok(effect) => {
							effect.play(&mut manager);
						}
						Err(err) => {
							return Err(eyre!("{} ", err.to_string()).suggestion("Make sure you are using a valid effect"));
						}
					}
				}
			}
			_ => {
				let effect: Effects = Effects::from_str(input).unwrap();
				let speed = matches.value_of("speed").unwrap_or_default().parse::<u8>().unwrap_or(1);
				let brightness = matches.value_of("brightness").unwrap_or_default().parse::<u8>().unwrap_or(1);

				let rgb_array: [u8; 12] = match effect {
					Effects::Static | Effects::Breath | Effects::Swipe | Effects::Fade => {
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
					#[cfg(target_os = "windows")]
					Effects::Temperature => {
						panic!("This effect is not supported on Windows");
					}
					_ => [0; 12],
				};

				let direction: Direction = match effect {
					Effects::Wave | Effects::SmoothWave | Effects::Swipe => {
						let direction = if let Some(value) = matches.value_of("direction") {
							Direction::from_str(value).expect("Invalid direction")
						} else {
							println!("This effect requires a direction.");
							process::exit(0);
						};
						direction
					}
					_ => Direction::Right,
				};

				if let Some(filename) = matches.value_of("save") {
					let profile = Profile::new(rgb_array, effect, direction, speed, brightness, [false; 5]);
					profile.save(filename).expect("Failed to save.");
				}

				manager.set_effect(effect, direction, &rgb_array, speed, brightness);
			}
		}

		Ok(())
	} else {
		let exec_name = env::current_exe().unwrap().file_name().unwrap().to_string_lossy().into_owned();
		println!("No subcommands found, starting in GUI mode. To view the possible subcommands type \"{} --help\".", exec_name);
		crate::gui::app::App::start_ui();
		Ok(())
	}
}
