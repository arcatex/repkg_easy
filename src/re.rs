use crate::os;
use std::ffi::OsStr;
use std::fs::{self};
use std::io;
use std::path::{Path, PathBuf};

const DEFAULT_SUFFIX: [&str; 3] = ["jpg", "png", "jpeg"];
#[derive(Default)]
pub struct Param {
    pub target: String,               // 指定目录
    pub saved: String,                // 保存目录
    pub as_title: bool,               // 以名称创建文件夹
    pub all_combine: bool,            // 所有文件合并到一个文件夹
    pub cobo_status: usize,           // 0 "以文件夹分类"; 1 "合并到文件夹"; 2 "分类和合并"
    pub addition_suffix: Vec<String>, // 需要添加保存的后缀名称
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

pub fn extract(param: Param) -> Result<usize, String> {
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

    // 拼接命令
    let mut condition = "".to_string();
    if param.as_title {
        condition.push_str("-n ");
    }
    let result = format!(
        "extract {}-o {} {}",
        condition,
        folder_path.to_str().unwrap(),
        param.target
    );

    println!("{}", result);

    os::process_repkg(&result);
    let mut extensions = DEFAULT_SUFFIX.map(|s| s.to_string()).to_vec();
    // 添加指定后缀
    for ele in param.addition_suffix.into_iter() {
        if !ele.is_empty() {
            extensions.push(ele);
        }
    }
    let files = search_files_with_extension(&folder_path.to_string_lossy(), &extensions);

    let target_directory = &param.saved;
    let file_len = files.len();
    if files.is_empty() {
        println!(
            "No files found with the specified extensions in the directory {}",
            path
        );
    } else {
        println!("Found {} files. Moving them...", file_len);
        println!("tmp save: {}", folder_path.to_str().unwrap());
        // 将文件移动到目标目录
        if let Err(e) = move_files_to_directory(
            files,
            folder_path.to_str().unwrap(),
            target_directory,
            param.cobo_status,
        ) {
            eprintln!("Error moving files: {}", e);
        }
    }

    // 结束前，删除临时目录
    fs::remove_dir_all(folder_path)
        .map_err(|e| format!("Failed to delete temporary convert file: {}", e))?;
    let temp_exe_path = Path::new("RePKG_temp.exe");
    fs::remove_file(temp_exe_path)
        .map_err(|e| format!("Failed to delete temporary EXE file: {}", e))?;
    return Ok(file_len);
}

fn search_files_with_extension(directory: &str, extensions: &[String]) -> Vec<PathBuf> {
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
                            if extensions.iter().any(|e| ext == e.as_str()) {
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

fn move_files_to_directory(
    files: Vec<PathBuf>,
    source_directory: &str,
    target_root_directory: &str,
    combo: usize,
) -> io::Result<()> {
    match combo {
        0 => move_files_for_directory(files, source_directory, target_root_directory),
        1 => move_files_com_directory(files, source_directory, target_root_directory),
        2 => move_files_all_directory(files, source_directory, target_root_directory),
        _ => move_files_for_directory(files, source_directory, target_root_directory),
    }
}

fn move_files_for_directory(
    files: Vec<PathBuf>,
    source_directory: &str,
    target_root_directory: &str,
) -> io::Result<()> {
    // 遍历所有找到的文件
    for file in files {
        // 获取文件的父目录路径，去除源目录的前缀，以便保留目录结构
        let relative_path = match file.strip_prefix(source_directory) {
            Ok(rp) => rp,
            Err(e) => {
                eprintln!("Failed to strip prefix from {:?}: {}", file, e);
                continue; // 跳过当前文件，继续处理下一个文件
            }
        };
        let first_folder = relative_path
            .iter()
            .next()
            .unwrap_or_else(|| OsStr::new(""));

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

fn move_files_com_directory(
    files: Vec<PathBuf>,
    source_directory: &str,
    target_root_directory: &str,
) -> io::Result<()> {
    // 遍历所有找到的文件
    for file in files {
        // 获取文件的父目录路径，去除源目录的前缀，以便保留目录结构
        let relative_path = match file.strip_prefix(source_directory) {
            Ok(rp) => rp,
            Err(e) => {
                eprintln!("Failed to strip prefix from {:?}: {}", file, e);
                continue; // 跳过当前文件，继续处理下一个文件
            }
        };

        let first_folder = relative_path
            .iter()
            .next()
            .unwrap_or_else(|| OsStr::new(""));

        println!("parent dir: {}", first_folder.to_string_lossy());
        // 拼接目标目录结构
        let target_directory = Path::new(target_root_directory).join("pics");
        println!("target dir: {}", target_directory.to_string_lossy());
        // 确保目标目录存在
        fs::create_dir_all(&target_directory)?;

        // 获取文件名，并将文件名前加上 parent_folder
        let new_file_name = format!(
            "{}-{}",
            first_folder.to_string_lossy(),
            file.file_name().unwrap().to_string_lossy()
        );

        // 目标文件的完整路径
        let target_path = target_directory.join(new_file_name);

        println!("new file path: {}", target_path.to_string_lossy());
        // 执行文件移动
        match fs::rename(&file, &target_path) {
            Ok(_) => println!("Successfully moved: {:?} to {:?}", file, target_path),
            Err(e) => eprintln!("Failed to move {:?} to {:?}: {}", file, target_path, e),
        }
    }

    Ok(())
}

fn move_files_all_directory(
    files: Vec<PathBuf>,
    source_directory: &str,
    target_root_directory: &str,
) -> io::Result<()> {
    // 遍历所有找到的文件
    for file in files {
        // 获取文件的父目录路径，去除源目录的前缀，以便保留目录结构
        let relative_path = match file.strip_prefix(source_directory) {
            Ok(rp) => rp,
            Err(e) => {
                eprintln!("Failed to strip prefix from {:?}: {}", file, e);
                continue; // 跳过当前文件，继续处理下一个文件
            }
        };
        let first_folder = relative_path
            .iter()
            .next()
            .unwrap_or_else(|| OsStr::new(""));
        // 拼接目标目录结构
        let target_directory = Path::new(target_root_directory).join(first_folder);
        // 确保目标目录存在
        fs::create_dir_all(&target_directory)?;
        // 目标文件的完整路径
        let target_path = target_directory.join(file.file_name().unwrap());
        match fs::copy(&file, &target_path) {
            Ok(_) => println!(
                "Successfully copied to first folder: {:?} -> {:?}",
                file, target_path
            ),
            Err(e) => eprintln!(
                "Failed to copy to first folder: {:?} -> {:?}: {}",
                file, target_path, e
            ),
        }
        println!("new file path: {}", target_path.to_string_lossy());

        // 拼接目标目录结构
        let target_directory = Path::new(target_root_directory).join("AAA-pics");
        println!("target dir: {}", target_directory.to_string_lossy());
        // 确保目标目录存在
        fs::create_dir_all(&target_directory)?;

        // 获取文件名，并将文件名前加上 parent_folder
        let new_file_name = format!(
            "{}-{}",
            first_folder.to_string_lossy(),
            file.file_name().unwrap().to_string_lossy()
        );

        // 目标文件的完整路径
        let target_path = target_directory.join(new_file_name);

        println!("new file path: {}", target_path.to_string_lossy());
        // 执行文件移动
        match fs::rename(&file, &target_path) {
            Ok(_) => println!("Successfully moved: {:?} to {:?}", file, target_path),
            Err(e) => eprintln!("Failed to move {:?} to {:?}: {}", file, target_path, e),
        }
    }

    Ok(())
}
