//! Where the catalog puts each navigation bar, so that the visual checks know where to read its pixels.
//!
//! The bars sit on a grid below the icon buttons': one row per arrangement, one column per icon position and label
//! visibility. Each bar holds the same three items: a selected one, an enabled one and a disabled one, so that every
//! color of an item is on screen. Each bar's top left corner is at a fixed place.

use nsh_core::geometry::Points;
use nsh_m3e::navigation_bar::NavigationBarArrangement;
use nsh_m3e::navigation_item::{NavigationItemIconPosition, NavigationItemLabel};
use nsh_m3e::tokens::androidx::navigation_bar_tokens;

use crate::grid::{MARGIN, ROW_GAP};

/// Every arrangement, in the order of the rows.
pub const ARRANGEMENTS: [NavigationBarArrangement; 2] = [
    NavigationBarArrangement::EqualWeight,
    NavigationBarArrangement::Centered,
];

/// Every icon position, in the order of the columns.
pub const ICON_POSITIONS: [NavigationItemIconPosition; 2] = [
    NavigationItemIconPosition::Top,
    NavigationItemIconPosition::Start,
];

/// Every label visibility, in the order of the columns within an icon position.
pub const LABELS: [NavigationItemLabel; 2] =
    [NavigationItemLabel::Shown, NavigationItemLabel::Hidden];

/// The state an item of a bar is in.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ItemState {
    /// It is the current destination.
    #[default]
    Selected,
    /// It is enabled and not the current destination.
    Unselected,
    /// It takes no presses.
    Disabled,
}

impl ItemState {
    /// Its label, which the bar shows and screen readers say, and the test scripts name (`label:Search`).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Unselected => "Search",
            Self::Disabled => "Library",
            // `ItemState` is non-exhaustive; the default state is the selected one.
            _ => "Home",
        }
    }
}

/// The items of every bar, in order.
pub const ITEMS: [ItemState; 3] = [
    ItemState::Selected,
    ItemState::Unselected,
    ItemState::Disabled,
];

/// How many items a bar holds, as the number the places of the items are computed with.
const ITEM_COUNT: f32 = 3.0;

/// The width of a bar whose items share it equally. A layout value of the catalog, not of Material: three items of
/// 160 points, each wider than its widest content (the pill, 56 points, and the label).
pub const EQUAL_WEIGHT_WIDTH: Points = Points(480.0);

/// The width of a bar whose items are centered. A layout value of the catalog, not of Material: its group of three
/// items is 60 % of it (androidx's `10 · (items + 3)` percent, `ShortNavigationBar.kt`, lines 539-619), so 120 points
/// for each item, again wider than its content.
pub const CENTERED_WIDTH: Points = Points(600.0);

/// The width of a column: more than the widest bar, the centered one. A layout value of the catalog, not of Material.
pub const COLUMN_WIDTH: Points = Points(640.0);

/// The height of a bar: its `ContainerHeight` token. Every item of the catalog is as tall as that or less, with the
/// system bars' insets at zero.
pub const BAR_HEIGHT: Points = navigation_bar_tokens::CONTAINER_HEIGHT;

/// The top of the bars' grid: below the switches' rows and their last gap, which are below the icon buttons'.
#[must_use]
pub fn top() -> Points {
    Points(crate::switch::top().0 + crate::switch::block_height().0)
}

/// The bottom of the bars' grid: the last row and its gap end there.
#[must_use]
pub fn bottom() -> Points {
    let mut bottom = top().0;
    for _ in ARRANGEMENTS {
        bottom += BAR_HEIGHT.0 + ROW_GAP.0;
    }
    Points(bottom)
}

/// One navigation bar of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NavCell {
    /// How its items share it.
    pub arrangement: NavigationBarArrangement,
    /// Where its items' icons sit.
    pub icon_position: NavigationItemIconPosition,
    /// Whether its items show their labels.
    pub label: NavigationItemLabel,
}

impl NavCell {
    /// The bar with this arrangement, icon position and label visibility.
    #[must_use]
    pub const fn new(
        arrangement: NavigationBarArrangement,
        icon_position: NavigationItemIconPosition,
        label: NavigationItemLabel,
    ) -> Self {
        Self {
            arrangement,
            icon_position,
            label,
        }
    }

    /// Every bar of the catalog, row by row.
    pub fn all() -> impl Iterator<Item = Self> {
        ARRANGEMENTS.into_iter().flat_map(|arrangement| {
            ICON_POSITIONS.into_iter().flat_map(move |icon_position| {
                LABELS
                    .into_iter()
                    .map(move |label| Self::new(arrangement, icon_position, label))
            })
        })
    }

    /// The test id of the view around it, which the test scripts name (`id:navigation-equal-top-shown`).
    #[must_use]
    pub fn test_id(self) -> String {
        let arrangement = match self.arrangement {
            NavigationBarArrangement::Centered => "centered",
            _ => "equal",
        };
        let position = match self.icon_position {
            NavigationItemIconPosition::Start => "start",
            _ => "top",
        };
        let label = match self.label {
            NavigationItemLabel::Hidden => "hidden",
            _ => "shown",
        };
        format!("navigation-{arrangement}-{position}-{label}")
    }

    /// Its width.
    #[must_use]
    pub fn width(self) -> Points {
        match self.arrangement {
            NavigationBarArrangement::Centered => CENTERED_WIDTH,
            _ => EQUAL_WEIGHT_WIDTH,
        }
    }

    /// Its height: the bar's token, as every item fits in it.
    #[must_use]
    pub fn height(self) -> Points {
        BAR_HEIGHT
    }

    /// The top left corner of its container in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let column = position(&ICON_POSITIONS, self.icon_position) * LABELS.len()
            + position(&LABELS, self.label);
        let mut x = MARGIN.0;
        for _ in 0..column {
            x += COLUMN_WIDTH.0;
        }
        let mut y = top().0;
        for arrangement in ARRANGEMENTS {
            if arrangement == self.arrangement {
                return (Points(x), Points(y));
            }
            y += BAR_HEIGHT.0 + ROW_GAP.0;
        }
        (Points(x), Points(y))
    }

    /// How wide each item is: a third of the bar when the items share it, and a third of the group when they are
    /// centered (60 % of the bar).
    #[must_use]
    pub fn item_width(self) -> Points {
        match self.arrangement {
            NavigationBarArrangement::Centered => Points(self.width().0 * 0.6 / ITEM_COUNT),
            _ => Points(self.width().0 / ITEM_COUNT),
        }
    }

    /// How far the first item is from the bar's left edge: nothing when the items share the bar, and the margin of
    /// the group (20 % of the bar on each side) when they are centered.
    #[must_use]
    pub fn items_start(self) -> Points {
        match self.arrangement {
            NavigationBarArrangement::Centered => Points(self.width().0 * 0.2),
            _ => Points(0.0),
        }
    }

    /// The middle of the item in `state`, in the viewport, in points.
    #[must_use]
    pub fn item_center(self, state: ItemState) -> (Points, Points) {
        let (x, y) = self.origin();
        let width = self.item_width().0;
        let mut left = x.0 + self.items_start().0;
        for item in ITEMS {
            if item == state {
                break;
            }
            left += width;
        }
        (
            Points(left + width / 2.0),
            Points(y.0 + self.height().0 / 2.0),
        )
    }
}

/// Where `item` is in `items`, or 0 for an item the catalog does not show.
fn position<T: PartialEq + Copy>(items: &[T], item: T) -> usize {
    items.iter().position(|i| *i == item).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::grid;

    #[test]
    fn every_bar_is_once_in_the_catalog() {
        let cells: Vec<NavCell> = NavCell::all().collect();
        assert_eq!(
            cells.len(),
            ARRANGEMENTS.len() * ICON_POSITIONS.len() * LABELS.len()
        );
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
    }

    #[test]
    fn the_item_count_is_the_items() {
        let count: f32 = ITEMS.iter().map(|_| 1.0).sum();
        assert_eq!(count.to_bits(), ITEM_COUNT.to_bits());
        let states: HashSet<ItemState> = ITEMS.into_iter().collect();
        assert_eq!(states.len(), ITEMS.len());
    }

    #[test]
    fn the_rows_and_columns_are_where_the_layout_says() {
        let first = NavCell::default();
        // Below the switches, which are below the icon buttons: the switches end at 2,760 points, the last row's gap
        // included.
        assert_eq!(top(), Points(2760.0));
        assert_eq!(first.origin(), (Points(24.0), Points(2760.0)));
        // Hidden labels are the second column of an icon position, the start icon the third.
        let hidden = NavCell {
            label: NavigationItemLabel::Hidden,
            ..first
        };
        assert_eq!(hidden.origin(), (Points(664.0), Points(2760.0)));
        let start = NavCell {
            icon_position: NavigationItemIconPosition::Start,
            ..first
        };
        assert_eq!(start.origin(), (Points(1304.0), Points(2760.0)));
        // Centered is the second row: below the first bar (64 points) and the gap.
        let centered = NavCell {
            arrangement: NavigationBarArrangement::Centered,
            ..first
        };
        assert_eq!(centered.origin(), (Points(24.0), Points(2840.0)));
        let last = NavCell::new(
            NavigationBarArrangement::Centered,
            NavigationItemIconPosition::Start,
            NavigationItemLabel::Hidden,
        );
        assert_eq!(last.origin(), (Points(1944.0), Points(2840.0)));
        assert_eq!(bottom(), Points(2920.0));
    }

    #[test]
    fn the_bars_fit_their_columns_and_the_viewport() {
        let (width, height) = grid::viewport();
        for cell in NavCell::all() {
            let (x, y) = cell.origin();
            assert!(cell.width().0 < COLUMN_WIDTH.0, "{cell:?}");
            assert!(x.0 + COLUMN_WIDTH.0 <= width.0 - MARGIN.0, "{cell:?}");
            assert!(y.0 >= top().0, "{cell:?}");
            assert!(y.0 + cell.height().0 <= height.0 - MARGIN.0, "{cell:?}");
        }
        // The last row and its gap end at the bottom margin, which is where the viewport ends.
        assert_eq!(bottom().0 + MARGIN.0, height.0);
    }

    #[test]
    fn the_items_are_where_the_arrangement_puts_them() {
        let equal = NavCell::default();
        // A third of 480 points each, from the bar's left edge (24 points in).
        assert_eq!(equal.item_width(), Points(160.0));
        assert_eq!(equal.items_start(), Points(0.0));
        let centers: Vec<f32> = ITEMS
            .into_iter()
            .map(|s| equal.item_center(s).0.0)
            .collect();
        assert_eq!(centers, [104.0, 264.0, 424.0]);
        // Every item is in the middle of the bar's height (2,664 and half of 64 points).
        assert_eq!(equal.item_center(ItemState::Selected).1, Points(2792.0));
        let centered = NavCell {
            arrangement: NavigationBarArrangement::Centered,
            ..equal
        };
        // A group of 360 points, 120 from each edge of 600, three items of 120.
        assert_eq!(centered.item_width(), Points(120.0));
        assert_eq!(centered.items_start(), Points(120.0));
        let centers: Vec<f32> = ITEMS
            .into_iter()
            .map(|s| centered.item_center(s).0.0)
            .collect();
        assert_eq!(centers, [204.0, 324.0, 444.0]);
        assert_eq!(centered.item_center(ItemState::Disabled).1, Points(2872.0));
    }

    #[test]
    fn labels_and_ids_name_the_bar_and_its_items() {
        let cell = NavCell::new(
            NavigationBarArrangement::Centered,
            NavigationItemIconPosition::Start,
            NavigationItemLabel::Hidden,
        );
        assert_eq!(cell.test_id(), "navigation-centered-start-hidden");
        assert_eq!(NavCell::default().test_id(), "navigation-equal-top-shown");
        let ids: Vec<String> = NavCell::all().map(NavCell::test_id).collect();
        assert_eq!(
            ids,
            [
                "navigation-equal-top-shown",
                "navigation-equal-top-hidden",
                "navigation-equal-start-shown",
                "navigation-equal-start-hidden",
                "navigation-centered-top-shown",
                "navigation-centered-top-hidden",
                "navigation-centered-start-shown",
                "navigation-centered-start-hidden",
            ]
        );
        let labels: Vec<&str> = ITEMS.into_iter().map(ItemState::label).collect();
        assert_eq!(labels, ["Home", "Search", "Library"]);
    }

    #[test]
    fn a_bar_is_as_tall_as_its_token() {
        assert_eq!(BAR_HEIGHT, Points(64.0));
        assert_eq!(NavCell::default().height(), BAR_HEIGHT);
    }
}
