//! The list items of the catalog, and where it puts them, so that the visual checks know where to read their pixels.
//!
//! The list items sit on a third grid, below the icon buttons': one column per kind of item, one row per number of
//! lines. Every item has a leading and a trailing icon, and is [`WIDTH`] points wide whatever its text.

use nsh_core::geometry::Points;
use nsh_core::prelude::*;
use nsh_m3e::list_item::{clickable_list_item, list_item};
use nsh_m3e::tokens::androidx::list_tokens;

use crate::ICON;
use crate::grid::{self, MARGIN, ROW_GAP};

/// How many lines an item shows, in the order of the rows: the headline alone, with supporting text, and with an
/// overline and supporting text.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Lines {
    /// The headline.
    #[default]
    One,
    /// The headline and the supporting text.
    Two,
    /// The overline, the headline and the supporting text.
    Three,
}

/// Every number of lines, in the order of the rows.
pub const LINES: [Lines; 3] = [Lines::One, Lines::Two, Lines::Three];

/// What an item is, in the order of the columns.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A list item that takes no press: androidx's `ListItem(modifier, …)`.
    #[default]
    Plain,
    /// A list item that is pressed: `ListItem(onClick, …)`.
    Clickable,
    /// A pressed list item that is the selected one of its list: `ListItem(selected = true, onClick, …)`.
    Selected,
    /// A pressed list item that takes no press, hover or focus, with the disabled colors.
    Disabled,
}

/// Every kind of item, in the order of the columns.
pub const KINDS: [Kind; 4] = [Kind::Plain, Kind::Clickable, Kind::Selected, Kind::Disabled];

/// The width of every item. A layout value of the catalog, not of Material: the catalog is a test page.
pub const WIDTH: Points = Points(360.0);

/// The width of a column: an item and the space before the next one. A layout value of the catalog, not of Material.
pub const COLUMN_WIDTH: Points = Points(400.0);

/// The height of an item with `lines`: its container height token, as the list item takes it.
#[must_use]
pub fn height(lines: Lines) -> Points {
    match lines {
        Lines::Two => list_tokens::ITEM_TWO_LINE_CONTAINER_HEIGHT,
        Lines::Three => list_tokens::ITEM_THREE_LINE_CONTAINER_HEIGHT,
        // `Lines` is non-exhaustive: a count added later is drawn as one line until the catalog shows it.
        _ => list_tokens::ITEM_ONE_LINE_CONTAINER_HEIGHT,
    }
}

/// The top of the list items' grid: below the icon buttons' rows, each followed by its gap.
#[must_use]
pub fn top() -> Points {
    grid::icon_bottom()
}

/// The bottom of the list items' grid: below its rows, each followed by its gap.
#[must_use]
pub fn bottom() -> Points {
    let mut bottom = top().0;
    for lines in LINES {
        bottom += height(lines).0 + ROW_GAP.0;
    }
    Points(bottom)
}

/// One list item of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ListCell {
    /// How many lines it shows.
    pub lines: Lines,
    /// What it is.
    pub kind: Kind,
}

impl ListCell {
    /// The list item of this number of lines and kind.
    #[must_use]
    pub const fn new(lines: Lines, kind: Kind) -> Self {
        Self { lines, kind }
    }

    /// Every list item of the catalog, column by column.
    pub fn all() -> impl Iterator<Item = Self> {
        KINDS
            .into_iter()
            .flat_map(|kind| LINES.into_iter().map(move |lines| Self::new(lines, kind)))
    }

    /// Its headline: the name of its kind.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self.kind {
            Kind::Clickable => "Clickable",
            Kind::Selected => "Selected",
            Kind::Disabled => "Disabled",
            // `Kind` is non-exhaustive; the default kind is the plain one.
            _ => "Plain",
        }
    }

    /// Whether it has an overline.
    #[must_use]
    pub fn has_overline(self) -> bool {
        self.lines == Lines::Three
    }

    /// Whether it has supporting text.
    #[must_use]
    pub fn has_supporting(self) -> bool {
        matches!(self.lines, Lines::Two | Lines::Three)
    }

    /// Whether it takes presses: a clickable item that is not disabled.
    #[must_use]
    pub fn takes_presses(self) -> bool {
        matches!(self.kind, Kind::Clickable | Kind::Selected)
    }

    /// Its height, in points.
    #[must_use]
    pub fn height(self) -> Points {
        height(self.lines)
    }

    /// The test id of the view around it, which the test scripts name (`id:list-two-clickable`).
    #[must_use]
    pub fn test_id(self) -> String {
        let lines = match self.lines {
            Lines::Two => "two",
            Lines::Three => "three",
            _ => "one",
        };
        format!("list-{lines}-{}", self.label().to_lowercase())
    }

    /// The top left corner of its container in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let mut x = MARGIN.0;
        for kind in KINDS {
            if kind == self.kind {
                break;
            }
            x += COLUMN_WIDTH.0;
        }
        let mut y = top().0;
        for lines in LINES {
            if lines == self.lines {
                break;
            }
            y += height(lines).0 + ROW_GAP.0;
        }
        (Points(x), Points(y))
    }
}

/// The list item of `cell`, with the catalog's icon on each side.
#[must_use]
pub fn item(cell: ListCell) -> AnyElement {
    if cell.kind == Kind::Plain {
        let mut item = list_item()
            .headline(cell.label())
            .leading_icon(ICON)
            .trailing_icon(ICON);
        if cell.has_overline() {
            item = item.overline("Overline");
        }
        if cell.has_supporting() {
            item = item.supporting("Supporting text");
        }
        return item.into_element();
    }
    let mut item = clickable_list_item()
        .headline(cell.label())
        .leading_icon(ICON)
        .trailing_icon(ICON)
        .disabled(cell.kind == Kind::Disabled);
    if cell.has_overline() {
        item = item.overline("Overline");
    }
    if cell.has_supporting() {
        item = item.supporting("Supporting text");
    }
    if cell.kind == Kind::Selected {
        item = item.selected(true);
    }
    item.into_element()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn every_list_item_is_once_in_the_catalog() {
        let cells: Vec<ListCell> = ListCell::all().collect();
        assert_eq!(cells.len(), KINDS.len() * LINES.len());
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
    fn the_heights_are_the_container_height_tokens() {
        let heights: Vec<f32> = LINES.into_iter().map(|lines| height(lines).0).collect();
        assert_eq!(heights, [56.0, 72.0, 88.0]);
        assert_eq!(
            ListCell::new(Lines::Two, Kind::Selected).height(),
            Points(72.0)
        );
    }

    #[test]
    fn the_rows_and_columns_are_where_the_layout_says() {
        // Below the icon buttons: the 2,664 points their grid ends at, each row followed by its gap.
        assert_eq!(top(), Points(2664.0));
        let first = ListCell::new(Lines::One, Kind::Plain);
        assert_eq!(first.origin(), (Points(24.0), Points(2664.0)));
        // Clickable is the second column, 400 points on.
        let clickable = ListCell::new(Lines::One, Kind::Clickable);
        assert_eq!(clickable.origin(), (Points(424.0), Points(2664.0)));
        let disabled = ListCell::new(Lines::One, Kind::Disabled);
        assert_eq!(disabled.origin(), (Points(1224.0), Points(2664.0)));
        // The second row is below the one-line row (56 points and the gap), the third below the two-line one.
        let two = ListCell::new(Lines::Two, Kind::Plain);
        assert_eq!(two.origin(), (Points(24.0), Points(2736.0)));
        let three = ListCell::new(Lines::Three, Kind::Selected);
        assert_eq!(three.origin(), (Points(824.0), Points(2824.0)));
        // The last row and its gap end at `bottom`.
        assert_eq!(bottom(), Points(2928.0));
    }

    #[test]
    fn the_items_fit_their_columns() {
        const { assert!(WIDTH.0 < COLUMN_WIDTH.0) };
        let (width, height) = grid::viewport();
        for cell in ListCell::all() {
            let (x, y) = cell.origin();
            assert!(x.0 + WIDTH.0 <= width.0 - MARGIN.0, "{cell:?}");
            assert!(y.0 + cell.height().0 <= height.0 - MARGIN.0, "{cell:?}");
        }
    }

    #[test]
    fn labels_and_ids_name_the_list_item() {
        let labels: Vec<&str> = KINDS
            .into_iter()
            .map(|kind| ListCell::new(Lines::One, kind).label())
            .collect();
        assert_eq!(labels, ["Plain", "Clickable", "Selected", "Disabled"]);
        let ids: Vec<String> = LINES
            .into_iter()
            .map(|lines| ListCell::new(lines, Kind::Clickable).test_id())
            .collect();
        assert_eq!(
            ids,
            [
                "list-one-clickable",
                "list-two-clickable",
                "list-three-clickable"
            ]
        );
        assert_eq!(
            ListCell::new(Lines::Three, Kind::Disabled).test_id(),
            "list-three-disabled"
        );
    }

    #[test]
    fn the_lines_say_which_texts_an_item_has() {
        for (lines, overline, supporting) in [
            (Lines::One, false, false),
            (Lines::Two, false, true),
            (Lines::Three, true, true),
        ] {
            let cell = ListCell::new(lines, Kind::Plain);
            assert_eq!(cell.has_overline(), overline, "{lines:?}");
            assert_eq!(cell.has_supporting(), supporting, "{lines:?}");
        }
    }

    #[test]
    fn only_the_clickable_and_selected_items_take_presses() {
        let takes: Vec<bool> = KINDS
            .into_iter()
            .map(|kind| ListCell::new(Lines::One, kind).takes_presses())
            .collect();
        assert_eq!(takes, [false, true, true, false]);
    }
}
