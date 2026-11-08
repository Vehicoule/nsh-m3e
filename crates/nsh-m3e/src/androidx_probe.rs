//! The vendored androidx files that the literals of the progress indicators cite, and the line of each, so that a
//! test fails when a bump of the pinned commit moves or changes what a citation points at (ADR 0012).

/// `ProgressIndicator.kt`: the flat indicators, their defaults and their constants.
pub(crate) const PROGRESS_INDICATOR_KT: &str =
    include_str!("../../../reference/androidx-material3-components/ProgressIndicator.kt");

/// `MotionTokens.kt`: the curves and durations of the Material 3 motion tokens.
pub(crate) const MOTION_TOKENS_KT: &str =
    include_str!("../../../reference/androidx-material3-tokens/MotionTokens.kt");

/// The line `number` (counted from 1) of `file`, without the indentation around it.
pub(crate) fn line(file: &str, number: usize) -> &str {
    number
        .checked_sub(1)
        .and_then(|index| file.lines().nth(index))
        .map_or("", str::trim)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_is_counted_from_one_and_trimmed() {
        let file = "first\n    second  \nthird";
        assert_eq!(line(file, 1), "first");
        assert_eq!(line(file, 2), "second");
        assert_eq!(line(file, 3), "third");
        // Past the end, and before the start, there is nothing.
        assert_eq!(line(file, 4), "");
        assert_eq!(line(file, 0), "");
    }
}
