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
        Path::new("/eos/experiment/ALPHAg/midasdata_old/")
    );
    assert_eq!(
        Host::Alpha03.path_to_data(),
        Path::new("/daq/alpha_data0/acapra/alphag/midasdata/")
    );
}

#[test]
fn lxplus_filename_lz4() {
    assert_eq!(
        Host::Lxplus.filename(1, Some(Extension::Lz4)),
        Pattern::new("run00001sub*.mid.lz4").unwrap()
    );
    assert_eq!(
        Host::Lxplus.filename(12, Some(Extension::Lz4)),
        Pattern::new("run00012sub*.mid.lz4").unwrap()
    );
    assert_eq!(
        Host::Lxplus.filename(123, Some(Extension::Lz4)),
        Pattern::new("run00123sub*.mid.lz4").unwrap()
    );
    assert_eq!(
        Host::Lxplus.filename(1234, Some(Extension::Lz4)),
        Pattern::new("run01234sub*.mid.lz4").unwrap()
    );
    assert_eq!(
        Host::Lxplus.filename(12345, Some(Extension::Lz4)),
        Pattern::new("run12345sub*.mid.lz4").unwrap()
    );
    assert_eq!(
        Host::Lxplus.filename(123456, Some(Extension::Lz4)),
        Pattern::new("run123456sub*.mid.lz4").unwrap()
    );
}

#[test]
fn lxplus_filename() {
    assert_eq!(
        Host::Lxplus.filename(1, None),
        Pattern::new("run00001sub*.mid").unwrap()
    );
    assert_eq!(
        Host::Lxplus.filename(12, None),
        Pattern::new("run00012sub*.mid").unwrap()
    );
    assert_eq!(
        Host::Lxplus.filename(123, None),
        Pattern::new("run00123sub*.mid").unwrap()
    );
    assert_eq!(
        Host::Lxplus.filename(1234, None),
        Pattern::new("run01234sub*.mid").unwrap()
    );
    assert_eq!(
        Host::Lxplus.filename(12345, None),
        Pattern::new("run12345sub*.mid").unwrap()
    );
    assert_eq!(
        Host::Lxplus.filename(123456, None),
        Pattern::new("run123456sub*.mid").unwrap()
    );
}

#[test]
fn alpha03_filename() {
    assert_eq!(
        Host::Alpha03.filename(1, None),
        Pattern::new("run00001sub*.mid").unwrap()
    );
    assert_eq!(
        Host::Alpha03.filename(12, None),
        Pattern::new("run00012sub*.mid").unwrap()
    );
    assert_eq!(
        Host::Alpha03.filename(123, None),
        Pattern::new("run00123sub*.mid").unwrap()
    );
    assert_eq!(
        Host::Alpha03.filename(1234, None),
        Pattern::new("run01234sub*.mid").unwrap()
    );
    assert_eq!(
        Host::Alpha03.filename(12345, None),
        Pattern::new("run12345sub*.mid").unwrap()
    );

    assert_eq!(
        Host::Alpha03.filename(123456, None),
        Pattern::new("run123456sub*.mid").unwrap()
    );
}

#[test]
fn alpha03_filename_lz4() {
    assert_eq!(
        Host::Alpha03.filename(1, Some(Extension::Lz4)),
        Pattern::new("run00001sub*.mid.lz4").unwrap()
    );
    assert_eq!(
        Host::Alpha03.filename(12, Some(Extension::Lz4)),
        Pattern::new("run00012sub*.mid.lz4").unwrap()
    );
    assert_eq!(
        Host::Alpha03.filename(123, Some(Extension::Lz4)),
        Pattern::new("run00123sub*.mid.lz4").unwrap()
    );
    assert_eq!(
        Host::Alpha03.filename(1234, Some(Extension::Lz4)),
        Pattern::new("run01234sub*.mid.lz4").unwrap()
    );
    assert_eq!(
        Host::Alpha03.filename(12345, Some(Extension::Lz4)),
        Pattern::new("run12345sub*.mid.lz4").unwrap()
    );
    assert_eq!(
        Host::Alpha03.filename(123456, Some(Extension::Lz4)),
        Pattern::new("run123456sub*.mid.lz4").unwrap()
    );
}
