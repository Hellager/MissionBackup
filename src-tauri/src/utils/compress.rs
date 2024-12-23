//! The `compressor` module is able to create archieves for file or directory.
//! The following compression formats are supported: zip, tar.gz, tar.bz2, tar.xz, 7z.

#[allow(dead_code)]
/// Prepares packing for `from` path and `to path.
/// Returns the new `from` path
///
/// # Arguments
///
/// * `from` - A string slice that holds the source path
/// * `to` - A string slice that holds the save path
///
/// # Examples
///
/// ```
/// use compress::compressor::prepare_packing;
///
/// let file = "path\\for\\target\\file";
/// let dir = "path\\for\\target\\dir";
/// let save_file = "path\\for\\save\\file.format";
/// let save_dir = "path\\for\\save\\dir.format";
///
/// let packing_file = prepare_packing(file, save_file).except("some error occurs");
/// let packing_dir = prepare_packing(dir, save_dir).except("some error occurs");
/// assert_eq!(packing_file, "path\\for\\save\\packing\\file.format".to_string());
/// assert_eq!(packing_dir, "path\\for\\target\\dir".to_string());
/// ```
fn prepare_packing(from: &str, to: &str) -> Result<String, std::io::Error> {
    use std::fs::{copy, create_dir_all, File};
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    let src_path = Path::new(from);
    let save_path = Path::new(to);
    // Check whether `from` path exists
    if !Path::new(from).exists() {
        return Err(Error::from(ErrorKind::NotFound));
    }
    // Check whether child path
    if save_path.starts_with(src_path) {
        return Err(Error::from(ErrorKind::InvalidData));
    }

    if let Some(prefix) = save_path.parent() {
        // Create the `to` file.
        create_dir_all(prefix)?;
        File::create(save_path)?;

        // If `from` path is file, creates a directory `packing` at the parent path of `to` path.
        // Copy `from` to `packing` and returns the new `from` path inside `packing` directory.
        if src_path.is_file() {
            let _path = prefix.join("packing");
            let packing_dir = _path.as_path();
            let file_name = src_path.file_name();
            match file_name {
                Some(name) => {
                    create_dir_all(packing_dir)?;
                    copy(from, packing_dir.join(name.to_str().unwrap()))?;
                    return Ok(packing_dir.display().to_string());
                }
                None => {
                    return Err(Error::from(ErrorKind::NotFound));
                }
            }
        }
    }

    Ok(from.to_string())
}

#[allow(dead_code)]
/// Finishes packing for `from` path and `to path.
///
/// # Arguments
///
/// * `from` - A string slice that holds the source path
/// * `to` - A string slice that holds the save path
///
/// # Examples
///
/// ```
/// use compress::compressor::finish_packing;
///
/// let file = "path\\for\\target\\file";
/// let save_file = "path\\for\\save\\file.format";
///
/// let _ = finish_packing(file, save_file).except("some error occurs");
/// assert_eq!(Path::new("path\\for\\save\\packing").exists(), false);
/// ```
fn finish_packing(from: &str, to: &str) -> Result<(), std::io::Error> {
    use std::fs::remove_dir_all;
    use std::path::Path;

    // If `from` path is file, delete the `packing` directory.
    if Path::new(from).is_file() {
        if let Some(prefix) = Path::new(to).parent() {
            remove_dir_all(prefix.join("packing"))?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
/// Creates zip archive for directory.
///
/// # Arguments
///
/// * `it` - Iterator to walk directory
/// * `prefix` - A string slice that hold the directory path
/// * `writer` - Write target that stores the archive data
/// * `method` - The zip compression method
///
/// # Examples
///
/// ```
/// use compress::compressor::prepare_packing;
/// use std::io::{Error, ErrorKind};
/// use std::fs::OpenOptions;
///
/// let dir = "path\\for\\target\\dir";
/// let save_dir = "path\\for\\save\\dir.format";
///
/// let walkdir = walkdir::WalkDir::new(dir);
/// let iter = walkdir.into_iter();
/// let saver = OpenOptions::new().write(true).open(save_dir)?;
/// if let Err(_) = zip_dir(&mut iter.filter_map(|e| e.ok()), &dir, saver, zip::CompressionMethod::Deflated) {
///     return Err(Error::from(ErrorKind::Other));
/// }
/// ```
fn zip_dir<T>(
    it: &mut dyn Iterator<Item = walkdir::DirEntry>,
    prefix: &std::path::Path,
    writer: T,
    method: zip::CompressionMethod,
) -> anyhow::Result<()>
where
    T: std::io::Write + std::io::Seek,
{
    use anyhow::Context;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::Path;

    let mut zip = zip::ZipWriter::new(writer);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let prefix = Path::new(prefix);
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(prefix).unwrap();
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .with_context(|| format!("{name:?} Is a Non UTF-8 Path"))?;

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            // info!("adding file {:?} as {:?} ...", path, name);
            zip.start_file(path_as_string, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            // info!("adding dir {:?} as {:?} ...", path, name);
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

#[allow(dead_code)]
/// Creates zip archive for file or directory.
///
/// # Arguments
///
/// * `from` - A string slice that holds the source path
/// * `to` - A string slice that holds the save path
///
/// # Examples
///
/// ```
/// use compress::compressor::create_zip_archive;
///
/// let src = "path\\for\\target";
/// let dst = "path\\for\\save.zip";
///
/// let _ = create_zip_archive(src, dst)?;
/// assert_eq!(Path::new("path\\for\\save.zip").exists(), true);
/// ```
fn create_zip_archive(from: &str, to: &str) -> Result<(), std::io::Error> {
    use std::fs::OpenOptions;
    use std::io::{Error, ErrorKind};
    if let Ok(src) = prepare_packing(from, to) {
        let prefix = src.clone();
        let walkdir = walkdir::WalkDir::new(src);
        let iter = walkdir.into_iter();

        let save_file = OpenOptions::new().write(true).open(to)?;
        if let Err(_) = zip_dir(
            &mut iter.filter_map(|e| e.ok()),
            &std::path::Path::new(&prefix),
            save_file,
            zip::CompressionMethod::Deflated,
        ) {
            return Err(Error::from(ErrorKind::Other));
        }
    }

    finish_packing(from, to)?;

    Ok(())
}

#[allow(dead_code)]
/// Creates tar package for file or directory.
///
/// # Arguments
///
/// * `from` - A string slice that holds the source path
/// * `to` - A string slice that holds the save path
///
/// # Examples
///
/// ```
/// use compress::compressor::create_tar_archive;
///
/// let src = "path\\for\\target";
/// let dst = "path\\for\\save.tar";
///
/// let _ = create_tar_archive(src, dst)?;
/// assert_eq!(Path::new("path\\for\\save.tar").exists(), true);
/// ```
fn create_tar_package(from: &str, to: &str) -> Result<(), std::io::Error> {
    use std::fs::OpenOptions;
    use std::io::{Error, ErrorKind};
    if let Ok(src) = prepare_packing(from, to) {
        let save_file = OpenOptions::new().write(true).open(to)?;
        let mut tar_builder = tar::Builder::new(save_file);

        if let Err(_) = tar_builder.append_dir_all("", src) {
            return Err(Error::from(ErrorKind::Other));
        }
    }

    finish_packing(from, to)?;

    Ok(())
}

#[allow(dead_code)]
/// Creates tar.gz archive for file or directory.
///
/// # Arguments
///
/// * `from` - A string slice that holds the source path
/// * `to` - A string slice that holds the save path
///
/// # Examples
///
/// ```
/// use compress::compressor::create_tar_gz_archive;
///
/// let src = "path\\for\\target";
/// let dst = "path\\for\\save.tar.gz";
///
/// let _ = create_tar_gz_archive(src, dst)?;
/// assert_eq!(Path::new("path\\for\\save.tar.gz").exists(), true);
/// ```
fn create_tar_gz_archive(from: &str, to: &str) -> Result<(), std::io::Error> {
    use flate2::write::GzEncoder;
    use std::fs::OpenOptions;
    use std::io::{Error, ErrorKind};

    if let Ok(src) = prepare_packing(from, to) {
        let save_file = OpenOptions::new().write(true).open(to)?;
        let encoder = GzEncoder::new(save_file, flate2::Compression::default());
        let mut tar_builder = tar::Builder::new(encoder);

        if let Err(_) = tar_builder.append_dir_all("", src) {
            return Err(Error::from(ErrorKind::Other));
        }
    }

    finish_packing(from, to)?;

    Ok(())
}

#[allow(dead_code)]
/// Creates tar.bz2 archive for file or directory.
///
/// # Arguments
///
/// * `from` - A string slice that holds the source path
/// * `to` - A string slice that holds the save path
///
/// # Examples
///
/// ```
/// use compress::compressor::create_tar_bz2_archive;
///
/// let src = "path\\for\\target";
/// let dst = "path\\for\\save.tar.gz";
///
/// let _ = create_tar_bz2_archive(src, dst)?;
/// assert_eq!(Path::new("path\\for\\save.tar.bz2").exists(), true);
/// ```
fn create_tar_bz2_archive(from: &str, to: &str) -> Result<(), std::io::Error> {
    use bzip2::write::BzEncoder;
    use std::fs::OpenOptions;
    use std::io::{Error, ErrorKind};

    if let Ok(src) = prepare_packing(from, to) {
        let save_file = OpenOptions::new().write(true).open(to)?;
        let encoder = BzEncoder::new(save_file, bzip2::Compression::best());
        let mut tar_builder = tar::Builder::new(encoder);

        if let Err(_) = tar_builder.append_dir_all("", src) {
            return Err(Error::from(ErrorKind::Other));
        }
    }

    finish_packing(from, to)?;

    Ok(())
}

#[allow(dead_code)]
/// Creates tar.xz archive for file or directory.
///
/// # Arguments
///
/// * `from` - A string slice that holds the source path
/// * `to` - A string slice that holds the save path
///
/// # Examples
///
/// ```
/// use compress::compressor::create_tar_xz_archive;
///
/// let src = "path\\for\\target";
/// let dst = "path\\for\\save.tar.xz";
///
/// let _ = create_tar_xz_archive(src, dst)?;
/// assert_eq!(Path::new("path\\for\\save.tar.xz").exists(), true);
/// ```
fn create_tar_xz_archive(from: &str, to: &str) -> Result<(), std::io::Error> {
    use std::fs::OpenOptions;
    use std::io::{Error, ErrorKind};
    use xz2::write::XzEncoder;

    if let Ok(src) = prepare_packing(from, to) {
        let save_file = OpenOptions::new().write(true).open(to)?;
        let encoder = XzEncoder::new(save_file, 6);
        let mut tar_builder = tar::Builder::new(encoder);

        if let Err(_) = tar_builder.append_dir_all("", src) {
            return Err(Error::from(ErrorKind::Other));
        }
    }

    finish_packing(from, to)?;

    Ok(())
}

#[allow(dead_code)]
/// Creates 7z archive for file or directory.
///
/// # Arguments
///
/// * `from` - A string slice that holds the source path
/// * `to` - A string slice that holds the save path
///
/// # Examples
///
/// ```
/// use compress::compressor::create_7zip_archive;
///
/// let src = "path\\for\\target";
/// let dst = "path\\for\\save.7z";
///
/// let _ = create_7zip_archive(src, dst)?;
/// assert_eq!(Path::new("path\\for\\save.7z").exists(), true);
/// ```
fn create_7zip_archive(from: &str, to: &str) -> Result<(), std::io::Error> {
    use std::io::{Error, ErrorKind};

    if let Err(_) = sevenz_rust::compress_to_path(from, to) {
        return Err(Error::from(ErrorKind::Other));
    }

    Ok(())
}

#[allow(dead_code)]
/// Creates archive for file or directory.
///
/// # Arguments
///
/// * `from` - A string slice that holds the source path
/// * `to` - A string slice that holds the save path
///
/// # Examples
///
/// ```
/// use compress::compressor::create_archive;
///
/// let src = "path\\for\\target";
/// let dst = "path\\for\\save.zip";
///
/// let _ = create_archive(src, dst)?;
/// assert_eq!(Path::new("path\\for\\save.zip").exists(), true);
/// ```
pub fn create_archive(from: &str, to: &str) -> Result<(), std::io::Error> {
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    let src_path = Path::new(from);
    let save_path = Path::new(to);
    // Check whether `from` path exists
    if !Path::new(from).exists() {
        return Err(Error::from(ErrorKind::NotFound));
    }
    // Check whether child path
    if save_path.starts_with(src_path) {
        return Err(Error::from(ErrorKind::InvalidData));
    }

    let mut save_format: Option<&str> = None;
    let support_formats = Vec::from(["zip", "tar.gz", "tar.bz2", "tar.xz", "7z"]);
    for item in &support_formats {
        if to.ends_with(item) {
            save_format = Some(item);
            break;
        }
    }

    if let Some(format) = save_format {
        match format {
            "zip" => {
                create_zip_archive(from, to)?;
            }
            "tar.gz" => {
                create_tar_gz_archive(from, to)?;
            }
            "tar.bz2" => {
                create_tar_bz2_archive(from, to)?;
            }
            "tar.xz" => {
                create_tar_xz_archive(from, to)?;
            }
            "7z" => {
                create_7zip_archive(from, to)?;
            }
            _ => {
                return Err(Error::from(ErrorKind::Unsupported));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestPack {
        pub file_from: String,
        pub file_to: String,
        pub dir_from: String,
        pub dir_to: String,
    }

    fn test_build_pack(format: &str) -> Result<TestPack, std::io::Error> {
        use std::env::current_dir;
        use std::fs::{copy, create_dir_all, OpenOptions};
        use std::io::Write;
        use std::io::{Error, ErrorKind};

        if let Ok(home_dir) = current_dir() {
            let test_compress_path = &home_dir.join(format!("test_compress_{}", format));
            let _ = create_dir_all(test_compress_path)?;

            let file_path = test_compress_path
                .join("test_file.txt")
                .display()
                .to_string();
            println!("test file_path: {}", &file_path);
            let mut test_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(file_path.clone().as_str())
                .unwrap();
            test_file.write_all("Hello world!".as_bytes()).unwrap();

            let dir_path = &test_compress_path.join("test_dir");
            let _ = create_dir_all(dir_path)?;
            let _ = copy(&file_path, dir_path.join("test_file.txt"))?;

            return Ok(TestPack {
                file_from: file_path.clone(),
                file_to: format!("{}.{}", file_path.clone(), format),
                dir_from: dir_path.display().to_string(),
                dir_to: format!("{}.{}", dir_path.display().to_string(), format),
            });
        }

        Err(Error::from(ErrorKind::NotFound))
    }

    #[test]
    fn test_prepare_and_finish_packing() {
        use std::env::current_dir;
        use std::fs::remove_dir_all;
        use std::path::Path;

        if let Ok(to_compress) = test_build_pack("zip") {
            let re_from_file = prepare_packing(
                &to_compress.file_from.as_str(),
                &to_compress.file_to.as_str(),
            )
            .unwrap();
            let re_from_dir =
                prepare_packing(&to_compress.dir_from.as_str(), &to_compress.dir_to.as_str())
                    .unwrap();

            assert_eq!(Path::new(re_from_file.as_str()).exists(), true);
            assert_eq!(Path::new(re_from_dir.as_str()).exists(), true);

            let _ = remove_dir_all(
                current_dir()
                    .expect("")
                    .join(format!("test_compress_{}", "zip")),
            )
            .unwrap();
        }
    }

    #[test]
    #[ignore] // test single
    fn test_create_zip_archive() {
        use std::env::current_dir;
        use std::fs::remove_dir_all;
        use std::path::Path;

        if let Ok(to_compress) = test_build_pack("zip") {
            let _ = create_zip_archive(
                &to_compress.file_from.as_str(),
                &to_compress.file_to.as_str(),
            );
            let _ =
                create_zip_archive(&to_compress.dir_from.as_str(), &to_compress.dir_to.as_str());

            assert_eq!(Path::new(to_compress.file_to.as_str()).exists(), true);
            assert_eq!(Path::new(to_compress.dir_to.as_str()).exists(), true);

            let _ = remove_dir_all(
                current_dir()
                    .expect("")
                    .join(format!("test_compress_{}", "zip")),
            )
            .unwrap();
        }
    }

    #[test]
    fn test_create_tar_package() {
        use std::env::current_dir;
        use std::fs::remove_dir_all;
        use std::path::Path;

        if let Ok(to_compress) = test_build_pack("tar") {
            let _ = create_tar_package(
                &to_compress.file_from.as_str(),
                &to_compress.file_to.as_str(),
            );
            let _ =
                create_tar_package(&to_compress.dir_from.as_str(), &to_compress.dir_to.as_str());

            assert_eq!(Path::new(to_compress.file_to.as_str()).exists(), true);
            assert_eq!(Path::new(to_compress.dir_to.as_str()).exists(), true);

            let _ = remove_dir_all(
                current_dir()
                    .expect("")
                    .join(format!("test_compress_{}", "tar")),
            )
            .unwrap();
        }
    }

    #[test]
    fn test_create_tar_gz_archive() {
        use std::env::current_dir;
        use std::fs::remove_dir_all;
        use std::path::Path;

        if let Ok(to_compress) = test_build_pack("tar.gz") {
            let _ = create_tar_gz_archive(
                &to_compress.file_from.as_str(),
                &to_compress.file_to.as_str(),
            );
            let _ =
                create_tar_gz_archive(&to_compress.dir_from.as_str(), &to_compress.dir_to.as_str());

            assert_eq!(Path::new(to_compress.file_to.as_str()).exists(), true);
            assert_eq!(Path::new(to_compress.dir_to.as_str()).exists(), true);

            let _ = remove_dir_all(
                current_dir()
                    .expect("")
                    .join(format!("test_compress_{}", "tar.gz")),
            )
            .unwrap();
        }
    }

    #[test]
    fn test_create_tar_bz2_archive() {
        use std::env::current_dir;
        use std::fs::remove_dir_all;
        use std::path::Path;

        if let Ok(to_compress) = test_build_pack("tar.bz2") {
            let _ = create_tar_bz2_archive(
                &to_compress.file_from.as_str(),
                &to_compress.file_to.as_str(),
            );
            let _ = create_tar_bz2_archive(
                &to_compress.dir_from.as_str(),
                &to_compress.dir_to.as_str(),
            );

            assert_eq!(Path::new(to_compress.file_to.as_str()).exists(), true);
            assert_eq!(Path::new(to_compress.dir_to.as_str()).exists(), true);

            let _ = remove_dir_all(
                current_dir()
                    .expect("")
                    .join(format!("test_compress_{}", "tar.bz2")),
            )
            .unwrap();
        }
    }

    #[test]
    fn test_create_tar_xz_archive() {
        use std::env::current_dir;
        use std::fs::remove_dir_all;
        use std::path::Path;

        if let Ok(to_compress) = test_build_pack("tar.xz") {
            let _ = create_tar_xz_archive(
                &to_compress.file_from.as_str(),
                &to_compress.file_to.as_str(),
            );
            let _ =
                create_tar_xz_archive(&to_compress.dir_from.as_str(), &to_compress.dir_to.as_str());

            assert_eq!(Path::new(to_compress.file_to.as_str()).exists(), true);
            assert_eq!(Path::new(to_compress.dir_to.as_str()).exists(), true);

            let _ = remove_dir_all(
                current_dir()
                    .expect("")
                    .join(format!("test_compress_{}", "tar.xz")),
            )
            .unwrap();
        }
    }

    #[test]
    #[ignore] // test single
    fn test_create_7zip_archive() {
        use std::env::current_dir;
        use std::fs::remove_dir_all;
        use std::path::Path;

        if let Ok(to_compress) = test_build_pack("7z") {
            let _ = create_7zip_archive(
                &to_compress.file_from.as_str(),
                &to_compress.file_to.as_str(),
            );
            let _ =
                create_7zip_archive(&to_compress.dir_from.as_str(), &to_compress.dir_to.as_str());

            assert_eq!(Path::new(to_compress.file_to.as_str()).exists(), true);
            assert_eq!(Path::new(to_compress.dir_to.as_str()).exists(), true);

            let _ = remove_dir_all(
                current_dir()
                    .expect("")
                    .join(format!("test_compress_{}", "7z")),
            )
            .unwrap();
        }
    }

    #[test]
    fn test_create_archive() {
        use std::env::current_dir;
        use std::fs::remove_dir_all;
        use std::path::Path;

        if let Ok(to_compress) = test_build_pack("7z") {
            let _ = create_archive(
                &to_compress.file_from.as_str(),
                &to_compress.file_to.as_str(),
            );
            let _ = create_archive(&to_compress.dir_from.as_str(), &to_compress.dir_to.as_str());

            assert_eq!(Path::new(to_compress.file_to.as_str()).exists(), true);
            assert_eq!(Path::new(to_compress.dir_to.as_str()).exists(), true);

            let _ = remove_dir_all(
                current_dir()
                    .expect("")
                    .join(format!("test_compress_{}", "7z")),
            )
            .unwrap();
        }
    }
}
