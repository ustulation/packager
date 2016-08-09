//! #Linux package generator

// -----------------------------------------------------------------------------------------------
#![forbid(bad_style, exceeding_bitshifts, mutable_transmutes, no_mangle_const_items,
          unknown_crate_types, warnings)]
#![deny(deprecated, drop_with_repr_extern, improper_ctypes, missing_docs,
        non_shorthand_field_patterns, overflowing_literals, plugin_as_library,
        private_no_mangle_fns, private_no_mangle_statics, stable_features, unconditional_recursion,
        unknown_lints, unsafe_code, unused, unused_allocation, unused_attributes,
        unused_comparisons, unused_features, unused_parens, while_true)]
#![warn(trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results)]
#![allow(box_pointers, fat_ptr_transmutes, missing_copy_implementations,
         missing_debug_implementations, variant_size_differences)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", deny(clippy, unicode_not_nfc, wrong_pub_self_convention,
                                   option_unwrap_used))]#[macro_use]
// -----------------------------------------------------------------------------------------------

extern crate unwrap;

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const PACKAGE_OUTER_DIR_NAME: &'static str = "maidsafe";
const POST_INSTALL_SCRIPT_NAME: &'static str = "post_install.sh";

fn get_input() -> String {
    let mut input = String::new();
    let _ = unwrap!(io::stdin().read_line(&mut input));

    input.trim().to_string()
}

fn source_dir_sanitiy_check<P: AsRef<Path>>(src: P) {
    if !src.as_ref().is_dir() {
        panic!("Source path is not a valid directory.");
    }

    let log_toml = src.as_ref().join("log.toml");
    let crust_config = src.as_ref().join("safe_launcher.crust.config");
    let _ = unwrap!(File::open(log_toml));
    let _ = unwrap!(File::open(crust_config));
}

fn main() {
    println!("\n\t================= Linux Package Creator =================\n");

    println!("Enter source path:");
    let src_path = PathBuf::from(get_input());
    source_dir_sanitiy_check(&src_path);

    println!("\nEnter destination path:");
    let dest_path = PathBuf::from(get_input());

    let maidsafe_path = dest_path.join(PACKAGE_OUTER_DIR_NAME);
    if maidsafe_path.is_dir() {
        println!("Dir called \"maidsafe\" already exists in destination - overwrite ? [y/n]:");
        let choice = get_input().to_lowercase();
        if choice != "y" && choice != "yes" {
            return println!("Aborting...");
        }

        unwrap!(fs::remove_dir_all(&maidsafe_path));
    }

    unwrap!(fs::create_dir(&maidsafe_path));

    let mut status = unwrap!(Command::new("cp")
        .arg("-r")
        .arg(src_path)
        .arg(maidsafe_path)
        .status());

    if !status.success() {
        panic!("Could not copy source. Process exited with error code: {:?}",
               status);
    }

    let post_install_script = dest_path.join(POST_INSTALL_SCRIPT_NAME);
    let mut fh = unwrap!(File::create(post_install_script));
    unwrap!(fh.write_all(b"#!/bin/sh\nln -s /opt/maidsafe/safe_launcher/safe_launcher /usr/bin/safe_launcher"));
    unwrap!(fh.sync_all());

    println!("\nEnter package name:");
    let package_name = get_input();

    // println!("\nEnter author name:");
    // let author = get_input();

    status = unwrap!(Command::new("fpm")
        .current_dir(&dest_path)
        .args(&["--prefix", "/opt"])
        .args(&["--after-install", POST_INSTALL_SCRIPT_NAME])
        .args(&["-s", "dir"])
        .args(&["-t", "deb"])
        .arg("-n")
        .arg(&package_name)
        .arg("maidsafe")
        .status());

    if !status.success() {
        panic!("Could not create \"deb\" package. Process exited with error code: {:?}",
               status);
    }

    status = unwrap!(Command::new("fpm")
        .current_dir(&dest_path)
        .args(&["--prefix", "/opt"])
        .args(&["--after-install", POST_INSTALL_SCRIPT_NAME])
        .args(&["-s", "dir"])
        .args(&["-t", "rpm"])
        .arg("-n")
        .arg(&package_name)
        .arg("maidsafe")
        .status());

    if !status.success() {
        panic!("Could not create \"rpm\" package. Process exited with error code: {:?}",
               status);
    }

    status = unwrap!(Command::new("fpm")
        .current_dir(dest_path)
        .args(&["-s", "dir"])
        .args(&["-t", "tar"])
        .arg("-n")
        .arg(package_name)
        .arg("maidsafe")
        .status());

    if !status.success() {
        panic!("Could not create \"tar\" package. Process exited with error code: {:?}",
               status);
    }

    println!("\n\t=========================================================\n");
}
