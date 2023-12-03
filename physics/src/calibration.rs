// This is a macro definition for a convenient way to include multiple files
// as byte arrays. It takes a directory and multiple file names.
// The contents of all files are embedded at compile time.
macro_rules! includes {
    ($dname:ident = $dir:expr; $($fname:ident = $fpath:expr,)*) => (
        $(
            const $fname: &'static [u8] = include_bytes!(concat!($dir, $fpath));
        )*
    )
}

// Anode wires calibration
pub(crate) mod wires;
// Pad calibration
pub(crate) mod pads;
