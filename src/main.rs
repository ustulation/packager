//! #Linux package generator

// -----------------------------------------------------------------------------------------------
// #![forbid(bad_style, exceeding_bitshifts, mutable_transmutes, no_mangle_const_items,
//           unknown_crate_types, warnings)]
// #![deny(deprecated, drop_with_repr_extern, improper_ctypes, missing_docs,
//         non_shorthand_field_patterns, overflowing_literals, plugin_as_library,
//         private_no_mangle_fns, private_no_mangle_statics, stable_features, unconditional_recursion,
//         unknown_lints, unsafe_code, unused, unused_allocation, unused_attributes,
//         unused_comparisons, unused_features, unused_parens, while_true)]
// #![warn(trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
//         unused_qualifications, unused_results)]
// #![allow(box_pointers, fat_ptr_transmutes, missing_copy_implementations,
//          missing_debug_implementations, variant_size_differences)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", deny(clippy, unicode_not_nfc, wrong_pub_self_convention,
                                   option_unwrap_used))]#[macro_use]
// -----------------------------------------------------------------------------------------------

extern crate config_file_handler;
extern crate rustc_serialize;
#[macro_use]
extern crate unwrap;

use config_file_handler::FileHandler;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{self, Command};

const POST_INSTALL_DEMO_APP_SCRIPT_NAME: &'static str = "post-install-demo-app.sh";
const POST_REMOVE_DEMO_APP_SCRIPT_NAME: &'static str = "post-remove-demo-app.sh";

macro_rules! x {
    ($result:expr, $msg:expr) => {
        match $result {
            Ok(v)  => v,
            Err(e) => {
                let decorator = ::std::iter::repeat('-').take(50).collect::<String>();
                println!("\n {}\n| {}: {:?}\n {}", decorator, $msg, e, decorator);
                println!("Aborting...\n\n");
                process::exit(-1);
            },
        }
    }
}

pub fn abort_if(cond: bool, msg: &str) {
    if cond {
        println!("ERROR: {}\nAborting...", msg);
        process::exit(-1);
    }
}

pub fn run(cmd: &mut Command, msg: &str) {
    let status = x!(cmd.status(), msg);
    abort_if(!status.success(),
             &format!("{} - Status: {:?}", msg, status));
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone)]
struct Config {
    output_dir: String,
    demo_app: Option<DemoApp>,
    safe_launcher: Option<SafeLauncher>,
}

#[derive(Debug)]
struct ConfigImpl {
    pub output_dir: PathBuf,
    pub demo_app: Option<DemoAppImpl>,
    pub safe_launcher: Option<SafeLauncherImpl>,
}

impl Into<ConfigImpl> for Config {
    fn into(self) -> ConfigImpl {
        ConfigImpl {
            output_dir: PathBuf::from(self.output_dir),
            demo_app: self.demo_app.map(|v| v.into()),
            safe_launcher: self.safe_launcher.map(|v| v.into()),
        }
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone)]
struct DemoApp {
    demo_app_project_dir: String,
    build: bool,
}

#[derive(Debug)]
struct DemoAppImpl {
    demo_app_project_dir: PathBuf,
    build: bool,
}

impl Into<DemoAppImpl> for DemoApp {
    fn into(self) -> DemoAppImpl {
        DemoAppImpl {
            demo_app_project_dir: PathBuf::from(self.demo_app_project_dir),
            build: self.build,
        }
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone)]
struct SafeLauncher {
    launcher_project_dir: String,
    log_toml_path: String,
    crust_config_path: String,
    safe_core: SafeCore,
}

#[derive(Debug)]
struct SafeLauncherImpl {
    pub launcher_project_dir: PathBuf,
    pub log_toml_path: PathBuf,
    pub crust_config_path: PathBuf,
    pub safe_core: SafeCoreImpl,
}

impl Into<SafeLauncherImpl> for SafeLauncher {
    fn into(self) -> SafeLauncherImpl {
        SafeLauncherImpl {
            launcher_project_dir: PathBuf::from(self.launcher_project_dir),
            log_toml_path: PathBuf::from(self.log_toml_path),
            crust_config_path: PathBuf::from(self.crust_config_path),
            safe_core: self.safe_core.into(),
        }
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone)]
struct SafeCore {
    project_dir: String,
    relative_target_dir: Option<String>,
    use_nightly: bool,
    clean: bool,
    release_build: bool,
    run_tests: bool,
}

#[derive(Debug)]
struct SafeCoreImpl {
    pub project_dir: PathBuf,
    pub relative_target_dir: Option<PathBuf>,
    pub use_nightly: bool,
    pub clean: bool,
    pub release_build: bool,
    pub run_tests: bool,
}

impl Into<SafeCoreImpl> for SafeCore {
    fn into(self) -> SafeCoreImpl {
        SafeCoreImpl {
            project_dir: PathBuf::from(self.project_dir),
            relative_target_dir: self.relative_target_dir.map(PathBuf::from),
            use_nightly: self.use_nightly,
            clean: self.clean,
            release_build: self.release_build,
            run_tests: self.run_tests,
        }
    }
}

fn get_input() -> String {
    let mut input = String::new();
    let _ = unwrap!(io::stdin().read_line(&mut input));

    input.trim().to_string()
}

fn main() {
    println!("\n\t================= Linux Package Creator =================\n");

    let config: ConfigImpl = x!(x!(FileHandler::<Config>::open("linux-packager.config", false),
                                   "Could not find config file")
                                    .read_file(),
                                "Could not read config file")
        .into();

    abort_if(config.demo_app.is_none() && config.safe_launcher.is_none(),
             "Atleast one should be non-null. Nothing to be done as all options are null.");

    if config.output_dir.is_dir() {
        if x!(config.output_dir.read_dir(),
              "Could not read output directory.")
            .count() != 0 {
            println!("Output directory {:?} is not empty. Contents will be overwritten - continue ? [y/n]",
                     config.output_dir.to_str().expect("Could not convert pathbuf to string."));
            let choice = get_input().to_lowercase();
            if choice != "yes" && choice != "y" {
                abort_if(true, "Output directory not empty.");
            }

            x!(fs::remove_dir_all(&config.output_dir),
               "Could not clear output directory.");
            x!(fs::create_dir(&config.output_dir),
               "Could not create output directory.");
        }
    } else {
        x!(fs::create_dir(&config.output_dir),
           "Could not create output directory.");
    }

    if let Some(demo_app) = config.demo_app {
        println!("\n############ SAFE Demo App: Start ############");

        let mut demo_app_project_dir = demo_app.demo_app_project_dir;

        abort_if(!demo_app_project_dir.is_dir(),
                 "Invalid Demo App Project directory.");
        demo_app_project_dir.push("demo_app");
        abort_if(!demo_app_project_dir.is_dir(),
                 "Invalid Demo App Project directory.");

        print!("###################### Creating working directory for Demo App...");
        x!(io::stdout().flush(), "Could not flush Stdout");
        let demo_app_package_dir = config.output_dir.join("demo-app-packages");
        x!(fs::create_dir(&demo_app_package_dir),
           "Could not create output directory for Demo App.");
        println!(" Ok");

        print!("###################### Creating post-install script...");
        x!(io::stdout().flush(), "Could not flush Stdout");
        let post_install_script = demo_app_package_dir.join(POST_INSTALL_DEMO_APP_SCRIPT_NAME);
        let mut fh = x!(File::create(post_install_script),
                        "Could not create post install script for Demo App.");
        x!(fh.write_all(b"#!/bin/sh\nln -fs /opt/maidsafe/safe_demo_app/safe_demo_app /usr/bin/safe_demo_app\n"),
           "Could not write post install script for Demo App.");
        x!(fh.sync_all(),
           "Could not sync file for post install script for Demo App.");
        println!(" Ok");

        print!("###################### Creating post-remove script...");
        x!(io::stdout().flush(), "Could not flush Stdout");
        let post_remove_script = demo_app_package_dir.join(POST_REMOVE_DEMO_APP_SCRIPT_NAME);
        let mut fh = x!(File::create(post_remove_script),
                        "Could not create post remove script for Demo App.");
        x!(fh.write_all(b"#!/bin/sh\nrm /usr/bin/safe_demo_app\n"),
           "Could not write post remove script for Demo App.");
        x!(fh.sync_all(),
           "Could not sync file for post remove script for Demo App.");
        println!(" Ok");

        if demo_app.build {
            println!("###################### Building Demo App...\n");
            run(&mut Command::new("npm").current_dir(&demo_app_project_dir).arg("prune"),
                "\"npm prune\" failed.");
            run(&mut Command::new("npm").current_dir(&demo_app_project_dir).arg("install"),
                "\"npm install\" failed.");
            run(&mut Command::new("npm").current_dir(&demo_app_project_dir).args(&["run", "package"]),
                "\"npm run package\" failed.");
        }

        let dest_maidsafe_dir = demo_app_package_dir.join("maidsafe");
        x!(fs::create_dir(&dest_maidsafe_dir),
           "Could not create destination maidsafe directory for Demo App.");

        let safe_demo_app_in_maidsafe_dir = dest_maidsafe_dir.join("safe_demo_app");

        run(&mut Command::new("sh")
                .current_dir(&demo_app_project_dir)
                .arg("-c")
                .arg(format!("cp -r app_dist/safe_demo* {}",
                             safe_demo_app_in_maidsafe_dir.to_str().expect("Could not convert path to string"))),
            "Could not copy files to destination package directory.");

        print!("\n###################### Establising Demo App version...");
        x!(io::stdout().flush(), "Could not flush Stdout");
        let version_file = safe_demo_app_in_maidsafe_dir.join("version");
        let mut version = String::new();
        let _ = x!(x!(File::open(&version_file), "\"version\" file not found.").read_to_string(&mut version),
                   "Could not read the version file.");
        version = version.trim().to_string();
        println!(" Ok -> {}", version);

        let file_type = String::from_utf8_lossy(&x!(Command::new("file")
                                                        .current_dir(&safe_demo_app_in_maidsafe_dir)
                                                        .arg("safe_demo_app")
                                                        .output(),
                                                    "Could not run \"file\" over executable")
                .stdout)
            .into_owned();

        print!("###################### Establising Demo App architecture...");
        x!(io::stdout().flush(), "Could not flush Stdout");
        let (fpm_arch, tar_arch) = if file_type.contains("ELF 64") {
            ("x86_64".to_string(), "x64".to_string())
        } else if file_type.contains("ELF 32") {
            ("i386".to_string(), "x86".to_string())
        } else {
            println!("Could not guage architecture of build for Demo App");
            process::exit(-1);
        };
        println!(" Ok -> {}\n", fpm_arch);

        let demo_app_tar_base_name = format!("safe_demo_app-v{}-linux-{}", version, tar_arch);

        run(&mut Command::new("sh")
                .current_dir(&demo_app_project_dir)
                .arg("-c")
                .arg(format!("cp -r app_dist/safe_demo* {}",
                             demo_app_package_dir.join(&demo_app_tar_base_name)
                                 .to_str()
                                 .expect("Could not convert path to string"))),
            "Could not copy files to destination package directory.");

        print!("###################### Creating tar...");
        x!(io::stdout().flush(), "Could not flush Stdout");
        run(&mut Command::new("tar")
                .current_dir(&demo_app_package_dir)
                .arg("zcf")
                .arg(&format!("{}.tar.gz", demo_app_tar_base_name))
                .arg(&demo_app_tar_base_name),
            "Could not run \"tar\".");
        println!(" Ok");

        let mut fpm = format!("fpm --prefix /opt --after-install {} --after-remove {} -s dir -t deb -n safe_demo_app \
                               --version {} --architecture {} --license GPLv3 --vendor MaidSafe --maintainer \
                               \"MaidSafe Dev <dev@maidsafe.net>\" --description \"SAFE Demo App Installer\" --url \
                               \"http://maidsafe.net\" maidsafe",
                              POST_INSTALL_DEMO_APP_SCRIPT_NAME,
                              POST_REMOVE_DEMO_APP_SCRIPT_NAME,
                              version,
                              fpm_arch);

        println!("###################### Creating deb package...\n");
        run(&mut Command::new("sh")
                .current_dir(&demo_app_package_dir)
                .arg("-c")
                .arg(&fpm),
            "fpm failed for \".deb\".");

        fpm = fpm.replace("-t deb", "-t rpm");

        println!("\n###################### Creating rpm package...\n");
        run(&mut Command::new("sh")
                .current_dir(&demo_app_package_dir)
                .arg("-c")
                .arg(&fpm),
            "fpm failed for \".rpm\".");

        println!("\n###################### Finalising Demo App Packages...\n");
        run(&mut Command::new("sh")
                .current_dir(&demo_app_package_dir)
                .arg("-c")
                .arg("mv *.rpm *.deb *.tar.gz ../"),
            "Could not copy files to destination package directory.");

        println!("############ SAFE Demo App: Finish ############");
    }

    println!("\n\t=========================================================\n");
}
