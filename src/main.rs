use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use nix::sys::ptrace;
use nix::unistd::Pid;
use std::io::{Seek, SeekFrom, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let pid_num: i32 = args[1].parse()?;
    let pid = Pid::from_raw(pid_num);
    let maps_path = format!("/proc/{}/maps", pid_num);
    
    let f = File::open(maps_path)?;
    let reader = BufReader::new(f);
   
    let mut result = None;
    for line in reader.lines(){
        let line = line.unwrap();
        if line.contains("[heap]"){
            let parts: Vec<&str> = line.split_whitespace().collect();
            let addr_range = parts[0];
            let mut addr_parts = addr_range.split("-");
            let start_str = addr_parts.next().unwrap();
            let end_str = addr_parts.next().unwrap();
            let start = u64::from_str_radix(start_str, 16)?;
            let end = u64::from_str_radix(end_str, 16)?;
            println!("[+] Heap Range: 0x{} - 0x{}", start, end);
            result = Some((start, end));
            break;
        }
    }
    
    let (start, end) = result.ok_or("Heap section not found")?;

    ptrace::attach(pid)?;
    nix::sys::wait::waitpid(pid,  None)?;
    
    let mem_path = format!("/proc/{}/mem", pid_num);
    let mut mem_file = File::open(&mem_path)?;

    mem_file.seek(SeekFrom::Start(start))?;
    let mut buffer = vec![0; (end - start) as usize];
    mem_file.read_exact(&mut buffer)?;

    for (i, byte) in buffer.iter().enumerate() {
        if i % 16 == 0{
            print!("\n0x{:08x}: ", start + i as u64);
        }
        print!("{:02x} ", byte);
    }
    println!();
    ptrace::detach(pid, None)?;

    Ok(())
}
