use super::*;

#[test]
fn host_display() {
    assert_eq!(Host::Lxplus.to_string(), String::from("lxplus.cern.ch"));
    assert_eq!(Host::Alpha03.to_string(), String::from("alpha03.triumf.ca"));
}

#[test]
fn host_path_to_data() {
    assert_eq!(
        Host::Lxplus.path_to_data(),
        PathBuf::from("/eos/experiment/ALPHAg/midasdata_old/")
    );
    assert_eq!(
        Host::Alpha03.path_to_data(),
        PathBuf::from("/daq/alpha_data0/acapra/alphag/midasdata/")
    );
}

#[test]
fn lxplus_filename_lz4() {
    assert_eq!(
        Host::Lxplus.filename(1, Some(Extension::Lz4)),
        String::from("run00001sub*.mid.lz4")
    );
    assert_eq!(
        Host::Lxplus.filename(12, Some(Extension::Lz4)),
        String::from("run00012sub*.mid.lz4")
    );
    assert_eq!(
        Host::Lxplus.filename(123, Some(Extension::Lz4)),
        String::from("run00123sub*.mid.lz4")
    );
    assert_eq!(
        Host::Lxplus.filename(1234, Some(Extension::Lz4)),
        String::from("run01234sub*.mid.lz4")
    );
    assert_eq!(
        Host::Lxplus.filename(12345, Some(Extension::Lz4)),
        String::from("run12345sub*.mid.lz4")
    );
    assert_eq!(
        Host::Lxplus.filename(123456, Some(Extension::Lz4)),
        String::from("run123456sub*.mid.lz4")
    );
}

#[test]
fn lxplus_filename() {
    assert_eq!(
        Host::Lxplus.filename(1, None),
        String::from("run00001sub*.mid")
    );
    assert_eq!(
        Host::Lxplus.filename(12, None),
        String::from("run00012sub*.mid")
    );
    assert_eq!(
        Host::Lxplus.filename(123, None),
        String::from("run00123sub*.mid")
    );
    assert_eq!(
        Host::Lxplus.filename(1234, None),
        String::from("run01234sub*.mid")
    );
    assert_eq!(
        Host::Lxplus.filename(12345, None),
        String::from("run12345sub*.mid")
    );
    assert_eq!(
        Host::Lxplus.filename(123456, None),
        String::from("run123456sub*.mid")
    );
}

#[test]
fn alpha03_filename() {
    assert_eq!(
        Host::Alpha03.filename(1, None),
        String::from("run00001sub*.mid")
    );
    assert_eq!(
        Host::Alpha03.filename(12, None),
        String::from("run00012sub*.mid")
    );
    assert_eq!(
        Host::Alpha03.filename(123, None),
        String::from("run00123sub*.mid")
    );
    assert_eq!(
        Host::Alpha03.filename(1234, None),
        String::from("run01234sub*.mid")
    );
    assert_eq!(
        Host::Alpha03.filename(12345, None),
        String::from("run12345sub*.mid")
    );

    assert_eq!(
        Host::Alpha03.filename(123456, None),
        String::from("run123456sub*.mid")
    );
}

#[test]
fn alpha03_filename_lz4() {
    assert_eq!(
        Host::Alpha03.filename(1, Some(Extension::Lz4)),
        String::from("run00001sub*.mid.lz4")
    );
    assert_eq!(
        Host::Alpha03.filename(12, Some(Extension::Lz4)),
        String::from("run00012sub*.mid.lz4")
    );
    assert_eq!(
        Host::Alpha03.filename(123, Some(Extension::Lz4)),
        String::from("run00123sub*.mid.lz4")
    );
    assert_eq!(
        Host::Alpha03.filename(1234, Some(Extension::Lz4)),
        String::from("run01234sub*.mid.lz4")
    );
    assert_eq!(
        Host::Alpha03.filename(12345, Some(Extension::Lz4)),
        String::from("run12345sub*.mid.lz4")
    );
    assert_eq!(
        Host::Alpha03.filename(123456, Some(Extension::Lz4)),
        String::from("run123456sub*.mid.lz4")
    );
}
