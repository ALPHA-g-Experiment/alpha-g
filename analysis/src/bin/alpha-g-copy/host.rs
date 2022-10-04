use crate::Extension;
use clap::ValueEnum;
use glob::Pattern;
use std::fmt;
use std::path::Path;

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
    pub fn path_to_data(&self) -> &Path {
        match self {
            Host::Lxplus => Path::new("/eos/experiment/ALPHAg/midasdata_old"),
            Host::Alpha03 => Path::new("/daq/alpha_data0/acapra/alphag/midasdata"),
        }
    }
    /// Return the Unix shell style pattern of all files from a single run number.
    /// This does not include the /path/to/data.
    pub fn filename(&self, run_number: u32, extension: Option<Extension>) -> Pattern {
        let extension = extension.map_or(String::new(), |e| e.to_string());
        match self {
            Host::Lxplus | Host::Alpha03 => {
                Pattern::new(&format!("run{run_number:05}sub*.mid{extension}")).unwrap()
            }
        }
    }
}

#[cfg(test)]
mod tests;
