use reqwest::blocking::get;
use std::fs::{self, File};
use std::io::{self, Cursor};
use zip::ZipArchive;

pub const GIT_MODEL_PATH: &str = "https://github.com/harloc-AI/rustai_abalone/raw/main/src/magister_zero.zip";

fn download_and_extract_zip(url: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // download .zip file
    let response = get(url)?;
    let bytes = response.bytes()?;

    // open .zip file bytes
    let cursor = Cursor::new(bytes);
    let mut zip = ZipArchive::new(cursor)?;

    // create target directory if it does not exist
    fs::create_dir_all(output_dir)?;

    // extract from .zip
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let outpath = std::path::Path::new(output_dir).join(file.name());

        if file.is_dir() {
            // create folder
            fs::create_dir_all(&outpath)?;
        } else {
            // extract file
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

    }

    Ok(())
}

pub fn download_model(output_dir: &str) -> String {
    let model_path = std::path::Path::new(output_dir).join("magister_zero_save_unwrap");
    if let Err(e) = download_and_extract_zip(GIT_MODEL_PATH, output_dir) {
        eprintln!("Error: {}", e);
    }
    let model_path_str = model_path.to_str().unwrap().to_string();
    model_path_str
}