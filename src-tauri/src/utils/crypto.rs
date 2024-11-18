//! The `crypto` module contains functions about crypto.

#[allow(dead_code)]
/// Encodes str to base64 string.
/// Takes care if the input data is url, see [URL-safe alphabet](https://docs.rs/base64/latest/base64/#url-safe-alphabet).
///
/// # Arguments
///
/// * `data` - A string that holds the text data
///
/// # Examples
///
/// ```
/// use crypto::encoding_base64;
///
/// assert_eq!(encode_base64("Hello world!").expect("Error!"), "SGVsbG8gd29ybGQh");
/// ```
pub fn encode_base64(data: &str) -> Result<String, ()> {
    use base64::{
        engine::general_purpose::STANDARD, engine::general_purpose::URL_SAFE, Engine as _,
    };
    use url::Url;

    match Url::parse(data) {
        Ok(_) => Ok(STANDARD.encode(data)),
        Err(_) => Ok(URL_SAFE.encode(data)),
    }
}

#[allow(dead_code)]
/// Decodes base64 data to normal string.
/// Takes care if the origin data is url, see [URL-safe alphabet](https://docs.rs/base64/latest/base64/#url-safe-alphabet).
///
/// # Arguments
///
/// * `data` - A string that holds the encoded data.
/// * `is_url` - Option if the origin data is url.
///
/// # Examples
///
/// ```
/// use crypto::decode_base64;
///
/// assert_eq!(decode_base64("SGVsbG8gd29ybGQh", None).expect("Error!"), "Hello world!");
/// ```
pub fn decode_base64(data: &str, is_url: Option<bool>) -> Result<String, base64::DecodeError> {
    use base64::{
        engine::general_purpose::STANDARD, engine::general_purpose::URL_SAFE, DecodeError,
        Engine as _,
    };

    if let Some(true) = is_url {
        match URL_SAFE.decode(data) {
            Ok(arr) => {
                if let Ok(res) = String::from_utf8(arr) {
                    return Ok(res);
                } else {
                    return Err(DecodeError::from(DecodeError::InvalidPadding));
                }
            }
            Err(error) => {
                // println!("decode error, errMsg: {:?}", error);
                return Err(error);
            }
        }
    } else {
        match STANDARD.decode(data) {
            Ok(arr) => {
                if let Ok(res) = String::from_utf8(arr) {
                    return Ok(res);
                } else {
                    return Err(DecodeError::from(DecodeError::InvalidPadding));
                }
            }
            Err(error) => {
                // println!("decode error, errMsg: {:?}", error);
                return Err(error);
            }
        }
    }
}

#[allow(dead_code)]
/// Encodes file to Sha256 string.
///
/// # Arguments
///
/// * `path` - A string that holds the file path.
///
/// # Examples
///
/// ```
/// use crypto::encode_sha2_file;
///
/// let path = "path\\to\\encoded";
///
/// match encode_sha2_file(path) {
///     Ok(data) => {
///         println!("File {} encoded data: {}", path, data);
///     },
///     Err(error) => {
///         println!("Failed to encode file {}, errMsg: {:?}", path, error);
///     }
/// }
/// ```
pub fn encode_sha2_file(path: &str) -> Result<String, std::io::Error> {
    use sha2::{Digest, Sha256};
    use std::fs::read;
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    let input = Path::new(path);
    if !input.exists() {
        return Err(Error::from(ErrorKind::NotFound));
    }

    let mut hasher = Sha256::new();

    match read(path) {
        Ok(data) => {
            hasher.update(data);
            return Ok(format!("{:X}", hasher.finalize()));
        }
        Err(_error) => {
            // println!("encode sha2 file for {} failed, errMsg: {:?}", path, _error);
            return Err(Error::from(ErrorKind::Other));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode_base64() {
        let normal_text = encode_base64("Hello world!").expect("");
        assert_eq!(normal_text, "SGVsbG8gd29ybGQh".to_string());
    }

    #[test]
    fn test_decode_base64() {
        let normal_text = decode_base64("SGVsbG8gd29ybGQh", None).expect("");
        assert_eq!(normal_text, "Hello world!".to_string());
    }

    #[test]
    fn test_encode_sha2_file() {
        use std::env::current_dir;
        use std::fs::{remove_file, OpenOptions};
        use std::io::Write;

        if let Ok(path) = current_dir() {
            let file_path = path.join("test_file.txt").display().to_string();
            let mut test_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(file_path.clone().as_str())
                .unwrap();
            test_file.write_all("Hello world!".as_bytes()).unwrap();

            let encoded_data = encode_sha2_file(file_path.as_str()).expect("");
            assert_eq!(
                encoded_data,
                "C0535E4BE2B79FFD93291305436BF889314E4A3FAEC05ECFFCBB7DF31AD9E51A".to_string()
            );

            remove_file(file_path).expect("");
        }
    }
}
