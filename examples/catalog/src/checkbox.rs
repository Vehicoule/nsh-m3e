//! Where the catalog puts each checkbox, so that the visual checks know where to read its pixels.
//!
//! The checkboxes sit on a small grid below the icon buttons' (see [`crate::grid`]): one column per state, one row
//! per availability, the enabled row above the disabled one. A checkbox is laid out as its state layer, a square of
//! `CheckboxTokens.StateLayerSize` points; its box is the `CheckboxTokens.ContainerSize` square in the middle of it.
//!
//! The grid's top is where the page's other grids end, so this grid is the last block of the page, and
//! [`viewport`] is the page's size with it. `button.test.tsx` opens the page at [`crate::grid::viewport`], which stops
//! above this grid; `checkbox.test.tsx` opens it at [`viewport`].

use nsh_core::geometry::Points;
use nsh_core::prelude::*;
use nsh_m3e::checkbox::{CheckboxState, checkbox};
use nsh_m3e::tokens::androidx::checkbox_tokens;

use crate::grid::{self, AVAILABILITIES, Availability, MARGIN, ROW_GAP};

/// Every state, in the order of the columns.
pub const STATES: [CheckboxState; 3] = [
    CheckboxState::Unchecked,
    CheckboxState::Checked,
    CheckboxState::Indeterminate,
];

/// The space between two columns, from the edge of one state layer to the edge of the next. A layout value of the
/// catalog, not of Material: the catalog is a test page.
pub const COLUMN_GAP: Points = Points(24.0);

/// The side of a checkbox's layout, in points: its state layer's size, `CheckboxTokens.StateLayerSize`.
pub const SIDE: Points = checkbox_tokens::STATE_LAYER_SIZE;

/// The top of the checkboxes' grid: where the rows of the grids above it end, one row gap after their last row (the
/// bottom margin of [`grid::viewport`] starts there).
#[must_use]
pub fn top() -> Points {
    Points(grid::viewport().1.0 - MARGIN.0)
}

/// The size of the viewport the catalog is opened in to show the checkboxes, width and height: the page's width, as
/// the checkboxes' grid is narrower than the buttons', and the height of the page with this grid and the margin
/// below it.
#[must_use]
pub fn viewport() -> (Points, Points) {
    let mut height = top().0 + MARGIN.0;
    for _ in AVAILABILITIES {
        height += SIDE.0 + ROW_GAP.0;
    }
    (grid::viewport().0, Points(height))
}

/// One checkbox of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CheckboxCell {
    /// What it shows.
    pub state: CheckboxState,
    /// Whether it takes presses.
    pub availability: Availability,
}

impl CheckboxCell {
    /// The checkbox in this state and availability.
    #[must_use]
    pub const fn new(state: CheckboxState, availability: Availability) -> Self {
        Self {
            state,
            availability,
        }
    }

    /// Every checkbox of the catalog, column by column.
    pub fn all() -> impl Iterator<Item = Self> {
        STATES.into_iter().flat_map(|state| {
            AVAILABILITIES
                .into_iter()
                .map(move |availability| Self::new(state, availability))
        })
    }

    /// Its label, which screen readers say: the name of its state.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self.state {
            CheckboxState::Checked => "Checked",
            CheckboxState::Indeterminate => "Indeterminate",
            // `CheckboxState` is non-exhaustive; the default state is the unchecked one.
            _ => "Unchecked",
        }
    }

    /// The test id of the view around it, which the test scripts name (`id:checkbox-checked-enabled`).
    #[must_use]
    pub fn test_id(self) -> String {
        let availability = match self.availability {
            Availability::Disabled => "disabled",
            Availability::Enabled => "enabled",
        };
        format!("checkbox-{}-{availability}", self.label().to_lowercase())
    }

    /// The top left corner of its state layer in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let column = position(&STATES, self.state);
        let mut x = MARGIN.0;
        for _ in 0..column {
            x += SIDE.0 + COLUMN_GAP.0;
        }
        let mut y = top().0;
        for availability in AVAILABILITIES {
            if availability == self.availability {
                break;
            }
            y += SIDE.0 + ROW_GAP.0;
        }
        (Points(x), Points(y))
    }
}

/// Where `item` is in `items`, or 0 for an item the catalog does not show.
fn position<T: PartialEq + Copy>(items: &[T], item: T) -> usize {
    items.iter().position(|i| *i == item).unwrap_or(0)
}

/// `page` with every checkbox of the catalog at its place, each in a view with its test id.
#[must_use]
pub fn place(page: View) -> View {
    CheckboxCell::all().fold(page, |page, cell| {
        let (x, y) = cell.origin();
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .test_id(&cell.test_id())
                .child(
                    checkbox()
                        .label(cell.label())
                        .state(cell.state)
                        .disabled(cell.availability == Availability::Disabled),
                ),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex, PoisonError};

    use nsh_core::geometry::Bounds;
    use nsh_core::{AccessibilityRole, Checked, MountItem, NodeId};
    use nsh_engine::{EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::Catalog;

    /// The checkbox test script, `checkbox.test.tsx`, which opens the catalog at [`viewport`] and takes the
    /// screenshots `visual-checks` reads.
    const SCRIPT: &str = include_str!("../checkbox.test.tsx");

    #[test]
    fn every_checkbox_is_once_in_the_catalog() {
        let cells: Vec<CheckboxCell> = CheckboxCell::all().collect();
        assert_eq!(cells.len(), STATES.len() * AVAILABILITIES.len());
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
        // The grids above end where the viewport's bottom margin starts: 24 points of margin, then the 880 points
        // of the buttons' rows and gaps, the icon buttons' 1,760 and the navigation bars' 160.
        assert_eq!(top(), Points(2824.0));
        let first = CheckboxCell::new(CheckboxState::Unchecked, Availability::Enabled);
        assert_eq!(first.origin(), (Points(24.0), Points(2824.0)));
        // Checked is the second column, 40 points of state layer and 24 of gap further; indeterminate the third.
        let checked = CheckboxCell {
            state: CheckboxState::Checked,
            ..first
        };
        assert_eq!(checked.origin(), (Points(88.0), Points(2824.0)));
        let indeterminate = CheckboxCell {
            state: CheckboxState::Indeterminate,
            ..first
        };
        assert_eq!(indeterminate.origin(), (Points(152.0), Points(2824.0)));
        // The disabled row is below the enabled one: 40 points and a row gap.
        let disabled = CheckboxCell {
            availability: Availability::Disabled,
            ..first
        };
        assert_eq!(disabled.origin(), (Points(24.0), Points(2880.0)));
        assert_eq!(SIDE, Points(40.0));
    }

    #[test]
    fn the_viewport_holds_the_checkboxes_and_the_grids_above() {
        let (width, height) = viewport();
        // The page is as wide as the buttons' grid; the checkboxes' two rows and the margin below them make it 112
        // points taller than the icon buttons' viewport.
        assert_eq!((width, height), (Points(2608.0), Points(2960.0)));
        assert_eq!(width, grid::viewport().0);
        assert_eq!(height.0, grid::viewport().1.0 + 2.0 * (SIDE.0 + ROW_GAP.0));
        let mut bottom = Points(0.0);
        for cell in CheckboxCell::all() {
            let (x, y) = cell.origin();
            assert!(x.0 >= MARGIN.0, "{cell:?}");
            assert!(x.0 + SIDE.0 <= width.0 - MARGIN.0, "{cell:?}");
            // Below every icon button of the grid above, and a row gap clear of it.
            assert!(y.0 >= grid::viewport().1.0 - MARGIN.0, "{cell:?}");
            assert!(y.0 + SIDE.0 <= height.0 - MARGIN.0, "{cell:?}");
            bottom = Points(bottom.0.max(y.0 + SIDE.0));
        }
        // The last row and its gap end at the margin, as the other grids'.
        assert_eq!(bottom.0 + ROW_GAP.0, height.0 - MARGIN.0);
        // No checkbox takes the place of a button or an icon button.
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
            "await app.screenshot(`target/visual/checkbox-${scheme}-rest.png`);",
            "await app.hover(`id:checkbox-${state}-${availability}`);",
            "`target/visual/checkbox-${scheme}-hover-${state}-${availability}.png`",
            "for (const scheme of [\"light\", \"dark\"] as const) {",
        ] {
            assert!(SCRIPT.contains(line), "the script lacks: {line}");
        }
        // The states and availabilities it loops over are the grid's, in the grid's names.
        let ids: Vec<String> = CheckboxCell::all().map(CheckboxCell::test_id).collect();
        assert_eq!(
            ids,
            [
                "checkbox-unchecked-enabled",
                "checkbox-unchecked-disabled",
                "checkbox-checked-enabled",
                "checkbox-checked-disabled",
                "checkbox-indeterminate-enabled",
                "checkbox-indeterminate-disabled",
            ]
        );
        for name in [
            "unchecked",
            "checked",
            "indeterminate",
            "enabled",
            "disabled",
        ] {
            assert!(SCRIPT.contains(&format!("\"{name}\"")), "{name}");
        }
    }

    #[test]
    fn labels_and_ids_name_the_checkbox() {
        let cell = CheckboxCell::new(CheckboxState::Indeterminate, Availability::Disabled);
        assert_eq!(cell.label(), "Indeterminate");
        assert_eq!(cell.test_id(), "checkbox-indeterminate-disabled");
        let labels: Vec<&str> = STATES
            .into_iter()
            .map(|state| CheckboxCell { state, ..cell }.label())
            .collect();
        assert_eq!(labels, ["Unchecked", "Checked", "Indeterminate"]);
        assert_eq!(
            CheckboxCell {
                availability: Availability::Enabled,
                ..cell
            }
            .test_id(),
            "checkbox-indeterminate-enabled"
        );
    }

    /// Every checkbox is at its place on the grid, inside a view with its test id, as large as its state layer, and
    /// shows its state and availability to screen readers; no button of the page is a checkbox.
    #[test]
    fn every_checkbox_is_where_the_grid_says() {
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
        let mut checkboxes = HashMap::new();
        let mut parents = HashMap::new();
        for item in &items {
            match item {
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        ids.insert(test_id.to_string(), *id);
                    }
                    if props.accessibility.role == AccessibilityRole::Checkbox {
                        checkboxes.insert(
                            *id,
                            (
                                props.accessibility.state.checked,
                                props.accessibility.state.disabled,
                                props.accessibility.label.clone(),
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
        assert_eq!(checkboxes.len(), CheckboxCell::all().count());
        let child_of = |id: NodeId| {
            parents
                .iter()
                .find(|(_, parent)| **parent == id)
                .map(|(child, _)| *child)
                .unwrap()
        };
        for cell in CheckboxCell::all() {
            let id = ids[&cell.test_id()];
            let (x, y) = cell.origin();
            assert_eq!(
                (frames[&id].origin.x, frames[&id].origin.y),
                (x, y),
                "{cell:?}"
            );
            let root = child_of(id);
            assert_eq!(
                (frames[&root].size.width, frames[&root].size.height),
                (SIDE, SIDE),
                "{cell:?}"
            );
            let (checked, disabled, label) = &checkboxes[&root];
            let expected = match cell.state {
                CheckboxState::Checked => Checked::True,
                CheckboxState::Indeterminate => Checked::Mixed,
                _ => Checked::False,
            };
            assert_eq!(*checked, Some(expected), "{cell:?}");
            assert_eq!(
                *disabled,
                cell.availability == Availability::Disabled,
                "{cell:?}"
            );
            assert_eq!(label.as_deref(), Some(cell.label()), "{cell:?}");
        }
    }
}
