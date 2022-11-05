use std::{convert::TryInto, path::PathBuf, process, str::FromStr};

use clap::{arg, command, crate_name, Parser, Subcommand};
use color_eyre::{eyre::eyre, Help, Report};
use single_instance::SingleInstance;
use strum::IntoEnumIterator;

use crate::{
	custom_effect::CustomEffect,
	effects::EffectManager,
	enums::{Direction, Effects},
	profile::Profile,
};

#[macro_export]
macro_rules! clap_value_parser {
	($v: expr, $e: ty) => {{
		use clap::builder::TypedValueParser;
		clap::builder::PossibleValuesParser::new($v).map(|s| s.parse::<$e>().unwrap())
	}};
}

#[derive(Parser)]
#[command(
    author,
    version,
    long_about = None,
    name = "Legion Keyboard Control",
    // arg_required_else_help(true),
    rename_all = "PascalCase",
)]
struct Cli {
	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand)]
#[command(

    // rename_all = "PascalCase",
)]
enum Commands {
	/// Use an effect from the built-in set
	Set {
		/// The effect to be set
		#[arg(short, long, value_enum, value_parser, rename_all = "PascalCase")]
		effect: Effects,

		/// List of 4 RGB triplets. Example: 255,0,0,255,255,0,0,0,255,255,128,0
		#[arg(short, long, default_value = "0,0,0,0,0,0,0,0,0,0,0,0", value_parser = parse_colors)]
		colors: Option<[u8; 12]>,

		/// The brightness of the effect
		#[arg(short, long, default_value_t = 1, value_parser = clap_value_parser!(["1","2"], u8))]
		brightness: u8,

		/// The brightness of the effect
		#[arg(short, long, default_value_t = 1, value_parser = clap_value_parser!(["1","2","3","4"], u8))]
		speed: u8,

		/// The direction of the effect (If applicable)
		#[arg(short, long, value_enum)]
		direction: Option<Direction>,

		/// A filename to save the effect at
		#[arg(long, value_enum)]
		save: Option<String>,
	},

	/// List all the available effects
	List,

	/// Load a profile from a file
	LoadProfile {
		#[arg(short, long)]
		path: PathBuf,
	},

	/// Load an effect from a file
	LoadEffect {
		#[arg(short, long)]
		path: PathBuf,
	},

	/// Start the GUI
	Gui {
		/// Keep the window hidden on startup
		#[arg(short, long, default_value_t = false)]
		hidden: bool,
	},
}

fn parse_colors(arg: &str) -> Result<[u8; 12], String> {
	fn input_err<E>(_e: E) -> String {
		"Invalid input, please check you used the correct format for the colors".to_string()
	}

	let vec: Result<Vec<u8>, <u8 as FromStr>::Err> = arg.split(',').map(str::parse::<u8>).collect();
	let vec = vec.map_err(input_err);

	match vec {
		Ok(vec) => {
			let vec: Result<[u8; 12], Vec<u8>> = vec.try_into();

			vec.map_err(input_err)
		}
		Err(err) => Err(err),
	}
}

pub fn try_cli() -> Result<(), Report> {
	let cli = Cli::parse();

	let manager_result = EffectManager::new();

	match cli.command {
		Some(subcommand) => {
			// Early logic for specific subcommands
			match subcommand {
				Commands::Set { .. } | Commands::LoadEffect { .. } | Commands::LoadProfile { .. } => {
					let instance = SingleInstance::new(crate_name!()).unwrap();
					assert!(instance.is_single(), "Another instance of the program is already running, please close it before starting a new one.");
				}
				_ => {}
			}

			match subcommand {
				Commands::Set {
					effect,
					colors,
					brightness,
					speed,
					direction,
					save,
				} => {
					let mut manager = manager_result.unwrap();

					let direction = direction.unwrap_or_default();

					let rgb_array: [u8; 12] = if effect.takes_color_array() {
						colors.unwrap_or_else(|| {
							println!("This effect requires specifying the colors to use.");
							process::exit(0);
						})
					} else {
						[0; 12]
					};

					let profile = Profile {
						rgb_array,
						effect,
						direction,
						speed,
						brightness,
						ui_toggle_button_state: [false; 5],
					};

					if let Some(filename) = save {
						profile.save_profile(&filename).expect("Failed to save.");
					}

					manager.set_effect(profile);
				}
				Commands::List => {
					println!("List of available effects:");
					for (i, effect) in Effects::iter().enumerate() {
						println!("{i}. {}", effect);
					}
				}
				Commands::LoadProfile { path } => {
					let mut manager = manager_result.unwrap();

					match Profile::load_profile(path) {
						Ok(profile) => {
							manager.set_effect(profile);
						}
						Err(err) => {
							return Err(eyre!("{} ", err.to_string()).suggestion("Make sure you are using a valid profile."));
						}
					}
				}
				Commands::LoadEffect { path } => {
					let mut manager = manager_result.unwrap();

					match CustomEffect::from_file(path.to_string_lossy().to_string()) {
						Ok(effect) => {
							effect.play(&mut manager);
						}
						Err(err) => {
							return Err(eyre!("{} ", err.to_string()).suggestion("Make sure you are using a valid effect"));
						}
					}
				}

				Commands::Gui { hidden } => {
					crate::gui::app::App::start_ui(manager_result, hidden);
				}
			}
		}
		None => {
			let exec_name = std::env::current_exe().unwrap().file_name().unwrap().to_string_lossy().into_owned();
			println!("No subcommands found, starting in GUI mode. To view the possible subcommands type \"{} --help\".", exec_name);
			crate::gui::app::App::start_ui(manager_result, false);
		}
	}

	Ok(())
}
