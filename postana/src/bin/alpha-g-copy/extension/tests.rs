use super::*;

#[test]
fn extension_display() {
    assert_eq!(Extension::Lz4.to_string(), String::from(".lz4"));
}
