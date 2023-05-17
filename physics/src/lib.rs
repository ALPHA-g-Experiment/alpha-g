// Calibration
//
// I have the strong opinion that all calibration is an implementation detail
// and should not be exposed to the user.  This also means that after every
// calibration procedure, all binaries that use the calibration data should be
// recompiled and redistributed. This has the advantage that everyone is
// guaranteed to use the same (and correct) calibration settings.
//
// If there is ever a compelling reason to expose calibration to the user, I
// believe it should be moved to a separate `alpha_g_calibration` crate.
mod calibration;
