use std::fs::File;
use std::io::Write;
use zip::result::ZipError;
use zip::write::FileOptions;

pub fn create_zip_with_password(file_path: &str, files: Vec<Vec<u8>>, password: &str) -> Result<(), ZipError> {
    let file = File::create(file_path)?;
    let mut archive = zip::ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    for (i, buffer) in files.iter().enumerate() {
        archive.start_file_with_encryption(format!("file_{}.bin", i), options, zip::EncryptionMethod::ZipCrypto, password)?;
        archive.write_all(buffer)?;
    }

    archive.finish()?;
    Ok(())
}
