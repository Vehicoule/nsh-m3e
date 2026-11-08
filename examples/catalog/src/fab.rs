//! Where the catalog puts each floating action button (FAB), so that the visual checks know where to read its pixels.
//!
//! The FABs sit in one row below the checkboxes' grid (see [`crate::checkbox`]), the last block of the page: one column
//! per style and size, a style's four sizes side by side, their tops aligned. A FAB casts a shadow (ADR 0018), so the
//! columns leave room for it, and [`viewport`] is the page's size with this row.
//!
//! The row's top is where the page's other grids end, so `button.test.tsx`, `switch.test.tsx` and `checkbox.test.tsx`
//! open the page at viewports that stop above this row; `fab.test.tsx` opens it at [`viewport`].

use nsh_core::geometry::Points;
use nsh_core::prelude::*;
use nsh_m3e::fab::{FabSize, FabStyle, fab};
use nsh_m3e::tokens::androidx::{
    fab_baseline_tokens, fab_large_tokens, fab_medium_tokens, fab_small_tokens,
};

use crate::ICON;
use crate::checkbox;
use crate::grid::{self, MARGIN, ROW_GAP};

/// Every style, in the order of the columns.
pub const STYLES: [FabStyle; 2] = [FabStyle::PrimaryContainer, FabStyle::SecondaryContainer];

/// Every size, in the order of the columns within a style.
pub const SIZES: [FabSize; 4] = [
    FabSize::Small,
    FabSize::Baseline,
    FabSize::Medium,
    FabSize::Large,
];

/// The width of a column: more than the largest FAB (96 points) by 48, so that two FABs have surface between them, past
/// the reach of their shadows (14 points to the sides at the highest level, ADR 0018) on both sides of the middle of
/// the gap, where the visual checks read it. A layout value of the catalog, not of Material.
pub const COLUMN_WIDTH: Points = Points(144.0);

/// The top of the FABs' row: where the rows of the grids above it end, one row gap after their last row (the bottom
/// margin of [`checkbox::viewport`] starts there).
#[must_use]
pub fn top() -> Points {
    Points(checkbox::viewport().1.0 - MARGIN.0)
}

/// The height of the FABs' row and its gap: the tallest FAB and a row gap. The shadows reach further below a container
/// than the gap (20 points at the highest level, ADR 0018), so the margin below the row holds the rest of them.
#[must_use]
pub fn block_height() -> Points {
    let mut row = 0.0_f32;
    for size in SIZES {
        row = row.max(frame(size).1.0);
    }
    Points(row + ROW_GAP.0)
}

/// The size of the viewport the catalog is opened in to show the FABs, width and height: the page's width, as the
/// FABs' row is narrower than the buttons' grid, and the height of the page with this row and the margin below it.
#[must_use]
pub fn viewport() -> (Points, Points) {
    (
        grid::viewport().0,
        Points(top().0 + block_height().0 + MARGIN.0),
    )
}

/// The width and height of a FAB of `size`: its `ContainerWidth` and `ContainerHeight` tokens, as the FAB takes them.
#[must_use]
pub fn frame(size: FabSize) -> (Points, Points) {
    match size {
        FabSize::Small => (
            fab_small_tokens::CONTAINER_WIDTH,
            fab_small_tokens::CONTAINER_HEIGHT,
        ),
        FabSize::Medium => (
            fab_medium_tokens::CONTAINER_WIDTH,
            fab_medium_tokens::CONTAINER_HEIGHT,
        ),
        FabSize::Large => (
            fab_large_tokens::CONTAINER_WIDTH,
            fab_large_tokens::CONTAINER_HEIGHT,
        ),
        // `FabSize` is non-exhaustive: a size added later falls back to the default one until the catalog shows it.
        _ => (
            fab_baseline_tokens::CONTAINER_WIDTH,
            fab_baseline_tokens::CONTAINER_HEIGHT,
        ),
    }
}

/// One FAB of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct FabCell {
    /// Its color style.
    pub style: FabStyle,
    /// Its size.
    pub size: FabSize,
}

impl FabCell {
    /// The FAB of this style and size.
    #[must_use]
    pub const fn new(style: FabStyle, size: FabSize) -> Self {
        Self { style, size }
    }

    /// Every FAB of the catalog, column by column.
    pub fn all() -> impl Iterator<Item = Self> {
        STYLES
            .into_iter()
            .flat_map(|style| SIZES.into_iter().map(move |size| Self::new(style, size)))
    }

    /// Its label, which screen readers say: the name of its style.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self.style {
            FabStyle::SecondaryContainer => "Secondary container",
            // `FabStyle` is non-exhaustive; the default style is the primary container one.
            _ => "Primary container",
        }
    }

    /// The name of its style in a test id and a screenshot's file name.
    #[must_use]
    pub fn style_name(self) -> &'static str {
        match self.style {
            FabStyle::SecondaryContainer => "secondary-container",
            _ => "primary-container",
        }
    }

    /// The test id of the view around it, which the test scripts name (`id:fab-primary-container-baseline`).
    #[must_use]
    pub fn test_id(self) -> String {
        let size = match self.size {
            FabSize::Small => "small",
            FabSize::Medium => "medium",
            FabSize::Large => "large",
            _ => "baseline",
        };
        format!("fab-{}-{size}", self.style_name())
    }

    /// Its width and height, in points.
    #[must_use]
    pub fn frame(self) -> (Points, Points) {
        frame(self.size)
    }

    /// The top left corner of its container in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let column =
            grid::position(&STYLES, self.style) * SIZES.len() + grid::position(&SIZES, self.size);
        let mut x = MARGIN.0;
        for _ in 0..column {
            x += COLUMN_WIDTH.0;
        }
        (Points(x), top())
    }
}

/// `page` with every FAB of the catalog at its place, each in a view with its test id.
#[must_use]
pub fn place(page: View) -> View {
    FabCell::all().fold(page, |page, cell| {
        let (x, y) = cell.origin();
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .test_id(&cell.test_id())
                .child(
                    fab()
                        .icon(ICON)
                        .label(cell.label())
                        .style(cell.style)
                        .size(cell.size),
                ),
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

    /// The FAB test script, `fab.test.tsx`, which opens the catalog at [`viewport`] and takes the screenshots
    /// `visual-checks` reads.
    const SCRIPT: &str = include_str!("../fab.test.tsx");

    #[test]
    fn every_fab_is_once_in_the_catalog() {
        let cells: Vec<FabCell> = FabCell::all().collect();
        assert_eq!(cells.len(), STYLES.len() * SIZES.len());
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
    fn the_fabs_are_in_one_row_below_the_checkboxes() {
        // The checkboxes' grid ends at the margin before the checkboxes' viewport does: the FABs start there.
        assert_eq!(top(), Points(checkbox::viewport().1.0 - 24.0));
        assert_eq!(top(), Points(2872.0));
        let first = FabCell::new(FabStyle::PrimaryContainer, FabSize::Small);
        assert_eq!(first.origin(), (Points(24.0), top()));
        // The sizes are the columns of a style, 144 points apart; the second style starts after the fourth size.
        let baseline = FabCell {
            size: FabSize::Baseline,
            ..first
        };
        assert_eq!(baseline.origin(), (Points(168.0), top()));
        let secondary = FabCell {
            style: FabStyle::SecondaryContainer,
            ..first
        };
        assert_eq!(secondary.origin(), (Points(600.0), top()));
        let last = FabCell::new(FabStyle::SecondaryContainer, FabSize::Large);
        assert_eq!(last.origin(), (Points(1032.0), top()));
    }

    #[test]
    fn the_viewport_holds_the_fabs_and_the_grids_above() {
        let (width, height) = viewport();
        // The page is as wide as the buttons' grid; the FABs' row (the largest FAB's 96 points and a gap) and the
        // margin below it make it 136 points taller than the checkboxes' viewport, whose margin it replaces.
        assert_eq!((width, height), (Points(2608.0), Points(3008.0)));
        assert_eq!(width, grid::viewport().0);
        assert_eq!(block_height(), Points(112.0));
        assert_eq!(height.0, checkbox::viewport().1.0 + block_height().0);
        let mut bottom = Points(0.0);
        for cell in FabCell::all() {
            let (x, y) = cell.origin();
            let (cell_width, cell_height) = cell.frame();
            // A column is wider than its FAB, and the row ends inside the margin.
            assert!(cell_width.0 < COLUMN_WIDTH.0, "{cell:?}");
            assert!(x.0 >= MARGIN.0, "{cell:?}");
            assert!(x.0 + COLUMN_WIDTH.0 <= width.0 - MARGIN.0, "{cell:?}");
            // Below every checkbox of the grid above, and a row gap clear of it.
            assert!(y.0 >= checkbox::viewport().1.0 - MARGIN.0, "{cell:?}");
            assert!(y.0 + cell_height.0 <= height.0 - MARGIN.0, "{cell:?}");
            bottom = Points(bottom.0.max(y.0 + cell_height.0));
        }
        // The last row and its gap end at the margin, as the other grids'.
        assert_eq!(bottom.0 + ROW_GAP.0, height.0 - MARGIN.0);
        // No FAB takes the place of a checkbox, a switch, a button or an icon button.
        for cell in checkbox::CheckboxCell::all() {
            assert!(cell.origin().1.0 + checkbox::SIDE.0 <= top().0);
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
            "await app.screenshot(`target/visual/fab-${scheme}-rest.png`);",
            "await app.hover(`id:fab-${style}-baseline`);",
            "`target/visual/fab-${scheme}-hover-${style}.png`",
            "for (const scheme of [\"light\", \"dark\"] as const) {",
        ] {
            assert!(SCRIPT.contains(line), "the script lacks: {line}");
        }
        // The styles it loops over are the grid's, in the grid's names, and the baseline FAB is one of its sizes.
        let ids: Vec<String> = FabCell::all().map(FabCell::test_id).collect();
        assert_eq!(
            ids,
            [
                "fab-primary-container-small",
                "fab-primary-container-baseline",
                "fab-primary-container-medium",
                "fab-primary-container-large",
                "fab-secondary-container-small",
                "fab-secondary-container-baseline",
                "fab-secondary-container-medium",
                "fab-secondary-container-large",
            ]
        );
        for name in ["primary-container", "secondary-container"] {
            assert!(SCRIPT.contains(&format!("\"{name}\"")), "{name}");
        }
    }

    #[test]
    fn fab_sizes_are_their_tokens() {
        let frames: Vec<(f32, f32)> = SIZES
            .into_iter()
            .map(|size| {
                let (width, height) = frame(size);
                (width.0, height.0)
            })
            .collect();
        assert_eq!(
            frames,
            [(40.0, 40.0), (56.0, 56.0), (80.0, 80.0), (96.0, 96.0)]
        );
        let cell = FabCell::new(FabStyle::SecondaryContainer, FabSize::Medium);
        assert_eq!(cell.frame(), (Points(80.0), Points(80.0)));
    }

    #[test]
    fn labels_and_ids_name_the_fab() {
        let cell = FabCell::new(FabStyle::SecondaryContainer, FabSize::Large);
        assert_eq!(cell.label(), "Secondary container");
        assert_eq!(cell.style_name(), "secondary-container");
        assert_eq!(cell.test_id(), "fab-secondary-container-large");
        let labels: Vec<&str> = STYLES
            .into_iter()
            .map(|style| FabCell { style, ..cell }.label())
            .collect();
        assert_eq!(labels, ["Primary container", "Secondary container"]);
        let sizes: Vec<String> = SIZES
            .into_iter()
            .map(|size| FabCell { size, ..cell }.test_id())
            .collect();
        assert_eq!(
            sizes,
            [
                "fab-secondary-container-small",
                "fab-secondary-container-baseline",
                "fab-secondary-container-medium",
                "fab-secondary-container-large",
            ]
        );
        assert_eq!(
            FabCell {
                style: FabStyle::PrimaryContainer,
                ..cell
            }
            .test_id(),
            "fab-primary-container-large"
        );
    }

    /// Every FAB is at its place on the grid, inside a view with its test id, as large as its size says, and takes
    /// presses: it is a button for screen readers, with its style's label, never disabled.
    #[test]
    fn every_fab_is_where_the_grid_says() {
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
        let mut buttons = HashMap::new();
        let mut parents = HashMap::new();
        for item in &items {
            match item {
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        ids.insert(test_id.to_string(), *id);
                    }
                    if props.accessibility.role == AccessibilityRole::Button {
                        buttons.insert(
                            *id,
                            (
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
        let child_of = |id: NodeId| {
            parents
                .iter()
                .find(|(_, parent)| **parent == id)
                .map(|(child, _)| *child)
                .unwrap()
        };
        for cell in FabCell::all() {
            let id = ids[&cell.test_id()];
            let (x, y) = cell.origin();
            assert_eq!(
                (frames[&id].origin.x, frames[&id].origin.y),
                (x, y),
                "{cell:?}"
            );
            let button = child_of(id);
            let (width, height) = cell.frame();
            assert_eq!(
                (frames[&button].size.width, frames[&button].size.height),
                (width, height),
                "{cell:?}"
            );
            let (disabled, label) = &buttons[&button];
            assert!(!disabled, "{cell:?}");
            assert_eq!(label.as_deref(), Some(cell.label()), "{cell:?}");
        }
    }
}
