//! Marker module for JSON structure tracking.

use crate::constants::MARKERS;

/// Represents a JSON marker with its expected counterpart.
///
/// A marker is a character that indicates the beginning of a JSON structure
/// (such as '{' or '[') and has an expected closing counterpart ('}' or ']').
#[derive(Debug, Copy, Clone)]
pub struct Marker {
    /// The character that corresponds to the closing of this marker
    pub(crate) expected_counterpart: char,
}

impl Marker {
    /// Creates a new marker if the provided character is a valid opening marker.
    ///
    /// # Arguments
    ///
    /// * `marker` - A reference to the character to check.
    ///
    /// # Returns
    ///
    /// * `Some(Marker)` - If the character is a valid opening marker ('{' or '[').
    /// * `None` - If the character is not a valid opening marker.
    ///
    /// # Examples
    ///
    /// ```
    /// use surfing::parser::marker::Marker;
    ///
    /// let open_brace = Marker::new(&'{');
    /// assert!(open_brace.is_some());
    /// assert_eq!(open_brace.unwrap().is_counter_part(&'}'), true);
    ///
    /// let open_bracket = Marker::new(&'[');
    /// assert!(open_bracket.is_some());
    /// assert_eq!(open_bracket.unwrap().is_counter_part(&']'), true);
    ///
    /// let invalid = Marker::new(&'x');
    /// assert!(invalid.is_none());
    /// ```
    pub fn new(marker: &char) -> Option<Self> {
        for included_marker in MARKERS {
            if *marker == included_marker {
                if *marker == '{' {
                    return Some(Self {
                        expected_counterpart: '}',
                    });
                }

                if *marker == '[' {
                    return Some(Self {
                        expected_counterpart: ']',
                    });
                }
            }
        }

        None
    }

    /// Checks if the provided character is the corresponding counter part for this marker.
    ///
    /// # Arguments
    ///
    /// * `marker` - A reference to the character to check.
    ///
    /// # Returns
    ///
    /// * `true` - If the character is the expected counterpart for this marker.
    /// * `false` - If the character is not the expected counterpart.
    ///
    /// # Examples
    ///
    /// ```
    /// use surfing::parser::marker::Marker;
    ///
    /// let open_brace = Marker::new(&'{').unwrap();
    /// assert!(open_brace.is_counter_part(&'}'));
    /// assert!(!open_brace.is_counter_part(&']'));
    /// ```
    pub fn is_counter_part(&self, marker: &char) -> bool {
        self.expected_counterpart == *marker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_new() {
        let marker = Marker::new(&'{');
        assert!(marker.is_some());
        assert_eq!(marker.unwrap().expected_counterpart, '}');

        let marker = Marker::new(&'[');
        assert!(marker.is_some());
        assert_eq!(marker.unwrap().expected_counterpart, ']');

        let marker = Marker::new(&'x');
        assert!(marker.is_none());
    }

    #[test]
    fn test_marker_is_counter_part() {
        let marker = Marker::new(&'{').unwrap();
        assert!(marker.is_counter_part(&'}'));
        assert!(!marker.is_counter_part(&']'));

        let marker = Marker::new(&'[').unwrap();
        assert!(marker.is_counter_part(&']'));
        assert!(!marker.is_counter_part(&'}'));
    }
}
