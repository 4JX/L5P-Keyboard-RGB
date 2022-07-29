use std::{convert::TryInto, env, path::PathBuf, process, str::FromStr};

use clap::{crate_authors, crate_name, crate_version, Arg, Command};
use color_eyre::{eyre::eyre, Help, Report};
use single_instance::SingleInstance;

use crate::{
	custom_effect::CustomEffect,
	enums::{Direction, Effects},
	keyboard_manager::KeyboardManager,
	profile::Profile,
};

pub fn try_cli() -> Result<(), Report> {
	let colors_arg = Arg::new("colors")
		.help("List of 4 RGB triplets. Example: 255,0,0,255,255,0,0,0,255,255,128,0")
		.index(1)
		.required(true)
		.clone();
	let path_arg = Arg::new("path").help("A path to the file").index(1).required(true);

	let matches = Command::new("Legion Keyboard Control")
		.version(crate_version!())
		.author(crate_authors!())
		.arg(
			Arg::new("brightness")
				.help("The brightness of the effect")
				.takes_value(true)
				.short('b')
				.possible_values(&["1", "2"])
				.default_value("1"),
		)
		.arg(
			Arg::new("speed")
				.help("The speed of the effect")
				.takes_value(true)
				.short('s')
				.possible_values(&["1", "2", "3", "4"])
				.default_value("1"),
		)
		.arg(
			Arg::new("direction")
				.help("The direction of the effect (If applicable)")
				.takes_value(true)
				.short('d')
				.possible_values(&["Left", "Right"]),
		)
		.arg(Arg::new("save").help("Saves the typed profile").short('p').takes_value(true))
		.subcommand(Command::new("LoadProfile").about("Load a profile from a file").arg(path_arg.clone()))
		.subcommand(Command::new("LoadEffect").about("Load an effect from a file").arg(path_arg))
		.subcommand(Command::new("Static").about("Static effect").arg(colors_arg.clone()))
		.subcommand(Command::new("Breath").about("Breath effect").arg(colors_arg.clone()))
		.subcommand(Command::new("Smooth").about("Smooth effect"))
		.subcommand(Command::new("Wave").about("Wave effect"))
		.subcommand(Command::new("Lightning").about("Lightning effect"))
		.subcommand(Command::new("AmbientLight").about("AmbientLight effect"))
		.subcommand(Command::new("SmoothWave").about("SmoothWave effect"))
		.subcommand(Command::new("Swipe").about("Swipe effect").arg(colors_arg.clone()))
		.subcommand(Command::new("Disco").about("Disco effect"))
		.subcommand(Command::new("Christmas").about("Christmas effect"))
		.subcommand(Command::new("Fade").about("Fade effect").arg(colors_arg))
		.subcommand(Command::new("Temperature").about("Temperature effect"))
		.subcommand(Command::new("HiddenWindow").about("Loads the GUI but keeps the window hidden"))
		.get_matches();

	if let Some(input) = matches.subcommand_name() {
		if input == "HiddenWindow" {
			// Run the GUI, but don't show the window
			crate::gui::app::App::start_ui(false);
		} else {
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
						let path = PathBuf::from_str(path_string)?;

						match Profile::load_profile(path) {
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
						let profile = Profile {
							rgb_array,
							effect,
							direction,
							speed,
							brightness,
							ui_toggle_button_state: [false; 5],
						};

						profile.save_profile(filename).expect("Failed to save.");
					}

					manager.set_effect(effect, direction, &rgb_array, speed, brightness);
				}
			}
		}

		Ok(())
	} else {
		let exec_name = env::current_exe().unwrap().file_name().unwrap().to_string_lossy().into_owned();
		println!("No subcommands found, starting in GUI mode. To view the possible subcommands type \"{} --help\".", exec_name);
		crate::gui::app::App::start_ui(true);
		Ok(())
	}
}
