//! Where the catalog puts each row of tabs, so that the visual checks know where to read its pixels.
//!
//! The rows sit on a grid below the navigation bars': one row of cells per kind of row (primary, secondary), one column
//! per content (a label, an icon, an icon and a label). Each row holds the same three tabs: a selected one, an enabled
//! one and a disabled one, so that every color of a tab is on screen. Each row's top left corner is at a fixed place.

use nsh_core::geometry::Points;
use nsh_m3e::navigation_item::NavigationItemLabel;
use nsh_m3e::tokens::androidx::primary_navigation_tab_tokens;

use crate::grid::{MARGIN, ROW_GAP};
use crate::navigation;

/// The kind of a row of tabs, in the order of the rows.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum RowKind {
    /// `PrimaryTabRow`: the indicator is a short bar under the content of the selected tab.
    #[default]
    Primary,
    /// `SecondaryTabRow`: the indicator is as wide as the selected tab.
    Secondary,
}

/// Every kind of row, in the order of the rows.
pub const KINDS: [RowKind; 2] = [RowKind::Primary, RowKind::Secondary];

/// What the tabs of a row show, in the order of the columns.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Content {
    /// A label.
    #[default]
    Label,
    /// An icon; the label is hidden, and still the tab's name for screen readers.
    Icon,
    /// An icon before a label.
    Both,
}

/// Every content, in the order of the columns.
pub const CONTENTS: [Content; 3] = [Content::Label, Content::Icon, Content::Both];

impl Content {
    /// Whether the tabs have an icon.
    #[must_use]
    pub fn has_icon(self) -> bool {
        self != Self::Label
    }

    /// Whether the tabs show their labels.
    #[must_use]
    pub fn label_visibility(self) -> NavigationItemLabel {
        match self {
            Self::Icon => NavigationItemLabel::Hidden,
            _ => NavigationItemLabel::Shown,
        }
    }
}

/// The state a tab of a row is in.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum TabState {
    /// It is the current tab.
    #[default]
    Selected,
    /// It is enabled and not the current tab.
    Unselected,
    /// It takes no presses.
    Disabled,
}

impl TabState {
    /// Its label, which the row shows (or not) and screen readers say, and the test scripts name (`label:Albums`).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Unselected => "Albums",
            Self::Disabled => "Artists",
            // `TabState` is non-exhaustive; the default state is the selected one.
            _ => "Songs",
        }
    }
}

/// The tabs of every row, in order.
pub const TABS: [TabState; 3] = [TabState::Selected, TabState::Unselected, TabState::Disabled];

/// How many tabs a row holds, as the number the places of the tabs are computed with.
const TAB_COUNT: f32 = 3.0;

/// The width of a row. A layout value of the catalog, not of Material: three tabs of 160 points, each wider than its
/// widest content (an icon, a label and the padding around them: 16 + 24 + 8 + about 50 + 16).
pub const ROW_WIDTH: Points = Points(480.0);

/// The width of a column: more than a row. A layout value of the catalog, not of Material.
pub const COLUMN_WIDTH: Points = Points(640.0);

/// The height of a row: the tabs' token, as every tab of the catalog fits in it.
pub const ROW_HEIGHT: Points = primary_navigation_tab_tokens::CONTAINER_HEIGHT;

/// The top of the rows' grid: below the navigation bars and their last gap.
#[must_use]
pub fn top() -> Points {
    navigation::bottom()
}

/// The bottom of the rows' grid: the last row and its gap end there.
#[must_use]
pub fn bottom() -> Points {
    let mut bottom = top().0;
    for _ in KINDS {
        bottom += ROW_HEIGHT.0 + ROW_GAP.0;
    }
    Points(bottom)
}

/// One row of tabs of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TabCell {
    /// Its kind.
    pub kind: RowKind,
    /// What its tabs show.
    pub content: Content,
}

impl TabCell {
    /// The row of this kind with tabs showing this content.
    #[must_use]
    pub const fn new(kind: RowKind, content: Content) -> Self {
        Self { kind, content }
    }

    /// Every row of the catalog, kind by kind.
    pub fn all() -> impl Iterator<Item = Self> {
        KINDS.into_iter().flat_map(|kind| {
            CONTENTS
                .into_iter()
                .map(move |content| Self::new(kind, content))
        })
    }

    /// The test id of the view around it, which the test scripts name (`id:tabs-primary-label`).
    #[must_use]
    pub fn test_id(self) -> String {
        let kind = match self.kind {
            RowKind::Secondary => "secondary",
            _ => "primary",
        };
        let content = match self.content {
            Content::Icon => "icon",
            Content::Both => "both",
            _ => "label",
        };
        format!("tabs-{kind}-{content}")
    }

    /// Its width.
    #[must_use]
    pub fn width(self) -> Points {
        ROW_WIDTH
    }

    /// Its height: the tabs' token.
    #[must_use]
    pub fn height(self) -> Points {
        ROW_HEIGHT
    }

    /// The top left corner of its container in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let mut x = MARGIN.0;
        for _ in 0..position(&CONTENTS, self.content) {
            x += COLUMN_WIDTH.0;
        }
        let mut y = top().0;
        for kind in KINDS {
            if kind == self.kind {
                return (Points(x), Points(y));
            }
            y += ROW_HEIGHT.0 + ROW_GAP.0;
        }
        (Points(x), Points(y))
    }

    /// How wide each tab is: a third of the row.
    #[must_use]
    pub fn tab_width(self) -> Points {
        Points(self.width().0 / TAB_COUNT)
    }

    /// The left edge of the tab in `state`, in the viewport, in points.
    #[must_use]
    pub fn tab_left(self, state: TabState) -> Points {
        let (x, _) = self.origin();
        let mut left = x.0;
        for tab in TABS {
            if tab == state {
                break;
            }
            left += self.tab_width().0;
        }
        Points(left)
    }

    /// The middle of the tab in `state`, in the viewport, in points.
    #[must_use]
    pub fn tab_center(self, state: TabState) -> (Points, Points) {
        let (_, y) = self.origin();
        (
            Points(self.tab_left(state).0 + self.tab_width().0 / 2.0),
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
    fn every_row_is_once_in_the_catalog() {
        let cells: Vec<TabCell> = TabCell::all().collect();
        assert_eq!(cells.len(), KINDS.len() * CONTENTS.len());
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
    fn the_tab_count_is_the_tabs() {
        let count: f32 = TABS.iter().map(|_| 1.0).sum();
        assert_eq!(count.to_bits(), TAB_COUNT.to_bits());
        let states: HashSet<TabState> = TABS.into_iter().collect();
        assert_eq!(states.len(), TABS.len());
        let labels: HashSet<&str> = TABS.into_iter().map(TabState::label).collect();
        assert_eq!(labels.len(), TABS.len());
    }

    #[test]
    fn the_rows_and_columns_are_where_the_layout_says() {
        let first = TabCell::default();
        // Below the navigation bars: they end at 2,824 points, the last row's gap included.
        assert_eq!(top(), Points(2824.0));
        assert_eq!(first.origin(), (Points(24.0), Points(2824.0)));
        let icon = TabCell {
            content: Content::Icon,
            ..first
        };
        assert_eq!(icon.origin(), (Points(664.0), Points(2824.0)));
        let both = TabCell {
            content: Content::Both,
            ..first
        };
        assert_eq!(both.origin(), (Points(1304.0), Points(2824.0)));
        // Secondary is the second row: below the first row (48 points) and the gap.
        let secondary = TabCell {
            kind: RowKind::Secondary,
            ..first
        };
        assert_eq!(secondary.origin(), (Points(24.0), Points(2888.0)));
        assert_eq!(bottom(), Points(2952.0));
    }

    #[test]
    fn the_rows_fit_their_columns_and_the_viewport() {
        let (width, height) = grid::viewport();
        for cell in TabCell::all() {
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
    fn the_tabs_are_a_third_of_the_row_each() {
        let row = TabCell::default();
        assert_eq!(row.tab_width(), Points(160.0));
        let centers: Vec<f32> = TABS.into_iter().map(|s| row.tab_center(s).0.0).collect();
        assert_eq!(centers, [104.0, 264.0, 424.0]);
        // Every tab is in the middle of the row's height (2,824 and half of 48 points).
        assert_eq!(row.tab_center(TabState::Selected).1, Points(2848.0));
        assert_eq!(row.tab_left(TabState::Disabled), Points(344.0));
    }

    #[test]
    fn the_icon_column_hides_the_label_and_the_label_column_has_no_icon() {
        assert!(!Content::Label.has_icon());
        assert!(Content::Icon.has_icon());
        assert!(Content::Both.has_icon());
        assert_eq!(
            Content::Icon.label_visibility(),
            NavigationItemLabel::Hidden
        );
        assert_eq!(Content::Both.label_visibility(), NavigationItemLabel::Shown);
        assert_eq!(
            Content::Label.label_visibility(),
            NavigationItemLabel::Shown
        );
    }
}
