use std::path::{Path, PathBuf};

use clap::{crate_authors, crate_version, Arg, ArgAction, ArgMatches, Command};

use merge_tool::changelog::extract_version_from_changelog_file;
use merge_tool::config::Config;
use merge_tool::git_description::retrieve_description;
use merge_tool::process::{self, GenerateOptions};
use std::process::exit;
use std::str::FromStr;

fn main() {
    env_logger::init();

    let matches = Command::new("merge_tool")
        .author(crate_authors!())
        .version(crate_version!())
        .author("Raphael Bernhard <beraphae@gmail.com>")
        .about("Merge firmwares like in 1999")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("generate")
                .about("Create a bootload script, merge firmware files and write a info.json file")
            .arg(
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .value_name("FILE")
                    .help("Set a config file. Defaults to config.gctmrg."),
            )
            .arg(
                Arg::new("output-dir")
                    .short('o')
                    .long("output-dir")
                    .value_name("FILE")
                    .help("Output folder for generated files. Defaults to `<config-file-dir>/out`"),
            )
            .arg(
                Arg::new("use-backdoor")
                    .long("use-backdoor")
                    .action(ArgAction::SetTrue)
                    .help("Use the backdoor to validate the firmware image."),
            )
            .arg(
                Arg::new("repo-path")
                    .long("repo-path")
                    .value_name("FILE")
                    .help("Path to the git repository (or any file within the repository). Defaults to the config file path."),
            )
            .arg(
                Arg::new("timestamp")
                    .short('t')
                    .long("timestamp")
                    .value_name("TIMESTAMP")
                    .help("Timestamp to use for the generated files in RFC3339. Defaults to the current time.")
            )
        )
        .subcommand(
            Command::new("get-version")
                .about("Extract version information from changelog")
                .arg(
                    Arg::new("changelog")
                        .short('c')
                        .long("changelog")
                        .value_name("FILE")
                        .help("Set a changelog file. Defaults to CHANGELOG.md."),
                )
                .arg(
                    Arg::new("prerelease")
                        .short('p')
                        .long("prerelease")
                        .action(ArgAction::SetTrue)
                        .help("Include prerelease versions in the output."),
                )
                .arg(
                    Arg::new("timestamp")
                        .short('t')
                        .long("timestamp")
                        .value_name("TIMESTAMP")
                        .help("Timestamp to use for the generated files in RFC3339. Defaults to the current time.")
                )
            )
        .subcommand(
            Command::new("bundle")
                .about("Bundle the firmware files")
                .arg(
                    Arg::new("info")
                        .short('i')
                        .long("info")
                        .value_name("FILE")
                        .help("Info file to bundle. Defaults to info.json."),
                ).arg(
                    Arg::new("output-dir")
                        .short('o')
                        .long("output-dir")
                        .value_name("DIRECTORY")
                        .help("Output directory for the bundled firmware files."),
                )
                .arg(
                    Arg::new("versioned")
                        .long("versioned")
                        .action(ArgAction::SetTrue)
                        .help("Use versioned output directory."),
                )
        )
        .subcommand(
            Command::new("merge-packages")
            .about("Merge multiple firmware packages")
            .arg(
                Arg::new("output-file")
                    .short('o')
                    .long("output-file")
                    .value_name("FILE")
                    .help("Output file for the merged firmware package."),
            ).arg(
                Arg::new("packages")
                    .short('p')
                    .long("packages")
                    .value_name("FILES")
                    .action(clap::ArgAction::Append)
                    .help("Input files to merge."),
            )
    ).get_matches();

    if let Some(matches) = matches.subcommand_matches("generate") {
        let options = get_generation_options(&matches);

        if let Err(err) = process::generate(options) {
            println!("Error: Could not generate firmware: {}", err);
            exit(1);
        }
    }

    if let Some(matches) = matches.subcommand_matches("bundle") {
        let info = matches
            .get_one::<String>("info")
            .cloned()
            .unwrap_or("info.json".to_string());
        let info = Path::new(&info);
        let Some(output_dir) = matches.get_one::<String>("output-dir") else {
            println!("Error: No output directory specified.");
            exit(1);
        };
        let versioned = matches.contains_id("versioned");
        let output_dir = Path::new(output_dir);

        if let Err(err) = process::bundle(info, output_dir, versioned) {
            println!("Error: Could not bundle firmware: {}", err);
            exit(1);
        }
    }

    if let Some(matches) = matches.subcommand_matches("get-version") {
        let changelog = matches
            .get_one::<String>("changelog")
            .cloned()
            .unwrap_or("CHANGELOG.md".to_string());
        let mut version = match extract_version_from_changelog_file(changelog.as_ref()) {
            Ok(version) => version,
            Err(err) => {
                println!("Error: Could not extract version from changelog: {}", err);
                exit(1);
            }
        };
        let prerelease = matches.contains_id("prerelease");
        if prerelease {
            let repo = std::env::current_dir().unwrap();
            let git_description = match retrieve_description(&repo) {
                Ok(git_description) => git_description,
                Err(err) => {
                    println!("Error: Could not retrieve git description: {}", err);
                    exit(1);
                }
            };
            let date_time = parse_timestamp_arg(matches).unwrap_or_else(|| chrono::Utc::now());
            process::add_pre_release_info(&mut version, &date_time, &git_description);
        }
        println!("{}", version)
    }

    if let Some(matches) = matches.subcommand_matches("merge-packages") {
        let output_file = matches
            .get_one::<String>("output-file")
            .cloned()
            .unwrap_or("merged.fwpkg".to_string());
        let output_file = Path::new(&output_file);
        let packages: Vec<_> = matches
            .get_many::<String>("packages")
            .unwrap()
            .map(|x| Path::new(x))
            .collect();
        if let Err(err) = process::merge_app_packages(&packages, &output_file) {
            println!("Error: Could not merge packages: {}", err);
            exit(1);
        }
    }
}

fn get_generation_options(matches: &ArgMatches) -> GenerateOptions {
    let config = matches
        .get_one::<String>("config")
        .cloned()
        .unwrap_or("config.gctmrg".to_string());
    let config_path = Path::new(&config);
    if !config_path.exists() {
        println!("Config file does not exist.");
        exit(1);
    }
    log::debug!("Fetching Config from: {:?}", config_path);
    let config_dir = match Config::get_config_dir(config_path) {
        Ok(ret) => ret,
        Err(err) => {
            println!("Cannot retrieve parent directory of config file: {}", err);
            exit(1);
        }
    };

    let output_dir = matches
        .get_one::<String>("output-dir")
        .and_then(|x| PathBuf::from_str(x).ok())
        .unwrap_or(config_dir.join("out"));
    log::debug!("Output directory: {:?}", output_dir);

    let mut config = match Config::load_from_file(&config_path) {
        Ok(config) => config,
        Err(err) => {
            println!("Cannot load config: {}", err);
            exit(1);
        }
    };

    let use_backdoor = matches.get_one::<bool>("use-backdoor").unwrap();
    if *use_backdoor {
        config.use_backdoor = true;
    }

    let repo_dir = matches
        .get_one::<String>("repo-path")
        .and_then(|x| PathBuf::from_str(x).ok());

    if let Some(timestamp) = parse_timestamp_arg(matches) {
        config.build_time = timestamp;
    }

    GenerateOptions {
        config,
        output_dir,
        config_dir,
        repo_dir,
    }
}

fn parse_timestamp_arg(matches: &ArgMatches) -> Option<chrono::DateTime<chrono::Utc>> {
    let Some(timestamp) = matches.get_one::<String>("timestamp") else {
        return None;
    };
    match chrono::DateTime::parse_from_rfc3339(timestamp) {
        Ok(ret) => Some(ret.with_timezone(&chrono::Utc)),
        Err(err) => {
            println!("Invalid timestamp format: {}", err);
            exit(1);
        }
    }
}
