use std::{fs::File, io::{self, Read}, path::Path};

pub struct ProcessInfo {
    pub pid: usize,
    pub comm: String,
    pub maps: Vec<MemoryMap>,
}

struct MemoryMap {
    start: usize,
    end: usize,
    perms: String,
    path: String,
}

pub fn get_process_info(pid_str: &String) -> io::Result<ProcessInfo> {
    let pid: usize = pid_str
        .parse()
        .expect("Please provide a valid PID");
    if !Path::new(&format!("/proc/{}", pid)).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("No process with PID {}", pid),
        ));
    }
    let comm = get_comm(pid)?;
    let maps = get_maps(pid)?;

    Ok(ProcessInfo { pid, comm, maps })
}

fn get_comm(pid: usize) -> io::Result<String> {
    let path = format!("/proc/{}/comm", pid);
    let mut file = File::open(path)?;
    let mut comm_str = String::new();
    
    file.read_to_string(&mut comm_str)?;

    Ok(comm_str.trim().to_string())
}

fn get_maps(pid: usize) -> io::Result<Vec<MemoryMap>> {
    let mut maps = Vec::new();
    
    let path = format!("/proc/{}/maps", pid);
    let mut file = File::open(path)?;
    let mut maps_str = String::new();

    file.read_to_string(&mut maps_str)?;

    for line in maps_str.lines() {
        let split: Vec<&str> = line.split_whitespace().collect();
        let start_end: Vec<&str> = split[0].split('-').collect();
        let perms = split[1];
        let path = if split.len() > 5 {
            split[5].to_string()
        } else {
            String::new()
        };

        let map = MemoryMap {
            start: usize::from_str_radix(start_end[0], 16).unwrap(),
            end: usize::from_str_radix(start_end[1], 16).unwrap(),
            perms: perms.to_string(),
            path,
        };

        maps.push(map);
    }

    Ok(maps)
}