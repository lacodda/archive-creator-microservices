use crate::api::task::FileInfo;
use std::fs::File;
use std::io::Write;
use zip::{result::ZipError, write::SimpleFileOptions, AesMode, CompressionMethod};

pub fn create_zip_with_password(file_path: &str, files: Vec<FileInfo>, password: &str) -> Result<(), ZipError> {
    let file = File::create(file_path)?;
    let mut archive = zip::ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .with_aes_encryption(AesMode::Aes256, password)
        .unix_permissions(0o755);

    for file_info in files {
        archive.start_file(file_info.filename, options)?;
        archive.write_all(&file_info.content)?;
    }

    archive.finish()?;
    Ok(())
}
