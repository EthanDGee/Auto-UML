use glob::glob;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let staged_dir = Path::new(&out_dir).join("staged_languages");

    // Clean and create the destination directory
    if staged_dir.exists() {
        fs::remove_dir_all(&staged_dir).unwrap();
    }
    fs::create_dir_all(&staged_dir).unwrap();

    // Use pattern matching to find all config.yaml files
    let pattern = "languages/*/config.yaml";
    for path in glob(pattern)
        .expect("Failed to read glob pattern")
        .flatten()
    {
        if let Some(lang_name) = path.parent().and_then(|p| p.file_name()) {
            let lang_dest = staged_dir.join(lang_name);
            fs::create_dir_all(&lang_dest).unwrap();

            // Copy the file to the staged directory
            fs::copy(&path, lang_dest.join("config.yaml")).unwrap();
        }
    }

    // Export the staged directory path so include_dir! can use it
    println!("cargo:rustc-env=STAGED_LANGS_DIR={}", staged_dir.display());
    println!("cargo:rerun-if-changed=languages");
}
