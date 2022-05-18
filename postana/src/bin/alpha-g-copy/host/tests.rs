use super::*;

#[test]
fn host_display() {
    assert_eq!(Host::Lxplus.to_string(), String::from("lxplus.cern.ch"));
    assert_eq!(Host::Alpha03.to_string(), String::from("alpha03.triumf.ca"));
}

#[test]
fn host_data_path() {
    assert_eq!(
        Host::Lxplus.data_path(),
        PathBuf::from("/eos/experiment/ALPHAg/midasdata_old/")
    );
    assert_eq!(
        Host::Alpha03.data_path(),
        PathBuf::from("/daq/alpha_data0/acapra/alphag/midasdata/")
    );
}

#[test]
fn host_filename_pattern() {
    assert_eq!(
        Host::Lxplus.filename_pattern(1),
        String::from("run00001sub*.mid")
    );
    assert_eq!(
        Host::Lxplus.filename_pattern(12),
        String::from("run00012sub*.mid")
    );
    assert_eq!(
        Host::Lxplus.filename_pattern(123),
        String::from("run00123sub*.mid")
    );
    assert_eq!(
        Host::Lxplus.filename_pattern(1234),
        String::from("run01234sub*.mid")
    );
    assert_eq!(
        Host::Lxplus.filename_pattern(12345),
        String::from("run12345sub*.mid")
    );
    assert_eq!(
        Host::Lxplus.filename_pattern(123456),
        String::from("run123456sub*.mid")
    );

    assert_eq!(
        Host::Alpha03.filename_pattern(1),
        String::from("run00001sub*.mid")
    );
    assert_eq!(
        Host::Alpha03.filename_pattern(12),
        String::from("run00012sub*.mid")
    );
    assert_eq!(
        Host::Alpha03.filename_pattern(123),
        String::from("run00123sub*.mid")
    );
    assert_eq!(
        Host::Alpha03.filename_pattern(1234),
        String::from("run01234sub*.mid")
    );
    assert_eq!(
        Host::Alpha03.filename_pattern(12345),
        String::from("run12345sub*.mid")
    );
    assert_eq!(
        Host::Alpha03.filename_pattern(123456),
        String::from("run123456sub*.mid")
    );
}
