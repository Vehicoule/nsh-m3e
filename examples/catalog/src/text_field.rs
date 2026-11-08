//! Where the catalog puts each text field, so that the visual checks know where to read its pixels.
//!
//! The text fields sit below the checkboxes' grid (see [`crate::checkbox`]), in two rows: the filled fields above the
//! outlined ones. Each row has one column per status, first for the plain fields (empty, so that the label rests over
//! the text), then for the rich ones (a value, a leading and a trailing icon, so that the label floats). Every field has
//! supporting text. Each field's top left corner is at a fixed place, whatever its style.
//!
//! The grid's top is where the checkboxes' viewport ends, so this grid is the last block of the page, and [`viewport`]
//! is the page's size with it. `button.test.tsx` and `checkbox.test.tsx` open the page at the viewports that stop above
//! this grid; `text_field.test.tsx` opens it at [`viewport`].

use nsh_core::geometry::Points;
use nsh_core::prelude::*;
use nsh_m3e::text_field::{TextFieldStatus, TextFieldStyle, text_field};

use crate::ICON;
use crate::checkbox;
use crate::grid::{self, COLUMN_WIDTH, MARGIN, ROW_GAP};

/// Every style, in the order of the rows.
pub const STYLES: [TextFieldStyle; 2] = [TextFieldStyle::Filled, TextFieldStyle::Outlined];

/// Every status the catalog shows, in the order of the columns within a kind of content. The read-only statuses look
/// as the enabled and error ones do.
pub const STATUSES: [TextFieldStatus; 3] = [
    TextFieldStatus::Enabled,
    TextFieldStatus::Error,
    TextFieldStatus::Disabled,
];

/// What a text field of the catalog holds.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Content {
    /// No text and no icon: the label rests over the text.
    #[default]
    Plain,
    /// A value, a leading and a trailing icon: the label floats.
    Rich,
}

/// Every kind of content, in the order of the columns.
pub const CONTENTS: [Content; 2] = [Content::Plain, Content::Rich];

/// The label of every text field.
pub const LABEL: &str = "Label";

/// The value of a rich text field.
pub const VALUE: &str = "Value";

/// The supporting text of every text field.
pub const SUPPORTING_TEXT: &str = "Supporting text";

/// The width of every text field: the smallest one a text field has (androidx `TextFieldDefaults.MinWidth`). A layout
/// value of the catalog, not of Material: the catalog sets the width so that no field depends on its neighbors'.
pub const FIELD_WIDTH: Points = Points(280.0);

/// The height of a cell, the room of a field: the tallest one is the outlined one with supporting text. Its container
/// is 56 points high (`OutlinedTextFieldTokens.ContainerHeight`), the label floats half of its 16-point line above it
/// (8 points), and the supporting text is 4 points below it, in a 16-point line (androidx `SupportingTopPadding` and
/// `MinSupportingTextLineHeight`). A layout value of the catalog, not of Material; the catalog's tests check that
/// every field fits.
pub const CELL_HEIGHT: Points = Points(84.0);

/// The top of the text fields' rows: where the checkboxes' viewport ends, less its bottom margin, which a row gap
/// follows the checkboxes' last row with.
#[must_use]
pub fn top() -> Points {
    Points(checkbox::viewport().1.0 - MARGIN.0)
}

/// The bottom of the text fields' rows: below the last row, which is followed by its gap.
#[must_use]
pub fn bottom() -> Points {
    let mut bottom = top().0;
    for _ in STYLES {
        bottom += CELL_HEIGHT.0 + ROW_GAP.0;
    }
    Points(bottom)
}

/// The size of the viewport the catalog is opened in to show the text fields, width and height: the page's width, as the
/// rows of text fields are narrower than the buttons' grid, and the height of the page with the rows and the margin
/// below them.
#[must_use]
pub fn viewport() -> (Points, Points) {
    (grid::viewport().0, Points(bottom().0 + MARGIN.0))
}

/// One text field of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Cell {
    /// Which text field it is.
    pub style: TextFieldStyle,
    /// What it holds.
    pub content: Content,
    /// Whether it can be edited, and whether it shows an error.
    pub status: TextFieldStatus,
}

impl Cell {
    /// The text field of this style, content and status.
    #[must_use]
    pub const fn new(style: TextFieldStyle, content: Content, status: TextFieldStatus) -> Self {
        Self {
            style,
            content,
            status,
        }
    }

    /// Every text field of the catalog, row by row.
    pub fn all() -> impl Iterator<Item = Self> {
        STYLES.into_iter().flat_map(|style| {
            CONTENTS.into_iter().flat_map(move |content| {
                STATUSES
                    .into_iter()
                    .map(move |status| Self::new(style, content, status))
            })
        })
    }

    /// Its name, the style, the content and the status: `filled-plain-enabled`. The test scripts name the fields by it.
    #[must_use]
    pub fn name(self) -> String {
        let style = match self.style {
            TextFieldStyle::Outlined => "outlined",
            // `TextFieldStyle` is non-exhaustive; the default style is the filled one.
            _ => "filled",
        };
        let content = match self.content {
            Content::Rich => "rich",
            Content::Plain => "plain",
        };
        let status = match self.status {
            TextFieldStatus::Error => "error",
            TextFieldStatus::Disabled => "disabled",
            // The statuses the catalog does not show look as the enabled one does, or as the error one: the name of
            // a cell the catalog does not have is never looked up.
            _ => "enabled",
        };
        format!("{style}-{content}-{status}")
    }

    /// The test id of the view around it, which the test scripts name (`id:text-field-filled-plain-enabled`).
    #[must_use]
    pub fn test_id(self) -> String {
        format!("text-field-{}", self.name())
    }

    /// The top left corner of its view in the viewport, in points. For an outlined field it is the top of the label's
    /// overhang, above the container.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let column =
            position(&CONTENTS, self.content) * STATUSES.len() + position(&STATUSES, self.status);
        let mut x = MARGIN.0;
        for _ in 0..column {
            x += COLUMN_WIDTH.0;
        }
        let mut y = top().0;
        for _ in 0..position(&STYLES, self.style) {
            y += CELL_HEIGHT.0 + ROW_GAP.0;
        }
        (Points(x), Points(y))
    }
}

/// Where `item` is in `items`, or 0 for an item the catalog does not show.
fn position<T: PartialEq + Copy>(items: &[T], item: T) -> usize {
    items.iter().position(|i| *i == item).unwrap_or(0)
}

/// `page` with every text field of the catalog at its place, each in a view with its test id, as wide as
/// [`FIELD_WIDTH`].
#[must_use]
pub fn place(page: View) -> View {
    Cell::all().fold(page, |page, cell| {
        let (x, y) = cell.origin();
        let rich = cell.content == Content::Rich;
        let mut field = text_field()
            .label(LABEL)
            .value(if rich { VALUE } else { "" })
            .style(cell.style)
            .status(cell.status)
            .supporting_text(SUPPORTING_TEXT);
        if rich {
            field = field.leading_icon(Some(ICON)).trailing_icon(Some(ICON));
        }
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .w(FIELD_WIDTH)
                .test_id(&cell.test_id())
                .child(field),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex, PoisonError};

    use nsh_core::geometry::{Bounds, Size};
    use nsh_core::{AccessibilityRole, MountItem, NodeId, ViewKind};
    use nsh_engine::{EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::Catalog;

    /// The text field test script, `text_field.test.tsx`, which opens the catalog at [`viewport`] and takes the
    /// screenshots `visual-checks` reads.
    const SCRIPT: &str = include_str!("../text_field.test.tsx");

    #[test]
    fn every_text_field_is_once_in_the_catalog() {
        let cells: Vec<Cell> = Cell::all().collect();
        assert_eq!(cells.len(), STYLES.len() * CONTENTS.len() * STATUSES.len());
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
    fn the_rows_and_columns_are_where_the_layout_says() {
        let first = Cell::new(
            TextFieldStyle::Filled,
            Content::Plain,
            TextFieldStatus::Enabled,
        );
        // Below the checkboxes' viewport of 2,800 points, less the margin the viewport ends with.
        assert_eq!(top(), Points(2776.0));
        assert_eq!(first.origin(), (Points(24.0), Points(2776.0)));
        // Error is the second column of a content, disabled the third.
        let error = Cell {
            status: TextFieldStatus::Error,
            ..first
        };
        assert_eq!(error.origin(), (Points(344.0), Points(2776.0)));
        let disabled = Cell {
            status: TextFieldStatus::Disabled,
            ..first
        };
        assert_eq!(disabled.origin(), (Points(664.0), Points(2776.0)));
        // Rich is the second content, after the three plain statuses.
        let rich = Cell {
            content: Content::Rich,
            ..first
        };
        assert_eq!(rich.origin(), (Points(984.0), Points(2776.0)));
        // Outlined is the second row, below the first one (84 points and the gap).
        let outlined = Cell {
            style: TextFieldStyle::Outlined,
            ..first
        };
        assert_eq!(outlined.origin(), (Points(24.0), Points(2876.0)));
        let last = Cell::new(
            TextFieldStyle::Outlined,
            Content::Rich,
            TextFieldStatus::Disabled,
        );
        assert_eq!(last.origin(), (Points(1624.0), Points(2876.0)));
        assert_eq!(bottom(), Points(2976.0));
    }

    #[test]
    fn the_viewport_holds_the_text_fields_and_the_grids_above() {
        let (width, height) = viewport();
        // The page is as wide as the buttons' grid; the text fields' two rows and the margin below them make it 200
        // points taller than the checkboxes' viewport.
        assert_eq!((width, height), (Points(2608.0), Points(3000.0)));
        assert_eq!(width, grid::viewport().0);
        assert_eq!(
            height.0,
            checkbox::viewport().1.0 + 2.0 * (CELL_HEIGHT.0 + ROW_GAP.0)
        );
        for cell in Cell::all() {
            let (x, y) = cell.origin();
            // The field is narrower than its column, which ends before the margin.
            assert!(x.0 + FIELD_WIDTH.0 < x.0 + COLUMN_WIDTH.0, "{cell:?}");
            assert!(x.0 >= MARGIN.0, "{cell:?}");
            assert!(x.0 + COLUMN_WIDTH.0 <= width.0 - MARGIN.0, "{cell:?}");
            // Below every checkbox, and a row gap clear of it.
            assert!(y.0 >= top().0, "{cell:?}");
            assert!(
                y.0 + CELL_HEIGHT.0 <= height.0 - MARGIN.0 - ROW_GAP.0,
                "{cell:?}"
            );
        }
        // The last row and its gap end at the margin, as the other grids'.
        assert_eq!(bottom().0 + MARGIN.0, height.0);
        // No text field takes the place of a button, an icon button or a checkbox.
        for cell in grid::Cell::all() {
            assert!(cell.origin().1.0 + grid::height(cell.size).0 <= top().0);
        }
        for cell in grid::IconCell::all() {
            assert!(cell.origin().1.0 + cell.frame().1.0 <= top().0);
        }
        for cell in checkbox::CheckboxCell::all() {
            assert!(cell.origin().1.0 + checkbox::SIDE.0 <= top().0);
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
            "await app.screenshot(`target/visual/text-field-${scheme}-rest.png`);",
            "await app.type(`id:text-field-${style}-${field}`, \"\");",
            "`target/visual/text-field-${scheme}-focus-${style}-${field}.png`",
            "for (const scheme of [\"light\", \"dark\"] as const) {",
        ] {
            assert!(SCRIPT.contains(line), "the script lacks: {line}");
        }
        // The styles it loops over are the grid's, in the grid's names.
        for style in STYLES {
            let name = Cell {
                style,
                ..Cell::default()
            }
            .name();
            let style = name.split('-').next().unwrap_or_default();
            assert!(SCRIPT.contains(&format!("\"{style}\"")), "{style}");
        }
    }

    #[test]
    fn ids_name_the_text_field() {
        let cell = Cell::new(
            TextFieldStyle::Outlined,
            Content::Rich,
            TextFieldStatus::Disabled,
        );
        assert_eq!(cell.name(), "outlined-rich-disabled");
        assert_eq!(cell.test_id(), "text-field-outlined-rich-disabled");
        let styles: Vec<String> = STYLES
            .into_iter()
            .map(|style| Cell { style, ..cell }.test_id())
            .collect();
        assert_eq!(
            styles,
            [
                "text-field-filled-rich-disabled",
                "text-field-outlined-rich-disabled",
            ]
        );
        let contents: Vec<String> = CONTENTS
            .into_iter()
            .map(|content| Cell { content, ..cell }.test_id())
            .collect();
        assert_eq!(
            contents,
            [
                "text-field-outlined-plain-disabled",
                "text-field-outlined-rich-disabled",
            ]
        );
        let statuses: Vec<String> = STATUSES
            .into_iter()
            .map(|status| Cell { status, ..cell }.test_id())
            .collect();
        assert_eq!(
            statuses,
            [
                "text-field-outlined-rich-enabled",
                "text-field-outlined-rich-error",
                "text-field-outlined-rich-disabled",
            ]
        );
    }

    /// Every text field is at its place on the grid, in a view as wide as the catalog says and no taller than its cell,
    /// with what its cell holds: the value of a rich field, an input that takes text unless the field is disabled, the
    /// canvases of its icons and of the outlined style's outline, and the alert of an error's supporting text.
    #[test]
    fn every_text_field_is_where_the_grid_says_with_what_its_cell_holds() {
        // The viewport the test script opens the catalog in: the screen holds only what is in it, and the text fields
        // are far below the first buttons.
        let (width, height) = viewport();
        let mut app = Headless::start(EngineConfig {
            viewport: Size::new(width, height),
            ..EngineConfig::new(Root::of::<Catalog>())
        })
        .unwrap();
        let items: Arc<Mutex<Vec<MountItem>>> = Arc::default();
        let sink = Arc::clone(&items);
        app.observe(move |_, batch| {
            sink.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .extend(batch.items.iter().cloned());
        });
        app.wait_for(SUPPORTING_TEXT).unwrap();
        let items = items.lock().unwrap_or_else(PoisonError::into_inner).clone();

        let mut ids = HashMap::new();
        let mut frames: HashMap<NodeId, Bounds<Points>> = HashMap::new();
        let mut kinds = HashMap::new();
        let mut alerts = Vec::new();
        let mut inputs = HashMap::new();
        let mut parents = HashMap::new();
        for item in &items {
            match item {
                MountItem::Create { id, kind } => {
                    kinds.insert(*id, *kind);
                }
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        ids.insert(test_id.to_string(), *id);
                    }
                    if props.accessibility.role == AccessibilityRole::Alert {
                        alerts.push(*id);
                    }
                }
                MountItem::Input { id, config } => {
                    inputs.insert(*id, config.editable);
                }
                MountItem::Frame { id, frame } => {
                    frames.insert(*id, *frame);
                }
                MountItem::Insert { parent, child, .. } => {
                    parents.insert(*child, *parent);
                }
                MountItem::Remove { id } => {
                    parents.remove(id);
                }
                _ => {}
            }
        }
        // Whether `node` is `field` or inside it.
        let inside = |field: NodeId, mut node: NodeId| loop {
            if node == field {
                return true;
            }
            match parents.get(&node) {
                Some(parent) => node = *parent,
                None => return false,
            }
        };
        for cell in Cell::all() {
            let field = ids[&cell.test_id()];
            let (x, y) = cell.origin();
            let frame = frames[&field];
            assert_eq!((frame.origin.x, frame.origin.y), (x, y), "{cell:?}");
            assert_eq!(frame.size.width, FIELD_WIDTH, "{cell:?}");
            assert!(frame.size.height <= CELL_HEIGHT, "{cell:?}");
            let canvases = kinds
                .iter()
                .filter(|(id, kind)| **kind == ViewKind::Canvas && inside(field, **id))
                .count();
            let rich = cell.content == Content::Rich;
            let outline = usize::from(cell.style == TextFieldStyle::Outlined);
            assert_eq!(canvases, outline + if rich { 2 } else { 0 }, "{cell:?}");
            let in_error = cell.status == TextFieldStatus::Error;
            let alerts_inside = alerts.iter().filter(|id| inside(field, **id)).count();
            assert_eq!(alerts_inside, usize::from(in_error), "{cell:?}");
            let value = app
                .screen()
                .input_text(&format!("id:{}", cell.test_id()))
                .unwrap();
            let expected = if rich { VALUE } else { "" };
            assert_eq!(value, Some(expected), "{cell:?}");
            let editable: Vec<bool> = inputs
                .iter()
                .filter(|(id, _)| inside(field, **id))
                .map(|(_, editable)| *editable)
                .collect();
            assert_eq!(
                editable,
                [cell.status != TextFieldStatus::Disabled],
                "{cell:?}"
            );
        }
    }
}
