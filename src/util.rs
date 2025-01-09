use reqwest::blocking::get;
use std::fs::{self, File};
use std::io::{self, Cursor};
use std::path::Path;
use zip::ZipArchive;

const GIT_MODEL_PATH: &str = "https://github.com/harloc-AI/rustai_abalone/raw/main/src/magister_zero.zip";

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

/// downloads the tensorflow model
/// 
/// downloads the tensorflow model for the library from github and stores it in the given folder.
/// At the given path the new folder `magister_zero_unwrap_save` will be created. A String
/// representing that path will be returned
/// 
/// # Arguments
/// 
/// * `output_dir` - path to the target directory
/// 
/// # Returns
/// 
/// * `model_path` - path to the extracted model, that can be used to instantiate `MagisterLudi`
/// 
/// # Examples
/// 
/// ```rust
/// use rustai_abalone::util::download_model;
/// let model_path = download_model(".");
/// ```
pub fn download_model(output_dir: &str) -> String {
    let model_path = Path::new(output_dir).join("magister_zero_unwrap_save");
    if let Err(e) = download_and_extract_zip(GIT_MODEL_PATH, output_dir) {
        eprintln!("Error: {}", e);
    }
    let model_path_str = model_path.to_str().unwrap().to_string();
    model_path_str
}

/// checks whether the tensorflow model is present
/// 
/// checks the given path for the existance of the tensorflow model.
/// Either the path itself contains the model files or it contains
/// a folder named `magister_zero_unwrap_save`. 
/// Five files have to be present:
/// 
/// * `saved_model.pb`
/// * `keras_metadata.pb`
/// * `fingerprint.pb`
/// * `variables/variables.index`
/// * `variables/variables.data-00000-of-00001`
/// 
/// The path to the folder containing these files will be returned
/// 
/// # Arguments
/// 
/// * `model_str` - path to check for model presence
/// 
/// # Returns
/// 
/// * `model_path_str` - actual model path to instantiate `MagisterLudi` or None if the model is not present or `model_str` is not a valid path
/// 
/// # Examples
/// 
/// ```rust
/// use rustai_abalone::util::check_model_present;
/// let checked_path = check_model_present(".");
/// ```
pub fn check_model_present(model_str: &str) -> Option<String> {
    let model_path = Path::new(model_str);
    if !model_path.exists() {
        return None;
    }
    let mag_zero_path = model_path.join("magister_zero_unwrap_save");
    if mag_zero_path.exists() {
        return check_model_present(mag_zero_path.to_str().unwrap());
    }
    let to_check = [
        "saved_model.pb",
        "keras_metadata.pb",
        "fingerprint.pb",
        Box::leak(format!("variables{}variables.index", std::path::MAIN_SEPARATOR).into_boxed_str()),
        Box::leak(format!("variables{}variables.data-00000-of-00001", std::path::MAIN_SEPARATOR).into_boxed_str()),
    ];
    for file in to_check {
        if !model_path.join(file).exists(){
            return None;
        }
    }
    Some(model_str.to_string())
}