//! Where the catalog puts each badge, so that the visual checks know where to read its pixels.
//!
//! The badges sit in one row below the icon buttons' grid: a dot, then a label of one character, of another, of
//! three, and a longer text. Each is the badge of a [`badged_box`](nsh_m3e::badge::badged_box) around a small
//! standard icon button, the anchor, which is at a fixed place whatever the badge. Light and dark are the two
//! appearances the screenshots are taken in.

use nsh_core::geometry::Points;
use nsh_m3e::icon_button::{IconButtonSize, IconButtonWidth};
use nsh_m3e::tokens::androidx::badge_tokens;

use crate::grid::{self, ICON_COLUMN_WIDTH, MARGIN};

/// How far before the trailing edge of its anchor a dot starts, and how far it overlaps the anchor from the anchor's
/// top edge: androidx `Badge.kt`, `BadgeOffset` (line 218), which no token holds. A test pins it to the vendored file.
/// The checks' own statement of it, apart from the library's: if the library's constant moved, the pixels would not.
pub const DOT_OFFSET: Points = Points(6.0);

/// How far before the trailing edge of its anchor a badge with a label starts: androidx `Badge.kt`,
/// `BadgeWithContentHorizontalOffset` (line 209).
pub const LABEL_HORIZONTAL_OFFSET: Points = Points(12.0);

/// How far a badge with a label overlaps its anchor from the anchor's top edge: androidx `Badge.kt`,
/// `BadgeWithContentVerticalOffset` (line 211).
pub const LABEL_VERTICAL_OFFSET: Points = Points(14.0);

/// One badge of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum BadgeCell {
    /// No label: a dot.
    #[default]
    Dot,
    /// The label "1": one character, so a circle.
    One,
    /// The label "9": another character, so a circle too.
    Nine,
    /// The label "99+": a count past two digits, the longest a badge of Material shows.
    NinetyNinePlus,
    /// A longer text, which makes a wide pill.
    LongText,
}

/// Every badge, in the order of the columns.
pub const BADGES: [BadgeCell; 5] = [
    BadgeCell::Dot,
    BadgeCell::One,
    BadgeCell::Nine,
    BadgeCell::NinetyNinePlus,
    BadgeCell::LongText,
];

/// The size of every anchor: the small uniform icon button, whose icon is 24 points in a 40-point container.
#[must_use]
pub fn anchor() -> (Points, Points) {
    (
        grid::icon_width(IconButtonSize::Small, IconButtonWidth::Uniform),
        grid::icon_height(IconButtonSize::Small),
    )
}

impl BadgeCell {
    /// Every badge of the catalog, column by column.
    pub fn all() -> impl Iterator<Item = Self> {
        BADGES.into_iter()
    }

    /// Its label, or none for the dot.
    #[must_use]
    pub fn label(self) -> Option<&'static str> {
        match self {
            Self::One => Some("1"),
            Self::Nine => Some("9"),
            Self::NinetyNinePlus => Some("99+"),
            Self::LongText => Some("Sale 50%"),
            // `BadgeCell` is non-exhaustive: a badge added later is a dot until the catalog shows it.
            _ => None,
        }
    }

    /// The test id of the view around it and its anchor, which the test scripts name (`id:badge-99-plus`).
    #[must_use]
    pub fn test_id(self) -> &'static str {
        match self {
            Self::One => "badge-1",
            Self::Nine => "badge-9",
            Self::NinetyNinePlus => "badge-99-plus",
            Self::LongText => "badge-long-text",
            _ => "badge-dot",
        }
    }

    /// The top left corner of its anchor in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let column = BADGES.iter().position(|badge| *badge == self).unwrap_or(0);
        let mut x = MARGIN.0;
        for _ in 0..column {
            x += ICON_COLUMN_WIDTH.0;
        }
        (Points(x), grid::badge_top())
    }

    /// The top left corner of the badge in the viewport, in points, as androidx's `BadgedBox` places it: a dot
    /// against the top end corner of its anchor, a label starting 12 points before the anchor's end and overlapping
    /// it by 14 from its top edge, which puts its top 2 points above.
    #[must_use]
    pub fn badge_origin(self) -> (Points, Points) {
        let (x, y) = self.origin();
        let (width, _) = anchor();
        let (horizontal, vertical) = if self.label().is_some() {
            (LABEL_HORIZONTAL_OFFSET, LABEL_VERTICAL_OFFSET)
        } else {
            (DOT_OFFSET, DOT_OFFSET)
        };
        (
            Points(x.0 + width.0 - horizontal.0),
            Points(y.0 + vertical.0 - self.badge_height().0),
        )
    }

    /// The badge's height: a dot's size, or the least height of a label's container, whatever the label.
    #[must_use]
    pub fn badge_height(self) -> Points {
        if self.label().is_some() {
            badge_tokens::LARGE_SIZE
        } else {
            badge_tokens::SIZE
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    /// The vendored `Badge.kt`, at the pinned commit (ADR 0012).
    const BADGE_KT: &str =
        include_str!("../../../reference/androidx-material3-components/Badge.kt");

    /// Whether `Badge.kt` holds `lines`, each trimmed, one after the other.
    fn cites(lines: &[&str]) -> bool {
        let file: Vec<&str> = BADGE_KT.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn the_offsets_are_the_ones_badge_kt_writes() {
        assert!(cites(&["internal val BadgeOffset", "get() = 6.dp"]));
        assert_eq!(DOT_OFFSET, Points(6.0));
        assert!(cites(&[
            "internal val BadgeWithContentHorizontalOffset",
            "get() = 12.dp"
        ]));
        assert_eq!(LABEL_HORIZONTAL_OFFSET, Points(12.0));
        assert!(cites(&[
            "internal val BadgeWithContentVerticalOffset",
            "get() = 14.dp"
        ]));
        assert_eq!(LABEL_VERTICAL_OFFSET, Points(14.0));
        // A value that is not there is not found.
        assert!(!cites(&["internal val BadgeOffset", "get() = 7.dp"]));
    }

    #[test]
    fn every_badge_is_once_in_the_catalog() {
        let cells: Vec<BadgeCell> = BadgeCell::all().collect();
        assert_eq!(cells.len(), 5);
        assert_eq!(cells.iter().collect::<HashSet<_>>().len(), cells.len());
        assert_eq!(
            cells
                .iter()
                .map(|c| c.test_id())
                .collect::<HashSet<_>>()
                .len(),
            cells.len()
        );
        assert_eq!(
            cells
                .iter()
                .map(|c| c.origin())
                .collect::<HashSet<_>>()
                .len(),
            cells.len()
        );
        // The dot has no label; the others are different ones.
        let labels: Vec<Option<&str>> = cells.iter().map(|c| c.label()).collect();
        assert_eq!(
            labels,
            [None, Some("1"), Some("9"), Some("99+"), Some("Sale 50%")]
        );
    }

    #[test]
    fn the_badges_are_in_a_row_below_the_icon_buttons() {
        // The icon buttons' grid ends at the 24 points of margin and 1,760 of rows and gaps below its top, the
        // buttons' 880 above it: the badges' row starts at 2,664.
        assert_eq!(grid::badge_top(), Points(2664.0));
        let origins: Vec<(Points, Points)> = BadgeCell::all().map(BadgeCell::origin).collect();
        assert_eq!(
            origins,
            [
                (Points(24.0), Points(2664.0)),
                (Points(224.0), Points(2664.0)),
                (Points(424.0), Points(2664.0)),
                (Points(624.0), Points(2664.0)),
                (Points(824.0), Points(2664.0)),
            ]
        );
        // The anchor: the small uniform icon button, 40 points on each side.
        assert_eq!(anchor(), (Points(40.0), Points(40.0)));
    }

    #[test]
    fn a_dot_is_against_the_top_end_corner_and_a_label_overlaps_the_top_edge() {
        // A dot: 6 points from the anchor's end, so [34, 40], at the anchor's top; its height is its token's, 6.
        assert_eq!(
            BadgeCell::Dot.badge_origin(),
            (Points(24.0 + 34.0), Points(2664.0))
        );
        assert_eq!(BadgeCell::Dot.badge_height(), Points(6.0));
        // A label: 12 points before the end, so 28 from the anchor's left; 14 below its top, so 16 - 14 = 2 above.
        assert_eq!(
            BadgeCell::One.badge_origin(),
            (Points(224.0 + 28.0), Points(2664.0 - 2.0))
        );
        assert_eq!(
            BadgeCell::LongText.badge_origin(),
            (Points(824.0 + 28.0), Points(2664.0 - 2.0))
        );
        for cell in [
            BadgeCell::One,
            BadgeCell::Nine,
            BadgeCell::NinetyNinePlus,
            BadgeCell::LongText,
        ] {
            assert_eq!(cell.badge_height(), Points(16.0), "{cell:?}");
        }
    }

    #[test]
    fn ids_and_labels_name_the_badge() {
        let ids: Vec<&str> = BadgeCell::all().map(BadgeCell::test_id).collect();
        assert_eq!(
            ids,
            [
                "badge-dot",
                "badge-1",
                "badge-9",
                "badge-99-plus",
                "badge-long-text"
            ]
        );
        assert_eq!(BadgeCell::default(), BadgeCell::Dot);
    }
}
