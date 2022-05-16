use clap::ArgEnum;
use std::fmt;
use std::path::PathBuf;

/// Known hosts for ALPHA-g MIDAS files
#[derive(Clone, Copy, Debug, ArgEnum)]
pub enum Host {
    Lxplus,
    Alpha03,
}
impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Host::Lxplus => write!(f, "lxplus.cern.ch"),
            Host::Alpha03 => write!(f, "alpha03.triumf.ca"),
        }
    }
}

impl Host {
    /// Path to MIDAS files in a given host
    // Make sure that the path finishes with "/"
    pub fn data_path(&self) -> PathBuf {
        match self {
            Host::Lxplus => PathBuf::from("/eos/experiment/ALPHAg/midasdata_old/"),
            Host::Alpha03 => PathBuf::from("/daq/alpha_data0/acapra/alphag/midasdata/"),
        }
    }

    /// Given ALPHA-g run number; filename pattern for MIDAS files (without
    /// compression)
    pub fn filename_pattern(&self, run_number: u32) -> String {
        match self {
            Host::Lxplus | Host::Alpha03 => pattern_1(run_number),
        }
    }
}

// Pattern 1: runXXXXXsub*.mid
// i.e. run number is at least 5 characters with prefix of 0s
fn pattern_1(run_number: u32) -> String {
    let mut run_number = run_number.to_string();
    while run_number.len() < 5 {
        run_number.insert(0, '0');
    }
    String::from("run") + &run_number + "sub*.mid"
}
