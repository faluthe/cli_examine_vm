use std::{env, io::{self, Write}, process};

mod memory;
mod process_info;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <PID>", args[0]);
        process::exit(1);
    }

    let proc_info = process_info::get_process_info(&args[1])
        .expect("Failed to get process info");

    let mut examine = memory::Examine::new(1000);

    loop {
        println!("Enter command to examine {}({})...", proc_info.comm, proc_info.pid);
        println!("Commands: examine, maps, searchval, searchpat, settimeout, clear, quit");
        
        print!("> ");
        io::stdout().flush().unwrap();

        let mut command_str = String::new();
        let command_args = match io::stdin().read_line(&mut command_str) {
            Ok(_) => command_str.trim()
                .split_ascii_whitespace()
                .collect::<Vec<&str>>(),
            Err(error) => {
                eprintln!("Failed to read command: {error}");
                continue;
            }
        };

        if command_args.is_empty() {
            continue;
        }

        match command_args[0] {
            "examine" => {
                if let Some(&address) = command_args.get(1) {
                    let address = match memory::parse_address(address, &proc_info) {
                        Ok(address) => address,
                        Err(error) => {
                            eprintln!("{error}");
                            continue;
                        }
                    };
                    examine.examine(&proc_info, Some(address));
                } else {
                    examine.examine(&proc_info, None);
                }
            },
            "settimeout" => {
                if let Some(&timeout) = command_args.get(1) {
                    let timeout = match timeout.parse::<u16>() {
                        Ok(timeout) => timeout,
                        Err(error) => {
                            eprintln!("Failed to parse timeout: {error}");
                            continue;
                        }
                    };
                    examine.set_timeout(timeout);
                    println!("Timeout set to {} ms", timeout);
                } else {
                    eprintln!("Usage: settimeout <timeout>");
                }
            },
            "clear" => {
                print!("\x1b[H\x1b[J");
                io::stdout().flush().unwrap();
            },
            "maps" => process_info::print_maps(&proc_info),
            "quit" => break,
            cmd => eprintln!("Unknown command: {}", cmd),
        }
    }
}