use super::*;

#[test]
fn extension_display() {
    assert_eq!(Extension::Lz4.to_string(), String::from(".lz4"));
}

#[test]
fn get_local_files() {
    let patterns: Vec<String> = vec![String::from("*.toml")];
    let files = local_files(Path::new(env!("CARGO_MANIFEST_DIR")), &patterns);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].to_str().unwrap(), "Cargo.toml");
}

#[test]
fn arg_is_directory() {
    let path = String::from(env!("CARGO_MANIFEST_DIR")) + "/src";
    assert!(is_directory(&path).is_ok());

    let path = String::from(env!("CARGO_MANIFEST_DIR")) + "/README.md";
    assert!(is_directory(&path).is_err());
}
