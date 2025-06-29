mod collect;
mod displayer;
mod record_collection;
mod spawner;
mod util;

use std::fs;
use std::sync::OnceLock;

use spawner::spawn_executable;
use termion::color;

use crate::{
    collect::{CollectErr, FileCollection},
    configs::{Config, TargetConfig},
};

const DEFAULT_SOURCE: &str = "bin_test";

static CONFIG_VARS: OnceLock<Config> = OnceLock::new();

fn config_init() {
    const SOURCE_CONFIG: &str = "config.toml";

    // Read any argument eing passed
    // let args: Vec<String> = std::env::args().collect();
    // println!("Passed Arguments: {:?}", args);

    CONFIG_VARS
        .set({
            let setting_toml_str =
                fs::read_to_string(SOURCE_CONFIG).expect("Config file not found!");

            let setting =
                toml::from_str::<TargetConfig>(&setting_toml_str).expect("Unable to parse config!");

            println!("{:?}", setting);

            let target_toml_str =
                fs::read_to_string(setting.setting.config_path.to_owned() + "/config.toml")
                    .expect("Unable to locate config file!");
            let mut config =
                toml::from_str::<Config>(&target_toml_str).expect("Unale to parse target config!");

            config.target_config = setting.setting.clone();
            config
        })
        .expect("Global Config Variable is already initialized!");
}

#[inline(always)]
fn get_global_config_ref() -> &'static Config {
    CONFIG_VARS.get().expect("Uninitialized Global Config")
}

fn locate_bin_files() -> Result<FileCollection, CollectErr> {
    // Do some matching for configurations
    let target = get_global_config_ref().target_config.bin_target.as_str();

    println!(
        "{}[ Collecting Compiled Test in {}... ]{}",
        color::Fg(color::Yellow),
        target,
        color::Fg(color::Reset)
    );

    collect::collect_test_files(target)
}

fn main() {
    // Initialize static variables
    config_init();

    let file_collection = locate_bin_files().unwrap_or_else(|e| {
        eprintln!("There was a problem collecting files: {:?}", e);
        std::process::exit(1);
    });

    // println!("{:?}\n", file_collection);
    println!(
        "{}[ Setting up executables... ]{}",
        color::Fg(color::Yellow),
        color::Fg(color::Reset)
    );
    
    
    let collection = spawn_executable(file_collection);
    match collection {
        Some(c) => println!("{:#?}", c),
        None => println!("--- Collection is Empty! ---"),
    } 

    println!(
        "{}[ Finished Executing ]{}",
        color::Fg(color::Green),
        color::Fg(color::Reset)
    );
}

mod configs {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct TargetConfig {
        pub setting: TargetPath,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct TargetPath {
        pub config_path: String,
        pub bin_target: String,
    }

    impl Default for TargetPath {
        fn default() -> Self {
            Self {
                config_path: Default::default(),
                bin_target: Default::default(),
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct Config {
        #[serde(skip_deserializing)]
        pub target_config: TargetPath,
        pub path: ConfigPath,
        pub process: ConfigWorker,
    }

    // report out used for outputting test reports
    // error out are for internal error within the TestRunner
    #[derive(Debug, Deserialize)]
    pub struct ConfigPath {
        pub report_out: String,
        pub error_out: String,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct ConfigWorker {
        pub max_child_spawn: usize,
        pub worker_count: usize,
    }
}
