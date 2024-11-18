//! The `explorer` module contains functions about system explorer.

#[allow(dead_code)]
/// Show item in explorer.
///
/// # Arguments
///
/// * `path` - A string that holds the item path
///
/// # Examples
///
/// ```
/// use explorer::show_in_explorer;
///
/// show_in_explorer("to\\display\\item").unwrap();
/// ```
#[cfg(not(target_os = "linux"))]
pub fn show_in_explorer(path: &str) -> Result<(), std::io::Error> {
    use std::io::{Error, ErrorKind};
    use std::path::Path;
    #[cfg(target_os = "macos")]
    use std::path::PathBuf;
    use std::process::Command;

    if !Path::new(path).exists() {
        return Err(Error::from(ErrorKind::NotFound));
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(["/select,", &path]) // The comma after select is not a typo
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        let path_buf = PathBuf::from(&path);
        if path_buf.is_dir() {
            Command::new("open").args([&path]).spawn()?;
        } else {
            Command::new("open").args(["-R", &path]).spawn()?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
/// Show item in explorer.
///
/// # Arguments
///
/// * `path` - A string that holds the item path
/// * `dbus` - Dbus connection to prevent error on Linux Mint
///
/// # Examples
///
/// ```
/// use explorer::show_in_explorer;
///
/// show_in_explorer("to\\display\\item").unwrap();
/// ```
#[cfg(target_os = "linux")]
pub fn show_in_explorer(
    path: &str,
    dbus: Option<&dbus::blocking::SyncConnection>,
) -> Result<(), std::io::Error> {
    // https://github.com/tauri-apps/plugins-workspace/issues/999
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::Duration;

    if dbus.is_none() || path.contains(",") {
        let mut path_buf = PathBuf::from(&path);
        let new_path = match path_buf.is_dir() {
            true => path,
            false => {
                path_buf.pop();
                path_buf.into_os_string().into_string().unwrap()
            }
        };
        Command::new("xdg-open").arg(&new_path).spawn()?;
    } else {
        // https://docs.rs/dbus/latest/dbus/
        let dbus = dbus_guard.as_ref().unwrap();
        let proxy = dbus.with_proxy(
            "org.freedesktop.FileManager1",
            "/org/freedesktop/FileManager1",
            Duration::from_secs(5),
        );
        let (_,): (bool,) = proxy.method_call(
            "org.freedesktop.FileManager1",
            "ShowItems",
            (vec![format!("file://{path}")], ""),
        )?;
    }

    Ok(())
}

#[allow(dead_code)]
/// Removes the whole file or directory.
///
/// # Arguments
///
/// * `path` - A string that holds the file or directory path
///
/// # Examples
///
/// ```
/// use meta::remove_all;
///
/// remove_all("to\\remove\\file").unwrap();
/// remove_all("to\\remove\\directory").unwrap();
/// ```
pub fn remove_all(path: &str) -> Result<(), std::io::Error> {
    use std::fs::{remove_dir_all, remove_file};
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    let target = Path::new(path);

    if !target.exists() {
        // return Ok(());
        return Err(Error::from(ErrorKind::NotFound));
    }

    if target.is_file() {
        remove_file(target)?;
    } else if target.is_dir() {
        remove_dir_all(target)?;
    }

    Ok(())
}

#[allow(dead_code)]
/// Create the file or directory.
///
/// # Arguments
///
/// * `path` - A string that holds the file or directory path
///
/// # Examples
///
/// ```
/// use meta::create_all;
///
/// create_all("to\\create\\file", "file").unwrap();
/// create_all("to\\create\\directory", "dir").unwrap();
/// ```
pub fn create_all(path: &str, meta_type: &str) -> Result<(), std::io::Error> {
    use std::fs::{create_dir_all, OpenOptions};
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    if Path::new(path).exists() {
        return Err(Error::from(ErrorKind::AlreadyExists));
    }

    match meta_type {
        "file" => {
            if let Some(prefix) = Path::new(path).parent() {
                if !prefix.exists() {
                    create_dir_all(prefix)?;
                }
                let _ = OpenOptions::new().write(true).create_new(true).open(path)?;
            }
        }
        "dir" => {
            create_dir_all(path)?;
        }
        _ => {
            return Err(Error::from(ErrorKind::Unsupported));
        }
    }

    Ok(())
}

#[allow(dead_code)]
/// Recursively copies the whole directory.
///
/// See [How to copy a folder recursively in Rust?](https://stackoverflow.com/questions/26958489/how-to-copy-a-folder-recursively-in-rust)
///
/// # Arguments
///
/// * `src` - Path of source directory
/// * `dst` - Path of save directory
///
/// # Examples
///
/// ```
/// use meta::copy_dir_all;
///
/// copy_dir_all("to\\copy\\directory", "to\\save\\directory").unwrap();
/// ```
fn copy_dir_all(
    from: impl AsRef<std::path::Path>,
    to: impl AsRef<std::path::Path>,
) -> Result<(), std::io::Error> {
    use std::fs;

    fs::create_dir_all(&to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), to.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), to.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
/// Copies the whole file or directory.
///
/// # Arguments
///
/// * `from` - A string that holds the source path of file or directory
/// * `to` - A string that holds the save path of file or directory
///
/// # Examples
///
/// ```
/// use meta::copy_all;
///
/// copy_all("to\\copy\\file", "to\\save\\file").unwrap();
/// copy_all("to\\copy\\directory", "to\\save\\directory").unwrap();
/// ```
pub fn copy_all(from: &str, to: &str) -> Result<(), std::io::Error> {
    use std::fs::{copy, create_dir_all, OpenOptions};
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    let src = Path::new(from);
    let save = Path::new(to);
    if !src.exists() {
        return Err(Error::from(ErrorKind::NotFound));
    }
    if save.exists() {
        return Err(Error::from(ErrorKind::AlreadyExists));
    }

    if src.is_file() {
        if !save.exists() {
            if let Some(prefix) = save.parent() {
                if !prefix.exists() {
                    create_dir_all(prefix)?;
                }
            }

            let _ = OpenOptions::new().write(true).create_new(true).open(to)?;
        }

        copy(from, to)?;
    } else if src.is_dir() {
        copy_dir_all(from, to)?;
    }

    Ok(())
}

#[allow(dead_code)]
/// Copies the whole directory with build in .gitignore file.
///
/// # Arguments
///
/// * `from` - A string that holds the source path of  directory
/// * `to` - A string that holds the save path of directory
///
/// # Examples
///
/// ```
/// use meta::copy_dir_with_build_in_ignore;
///
/// copy_dir_with_build_in_ignore("to\\copy\\directory", "to\\save\\directory").unwrap();
/// ```
pub fn copy_dir_with_build_in_ignore(from: &str, to: &str) -> Result<(), std::io::Error> {
    use ignore::Walk;
    use std::fs::{copy, create_dir_all};
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    let src = Path::new(from);
    let save = Path::new(to);
    if !src.exists() {
        return Err(Error::from(ErrorKind::NotFound));
    }
    if save.exists() {
        return Err(Error::from(ErrorKind::AlreadyExists));
    }

    for result in Walk::new(src) {
        match result {
            Ok(entry) => {
                let cur_entry_path = entry.path();
                let entry_save = save.join(cur_entry_path.strip_prefix(from).unwrap_or(src));
                let entry_save_path = entry_save.as_path();

                if cur_entry_path.is_file() {
                    let _ = copy(cur_entry_path, entry_save_path)?;
                } else if cur_entry_path.is_dir() {
                    let _ = create_dir_all(entry_save_path)?;

                    let ignore_file = cur_entry_path.join(".gitignore");
                    if ignore_file.exists() {
                        let _ = copy(ignore_file, entry_save_path.join(".gitignore"))?;
                    }
                }
            }
            Err(_error) => {
                // println!("Failed to parse ignore file in dir {}, errMsg: {:?}", from, error);
                return Err(Error::from(ErrorKind::InvalidData));
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
/// Copies the whole directory with custom ignore strings.
///
/// # Arguments
///
/// * `from` - A string that holds the source path of  directory
/// * `to` - A string that holds the save path of directory
/// * `ignores` - A vec that contains the custom ignore strings
///
/// # Examples
///
/// ```
/// use meta::copy_dir_with_custom_ignores;
///
/// let ignores = vec!["debug", "bin", "target"];
/// copy_dir_with_custom_ignores("to\\copy\\directory", "to\\save\\directory", ignores).unwrap();
/// ```
pub fn copy_dir_with_custom_ignores(
    from: &str,
    to: &str,
    ignores: &Vec<String>,
) -> Result<(), std::io::Error> {
    use std::fs::{copy, create_dir_all};
    use std::io::{Error, ErrorKind};
    use std::path::Path;
    use walkdir::WalkDir;

    let src = Path::new(from);
    let save = Path::new(to);
    if !src.exists() {
        return Err(Error::from(ErrorKind::NotFound));
    }
    if save.exists() {
        return Err(Error::from(ErrorKind::AlreadyExists));
    }

    for result in WalkDir::new(src) {
        match result {
            Ok(entry) => {
                let cur_entry_path = entry.path();
                let entry_save = save.join(cur_entry_path.strip_prefix(from).unwrap_or(src));
                let entry_save_path = entry_save.as_path();

                if let Some(path_str) = entry.path().to_str() {
                    if ignores.iter().any(|item| path_str.contains(item.as_str())) {
                        continue;
                    }
                }

                if cur_entry_path.is_file() {
                    let _ = copy(cur_entry_path, entry_save_path)?;
                } else if cur_entry_path.is_dir() {
                    let _ = create_dir_all(entry_save_path)?;
                }
            }
            Err(_error) => {
                // println!("Failed to parse ignore file in dir {}, errMsg: {:?}", from, error);
                return Err(Error::from(ErrorKind::InvalidData));
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
/// Get path size.
///
/// # Arguments
///
/// * `path` - A string that holds the target path
///
/// # Examples
///
/// ```
/// use meta::get_path_size;
///
/// let size = get_path_size("to\\check\\path").unwrap();
/// println!("target path size is: {}", size);
/// ```
pub fn get_path_size(path: &str) -> Result<u64, std::io::Error> {
    use fs_extra::dir::get_size;
    use std::fs;
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    let target = Path::new(path);
    if !target.exists() {
        return Err(Error::from(ErrorKind::NotFound));
    }

    if target.is_dir() {
        if let Ok(dir_size) = get_size(path) {
            return Ok(dir_size);
        } else {
            return Err(Error::from(ErrorKind::Other));
        }
    } else {
        if let Ok(file) = fs::metadata(path) {
            return Ok(file.len());
        } else {
            return Err(Error::from(ErrorKind::Other));
        }
    }
}

#[allow(dead_code)]
/// Restricts the directory to wanted items count.
///
/// Will delete the ealiest created items.
///
/// # Arguments
///
/// * `path` - A string that holds the source path of directory
/// * `count` - Wanted directory items count
///
/// # Examples
///
/// ```
/// use meta::restrict_dir_items_by_count;
///
/// restrict_dir_subitems_count("to\\restrict\\directory", 3).unwrap();
/// ```
pub fn restrict_dir_subitems_count(path: &str, count: usize) -> Result<(), std::io::Error> {
    println!("cur path: {}, target cnt: {}", path, count);

    use std::fs::{metadata, read_dir};
    use std::io::{Error, ErrorKind};
    use std::path::Path;
    use std::time::UNIX_EPOCH;

    let target = Path::new(path);
    if !target.exists() {
        return Err(Error::from(ErrorKind::NotFound));
    }

    let mut dir_items: Vec<String> = vec![];
    for result in read_dir(path)? {
        match result {
            Ok(entry) => {
                let cur_entry_path = entry.path().display().to_string();
                dir_items.push(cur_entry_path);

                // let cur_created_time = metadata(&cur_entry_path)?.created()?;
                // println!("insert path: {:?}, time: {:?}", entry.path().display(), cur_created_time);
            }
            Err(error) => {
                println!("Failed to read dir {}, errMsg: {:?}", path, error);
                return Err(Error::from(ErrorKind::Other));
            }
        }
    }

    if dir_items.len() > count {
        dir_items.sort_by(|a, b| {
            metadata(a)
                .unwrap()
                .created()
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .cmp(
                    &metadata(b)
                        .unwrap()
                        .created()
                        .unwrap()
                        .duration_since(UNIX_EPOCH)
                        .unwrap(),
                )
        });

        for path in dir_items[..(dir_items.len() - count)].iter() {
            let _ = remove_all(path)?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
/// Restricts the directory to wanted size.
///
/// This will not recursively walk the children directory!
///
/// # Arguments
///
/// * `path` - A string that holds the source path of directory
/// * `size` - Wanted directory size in bytes
///
/// # Examples
///
/// ```
/// use meta::restrict_dir_items_by_size;
///
/// restrict_dir_subitems_size("to\\restrict\\directory", 3 * 1024).unwrap();
/// ```
pub fn restrict_dir_subitems_size(path: &str, size: u64) -> Result<(), std::io::Error> {
    use fs_extra::dir::get_size;
    use std::fs::read_dir;
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    let target = Path::new(path);
    if !target.exists() {
        return Err(Error::from(ErrorKind::NotFound));
    }

    for result in read_dir(path)? {
        match result {
            Ok(entry) => {
                if let Ok(dir_size) = get_size(path) {
                    if dir_size > size {
                        let _ = remove_all(entry.path().display().to_string().as_str())?;
                    }
                }
            }
            Err(error) => {
                println!("Failed to read dir {}, errMsg: {:?}", path, error);
                return Err(Error::from(ErrorKind::Other));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct TestPack {
        pub file_from: String,
        pub file_to: String,
        pub dir_from: String,
        pub dir_to: String,
    }

    fn test_build_pack(suffix: &str) -> Result<TestPack, std::io::Error> {
        use std::env::current_dir;
        use std::fs::{copy, create_dir_all, OpenOptions};
        use std::io::Write;
        use std::io::{Error, ErrorKind};

        if let Ok(home_dir) = current_dir() {
            let test_compress_path = &home_dir.join(format!("test_explorer_{}", suffix));
            let _ = create_dir_all(test_compress_path)?;

            let file_path = test_compress_path.join("test_file.txt");
            let mut test_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(file_path.clone())
                .unwrap();
            test_file.write_all("Hello world!".as_bytes()).unwrap();

            let dir_path = &test_compress_path.join("test_dir");
            let _ = create_dir_all(dir_path)?;
            let _ = copy(&file_path, dir_path.join("test_file.txt"))?;

            let ignore_file_path = dir_path.join(".gitignore");
            let mut ignore_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(ignore_file_path)
                .unwrap();
            ignore_file.write_all("test_file.txt".as_bytes()).unwrap();

            let bat_file_path = dir_path.join("test_code.bat");
            let mut bat_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(bat_file_path)
                .unwrap();
            bat_file
                .write_all("echo > Hello world!".as_bytes())
                .unwrap();

            let copy_save_path = test_compress_path.join("test_copy");
            let copy_dir_save_path = copy_save_path.join("test_dir");
            let copy_file_save_path = copy_save_path.join("test_file.txt");

            return Ok(TestPack {
                file_from: file_path.display().to_string(),
                file_to: copy_file_save_path.display().to_string(),
                dir_from: dir_path.display().to_string(),
                dir_to: copy_dir_save_path.display().to_string(),
            });
        }

        Err(Error::from(ErrorKind::NotFound))
    }

    // https://stackoverflow.com/questions/74809455/how-to-quickly-create-an-empty-file-of-a-specified-size-in-rust
    // https://users.rust-lang.org/t/idiomatic-way-to-generate-a-file-of-a-given-size/30407/2
    fn generate_specific_size_file(path: &str, size: usize) -> Result<(), std::io::Error> {
        use rand::Rng;
        use std::cmp;
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let mut rng = rand::thread_rng();
        let mut buffer = [0; 1024];
        let mut remaining_size = size;

        while remaining_size > 0 {
            let to_write = cmp::min(remaining_size, buffer.len());
            let buffer = &mut buffer[..to_write];
            rng.fill(buffer);
            writer.write(buffer)?;

            remaining_size -= to_write;
        }

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_show_in_explorer() {
        use std::env::current_dir;

        let home_dir = current_dir().unwrap();

        #[cfg(not(target_os = "linux"))]
        let _ = show_in_explorer(home_dir.display().to_string().as_str());

        #[cfg(target_os = "linux")]
        {
            let dbus = dbus::blocking::SyncConnection::new_session().ok();
            let _ = show_in_explorer(home_dir.display().to_string().as_str(), Some(dbus));
        }
    }

    #[test]
    fn test_create_remove_all() {
        use std::env::current_dir;
        use std::path::Path;

        let home_dir = current_dir().unwrap();
        let home_path = home_dir.as_path();

        let test_file_path = home_path.join("test_file_all.txt").display().to_string();
        let test_dir_path = home_path.join("test_dir_all").display().to_string();

        let _ = create_all(test_file_path.as_str(), "file").unwrap();
        let _ = create_all(test_dir_path.as_str(), "dir").unwrap();

        assert_eq!(Path::new(&test_file_path).exists(), true);
        assert_eq!(Path::new(&test_dir_path).exists(), true);

        let _ = remove_all(test_file_path.as_str()).unwrap();
        let _ = remove_all(test_dir_path.as_str()).unwrap();

        assert_eq!(Path::new(&test_file_path).exists(), false);
        assert_eq!(Path::new(&test_dir_path).exists(), false);
    }

    #[test]
    #[ignore]
    fn test_copy_all() {
        use same_file::is_same_file;
        use std::env::current_dir;
        use std::fs::remove_dir_all;

        if let Ok(copy_path) = test_build_pack("copy") {
            // println!("paths: {:?}", copy_path);
            let _ = copy_all(copy_path.file_from.as_str(), copy_path.file_to.as_str()).unwrap();
            let _ = copy_all(copy_path.dir_from.as_str(), copy_path.dir_to.as_str()).unwrap();

            // This will always return false, havn't figure out why.
            match is_same_file(copy_path.file_from.as_str(), copy_path.file_to.as_str()) {
                Ok(res) => {
                    assert_eq!(res, true);
                }
                Err(error) => {
                    panic!("test_copy_all error when compare same: {:?}", error);
                }
            }

            // This will always return false, havn't figure out why.
            match is_same_file(copy_path.dir_from.as_str(), copy_path.dir_to.as_str()) {
                Ok(res) => {
                    assert_eq!(res, true);
                }
                Err(error) => {
                    panic!("test_copy_all error when compare same: {:?}", error);
                }
            }

            let _ = remove_dir_all(current_dir().expect("").join("test_explorer")).unwrap();
        }
    }

    #[test]
    fn test_restrict_dir_subitems_count() {
        use std::env::current_dir;
        use std::fs::{read_dir, remove_dir_all};

        if let Ok(copy_path) = test_build_pack("count") {
            let _ = restrict_dir_subitems_count(copy_path.dir_from.as_str(), 1).unwrap();
            let subitems = read_dir(copy_path.dir_from).unwrap();

            assert_eq!(subitems.count(), 1);

            let _ = remove_dir_all(current_dir().expect("").join("test_explorer_count")).unwrap();
        }
    }

    #[test]
    fn test_restrict_dir_subitems_size() {
        use byte_unit::Byte;
        use fs_extra::dir::get_size;
        use std::env::current_dir;
        use std::fs::remove_dir_all;
        use std::path::Path;

        if let Ok(copy_path) = test_build_pack("size") {
            let file_size = Byte::parse_str("3 MiB", true).unwrap().as_u64();
            let wanted_size = Byte::parse_str("1 MiB", true).unwrap().as_u64();

            // Generate file
            let sized_file = Path::new(&copy_path.dir_from).join("sized_file.txt");
            let _ = generate_specific_size_file(
                sized_file.display().to_string().as_str(),
                file_size as usize,
            )
            .unwrap();

            // Shrink to 1Mb

            let _ = restrict_dir_subitems_size(copy_path.dir_from.as_str(), wanted_size).unwrap();
            let after_size = get_size(copy_path.dir_from.as_str()).unwrap();

            let res = after_size <= wanted_size;

            assert_eq!(res, true);

            let _ = remove_dir_all(current_dir().expect("").join("test_explorer_size")).unwrap();
        }
    }
}
