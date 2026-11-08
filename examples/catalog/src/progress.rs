//! Where the catalog puts each linear progress indicator, so that the visual checks know where to read its pixels.
//!
//! The indicators sit in one column in the strip to the right of the icon buttons' grid: the space that grid leaves
//! between its last column and the viewport's right margin, so that the viewport stays the size the buttons and the
//! icon buttons make. The flat ones come first and the wavy ones after them; in each block there is one row per
//! progress, the determinate ones from none of the work to all of it, and then the indeterminate one. The column is
//! at the left of the dividers' ([`grid::divider_left`]) and starts below them ([`top`]).
//!
//! They are on the page only under reduced motion ([`crate::Catalog`] says why), where they hold still, and the
//! screenshots of their own are taken so.
//!
//! The indicators are [`WIDTH`] points wide, narrower than the library's default of 240 points, which does not fit
//! in that strip. The default width is the library's own tests' to pin.

use nsh_core::geometry::{Points, percentage};
use nsh_core::prelude::*;
use nsh_m3e::linear_progress_indicator::linear_progress_indicator;
use nsh_m3e::progress::{Progress, ProgressShape};
use nsh_m3e::tokens::androidx::linear_progress_indicator_tokens;

use crate::grid::{self, DIVIDER_SPACE, DividerCell, MARGIN, ROW_GAP};

/// Every shape, in the order of the row blocks.
pub const SHAPES: [ProgressShape; 2] = [ProgressShape::Flat, ProgressShape::Wavy];

/// How much of the work the determinate indicators of a block show, in per cent, in the order of their rows. None and
/// all of it, the two thresholds of the wave (a tenth and 95 %, where it flattens), a progress inside the wave's range
/// and one near the end, where the track is a few points long.
pub const PERCENTS: [u8; 6] = [0, 10, 50, 90, 95, 100];

/// How wide each indicator is: less than the 144 points the strip beside the icon buttons' grid leaves between the
/// dividers' gutter and the viewport's right margin. A layout value of the catalog, not of Material.
pub const WIDTH: Points = Points(120.0);

/// What an indicator of the catalog shows of the work.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Reading {
    /// The extent of the work is unknown.
    #[default]
    Indeterminate,
    /// This many per cent of the work is done.
    Determinate(u8),
}

impl Reading {
    /// Every reading of a block, in the order of its rows: the determinate ones, then the indeterminate one.
    pub fn all() -> impl Iterator<Item = Self> {
        PERCENTS
            .into_iter()
            .map(Self::Determinate)
            .chain(std::iter::once(Self::Indeterminate))
    }

    /// The progress the indicator is given: a whole number of per cent as the fraction nearest to it.
    #[must_use]
    pub fn progress(self) -> Progress {
        match self {
            Self::Determinate(percent) => {
                Progress::Determinate(percentage(f32::from(percent) / 100.0))
            }
            Self::Indeterminate => Progress::Indeterminate,
        }
    }
}

/// One linear progress indicator of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ProgressCell {
    /// Whether its active line is straight or waves.
    pub shape: ProgressShape,
    /// How much of the work it shows.
    pub reading: Reading,
}

impl ProgressCell {
    /// The indicator of this shape and reading.
    #[must_use]
    pub const fn new(shape: ProgressShape, reading: Reading) -> Self {
        Self { shape, reading }
    }

    /// Every indicator of the catalog, block by block.
    pub fn all() -> impl Iterator<Item = Self> {
        SHAPES
            .into_iter()
            .flat_map(|shape| Reading::all().map(move |reading| Self::new(shape, reading)))
    }

    /// The progress it is given.
    #[must_use]
    pub fn progress(self) -> Progress {
        self.reading.progress()
    }

    /// Its label, which screen readers say: its shape and how much of the work it shows.
    #[must_use]
    pub fn label(self) -> String {
        let shape = match self.shape {
            ProgressShape::Wavy => "Wavy",
            _ => "Flat",
        };
        match self.reading {
            Reading::Determinate(percent) => format!("{shape} {percent} percent"),
            Reading::Indeterminate => format!("{shape} indeterminate"),
        }
    }

    /// The test id of the view around it, which the test scripts name (`id:linear-wavy-determinate-50`).
    #[must_use]
    pub fn test_id(self) -> String {
        let shape = match self.shape {
            ProgressShape::Wavy => "wavy",
            _ => "flat",
        };
        match self.reading {
            Reading::Determinate(percent) => format!("linear-{shape}-determinate-{percent}"),
            Reading::Indeterminate => format!("linear-{shape}-indeterminate"),
        }
    }

    /// Its height: the token of its shape, the stroke's for a flat one and the wave's room for a wavy one.
    #[must_use]
    pub fn height(self) -> Points {
        height(self.shape)
    }

    /// Its width and height, in points.
    #[must_use]
    pub fn frame(self) -> (Points, Points) {
        (WIDTH, self.height())
    }

    /// The top left corner of its container in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let mut y = top().0;
        for shape in SHAPES {
            for reading in Reading::all() {
                if (shape, reading) == (self.shape, self.reading) {
                    return (left(), Points(y));
                }
                y += height(shape).0 + ROW_GAP.0;
            }
        }
        (left(), Points(y))
    }
}

/// The height of an indicator of `shape`: `LinearProgressIndicatorTokens.Height` for a flat one and `WaveHeight` for a
/// wavy one.
#[must_use]
pub fn height(shape: ProgressShape) -> Points {
    match shape {
        ProgressShape::Wavy => linear_progress_indicator_tokens::WAVE_HEIGHT,
        // `ProgressShape` is non-exhaustive: a shape added later falls back to the flat one until the catalog shows
        // it.
        _ => linear_progress_indicator_tokens::HEIGHT,
    }
}

/// The left edge of the column: the dividers', past the icon buttons' grid and the gutter.
#[must_use]
pub fn left() -> Points {
    grid::divider_left()
}

/// The top of the column: below the dividers' column, which starts at the top of the icon buttons' grid, a row gap past
/// the last divider's space.
#[must_use]
pub fn top() -> Points {
    let mut top = grid::icon_top().0;
    for _ in DividerCell::all() {
        top += DIVIDER_SPACE.0 + ROW_GAP.0;
    }
    Points(top)
}

/// The right and bottom edges of the indicators and the margin around them: what the viewport must hold of them.
#[must_use]
pub fn extent() -> (Points, Points) {
    let mut bottom = top().0 + MARGIN.0;
    for shape in SHAPES {
        for _ in Reading::all() {
            bottom += height(shape).0 + ROW_GAP.0;
        }
    }
    (Points(left().0 + WIDTH.0 + MARGIN.0), Points(bottom))
}

/// The indicator of `cell` at its place on the page, inside a view with its test id.
#[must_use]
pub fn placed(cell: ProgressCell) -> impl IntoElement {
    let (x, y) = cell.origin();
    view()
        .absolute()
        .left(x)
        .top(y)
        .test_id(&cell.test_id())
        .child(
            linear_progress_indicator()
                .progress(cell.progress())
                .shape(cell.shape)
                .width(WIDTH)
                .label(cell.label()),
        )
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex, PoisonError};

    use nsh_core::geometry::Bounds;
    use nsh_core::platform::Environment;
    use nsh_core::{AccessibilityRole, MountItem, NodeId};
    use nsh_engine::{EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::Catalog;
    use crate::grid::{Cell, IconCell};

    #[test]
    fn every_indicator_is_once_in_the_catalog() {
        let cells: Vec<ProgressCell> = ProgressCell::all().collect();
        assert_eq!(cells.len(), SHAPES.len() * (PERCENTS.len() + 1));
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
                .map(|c| c.label())
                .collect::<HashSet<_>>()
                .len(),
            cells.len()
        );
        assert_eq!(
            cells
                .iter()
                .map(|c| c.origin().1.0.to_bits())
                .collect::<HashSet<_>>()
                .len(),
            cells.len()
        );
    }

    #[test]
    fn the_rows_are_where_the_layout_says() {
        use ProgressShape::{Flat, Wavy};
        // Past the icon buttons' 12 columns of 200 points, which start at the margin, and the gutter: where the
        // dividers' column is.
        assert_eq!(left(), Points(2440.0));
        assert_eq!(WIDTH, Points(120.0));
        let at = |shape, reading| ProgressCell::new(shape, reading).origin();
        // The icon buttons' grid starts at 904 and the dividers' column with it; its six spaces of 96 points and
        // their gaps (6 * 112) end at 1576, where the column starts.
        assert_eq!(grid::icon_top(), Points(904.0));
        assert_eq!(top(), Points(1576.0));
        assert_eq!(
            at(Flat, Reading::Determinate(0)),
            (Points(2440.0), Points(1576.0))
        );
        // Below a flat indicator (4 points) and the gap.
        assert_eq!(
            at(Flat, Reading::Determinate(10)),
            (Points(2440.0), Points(1596.0))
        );
        assert_eq!(
            at(Flat, Reading::Determinate(100)),
            (Points(2440.0), Points(1676.0))
        );
        assert_eq!(
            at(Flat, Reading::Indeterminate),
            (Points(2440.0), Points(1696.0))
        );
        // The wavy block starts below the seven flat rows (7 * 20 points), and a wavy row is 10 points and the gap.
        assert_eq!(
            at(Wavy, Reading::Determinate(0)),
            (Points(2440.0), Points(1716.0))
        );
        assert_eq!(
            at(Wavy, Reading::Determinate(10)),
            (Points(2440.0), Points(1742.0))
        );
        assert_eq!(
            at(Wavy, Reading::Indeterminate),
            (Points(2440.0), Points(1872.0))
        );
    }

    #[test]
    fn sizes_are_the_tokens() {
        assert_eq!(height(ProgressShape::Flat), Points(4.0));
        assert_eq!(height(ProgressShape::Wavy), Points(10.0));
        let cell = ProgressCell::new(ProgressShape::Wavy, Reading::Determinate(50));
        assert_eq!(cell.frame(), (Points(120.0), Points(10.0)));
        assert_eq!(cell.height(), Points(10.0));
        assert_eq!(
            ProgressCell::new(ProgressShape::Flat, Reading::Indeterminate).frame(),
            (Points(120.0), Points(4.0))
        );
    }

    #[test]
    fn the_indicators_stay_beside_the_icon_buttons_and_inside_the_viewport() {
        let (width, height) = grid::viewport();
        for cell in IconCell::all() {
            let (x, _) = cell.origin();
            // The widest icon button of a column (184 points) is narrower than the column.
            assert!(x.0 + cell.frame().0.0 <= left().0 - MARGIN.0, "{cell:?}");
        }
        // The buttons' rows are all above the column's top.
        for cell in Cell::all() {
            assert!(
                cell.origin().1.0 + grid::height(cell.size).0 <= grid::icon_top().0 - ROW_GAP.0
            );
        }
        for cell in ProgressCell::all() {
            let (x, y) = cell.origin();
            let (cell_width, cell_height) = cell.frame();
            assert!(x.0 >= left().0, "{cell:?}");
            assert!(y.0 >= top().0, "{cell:?}");
            assert!(x.0 + cell_width.0 + MARGIN.0 <= width.0, "{cell:?}");
            assert!(y.0 + cell_height.0 + MARGIN.0 <= height.0, "{cell:?}");
        }
        // The edges they need, a margin past the last of them, are inside what the catalog is opened in: the strip
        // changes nothing of the viewport.
        let (right, bottom) = extent();
        assert_eq!((right, bottom), (Points(2584.0), Points(1922.0)));
        assert!(right.0 <= width.0 && bottom.0 <= height.0);
        assert_eq!((width, height), (Points(2608.0), Points(2784.0)));
    }

    #[test]
    fn the_column_starts_below_the_dividers_and_never_over_them() {
        // The two columns are in the same strip, at the same left edge.
        assert_eq!(left(), grid::divider_left());
        for divider in DividerCell::all() {
            let (x, y) = divider.origin();
            assert_eq!(x, left(), "{divider:?}");
            // Each divider's space and a row gap end above the column's top, so that no indicator is on a divider's
            // space and the spaces' last row gap is the column's own.
            assert!(y.0 + DIVIDER_SPACE.0 + ROW_GAP.0 <= top().0, "{divider:?}");
        }
        // The column's top is the last of them, exactly: no room is left unused between the two columns.
        let last = DividerCell::all().last().map(|d| d.origin().1.0);
        assert_eq!(last, Some(top().0 - ROW_GAP.0 - DIVIDER_SPACE.0));
    }

    #[test]
    fn labels_and_ids_name_the_indicator() {
        use ProgressShape::{Flat, Wavy};
        let cell = ProgressCell::new(Wavy, Reading::Determinate(95));
        assert_eq!(cell.label(), "Wavy 95 percent");
        assert_eq!(cell.test_id(), "linear-wavy-determinate-95");
        let cell = ProgressCell::new(Flat, Reading::Indeterminate);
        assert_eq!(cell.label(), "Flat indeterminate");
        assert_eq!(cell.test_id(), "linear-flat-indeterminate");
        let ids: Vec<String> = Reading::all()
            .map(|reading| ProgressCell::new(Flat, reading).test_id())
            .collect();
        assert_eq!(
            ids,
            [
                "linear-flat-determinate-0",
                "linear-flat-determinate-10",
                "linear-flat-determinate-50",
                "linear-flat-determinate-90",
                "linear-flat-determinate-95",
                "linear-flat-determinate-100",
                "linear-flat-indeterminate",
            ]
        );
        assert_eq!(
            ProgressCell::default().test_id(),
            "linear-flat-indeterminate"
        );
    }

    #[test]
    fn a_reading_is_the_progress_the_indicator_is_given() {
        let fraction = |reading: Reading| match reading.progress() {
            Progress::Determinate(fraction) => Some(fraction.0),
            _ => None,
        };
        assert_eq!(fraction(Reading::Indeterminate), None);
        assert_eq!(Reading::default(), Reading::Indeterminate);
        // The fractions the library thresholds on, as the nearest `f32` to them: a tenth and 95 %, where its wave
        // flattens, are those numbers and not one step either side.
        assert_eq!(fraction(Reading::Determinate(0)), Some(0.0));
        assert_eq!(fraction(Reading::Determinate(10)), Some(0.1));
        assert_eq!(fraction(Reading::Determinate(50)), Some(0.5));
        assert_eq!(fraction(Reading::Determinate(95)), Some(0.95));
        assert_eq!(fraction(Reading::Determinate(100)), Some(1.0));
        let readings: Vec<Reading> = Reading::all().collect();
        assert_eq!(readings.len(), 7);
        assert_eq!(readings[6], Reading::Indeterminate);
        assert_eq!(readings[0], Reading::Determinate(0));
    }

    /// Starts the catalog as a user who asked for less motion, or not, sees it.
    fn start(reduce_motion: bool) -> Headless {
        Headless::start(EngineConfig {
            environment: Environment {
                reduce_motion,
                ..Environment::default()
            },
            ..EngineConfig::new(Root::of::<Catalog>())
        })
        .unwrap()
    }

    /// The page has no indicator unless the user asked for less motion, and every one of them then, so that nothing on
    /// it moves by itself in the screenshots of the other components (see [`Catalog`]).
    #[test]
    fn the_indicators_are_on_the_page_only_under_reduced_motion() {
        for (reduce_motion, expected) in [(false, 0), (true, ProgressCell::all().count())] {
            let mut app = start(reduce_motion);
            let items: Arc<Mutex<Vec<MountItem>>> = Arc::default();
            let sink = Arc::clone(&items);
            app.observe(move |_, batch| {
                sink.lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .extend(batch.items.iter().cloned());
            });
            app.wait_for("Filled").unwrap();
            let items = items.lock().unwrap_or_else(PoisonError::into_inner).clone();
            let bars = items
                .iter()
                .filter(|item| {
                    matches!(
                        item,
                        MountItem::ViewProps { props, .. }
                            if props.accessibility.role == AccessibilityRole::Progressbar
                    )
                })
                .count();
            let ids = items
                .iter()
                .filter(|item| {
                    matches!(
                        item,
                        MountItem::ViewProps { props, .. }
                            if props.test_id.as_ref().is_some_and(|id| id.starts_with("linear-"))
                    )
                })
                .count();
            assert_eq!(bars, expected, "reduce motion {reduce_motion}");
            assert_eq!(ids, expected, "reduce motion {reduce_motion}");
        }
    }

    /// Every indicator is at its place, inside a view with its test id, as a progress bar from 0 to 1 that says how
    /// far the work is and what it is, with the size its shape gives it.
    #[test]
    fn every_indicator_is_where_the_grid_says_and_a_progress_bar() {
        let mut app = start(true);
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
        let mut bars = HashMap::new();
        let mut parents = HashMap::new();
        for item in &items {
            match item {
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        ids.insert(test_id.to_string(), *id);
                    }
                    if props.accessibility.role == AccessibilityRole::Progressbar {
                        bars.insert(
                            *id,
                            (
                                props.accessibility.label.clone(),
                                props.accessibility.value.clone(),
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
        assert_eq!(bars.len(), ProgressCell::all().count());
        for cell in ProgressCell::all() {
            let id = ids[&cell.test_id()];
            let (x, y) = cell.origin();
            assert_eq!(
                (frames[&id].origin.x, frames[&id].origin.y),
                (x, y),
                "{cell:?}"
            );
            // The indicator is the one child of the view with the test id.
            let children: Vec<NodeId> = parents
                .iter()
                .filter(|(_, parent)| **parent == id)
                .map(|(child, _)| *child)
                .collect();
            let [indicator] = children[..] else {
                panic!("{cell:?}: one indicator in its view, not {children:?}");
            };
            let (width, height) = cell.frame();
            assert_eq!(
                (
                    frames[&indicator].size.width,
                    frames[&indicator].size.height
                ),
                (width, height),
                "{cell:?}"
            );
            let (label, value) = &bars[&indicator];
            assert_eq!(label.as_deref(), Some(cell.label().as_str()), "{cell:?}");
            let value = value.as_ref().unwrap();
            // From the reading, not from `cell.progress()` that gave the indicator its progress: the two would
            // agree on whatever it returned.
            let now = match cell.reading {
                Reading::Determinate(percent) => Some(f32::from(percent) / 100.0),
                Reading::Indeterminate => None,
            };
            assert_eq!(
                (value.min, value.max, value.now),
                (Some(0.0), Some(1.0), now),
                "{cell:?}"
            );
        }
    }
}
