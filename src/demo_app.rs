use config::DemoAppImpl;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{self, Command};
use utils::{abort_if, run};

const POST_INSTALL_DEMO_APP_SCRIPT_NAME: &'static str = "post-install-demo-app.sh";
const POST_REMOVE_DEMO_APP_SCRIPT_NAME: &'static str = "post-remove-demo-app.sh";

pub fn package_demo_app(demo_app: DemoAppImpl, output_dir: &Path) {
    println!("\n################## SAFE Demo App: Start ##################\n");

    let mut demo_app_project_dir = demo_app.demo_app_project_dir;

    abort_if(!demo_app_project_dir.is_dir(),
             "Invalid Demo App Project directory.");
    demo_app_project_dir.push("demo_app");
    abort_if(!demo_app_project_dir.is_dir(),
             "Invalid Demo App Project directory.");

    print!("###################### Creating working directory for Demo App...");
    x!(io::stdout().flush(), "Could not flush Stdout");
    let demo_app_package_dir = output_dir.join("demo-app-packages");
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
                         safe_demo_app_in_maidsafe_dir.display())),
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
    println!(" Ok -> {}", fpm_arch);

    let demo_app_tar_base_name = format!("safe_demo_app-v{}-linux-{}", version, tar_arch);

    run(&mut Command::new("sh")
            .current_dir(&demo_app_project_dir)
            .arg("-c")
            .arg(format!("cp -r app_dist/safe_demo* {}",
                         demo_app_package_dir.join(&demo_app_tar_base_name)
                             .display())),
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
                           --version {} --architecture {} --license GPLv3 --vendor MaidSafe --maintainer \"MaidSafe \
                           Dev <dev@maidsafe.net>\" --description \"SAFE Demo App Installer\" --url \
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

    print!("\n###################### Finalising Demo App Packages...");
    run(&mut Command::new("sh")
            .current_dir(&demo_app_package_dir)
            .arg("-c")
            .arg("mv *.rpm *.deb *.tar.gz ../"),
        "Could not copy files to destination package directory.");
    println!(" Ok -> Packages at {:?}", output_dir.display());

    println!("################## SAFE Demo App: Finish ##################");
}
