use std::fs::{File, remove_file};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{thread, time};
use rfd::FileDialog;

const EXE_BYTES: &[u8] = include_bytes!("../assets/RePKG.exe");

pub fn process_repkg(path: &str) -> Result<(), String> {
    // 将内嵌的 EXE 文件写入临时文件
    let temp_exe_path = Path::new("RePKG_temp.exe");
    let mut temp_exe = File::create(temp_exe_path)
        .map_err(|e| format!("Failed to create temporary EXE file: {}", e))?;
    temp_exe
        .write_all(EXE_BYTES)
        .map_err(|e| format!("Failed to write EXE content: {}", e))?;

    // 构建 cmd 命令
    let exe_path = temp_exe_path;

    // let current_dir = env::current_dir().expect("Failed to get current directory");
    // let exe_path = current_dir.join("assets").join("RePKG.exe");

    if !exe_path.exists() {
        eprintln!("Executable not found at {:?}", exe_path);
        return Err("Executable file not found".to_string());
    } else {
        println!("tmp exe succeed: {}", exe_path.to_str().unwrap());
    }
    
    let mut comm = String::from(r".\").replace("\\", r"\");
    comm.push_str(&exe_path.to_str().unwrap().replace("\\", "/"));
    // let mut com = exe_path.to_str().unwrap().replace("\\", "/");
    let path = path.replace("\\", "/");
    comm.push(' ');
    comm.push_str(&path);
    println!("com: {:?}", comm);
    thread::sleep(time::Duration::from_secs(1));
    // 手动将文件句柄释放，确保文件没有被占用
    drop(temp_exe); // 显式释放文件句柄
    let status = Command::new("cmd").args(&["/C", &comm]).status();

    match status {
        Ok(exit_status) if exit_status.success() => {
            println!("Successfully opened Explorer for: {}", path);
        }
        Ok(exit_status) => {
            eprintln!("Explorer exited with error: {:?}", exit_status.code());
        }
        Err(e) => {
            eprintln!("Failed to execute Explorer command: {}", e);
        }
    }

    // 检查程序是否已完全退出，删除临时文件
    // wait_for_process_and_cleanup(temp_exe_path)?;

    Ok(())
}

pub fn pick_folder() -> Result<String, String>{
    if let Some(path) = FileDialog::new().pick_folder() {
        return Ok(path.to_string_lossy().into_owned())
    } else {
        return Err(String::from(""));
    }
}
