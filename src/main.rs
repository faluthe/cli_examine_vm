use std::{env, io::{self, Write}, process};

use process_info::ProcessInfo;

mod process_info;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <PID>", args[0]);
        process::exit(1);
    }

    let proc_info = process_info::get_process_info(&args[1])
        .expect("Failed to get process info");

    loop {
        println!("Examining {}({})...", proc_info.comm, proc_info.pid);
        print!("Address: ");
        io::stdout().flush().unwrap();

        let mut address = String::new();
        let address = match io::stdin().read_line(&mut address) {
            Ok(_) => parse_address(&address.trim(), &proc_info),
            Err(error) => {
                eprintln!("Failed to read address: {error}");
                continue;
            }
        };
    }
}

fn parse_address(address: &str, proc: &ProcessInfo) -> usize {
    if address.starts_with("0x") {
        usize::from_str_radix(&address[2..], 16)
            .expect("Please provide a valid address")
    } else {
        address.parse()
            .expect("Please provide a valid address")
    }
}