use super::*;

#[test]
fn arg_is_directory() {
    let path = String::from(env!("CARGO_MANIFEST_DIR")) + "/src";
    assert!(is_directory(&path).is_ok());

    let path = String::from(env!("CARGO_MANIFEST_DIR")) + "/README.md";
    assert!(is_directory(&path).is_err());
}
