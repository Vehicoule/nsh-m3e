//! The list items of the catalog, and where it puts them, so that the visual checks know where to read their pixels.
//!
//! The list items sit on a grid below the FABs' row (see [`crate::fab`]), the last block of the page: one column per
//! kind of item, one row per number of lines. Every item has a leading and a trailing icon, and is [`WIDTH`] points
//! wide whatever its text. [`viewport`] is the page's size with this grid.
//!
//! The grid's top is where the FABs' row ends, a gap below it: `button.test.tsx`, `switch.test.tsx`,
//! `checkbox.test.tsx` and `fab.test.tsx` open the page at viewports that stop above it; `list_item.test.tsx` opens it
//! at [`viewport`].

use nsh_core::geometry::Points;
use nsh_core::prelude::*;
use nsh_m3e::list_item::{clickable_list_item, list_item};
use nsh_m3e::tokens::androidx::list_tokens;

use crate::grid::{self, MARGIN, ROW_GAP};
use crate::{ICON, fab};

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

/// The space between the FABs' row and the list items' grid: past the reach of the highest FAB's shadow, which is 20
/// points below it (ADR 0018). A layout value of the catalog, not of Material.
pub const GAP: Points = Points(48.0);

/// The top of the list items' grid: where the FABs' row ends, a gap below it (the bottom margin of [`fab::viewport`]
/// starts where the row and its row gap end).
#[must_use]
pub fn top() -> Points {
    Points(fab::viewport().1.0 - MARGIN.0 + GAP.0)
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

/// The size of the viewport the catalog is opened in to show the list items, width and height: the page's width, as
/// the grid is narrower than the buttons', and the height of the page with the grid and the margin below it.
#[must_use]
pub fn viewport() -> (Points, Points) {
    (grid::viewport().0, Points(bottom().0 + MARGIN.0))
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

/// `page` with every list item of the catalog at its place, each in a view with its test id and the width the catalog
/// gives it.
#[must_use]
pub fn place(page: View) -> View {
    ListCell::all().fold(page, |page, cell| {
        let (x, y) = cell.origin();
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .w(WIDTH)
                .test_id(&cell.test_id())
                .child(item(cell)),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex, PoisonError};

    use nsh_core::geometry::Bounds;
    use nsh_core::{AccessibilityRole, MountItem, NodeId};
    use nsh_engine::{EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::Catalog;

    /// The list item test script, `list_item.test.tsx`, which opens the catalog at [`viewport`] and takes the
    /// screenshots `visual-checks` reads.
    const SCRIPT: &str = include_str!("../list_item.test.tsx");

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
        // The FABs' row ends at the margin before the FABs' viewport does; the list items start a gap below it.
        assert_eq!(top(), Points(fab::viewport().1.0 - 24.0 + 48.0));
        assert_eq!(top(), Points(3032.0));
        let first = ListCell::new(Lines::One, Kind::Plain);
        assert_eq!(first.origin(), (Points(24.0), Points(3032.0)));
        // Clickable is the second column, 400 points on.
        let clickable = ListCell::new(Lines::One, Kind::Clickable);
        assert_eq!(clickable.origin(), (Points(424.0), Points(3032.0)));
        let disabled = ListCell::new(Lines::One, Kind::Disabled);
        assert_eq!(disabled.origin(), (Points(1224.0), Points(3032.0)));
        // The second row is below the one-line row (56 points and the gap), the third below the two-line one.
        let two = ListCell::new(Lines::Two, Kind::Plain);
        assert_eq!(two.origin(), (Points(24.0), Points(3104.0)));
        let three = ListCell::new(Lines::Three, Kind::Selected);
        assert_eq!(three.origin(), (Points(824.0), Points(3192.0)));
        // The last row and its gap end at `bottom`.
        assert_eq!(bottom(), Points(3296.0));
    }

    #[test]
    fn the_viewport_holds_the_list_items_and_the_grids_above() {
        let (width, height) = viewport();
        // The page is as wide as the buttons' grid; the list items' grid is three rows and their gaps, and the margin
        // below it.
        assert_eq!((width, height), (Points(2608.0), Points(3320.0)));
        assert_eq!(width, grid::viewport().0);
        assert_eq!(height.0, bottom().0 + MARGIN.0);
        const { assert!(WIDTH.0 < COLUMN_WIDTH.0) };
        let mut last = Points(0.0);
        for cell in ListCell::all() {
            let (x, y) = cell.origin();
            assert!(x.0 >= MARGIN.0, "{cell:?}");
            assert!(x.0 + WIDTH.0 <= width.0 - MARGIN.0, "{cell:?}");
            assert!(y.0 >= top().0, "{cell:?}");
            assert!(y.0 + cell.height().0 <= height.0 - MARGIN.0, "{cell:?}");
            last = Points(last.0.max(y.0 + cell.height().0));
        }
        // The last row and its gap end at the margin.
        assert_eq!(last.0 + ROW_GAP.0, height.0 - MARGIN.0);
        // A gap or more below every FAB: the highest FAB's shadow reaches 20 points below it (ADR 0018).
        for cell in fab::FabCell::all() {
            assert!(cell.origin().1.0 + cell.frame().1.0 + GAP.0 <= top().0);
        }
        // No list item takes the place of a checkbox, a switch, a button or an icon button.
        for cell in crate::checkbox::CheckboxCell::all() {
            assert!(cell.origin().1.0 + crate::checkbox::SIDE.0 <= top().0);
        }
        for cell in grid::Cell::all() {
            assert!(cell.origin().1.0 + grid::height(cell.size).0 <= top().0);
        }
        for cell in grid::IconCell::all() {
            assert!(cell.origin().1.0 + cell.frame().1.0 <= top().0);
        }
    }

    #[test]
    fn the_test_script_opens_the_catalog_at_the_viewport_and_takes_the_shots_visual_checks_reads() {
        let (width, height) = viewport();
        assert!(
            SCRIPT.contains(&format!(
                "const VIEWPORT = {{ width: {}, height: {} }};",
                width.0, height.0
            )),
            "the script's viewport is not {width:?} x {height:?}"
        );
        for line in [
            "await app.screenshot(`target/visual/list-${scheme}-rest.png`);",
            "await app.hover(`id:list-two-${kind}`);",
            "`target/visual/list-${scheme}-hover-${kind}.png`",
            "for (const scheme of [\"light\", \"dark\"] as const) {",
            "const KINDS = [\"clickable\", \"selected\"];",
        ] {
            assert!(SCRIPT.contains(line), "the script lacks: {line}");
        }
        // The items it hovers are the grid's two-line clickable and selected ones, in the grid's names.
        let ids: Vec<String> = [Kind::Clickable, Kind::Selected]
            .into_iter()
            .map(|kind| ListCell::new(Lines::Two, kind).test_id())
            .collect();
        assert_eq!(ids, ["list-two-clickable", "list-two-selected"]);
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

    /// Everything the catalog mounts on its first screen, in the order the engine sent it.
    fn mounted_items() -> Vec<MountItem> {
        let mut app = Headless::start(EngineConfig::new(Root::of::<Catalog>())).unwrap();
        let items: Arc<Mutex<Vec<MountItem>>> = Arc::default();
        let sink = Arc::clone(&items);
        app.observe(move |_, batch| {
            sink.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .extend(batch.items.iter().cloned());
        });
        app.wait_for("Filled").unwrap();
        let mounted = items.lock().unwrap_or_else(PoisonError::into_inner);
        mounted.clone()
    }

    /// Every list item is at its place on the list items' grid, inside a view with its test id and the width the
    /// catalog gives it; the item has its kind's role and state, the height of its lines, and the texts its lines
    /// show, and no others.
    #[test]
    fn every_list_item_is_where_the_grid_says() {
        let items = mounted_items();

        let mut ids = HashMap::new();
        let mut frames: HashMap<NodeId, Bounds<Points>> = HashMap::new();
        let mut roles = HashMap::new();
        let mut parents = HashMap::new();
        let mut texts = Vec::new();
        for item in &items {
            match item {
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        ids.insert(test_id.to_string(), *id);
                    }
                    roles.insert(
                        *id,
                        (props.accessibility.role, props.accessibility.state.disabled),
                    );
                }
                MountItem::Frame { id, frame } => {
                    frames.insert(*id, *frame);
                }
                MountItem::Insert { parent, child, .. } => {
                    parents.insert(*child, *parent);
                }
                MountItem::Text { id, text, .. } => {
                    texts.push((*id, text.text.clone()));
                }
                _ => {}
            }
        }
        let child_of = |id: NodeId| {
            parents
                .iter()
                .find(|(_, parent)| **parent == id)
                .map(|(child, _)| *child)
                .unwrap()
        };
        // The wrapper a node is inside of, if it is inside one of the list items'.
        let wrapper_of = |mut id: NodeId| {
            let wrappers: HashMap<NodeId, ListCell> = ListCell::all()
                .map(|cell| (ids[&cell.test_id()], cell))
                .collect();
            for _ in 0..parents.len() {
                if let Some(cell) = wrappers.get(&id) {
                    return Some(*cell);
                }
                id = *parents.get(&id)?;
            }
            None
        };
        for cell in ListCell::all() {
            let id = ids[&cell.test_id()];
            let (x, y) = cell.origin();
            let frame = frames[&id];
            assert_eq!((frame.origin.x, frame.origin.y), (x, y), "{cell:?}");
            assert_eq!(frame.size.width, WIDTH, "{cell:?}");
            let item = child_of(id);
            assert_eq!(frames[&item].size.height, cell.height(), "{cell:?}");
            assert_eq!(frames[&item].size.width, WIDTH, "{cell:?}");
            // The views of the item with a role: the plain item has none, the others have one.
            let with_role: Vec<(AccessibilityRole, bool)> = roles
                .iter()
                .filter(|(node, (role, _))| {
                    *role != AccessibilityRole::default() && wrapper_of(**node) == Some(cell)
                })
                .map(|(_, role)| *role)
                .collect();
            let expected = match cell.kind {
                Kind::Clickable => vec![(AccessibilityRole::Button, false)],
                Kind::Disabled => vec![(AccessibilityRole::Button, true)],
                Kind::Selected => vec![(AccessibilityRole::Radio, false)],
                _ => Vec::new(),
            };
            assert_eq!(with_role, expected, "{cell:?}");
            let mut shown: Vec<&str> = texts
                .iter()
                .filter(|(text, _)| wrapper_of(*text) == Some(cell))
                .map(|(_, text)| text.as_str())
                .collect();
            shown.sort_unstable();
            let mut expected = vec![cell.label()];
            if cell.has_overline() {
                expected.push("Overline");
            }
            if cell.has_supporting() {
                expected.push("Supporting text");
            }
            expected.sort_unstable();
            assert_eq!(shown, expected, "{cell:?}");
        }
        // Every selected item is a radio, and nothing else of the catalog is.
        let radios = ListCell::all()
            .filter(|cell| matches!(cell.kind, Kind::Selected))
            .count();
        assert_eq!(
            roles
                .values()
                .filter(|(role, _)| *role == AccessibilityRole::Radio)
                .count(),
            radios
        );
    }
}
