use std::{io::{self, Write}, num::ParseIntError, os::fd::AsFd, fmt};

use nix::{errno, ioctl_read_bad, libc, poll, sys::uio, unistd};

use crate::process_info::ProcessInfo;

pub struct Examine {
    timeout: u16,
}

impl Examine {
    pub fn new(timeout: u16) -> Self {
        Examine {
            timeout,
        }
    }

    pub fn set_timeout(&mut self, timeout: u16) {
        self.timeout = timeout;
    }

    pub fn examine(&self, proc_info: &ProcessInfo, address: Option<usize>) {
        let address = if let Some(a) = address {
            a
        } else {
            match Self::get_address_stdin(proc_info) {
                Ok(address) => address,
                Err(error) => {
                    eprintln!("Failed to get address: {error}");
                    return;
                }
            }
        };
        
        loop {
            // Clear/reset terminal
            print!("\x1b[H\x1b[J");
            io::stdout().flush().unwrap();
            
            // Do work
            let num_lines = num_terminal_lines().expect("Failed to get terminal lines");
            match self.examine_bytes(proc_info, address, num_lines as usize) {
                Ok(_) => (),
                Err(error) => {
                    eprintln!("Failed to examine address: {error}");
                    break;
                }
            }
    
            let stdin = io::stdin();
            let poll_fd = poll::PollFd::new(stdin.as_fd(), poll::PollFlags::POLLIN);
    
            // Wait for Enter key
            match poll::poll(&mut [poll_fd], self.timeout) {
                // Timed out
                Ok(0) => (),
                // Event occurred
                Ok(_) => {
                    // Clear stdin
                    let mut buf = String::new();
                    stdin.read_line(&mut buf).unwrap();
                    break;
                }
                Err(error) => {
                    eprintln!("Failed to poll stdin: {error}");
                    break;
                }
            }   
        }
    }

    fn examine_bytes(&self, proc_info: &ProcessInfo, address: usize, num_lines: usize) -> Result<(), errno::Errno> {
        // Header
        print!("0x{:x} in {}({}), {}ms timeout. Press 'Enter' to quit... ", address, proc_info.comm, proc_info.pid, self.timeout);

        // -1 for header
        let num_bytes = (num_lines - 1) * 16;
        let mut data = vec![0u8; num_bytes];
        let local_iov = io::IoSliceMut::new(&mut data);
        let remote_iov = uio::RemoteIoVec {
            base: address,
            len: num_bytes,
        };
    
        let bytes_read = uio::process_vm_readv(unistd::Pid::from_raw(proc_info.pid), &mut [local_iov], &[remote_iov])?;
        
        println!("Read {} bytes.", bytes_read);
        
        for (i, byte) in data.iter().enumerate() {
            if i % 16 == 0 {
                if i != 0 {
                    println!();
                }
                print!("{:08x}:", address + i);
            }
            print!(" {:02x}", byte);
        }

        io::stdout().flush().unwrap();
    
        Ok(())  
    }

    fn get_address_stdin(proc_info: &ProcessInfo) -> Result<usize, ParseAddressError> {
        let mut address = String::new();
    
        print!("Address: ");
        io::stdout().flush().unwrap();
    
        match io::stdin().read_line(&mut address) {
            Ok(_) => parse_address(&address.trim(), proc_info),
            Err(error) => Err(ParseAddressError::IoError(error)),
        }
    }
}

pub enum ParseAddressError {
    ParseIntError(ParseIntError),
    AddressNotInMap,
    IoError(io::Error),
}

impl From<ParseIntError> for ParseAddressError {
    fn from(error: ParseIntError) -> Self {
        ParseAddressError::ParseIntError(error)
    }
}

impl fmt::Display for ParseAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseAddressError::ParseIntError(error) => write!(f, "Failed to parse address: {error}"),
            ParseAddressError::AddressNotInMap => write!(f, "Address not in process map"),
            ParseAddressError::IoError(error) => write!(f, "Failed to read address: {error}"),
        }
    }
}

pub fn parse_address(address: &str, proc: &ProcessInfo) -> Result<usize, ParseAddressError> {
    let parsed_address = usize::from_str_radix(if address.starts_with("0x") { &address[2..] } else { address }, 16)?;

    for map in &proc.maps {
        if parsed_address >= map.start && parsed_address < map.end {
            return Ok(parsed_address);
        }
    }

    Err(ParseAddressError::AddressNotInMap)
}

// Only used in this module for now
fn num_terminal_lines() -> Result<u16, errno::Errno> {
    ioctl_read_bad!(tiocgwinsz, libc::TIOCGWINSZ, libc::winsize);

    let mut winsize = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    unsafe {
        tiocgwinsz(libc::STDOUT_FILENO, &mut winsize)?;
    };

    Ok(winsize.ws_row)
}