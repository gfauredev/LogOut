use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Get the output directory for generated files
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("exercises_data.rs");
    
    // Create assets directory if it doesn't exist
    let assets_dir = Path::new("assets");
    let exercises_dir = assets_dir.join("exercises");
    fs::create_dir_all(&exercises_dir).expect("Failed to create assets/exercises directory");
    
    // URLs for downloading
    const EXERCISES_JSON_URL: &str = "https://raw.githubusercontent.com/yuhonas/free-exercise-db/main/dist/exercises.json";
    const REPO_ZIP_URL: &str = "https://github.com/yuhonas/free-exercise-db/archive/refs/heads/main.zip";
    
    let download_json_path = Path::new(&out_dir).join("exercises.json");
    let download_zip_path = Path::new(&out_dir).join("free-exercise-db.zip");
    
    println!("cargo:warning=Downloading exercises.json from {}", EXERCISES_JSON_URL);
    
    // Download the exercises.json
    let json_success = Command::new("curl")
        .args(&["-L", "-o", download_json_path.to_str().unwrap(), EXERCISES_JSON_URL])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
        || Command::new("wget")
            .args(&["-O", download_json_path.to_str().unwrap(), EXERCISES_JSON_URL])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
    
    // Read the exercises JSON
    let exercises_json = if json_success && download_json_path.exists() {
        println!("cargo:warning=Successfully downloaded exercises.json");
        fs::read_to_string(&download_json_path)
            .expect("Failed to read downloaded exercises.json file")
    } else {
        panic!("Failed to download exercises.json. Please ensure curl or wget is installed.");
    };
    
    // Parse the JSON to validate it's correct
    let exercises: serde_json::Value = serde_json::from_str(&exercises_json)
        .expect("Failed to parse exercises.json");
    
    // Verify it's an array
    if !exercises.is_array() {
        panic!("exercises.json must contain an array of exercises");
    }
    
    // Download and extract exercise images if not already present
    // Check if we already have images
    let sample_exercise_dir = exercises_dir.join("3_4_Sit-Up");
    if !sample_exercise_dir.exists() {
        println!("cargo:warning=Downloading exercise images from {}", REPO_ZIP_URL);
        
        // Download the repository zip
        let zip_success = Command::new("curl")
            .args(&["-L", "-o", download_zip_path.to_str().unwrap(), REPO_ZIP_URL])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
            || Command::new("wget")
                .args(&["-O", download_zip_path.to_str().unwrap(), REPO_ZIP_URL])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        
        if zip_success && download_zip_path.exists() {
            println!("cargo:warning=Successfully downloaded exercise images");
            
            // Extract the zip file to OUT_DIR
            let extract_dir = Path::new(&out_dir).join("repo");
            fs::create_dir_all(&extract_dir).expect("Failed to create extract directory");
            
            let unzip_success = Command::new("unzip")
                .args(&["-q", "-o", download_zip_path.to_str().unwrap(), "-d", extract_dir.to_str().unwrap()])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            
            if unzip_success {
                println!("cargo:warning=Successfully extracted exercise images");
                
                // Copy exercises directory from extracted repo to assets
                let source_exercises = extract_dir.join("free-exercise-db-main").join("exercises");
                if source_exercises.exists() {
                    // Copy all exercise image directories
                    if let Ok(entries) = fs::read_dir(&source_exercises) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                let dir_name = path.file_name().unwrap();
                                let dest = exercises_dir.join(dir_name);
                                
                                // Create destination directory
                                fs::create_dir_all(&dest).ok();
                                
                                // Copy image files
                                if let Ok(files) = fs::read_dir(&path) {
                                    for file in files.flatten() {
                                        let file_path = file.path();
                                        if file_path.is_file() {
                                            let file_name = file_path.file_name().unwrap();
                                            let dest_file = dest.join(file_name);
                                            fs::copy(&file_path, &dest_file).ok();
                                        }
                                    }
                                }
                            }
                        }
                        println!("cargo:warning=Successfully copied exercise images to assets/exercises/");
                    }
                }
            } else {
                println!("cargo:warning=Failed to extract zip. Images will be loaded from CDN.");
            }
        } else {
            println!("cargo:warning=Failed to download exercise images. Images will be loaded from CDN.");
        }
    } else {
        println!("cargo:warning=Exercise images already present in assets/exercises/");
    }
    
    // Generate Rust code that will contain the JSON as a static string
    let generated_code = format!(
        r#####"
// This file is automatically generated by build.rs
// Do not edit manually

pub const EXERCISES_JSON: &str = r####"{}"####;
"#####,
        exercises_json
    );
    
    // Write the generated code to a file
    fs::write(&dest_path, generated_code)
        .expect("Failed to write generated exercises data");
    
    // Tell cargo to rerun if the assets directory changes
    println!("cargo:rerun-if-changed=assets/");
}
