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
#[macro_use]
extern crate unwrap;

extern crate config_file_handler;
extern crate rustc_serialize;

#[macro_use]
mod utils;

mod config;
mod demo_app;

use config::{Config, ConfigImpl};
use config_file_handler::FileHandler;
use std::fs;
use utils::{abort_if, get_input};

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
                     config.output_dir.display());
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
        demo_app::package_demo_app(demo_app, &config.output_dir);
    }

    println!("\n\t=========================================================\n");
}
