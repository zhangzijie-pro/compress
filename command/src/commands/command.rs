use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::process::Command;
use std::sync::{Mutex, RwLock};
use std::{env, fs};
use std::io::{self, BufRead, Error, ErrorKind, Read, Write};
use std::path::Path;

use lazy_static::lazy_static;


// whoami
pub fn whoami(session_context: &mut SessionContext) -> io::Result<(usize,String)>{
    let mut res = session_context.get_username();
    if session_context.user_state.root{
        res = "root".to_string()
    }

    Ok((STATUE_CODE,res))
}


// help 
pub fn help() -> String{
    let help = format!(
    "Usage: <command> [options] [arg]
\0\x1B[32mCommands:
    pwd     View current directory                         apt -i ..   Install package
    ls      View all files in the current directory        history     View past Commands
    cd      Change directory                               whoami  ||  apt -update version
    rm      Delete directory or file                       rn          Rename directory or file  
    touch   Create a new file                              mkdir       Create a new directory
    cat     View file only read                            mv          Move file's path
    python  Run code in python                             tar -zxvf:  Compression  
    html    Open html file                                 tar -xvf:   Decompression
    exit    Exit this process\0\x1B[0m\n"
    );

    help
}


// pwd
pub fn pwd() -> io::Result<(usize,String)>{
    let path = env::current_dir().unwrap().as_path().display().to_string();
    Ok((STATUE_CODE,path))
}


// ls
pub fn ls() -> io::Result<(usize,String)> { 
    let dir_path = Path::new("./");
    let mut result = String::new();

    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                result.push_str(&format!("{}    ", entry.file_name().into_string().unwrap()));
            } else {
                result.push_str(&format!("\x1B[32m{}    \x1B[0m", entry.path().display()));
            }
        }
        Ok((STATUE_CODE,result))
    } else {
        Err(Error::new(ErrorKind::NotFound, "Path is not a directory"))
    }
}


// ll
pub fn ll(context: &SessionContext) -> io::Result<(usize,String)>{
    let dir_path = Path::new("./");
    let mut result = String::new();
    let dirs = fs::read_dir(dir_path)?;
    for dir in dirs{
        let dir = dir?;
        let matadata = dir.metadata()?;
        let file_type = dir.file_type()?;

        // file type
        let file_type_str = if file_type.is_dir(){
            "d"
        }else if file_type.is_file(){
            "-"
        }else if file_type.is_symlink(){
            "l"
        }else{
            "?"
        };

        // file name
        let file_name = if file_type.is_dir(){
            format!("\x1B[32m{}    \x1B[0m", dir.path().display())
        }else if file_type.is_file(){
            dir.file_name().into_string().unwrap()
        }else{
            format!("\x1B[32m{}    \x1B[0m", dir.path().display())
        };


        // permission
        let uid = matadata.uid();
        let gid = matadata.gid();

        let output_o = match uid{
            1000=>context.get_username(),
            0=>"root".to_string(),
            _=>"-".to_string()
        };

        let output_p = match gid{
            1000=>context.get_username(),
            0=>"root".to_string(),
            _=>"-".to_string()
        };
        

        let size = matadata.len();
        

        // created time
        let path = dir.path();
        let s = path.as_os_str().to_str().unwrap();
        let time = file_create_time(s);

        // output
        result.push_str(&format!(
            "{} {}  {:>8}   {:>6} {}  {}\n",
            file_type_str,
            output_p,
            output_o,
            size,
            time,
            file_name
        ));
    }
    Ok((STATUE_CODE,result))
}


// history
lazy_static!{
    pub static ref HISTROY: Mutex<Vec<String>> = Mutex::new(Vec::new());
}


pub fn history_push(command: String){
    let mut history = HISTROY.lock().unwrap();
    history.push(command); 
}


pub fn history() -> Result<(usize,String),Error>{
    let s = HISTROY.lock().unwrap();
    for (i,c) in s.iter().enumerate(){
        println!("{}: {}",i,c);
    }

    let res = String::new().trim().to_owned();
    Ok((STATUE_CODE,res))
}


// cd
pub fn cd(path: &str) -> Result<(usize,String),Error>{
    let new_path = Path::new(path);
    env::set_current_dir(new_path)?;

    let res = format!("Successfully changed directory to {}.",path);
    Ok((STATUE_CODE,res))
}


// new dir
lazy_static!{
    pub static ref DIR: RwLock<&'static str> = RwLock::new("");
}


pub fn turn_dir(command: String, dir: String) -> Result<(usize,String),Error>{
    let mut dir_lock = DIR.write().unwrap();
    *dir_lock = Box::leak(dir.into_boxed_str());

    match command.as_str() {
        "mkdir" => {
            mkdir(&dir_lock)
        },
        "rm" => {
            rm(&dir_lock)
        },
        "cd" =>{
            cd(&dir_lock)
        }
        _ => {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported command",
            ))
        }
    }
}


// new file 
lazy_static! {
    pub static ref FILE: RwLock<&'static str> = RwLock::new("");
}


pub fn turn_file(command: String,file: String) -> Result<(usize,String), Error> {
    let mut file_lock = FILE.write().unwrap();
    *file_lock = Box::leak(file.into_boxed_str());

    match command.as_str() {
        "touch" => {
            touch(&file_lock)
        },
        "cat" => {
            cat(&file_lock)
        },
        "rm" => {
            rm(&file_lock)
        }
        _ =>{
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported command",
            ))
        }
    }
}


//touch
pub fn touch(file: &str) -> Result<(usize,String),std::io::Error>{
    if file.is_empty(){
        return Ok(empty_file());
    }
    let _ = fs::File::create_new(Path::new(file))?;

    let res = format!("Successfully created {}",file);
    Ok((STATUE_CODE,res))
}


// mkdir
pub fn mkdir(dir: &str) -> Result<(usize,String),std::io::Error>{
    if dir.is_empty(){
        return Ok(empty_dir());
    }
    let mut builder = fs::DirBuilder::new();
    let _ = builder.recursive(true).create(Path::new(dir));

    let res = format!("Successfully created {}",dir);
    Ok((STATUE_CODE,res))
}


// rm
pub fn rm(file: &str) -> Result<(usize,String),std::io::Error>{
    if file.is_empty(){
        return Ok(empty_file());
    }
    let filepath = Path::new(file);
    match filepath.is_dir() {
        true => {
            let _ = fs::remove_dir(filepath);
        },
        false => {
            let _ = fs::remove_file(filepath);
        }
    }

    let res = String::new().trim().to_owned();
    Ok((STATUE_CODE,res))
}


// rn mv
pub fn rename(source:&str,now:&str) -> std::io::Result<(usize,String)> {
    if source.is_empty(){
        return Ok(empty_file());
    }
    let _ = fs::rename(source, now);
    let res = String::new().trim().to_owned();
    Ok((STATUE_CODE,res))
}


// cat
pub fn cat(file: &str) -> Result<(usize,String),Error>{
    if file.is_empty(){
        return Ok(empty_file());
    }
    let f = fs::File::open(Path::new(file));
    let mut buffer = String::new();
    let _ = f.unwrap().read_to_string(&mut buffer);
    Ok((STATUE_CODE,buffer))
}


use crate::commands::download::{download_package, find_package};
use crate::get::get_hty::file_create_time;
use crate::run::run;
use crate::state_code::{empty_dir, empty_file, missing_pattern,  STATUE_CODE};
use super::download::update;
use crate::root::SessionContext;


// apt -install  xxx
pub fn apt(name: &str) -> io::Result<(usize,String)>{
    if name.is_empty(){
        return Ok(missing_pattern());
    }
    match find_package(name) {
        Some(package) => {
            if let Err(err) = download_package(&package) {
                eprintln!("Error: {}", err);
            }
        },
        None => {
            eprintln!("Package {} not found.", name);
        }
    }

    let res = format!("Successfully download Package {}",name);
    Ok((STATUE_CODE,res))
}


// apt -update xxx
pub fn update_new(version: &str) -> io::Result<(usize,String)>{
    if version.is_empty(){
        return Ok(missing_pattern());
    }
    match update(&version) {
        Ok(_) => {
            let script_path = dirs::home_dir().unwrap().join(".Tiks").join("update_script.sh");
            let output = Command::new("bash")
                .arg(script_path.clone())
                .output()
                .expect("Error: network error....");
            if output.status.success() {
                let _ = std::fs::remove_file(script_path);
            }
            let res = format!("Successfully Update version {}",version);
            Ok((STATUE_CODE,res))
        }
        Err(_) => {
            let err = format!("The current version is the latest one");
            return Ok((STATUE_CODE,err));
        },
    }

}


use tar::Archive;
use flate2::read::GzDecoder;
use flate2::Compression;
use flate2::write::GzEncoder;
use super::arg::{execute_command, execute_other_command, split, Commands};


pub fn zxvf(file: &str, to: &str) -> Result<(usize,String),std::io::Error>{
    if file.is_empty() || to.is_empty(){
        return Ok(missing_pattern());
    }
    let tar_gz = File::create(to)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_file(file, &mut File::open(file)?)?;
    Ok((STATUE_CODE,"Successfully Compression".to_string()))
}


pub fn xvf(to: &str) -> Result<(usize,String),std::io::Error>{
    if to.is_empty(){
        return Ok(missing_pattern());
    }
    let tar_gz = File::open(to)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(".")?;
    Ok((STATUE_CODE,"Successfully Decompression".to_string()))
}


// 重定向输出   > 
pub fn stdout_file(commands: Commands,session_context: &mut SessionContext) -> Result<(usize,String), std::io::Error>{
    let command = commands.command.clone();
    let arg = commands.arg.clone();
    let result = execute_command(&command, "", &arg, session_context)?.1;
    let mut file = File::create(arg[arg.len()-1].clone())?;
    file.write(result.as_bytes())?;
    Ok((STATUE_CODE,"write over!".to_string()))
}


// cp
#[allow(unused_assignments)]
pub fn cp(source:&str, to: &str) -> io::Result<(usize,String)>{
    if source.is_empty() || to.is_empty(){
        return Ok(missing_pattern());
    }

    let file = fs::read(source)?;

    let result = fs::write(to, file);
    let mut output = String::new();
    match result.is_ok(){
        true => {
            output = format!("Successfully to copy {}",to);
        },
        false =>{
            output = format!("Error: copy {} to {} failed",source,to);
        }
    }

    Ok((STATUE_CODE,output))
}


// sudo
#[allow(unused_assignments)]
pub fn sudo(session_context: &mut SessionContext)->io::Result<(usize,String)>{
    loop{
        let mut output = String::new();
        let user = session_context.get_username();
        println!("[sudo] password for {}:",user);
        let pd = rpassword::read_password().unwrap();
        let res = session_context.toggle_root(pd);
        if res.is_ok() {
            output = format!("Sucessfully to change root");
            return Ok((STATUE_CODE,output));
        } else {
            println!("Sorry, try again");
            continue;
        }
    }
}


// get time
use chrono;
pub fn get_time() -> io::Result<(usize,String)>{
    let now = chrono::Local::now();
    let time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    Ok((STATUE_CODE,time))
}

// grep
pub fn grep(pattern:&str,arg: &str) -> io::Result<(usize,String)>{
    if arg.is_empty(){
        return Ok(missing_pattern());
    }

    let mut output = String::new();

    if let Ok(file) = File::open(arg){
        let reader = io::BufReader::new(file);

        for line in reader.lines(){
            let line = line?;
            if line.contains(pattern){
                let replaced_string = line.replace(pattern, &&format!("\x1b[31m{}\x1b[0m", pattern));
                output.push_str(&replaced_string);
                output.push_str("\n");
            }
        }
    }else {
        let string_w = arg.split_whitespace();
        for i in string_w{
            if i.contains(pattern){
                let replaced_string = i.replace(pattern, &&format!("\x1b[31m{}\x1b[0m", pattern));
                output.push_str(&replaced_string);
                output.push_str("\n");
            }
        }
    }


    if output.is_empty(){
        let _ = output.trim();
    }

    Ok((STATUE_CODE,output))
}


// | pipe
#[allow(unused_assignments)]
pub fn pipe(command:Vec<String>) -> io::Result<(usize,String)>{
    let spilt_vec = command.split(|pipe| pipe.as_str()=="|");

    let mut output = String::new();
    let mut last_result = Vec::new();

    for i in spilt_vec{
        let i = i.to_vec();
        let commands = turn_command(i);
        let (command,option,mut arg) = split(commands);
        arg.append(&mut last_result);
        let mut result = (0,String::new());
        if arg.is_empty(){
            result = execute_other_command(&command, &option, &last_result).expect("Error: ...");
        }else{
            result = execute_other_command(&command, &option, &arg).expect("Error: ...");
        }
        last_result.push(result.1.clone());
        output = result.1;
    }

    Ok((STATUE_CODE,output))
}

// &
pub fn and(command:Vec<String>,session_context: &mut SessionContext){
    let commands = command.split(|x| x=="&");
    for c in commands{
        let v = c.to_vec();
        run(v, session_context)
    }
}

pub fn echo_print<T: std::fmt::Debug>(output: T) -> (usize,T){
    (STATUE_CODE,output)
}

// turn vec<_> to Commands
fn  turn_command(v: Vec<String>) -> Commands{
    let ouput = Commands::new(v);
    ouput
}