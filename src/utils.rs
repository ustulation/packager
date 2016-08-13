use std::io;
use std::process::{self, Command};

macro_rules! x {
    ($result:expr, $msg:expr) => {
        match $result {
            Ok(v)  => v,
            Err(e) => {
                let decorator = ::std::iter::repeat('-').take(50).collect::<String>();
                println!("\n {}\n| {}: {:?}\n {}", decorator, $msg, e, decorator);
                println!("Aborting...\n\n");
                ::std::process::exit(-1);
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

pub fn get_input() -> String {
    let mut input = String::new();
    let _ = unwrap!(io::stdin().read_line(&mut input));

    input.trim().to_string()
}
