//! Constants used throughout the library.

/// Markers that indicate the start of a JSON structure
pub const MARKERS: [char; 2] = ['{', '['];

/// All paired markers (opening and closing) used in JSON structures
pub const PAIRED_MARKERS: [char; 4] = ['{', '}', '[', ']'];
