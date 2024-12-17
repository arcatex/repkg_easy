use crate::os;
use std::fs::{self};
use std::io;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

#[derive(Default)]
pub struct Param {
    pub target: String, // 指定目录
    pub saved: String,  // 保存目录
    
}

// 用于处理路径是否存在，并且是文件或目录
fn check_path(path: &str) -> Result<(), String> {
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_file() {
                println!("{} 是一个文件", path);
                Ok(())
            } else if metadata.is_dir() {
                println!("{} 是一个目录", path);
                Ok(())
            } else {
                Err(format!("路径 {} 既不是文件也不是目录", path))
            }
        }
        Err(e) => Err(format!("无法获取元数据: {}", e)),
    }
}

pub fn extract(param: Param) -> Result<(), String> {
    let path = &param.target;

    // 检查 target 和 saved 路径
    if let Err(e) = check_path(path) {
        println!("{}", e);
        return Err(format!("源文件路径错误: {}", e));
    }

    let save = &param.saved;
    if let Err(e) = check_path(save) {
        println!("{}", e);
        return Err(format!("保存文件路径错误: {}", e));
    }

    // 在saved下创建一个临时文件夹
    // 拼接文件夹路径
    let folder_path = Path::new(&param.saved).join("tmp");

    // 使用 create_dir 创建文件夹
    fs::create_dir(&folder_path).map_err(|e| format!("Error creating directory: {}", e))?;

    // let tmpd = format!("{}/tmp")
    // 拼接命令
    let result = format!("extract -o {} {}", folder_path.to_str().unwrap(), param.target);

    println!("{}", result);

    os::process_repkg(&result);
    let extensions = vec!["jpg", "png"];
    let files = search_files_with_extension(&save, &extensions);

    let target_directory = &param.saved;
    if files.is_empty() {
        println!(
            "No files found with the specified extensions in the directory {}",
            path
        );
    } else {
        println!("Found {} files. Moving them...", files.len());
        println!("tmp save: {}", folder_path.to_str().unwrap());
        // 将文件移动到目标目录
        if let Err(e) = move_files_to_directory(files, folder_path.to_str().unwrap(), target_directory) {
            eprintln!("Error moving files: {}", e);
        }
    }

    // 结束前，删除临时目录
    fs::remove_dir_all(folder_path).map_err(|e| format!("Failed to delete temporary convert file: {}", e))?;
    let temp_exe_path = Path::new("RePKG_temp.exe");
    fs::remove_file(temp_exe_path).map_err(|e| format!("Failed to delete temporary EXE file: {}", e))?;
    return Ok(());
}

fn search_files_with_extension(directory: &str, extensions: &[&str]) -> Vec<PathBuf> {
    let mut result = Vec::new();

    // 递归读取目录内容
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let path = entry.path();

                    // 如果是文件并且具有指定的扩展名
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if extensions.iter().any(|&e| ext == e) {
                                result.push(path);
                            }
                        }
                    }
                    // 如果是目录，则递归搜索
                    else if path.is_dir() {
                        result.extend(search_files_with_extension(
                            path.to_str().unwrap(),
                            &extensions,
                        ));
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read entry: {}", e);
                }
            }
        }
    } else {
        eprintln!("Failed to read directory: {}", directory);
    }

    result
}

fn move_files_to_directory(files: Vec<PathBuf>, source_directory: &str, target_root_directory: &str) -> io::Result<()> {
    // 遍历所有找到的文件
    for file in files {
        // 获取文件的父目录路径，去除源目录的前缀，以便保留目录结构
        let relative_path = file.strip_prefix(source_directory)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;  // 使用 map_err 转换错误
        // 获取文件的直接父目录 (即一级子文件夹)
        let first_folder = relative_path.iter().next().unwrap_or_else(|| OsStr::new(""));

        println!("parent dir: {}", first_folder.to_string_lossy());
        // 拼接目标目录结构
        let target_directory = Path::new(target_root_directory).join(first_folder);
        println!("target dir: {}", target_directory.to_string_lossy());
        // 确保目标目录存在
        fs::create_dir_all(&target_directory)?;

        // 目标文件的完整路径
        let target_path = target_directory.join(file.file_name().unwrap());

        println!("new file path: {}", target_path.to_string_lossy());
        // 执行文件移动
        match fs::rename(&file, &target_path) {
            Ok(_) => println!("Successfully moved: {:?} to {:?}", file, target_path),
            Err(e) => eprintln!("Failed to move {:?} to {:?}: {}", file, target_path, e),
        }
    }

    Ok(())
}

