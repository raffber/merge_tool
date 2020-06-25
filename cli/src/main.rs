use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use clap::{App, Arg, SubCommand};

use merge_tool::config::Config;
use merge_tool::process;

fn main() {
    let matches = App::new("Merge Tool")
        .version("1.0")
        .author("Raphael Bernhard <beraphae@gmail.com>")
        .about("Merge firmwares like in 1999")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Set a config file. Defaults to config.gctmrg."),
        )
        .arg(
            Arg::with_name("output-dir")
                .short("o")
                .long("output-dir")
                .help("Output folder for generated files. Defaults to `<config-file-dir>/out`"),
        )
        .arg(
            Arg::with_name("use-backdoor")
                .long("use-backdoor")
                .help("Use the backdoor to validate the firmware image."),
        )
        .subcommand(SubCommand::with_name("script").about("Create a bootload script"))
        .subcommand(SubCommand::with_name("merge").about("Merge firmware files"))
        .subcommand(
            SubCommand::with_name("release")
                .about("Create a bootload script, merge files and release to git."),
        )
        .get_matches();
    let config = matches.value_of("config").unwrap_or("config.gctmrg");
    let config_path = Path::new(config);
    if !config_path.exists() {
        println!("Config file does not exist.");
        return;
    }
    let config_dir = match Config::get_config_dir(config_path) {
        Ok(ret) => ret,
        Err(err) => {
            println!("Cannot retrieve parent directory of config file: {}", err);
            return;
        }
    };
    let output_dir = matches
        .value_of("output-dir")
        .map(PathBuf::from)
        .unwrap_or(config_dir.join("out"));
    let mut config = match Config::load_from_file(&config_path) {
        Ok(config) => config,
        Err(err) => {
            println!("Cannot load config: {}", err);
            return;
        }
    };

    if let Some(_) = matches.subcommand_matches("script") {
        if let Err(err) = create_dir_all(&output_dir) {
            println!("Cannot create output directory: {}", err);
            return;
        }
        if let Err(err) = process::create_script(&mut config, &config_dir, &output_dir) {
            println!("Error: Could not create bootload script: {}", err);
        }
    }

    if let Some(_) = matches.subcommand_matches("merge") {
        if let Err(err) = create_dir_all(&output_dir) {
            println!("Cannot create output directory: {}", err);
            return;
        }
        match process::merge_all(&mut config, &config_dir) {
            Ok(fws) => {
                if let Err(err) = process::write_fws(&config, &fws, &output_dir) {
                    println!("Error: Couldn't write merged firmware to disk: {}", err);
                    return;
                }
            }
            Err(err) => {
                println!("Error: Couldn't merge firmware images: {}", err);
                return;
            }
        }
    }

    if let Some(_) = matches.subcommand_matches("release") {
        if let Err(err) = process::release(&mut config, &config_dir) {
            println!("Error: Could not release firmware: {}", err);
        }
    }
}
