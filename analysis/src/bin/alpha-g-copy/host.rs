use crate::Extension;
use clap::ValueEnum;
use std::fmt;
use std::path::PathBuf;

/// Known hosts for ALPHA-g MIDAS files
#[derive(Clone, Copy, Debug, ValueEnum)]
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
    pub fn path_to_data(&self) -> PathBuf {
        match self {
            Host::Lxplus => PathBuf::from("/eos/experiment/ALPHAg/midasdata_old"),
            Host::Alpha03 => PathBuf::from("/daq/alpha_data0/acapra/alphag/midasdata"),
        }
    }
    /// Return the Unix shell style pattern of all files from a single run number.
    /// This does not include the /path/to/data.
    // Using glob::Pattern instead of String sounds like a good idea, but every
    // usage of `filename()` in main needs the string anyway.
    // For some reason, the `glob` function in `glob::glob` does not work with
    // the `Pattern` type and requires a `&str`.
    pub fn filename(&self, run_number: u32, extension: Option<Extension>) -> String {
        let extension = extension.map_or(String::new(), |e| e.to_string());
        match self {
            Host::Lxplus | Host::Alpha03 => {
                format!("run{run_number:05}sub*.mid{extension}")
            }
        }
    }
}

#[cfg(test)]
mod tests;
