//! Where the catalog puts each radio button, so that the visual checks know where to read its pixels.
//!
//! The radio buttons sit in a block of their own below the other blocks of the page (the grids of [`crate::grid`], the
//! switches and the checkboxes): one column per selection, one row per availability, the enabled row above the
//! disabled one. Each radio button's top left corner is at a fixed place.
//!
//! [`crate::grid::viewport`] stays the viewport of the buttons' screenshots (`button.test.tsx`), and `radio.test.tsx`
//! opens the catalog in [`viewport`], which holds this block as well: the same cells at the same places, with the
//! block below the bottom edge of the smaller one.

use nsh_core::geometry::Points;
use nsh_core::prelude::*;
use nsh_m3e::radio::{FRAME, radio};

use crate::checkbox;
use crate::grid::{self, AVAILABILITIES, Availability, MARGIN};

/// Whether a radio button in the catalog is selected.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Selection {
    /// It is not selected.
    #[default]
    Unselected,
    /// It is selected.
    Selected,
}

/// Every selection, in the order of the columns.
pub const SELECTIONS: [Selection; 2] = [Selection::Unselected, Selection::Selected];

/// The width of a column: more than the state layer's 40 points (`RadioButtonTokens.StateLayerSize`), so that the
/// layer of a hovered radio button reaches no other. A layout value of the catalog, not of Material.
pub const COLUMN_WIDTH: Points = Points(64.0);

/// The space between two rows, past the radio button's own size: with it, the rows are further apart than the state
/// layer is wide. A layout value of the catalog, not of Material.
pub const ROW_GAP: Points = Points(32.0);

/// The test id of the view that holds the block, which the tests name.
pub const BLOCK_TEST_ID: &str = "radios";

/// The top of the block: a margin below the checkboxes' viewport, which is the last block before this one, so below
/// every button, icon button, switch and checkbox.
#[must_use]
pub fn top() -> Points {
    Points(checkbox::viewport().1.0 + MARGIN.0)
}

/// The height of the block: the rows of radio buttons and the gaps between them.
#[must_use]
pub fn block_height() -> Points {
    let mut height = 0.0;
    for _ in AVAILABILITIES {
        height += FRAME.0 + ROW_GAP.0;
    }
    Points(height - ROW_GAP.0)
}

/// The size of the viewport the radio buttons' screenshots are taken in, width and height: the page's width, and its
/// height with the block and a margin below it.
#[must_use]
pub fn viewport() -> (Points, Points) {
    let (width, _) = grid::viewport();
    (width, Points(top().0 + block_height().0 + MARGIN.0))
}

/// One radio button of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Cell {
    /// Whether it is selected.
    pub selection: Selection,
    /// Whether it takes presses.
    pub availability: Availability,
}

impl Cell {
    /// The radio button with this selection and availability.
    #[must_use]
    pub const fn new(selection: Selection, availability: Availability) -> Self {
        Self {
            selection,
            availability,
        }
    }

    /// Every radio button of the catalog, column by column.
    pub fn all() -> impl Iterator<Item = Self> {
        SELECTIONS.into_iter().flat_map(|selection| {
            AVAILABILITIES
                .into_iter()
                .map(move |availability| Self::new(selection, availability))
        })
    }

    /// Its label, which screen readers say: its selection.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self.selection {
            Selection::Selected => "Selected",
            // `Selection` is non-exhaustive; the default one is the unselected one.
            _ => "Unselected",
        }
    }

    /// Its name in the test ids and the screenshots' file names (`unselected-enabled`).
    #[must_use]
    pub fn slug(self) -> String {
        let availability = match self.availability {
            Availability::Disabled => "disabled",
            Availability::Enabled => "enabled",
        };
        format!("{}-{availability}", self.label().to_lowercase())
    }

    /// The test id of the view around it, which the test scripts name (`id:radio-unselected-enabled`).
    #[must_use]
    pub fn test_id(self) -> String {
        format!("radio-{}", self.slug())
    }

    /// The top left corner of its box in the block, in points.
    #[must_use]
    pub fn position(self) -> (Points, Points) {
        let column = position(&SELECTIONS, self.selection);
        let mut x = MARGIN.0;
        for _ in 0..column {
            x += COLUMN_WIDTH.0;
        }
        let mut y = 0.0;
        for availability in AVAILABILITIES {
            if availability == self.availability {
                break;
            }
            y += FRAME.0 + ROW_GAP.0;
        }
        (Points(x), Points(y))
    }

    /// The top left corner of its box in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let (x, y) = self.position();
        (x, Points(top().0 + y.0))
    }
}

/// The block of the catalog's radio buttons, at [`top`], each at its [`Cell::position`] inside it: the block is as
/// large as the cells, so that the radio buttons lie within the view that holds them (a view takes no press outside
/// its parent's bounds on every host).
#[must_use]
pub fn block() -> impl IntoElement {
    let (width, _) = viewport();
    Cell::all().fold(
        view()
            .absolute()
            .left(Points(0.0))
            .top(top())
            .w(width)
            .h(block_height())
            .test_id(BLOCK_TEST_ID),
        |block, cell| {
            let (x, y) = cell.position();
            block.child(
                view()
                    .absolute()
                    .left(x)
                    .top(y)
                    .test_id(&cell.test_id())
                    .child(
                        radio()
                            .label(cell.label())
                            .checked(cell.selection == Selection::Selected)
                            .disabled(cell.availability == Availability::Disabled),
                    ),
            )
        },
    )
}

/// Where `item` is in `items`, or 0 for an item the catalog does not show.
fn position<T: PartialEq + Copy>(items: &[T], item: T) -> usize {
    items.iter().position(|i| *i == item).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex, PoisonError};

    use nsh_core::geometry::Bounds;
    use nsh_core::{AccessibilityRole, Checked, MountItem, NodeId};
    use nsh_engine::{EngineConfig, Root};
    use nsh_m3e::tokens::androidx::radio_button_tokens;
    use nsh_testing::Headless;

    use super::*;
    use crate::Catalog;
    use crate::grid::{Cell as ButtonCell, IconCell};

    #[test]
    fn every_radio_button_is_once_in_the_catalog() {
        let cells: Vec<Cell> = Cell::all().collect();
        assert_eq!(cells.len(), SELECTIONS.len() * AVAILABILITIES.len());
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
        let first = Cell::new(Selection::Unselected, Availability::Enabled);
        // A margin of 24 points below the checkboxes' viewport: 2,896 points.
        assert_eq!(top(), Points(2920.0));
        assert_eq!(first.position(), (Points(24.0), Points(0.0)));
        assert_eq!(first.origin(), (Points(24.0), Points(2920.0)));
        // Selected is the second column, 64 points on.
        let selected = Cell {
            selection: Selection::Selected,
            ..first
        };
        assert_eq!(selected.origin(), (Points(88.0), Points(2920.0)));
        // Disabled is the second row: a radio button's 24 points and the 32-point gap below the first.
        let disabled = Cell {
            availability: Availability::Disabled,
            ..first
        };
        assert_eq!(disabled.origin(), (Points(24.0), Points(2976.0)));
        // Two rows of 24 points and the gap between them.
        assert_eq!(block_height(), Points(80.0));
    }

    #[test]
    fn the_viewport_holds_the_block_below_the_checkboxes_viewport_and_a_margin() {
        let (width, height) = viewport();
        // The page's width, and the checkboxes' 2,896 points of height with a margin of 24, the block's 80 and a
        // margin of 24.
        assert_eq!((width, height), (Points(2608.0), Points(3024.0)));
        assert_eq!(width, grid::viewport().0);
        // The buttons' own screenshots keep their viewport.
        assert_eq!(grid::viewport(), (Points(2608.0), Points(2784.0)));
        for cell in Cell::all() {
            let (x, y) = cell.origin();
            assert!(x.0 >= MARGIN.0, "{cell:?}");
            assert!(x.0 + FRAME.0 <= width.0 - MARGIN.0, "{cell:?}");
            // Under the checkboxes' viewport, so below every button, icon button, switch and checkbox.
            assert!(y.0 >= checkbox::viewport().1.0, "{cell:?}");
            assert!(y.0 + FRAME.0 <= height.0 - MARGIN.0, "{cell:?}");
        }
        // The last row ends at the margin.
        let last = Cell::new(Selection::Selected, Availability::Disabled);
        assert_eq!(last.origin().1.0 + FRAME.0 + MARGIN.0, height.0);
        // Every button and icon button is above the block.
        for cell in ButtonCell::all() {
            assert!(cell.origin().1.0 + grid::height(cell.size).0 <= top().0);
        }
        for cell in IconCell::all() {
            assert!(cell.origin().1.0 + cell.frame().1.0 <= top().0);
        }
        for cell in checkbox::CheckboxCell::all() {
            assert!(cell.origin().1.0 + checkbox::SIDE.0 <= top().0);
        }
    }

    #[test]
    fn the_state_layers_of_two_radio_buttons_never_meet() {
        // A hovered radio button's 40-point layer reaches 8 points past its box: no other radio button's layer is there.
        let layer = radio_button_tokens::STATE_LAYER_SIZE.0;
        let cells: Vec<Cell> = Cell::all().collect();
        for (i, a) in cells.iter().enumerate() {
            for b in &cells[i + 1..] {
                let (ax, ay) = a.origin();
                let (bx, by) = b.origin();
                // Centers' distance on the axis they differ on.
                let apart = (ax.0 - bx.0).abs().max((ay.0 - by.0).abs());
                assert!(apart >= layer, "{a:?} {b:?}");
            }
        }
    }

    #[test]
    fn labels_and_ids_name_the_radio_button() {
        let cell = Cell::new(Selection::Selected, Availability::Disabled);
        assert_eq!(cell.label(), "Selected");
        assert_eq!(cell.slug(), "selected-disabled");
        assert_eq!(cell.test_id(), "radio-selected-disabled");
        let ids: Vec<String> = Cell::all().map(Cell::test_id).collect();
        assert_eq!(
            ids,
            [
                "radio-unselected-enabled",
                "radio-unselected-disabled",
                "radio-selected-enabled",
                "radio-selected-disabled",
            ]
        );
    }

    /// Every radio button is at its place in the block, inside a view with its test id, with the selection, the
    /// availability and the label its cell says.
    #[test]
    fn every_radio_button_is_where_the_grid_says() {
        let mut app = Headless::start(EngineConfig::new(Root::of::<Catalog>())).unwrap();
        let items: Arc<Mutex<Vec<MountItem>>> = Arc::default();
        let sink = Arc::clone(&items);
        app.observe(move |_, batch| {
            sink.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .extend(batch.items.iter().cloned());
        });
        app.wait_for("Filled").unwrap();
        let items = items.lock().unwrap_or_else(PoisonError::into_inner).clone();

        let mut ids = HashMap::new();
        let mut frames: HashMap<NodeId, Bounds<Points>> = HashMap::new();
        let mut parents = HashMap::new();
        let mut radios = HashMap::new();
        for item in &items {
            match item {
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        ids.insert(test_id.to_string(), *id);
                    }
                    if props.accessibility.role == AccessibilityRole::Radio {
                        radios.insert(
                            *id,
                            (
                                props.accessibility.label.clone(),
                                props.accessibility.state.disabled,
                                props.accessibility.state.checked,
                            ),
                        );
                    }
                }
                MountItem::Frame { id, frame } => {
                    frames.insert(*id, *frame);
                }
                MountItem::Insert { parent, child, .. } => {
                    parents.insert(*child, *parent);
                }
                _ => {}
            }
        }
        assert_eq!(radios.len(), Cell::all().count());
        // The block is below the buttons' viewport and as large as its cells.
        let block = frames[&ids[BLOCK_TEST_ID]];
        let (width, _) = viewport();
        assert_eq!(
            (
                block.origin.x,
                block.origin.y,
                block.size.width,
                block.size.height
            ),
            (Points(0.0), top(), width, block_height())
        );
        for cell in Cell::all() {
            let id = ids[&cell.test_id()];
            assert_eq!(parents[&id], ids[BLOCK_TEST_ID], "{cell:?}");
            let (x, y) = cell.position();
            assert_eq!(
                (frames[&id].origin.x, frames[&id].origin.y),
                (x, y),
                "{cell:?}"
            );
            let radio = parents
                .iter()
                .find(|(_, parent)| **parent == id)
                .map(|(child, _)| *child)
                .unwrap();
            let (label, disabled, checked) = radios[&radio].clone();
            assert_eq!(label.as_deref(), Some(cell.label()), "{cell:?}");
            assert_eq!(
                disabled,
                cell.availability == Availability::Disabled,
                "{cell:?}"
            );
            assert_eq!(
                checked,
                Some(if cell.selection == Selection::Selected {
                    Checked::True
                } else {
                    Checked::False
                }),
                "{cell:?}"
            );
            // A radio button is its box, with no layout of its own around it.
            assert_eq!(
                (frames[&radio].size.width, frames[&radio].size.height),
                (FRAME, FRAME),
                "{cell:?}"
            );
        }
    }
}
