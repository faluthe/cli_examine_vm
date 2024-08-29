use std::{fs::File, io::{self, Read}, path::Path};

pub struct ProcessInfo {
    pub pid: i32,
    pub comm: String,
    pub maps: Vec<MemoryMap>,
}

pub struct MemoryMap {
    pub start: usize,
    pub end: usize,
    pub perms: String,
    pub path: String,
}

pub fn get_process_info(pid_str: &String) -> io::Result<ProcessInfo> {
    let pid: i32 = pid_str.parse()
        .expect("Please provide a valid PID");
    
    if !Path::new(&format!("/proc/{}", pid_str)).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("No process with PID {}", pid),
        ));
    }
    
    let comm = get_comm(pid)?;
    let maps = get_maps(pid)?;

    Ok(ProcessInfo { pid, comm, maps })
}

fn get_comm(pid: i32) -> io::Result<String> {
    let path = format!("/proc/{}/comm", pid);
    let mut file = File::open(path)?;
    let mut comm_str = String::new();
    
    file.read_to_string(&mut comm_str)?;

    Ok(comm_str.trim().to_string())
}

fn get_maps(pid: i32) -> io::Result<Vec<MemoryMap>> {
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

impl std::fmt::Display for MemoryMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}-{:016x} {} {}", self.start, self.end, self.perms, self.path)
    }
}

pub fn print_maps(proc_info: &ProcessInfo) {
    for map in &proc_info.maps {
        println!("{map}");
    }
}