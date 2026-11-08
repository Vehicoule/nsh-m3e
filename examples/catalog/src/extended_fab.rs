//! Where the catalog puts each extended floating action button, so that the visual checks know where to read its pixels.
//!
//! The extended FABs sit in a grid below the FABs' row (see [`crate::fab`]), the last block of the page: one row per
//! size, and in each row six columns, a style's three kinds side by side: the FAB expanded with its icon, collapsed to
//! its icon, and with no icon (which is always expanded). The columns leave room for an expanded FAB and for the shadow,
//! and the rows are far enough apart that no shadow reaches the next row. [`viewport`] is the page's size with this
//! grid.
//!
//! An expanded FAB's width is its label's, which each host measures with its own font, so the catalog never says how
//! wide one is: it says where its left edge is, how high it is, and that a collapsed one is as wide as it is high.
//!
//! The grid's top is where the FABs' row ends, so every other test script opens the page at a viewport that stops above
//! it; `extended_fab.test.tsx` opens it at [`viewport`].

use nsh_core::geometry::Points;
use nsh_core::prelude::*;
use nsh_m3e::extended_fab::{ExtendedFabExpansion, ExtendedFabSize, extended_fab};
use nsh_m3e::fab::FabStyle;
use nsh_m3e::tokens::androidx::{
    extended_fab_large_tokens, extended_fab_medium_tokens, extended_fab_small_tokens,
};

use crate::grid::{self, MARGIN};
use crate::{ICON, fab};

/// Every style, in the order of the column groups.
pub const STYLES: [FabStyle; 2] = [FabStyle::PrimaryContainer, FabStyle::SecondaryContainer];

/// Every size, in the order of the rows.
pub const SIZES: [ExtendedFabSize; 3] = [
    ExtendedFabSize::Small,
    ExtendedFabSize::Medium,
    ExtendedFabSize::Large,
];

/// Every kind, in the order of the columns within a style.
pub const KINDS: [Kind; 3] = [Kind::Expanded, Kind::Collapsed, Kind::NoIcon];

/// What every extended FAB of the catalog says. Not a design value: long enough that an expanded FAB with no icon is
/// wider than its height plus the room the checks read beyond it, whatever font a host measures it with, and short
/// enough to fit a column.
pub const LABEL: &str = "New message";

/// The width of a column: more than the widest expanded FAB (the large one with its icon) by the reach of the shadows
/// on both sides and the room the visual checks read the surface in, left of the next FAB. The catalog's test checks
/// that every FAB fits with that room, with the font of the headless host; the hosts' fonts are not measured here. A
/// layout value of the catalog, not of Material.
pub const COLUMN_WIDTH: Points = Points(360.0);

/// The space between two rows: more than the reach of the shadows below a container (20 points at the highest level,
/// ADR 0018) and the room the visual checks read the surface in, so that no shadow of one row touches the next. A layout
/// value of the catalog, not of Material.
pub const ROW_GAP: Points = Points(48.0);

/// What an extended FAB of the catalog shows.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Kind {
    /// The icon and the label.
    #[default]
    Expanded,
    /// The icon alone, in a square: the FAB made collapsed.
    Collapsed,
    /// The label alone, in a container whose content is centered; it has nothing to collapse to.
    NoIcon,
}

impl Kind {
    /// Its name in a test id.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Kind::Collapsed => "collapsed",
            Kind::NoIcon => "no-icon",
            Kind::Expanded => "expanded",
        }
    }

    /// Whether the FAB has an icon.
    #[must_use]
    pub const fn has_icon(self) -> bool {
        !matches!(self, Kind::NoIcon)
    }

    /// Whether the FAB is collapsed to its icon.
    #[must_use]
    pub const fn is_collapsed(self) -> bool {
        matches!(self, Kind::Collapsed)
    }
}

/// The top of the grid: where the FABs' row ends, at the margin the page's bottom had before this grid.
#[must_use]
pub fn top() -> Points {
    Points(fab::viewport().1.0 - MARGIN.0)
}

/// The height of an extended FAB of `size`: its `ContainerHeight` token.
#[must_use]
pub fn height(size: ExtendedFabSize) -> Points {
    match size {
        ExtendedFabSize::Medium => extended_fab_medium_tokens::CONTAINER_HEIGHT,
        ExtendedFabSize::Large => extended_fab_large_tokens::CONTAINER_HEIGHT,
        // `ExtendedFabSize` is non-exhaustive: a size added later falls back to the default one until the catalog shows
        // it.
        _ => extended_fab_small_tokens::CONTAINER_HEIGHT,
    }
}

/// The space before the icon of an extended FAB of `size` (or before its label, with no icon): its `LeadingSpace` token.
#[must_use]
pub fn leading_space(size: ExtendedFabSize) -> Points {
    match size {
        ExtendedFabSize::Medium => extended_fab_medium_tokens::LEADING_SPACE,
        ExtendedFabSize::Large => extended_fab_large_tokens::LEADING_SPACE,
        _ => extended_fab_small_tokens::LEADING_SPACE,
    }
}

/// The space after the label of an extended FAB of `size`: its `TrailingSpace` token.
#[must_use]
pub fn trailing_space(size: ExtendedFabSize) -> Points {
    match size {
        ExtendedFabSize::Medium => extended_fab_medium_tokens::TRAILING_SPACE,
        ExtendedFabSize::Large => extended_fab_large_tokens::TRAILING_SPACE,
        _ => extended_fab_small_tokens::TRAILING_SPACE,
    }
}

/// The side of the icon of an extended FAB of `size`: its `IconSize` token.
#[must_use]
pub fn icon_size(size: ExtendedFabSize) -> Points {
    match size {
        ExtendedFabSize::Medium => extended_fab_medium_tokens::ICON_SIZE,
        ExtendedFabSize::Large => extended_fab_large_tokens::ICON_SIZE,
        _ => extended_fab_small_tokens::ICON_SIZE,
    }
}

/// The height of the grid and of the margin above its first row, and so of every row's gap: the rows' heights, each
/// with a [`ROW_GAP`] after it.
#[must_use]
pub fn block_height() -> Points {
    let mut total = 0.0_f32;
    for size in SIZES {
        total += height(size).0 + ROW_GAP.0;
    }
    Points(total)
}

/// The size of the viewport the catalog is opened in to show the extended FABs, width and height: the page's width, as
/// the grid is narrower than the buttons', and the height of the page with this grid and the margin below it.
#[must_use]
pub fn viewport() -> (Points, Points) {
    (
        grid::viewport().0,
        Points(top().0 + block_height().0 + MARGIN.0),
    )
}

/// One extended FAB of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ExtendedFabCell {
    /// Its color style.
    pub style: FabStyle,
    /// Its size.
    pub size: ExtendedFabSize,
    /// What it shows.
    pub kind: Kind,
}

impl ExtendedFabCell {
    /// The extended FAB of this style, size and kind.
    #[must_use]
    pub const fn new(style: FabStyle, size: ExtendedFabSize, kind: Kind) -> Self {
        Self { style, size, kind }
    }

    /// Every extended FAB of the catalog, row by row.
    pub fn all() -> impl Iterator<Item = Self> {
        SIZES.into_iter().flat_map(|size| {
            STYLES.into_iter().flat_map(move |style| {
                KINDS
                    .into_iter()
                    .map(move |kind| Self::new(style, size, kind))
            })
        })
    }

    /// The name of its style in a test id and a screenshot's file name.
    #[must_use]
    pub fn style_name(self) -> &'static str {
        match self.style {
            FabStyle::SecondaryContainer => "secondary-container",
            // `FabStyle` is non-exhaustive; the default style is the primary container one.
            _ => "primary-container",
        }
    }

    /// The name of its size in a test id.
    #[must_use]
    pub fn size_name(self) -> &'static str {
        match self.size {
            ExtendedFabSize::Medium => "medium",
            ExtendedFabSize::Large => "large",
            _ => "small",
        }
    }

    /// The test id of the view around it, which the test scripts name
    /// (`id:extended-fab-primary-container-medium-expanded`).
    #[must_use]
    pub fn test_id(self) -> String {
        format!(
            "extended-fab-{}-{}-{}",
            self.style_name(),
            self.size_name(),
            self.kind.name()
        )
    }

    /// Its height, in points.
    #[must_use]
    pub fn height(self) -> Points {
        height(self.size)
    }

    /// The top left corner of its container in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let column =
            grid::position(&STYLES, self.style) * KINDS.len() + grid::position(&KINDS, self.kind);
        let mut x = MARGIN.0;
        for _ in 0..column {
            x += COLUMN_WIDTH.0;
        }
        let mut y = top().0;
        for size in SIZES.into_iter().take(grid::position(&SIZES, self.size)) {
            y += height(size).0 + ROW_GAP.0;
        }
        (Points(x), Points(y))
    }
}

/// `page` with every extended FAB of the catalog at its place, each in a view with its test id.
#[must_use]
pub fn place(page: View) -> View {
    ExtendedFabCell::all().fold(page, |page, cell| {
        let (x, y) = cell.origin();
        let expansion = if cell.kind.is_collapsed() {
            ExtendedFabExpansion::Collapsed
        } else {
            ExtendedFabExpansion::Expanded
        };
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .test_id(&cell.test_id())
                .child(
                    extended_fab()
                        .label(LABEL)
                        .icon(cell.kind.has_icon().then_some(ICON))
                        .style(cell.style)
                        .size(cell.size)
                        .expansion(expansion),
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

    /// The extended FAB test script, `extended_fab.test.tsx`, which opens the catalog at [`viewport`] and takes the
    /// screenshots `visual-checks` reads.
    const SCRIPT: &str = include_str!("../extended_fab.test.tsx");

    #[test]
    fn every_extended_fab_is_once_in_the_catalog() {
        let cells: Vec<ExtendedFabCell> = ExtendedFabCell::all().collect();
        assert_eq!(cells.len(), STYLES.len() * SIZES.len() * KINDS.len());
        assert_eq!(cells.len(), 18);
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
    fn the_extended_fabs_are_in_a_grid_below_the_fabs() {
        // The FABs' row ends at the margin before the FABs' viewport does: the grid starts there.
        assert_eq!(top(), Points(fab::viewport().1.0 - 24.0));
        assert_eq!(top(), Points(2984.0));
        let first = ExtendedFabCell::new(
            FabStyle::PrimaryContainer,
            ExtendedFabSize::Small,
            Kind::Expanded,
        );
        assert_eq!(first.origin(), (Points(24.0), top()));
        // The kinds are the columns of a style, 360 points apart; the second style starts after the third kind.
        let collapsed = ExtendedFabCell {
            kind: Kind::Collapsed,
            ..first
        };
        assert_eq!(collapsed.origin(), (Points(384.0), top()));
        let no_icon = ExtendedFabCell {
            kind: Kind::NoIcon,
            ..first
        };
        assert_eq!(no_icon.origin(), (Points(744.0), top()));
        let secondary = ExtendedFabCell {
            style: FabStyle::SecondaryContainer,
            ..first
        };
        assert_eq!(secondary.origin(), (Points(1104.0), top()));
        // The sizes are the rows, a FAB's height and a gap apart.
        let medium = ExtendedFabCell {
            size: ExtendedFabSize::Medium,
            ..first
        };
        assert_eq!(
            medium.origin(),
            (Points(24.0), Points(top().0 + 56.0 + 48.0))
        );
        let large = ExtendedFabCell {
            size: ExtendedFabSize::Large,
            ..first
        };
        assert_eq!(
            large.origin(),
            (Points(24.0), Points(top().0 + 56.0 + 48.0 + 80.0 + 48.0))
        );
        let last = ExtendedFabCell::new(
            FabStyle::SecondaryContainer,
            ExtendedFabSize::Large,
            Kind::NoIcon,
        );
        assert_eq!(last.origin(), (Points(1824.0), large.origin().1));
    }

    #[test]
    fn the_viewport_holds_the_grid_and_the_blocks_above() {
        let (width, height) = viewport();
        // The page is as wide as the buttons' grid; the grid's three rows and their gaps and the margin below make it
        // 376 points taller than the FABs' viewport, whose margin it replaces.
        assert_eq!((width, height), (Points(2608.0), Points(3384.0)));
        assert_eq!(width, grid::viewport().0);
        assert_eq!(block_height(), Points(376.0));
        assert_eq!(height.0, fab::viewport().1.0 + block_height().0);
        let mut bottom = Points(0.0);
        for cell in ExtendedFabCell::all() {
            let (x, y) = cell.origin();
            // A column holds its FAB with the room the checks read in on both sides, and the grid ends inside the page.
            assert!(x.0 >= MARGIN.0, "{cell:?}");
            assert!(x.0 + COLUMN_WIDTH.0 <= width.0 - MARGIN.0, "{cell:?}");
            assert!(y.0 >= fab::viewport().1.0 - MARGIN.0, "{cell:?}");
            assert!(y.0 + cell.height().0 <= height.0 - MARGIN.0, "{cell:?}");
            bottom = Points(bottom.0.max(y.0 + cell.height().0));
        }
        // The last row and its gap end at the margin, as the other grids'.
        assert_eq!(bottom.0 + ROW_GAP.0, height.0 - MARGIN.0);
        // The gap is more than the shadows reach below a container (20 points, ADR 0018) and the room the checks read in.
        const { assert!(ROW_GAP.0 >= 20.0 + 8.0) };
        // No extended FAB takes the place of a FAB or of anything above it.
        for cell in fab::FabCell::all() {
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
            "await app.screenshot(`target/visual/extended-fab-${scheme}-rest.png`);",
            "await app.hover(`id:extended-fab-${style}-medium-expanded`);",
            "`target/visual/extended-fab-${scheme}-hover-${style}.png`",
            "for (const scheme of [\"light\", \"dark\"] as const) {",
            "for (const size of SIZES) {",
            "for (const kind of KINDS) {",
        ] {
            assert!(SCRIPT.contains(line), "the script lacks: {line}");
        }
        // What it loops over are the grid's names, in the grid's order.
        for name in ["primary-container", "secondary-container"] {
            assert!(SCRIPT.contains(&format!("\"{name}\"")), "{name}");
        }
        let sizes: Vec<&str> = SIZES
            .into_iter()
            .map(|size| {
                ExtendedFabCell {
                    size,
                    ..ExtendedFabCell::default()
                }
                .size_name()
            })
            .collect();
        assert_eq!(sizes, ["small", "medium", "large"]);
        assert!(SCRIPT.contains("const SIZES = [\"small\", \"medium\", \"large\"];"));
        let kinds: Vec<&str> = KINDS.into_iter().map(Kind::name).collect();
        assert_eq!(kinds, ["expanded", "collapsed", "no-icon"]);
        assert!(SCRIPT.contains("const KINDS = [\"expanded\", \"collapsed\", \"no-icon\"];"));
    }

    #[test]
    fn heights_and_spaces_are_their_tokens() {
        let heights: Vec<f32> = SIZES.into_iter().map(|size| height(size).0).collect();
        assert_eq!(heights, [56.0, 80.0, 96.0]);
        let leading: Vec<f32> = SIZES
            .into_iter()
            .map(|size| leading_space(size).0)
            .collect();
        assert_eq!(leading, [16.0, 26.0, 28.0]);
        let trailing: Vec<f32> = SIZES
            .into_iter()
            .map(|size| trailing_space(size).0)
            .collect();
        assert_eq!(trailing, [16.0, 26.0, 28.0]);
        let icons: Vec<f32> = SIZES.into_iter().map(|size| icon_size(size).0).collect();
        assert_eq!(icons, [24.0, 28.0, 32.0]);
        let cell = ExtendedFabCell::new(
            FabStyle::SecondaryContainer,
            ExtendedFabSize::Medium,
            Kind::Collapsed,
        );
        assert_eq!(cell.height(), Points(80.0));
    }

    #[test]
    fn kinds_say_what_the_fab_shows() {
        assert!(Kind::Expanded.has_icon() && !Kind::Expanded.is_collapsed());
        assert!(Kind::Collapsed.has_icon() && Kind::Collapsed.is_collapsed());
        assert!(!Kind::NoIcon.has_icon() && !Kind::NoIcon.is_collapsed());
        assert_eq!(Kind::default(), Kind::Expanded);
    }

    #[test]
    fn names_and_ids_name_the_extended_fab() {
        let cell = ExtendedFabCell::new(
            FabStyle::SecondaryContainer,
            ExtendedFabSize::Large,
            Kind::NoIcon,
        );
        assert_eq!(cell.style_name(), "secondary-container");
        assert_eq!(cell.size_name(), "large");
        assert_eq!(
            cell.test_id(),
            "extended-fab-secondary-container-large-no-icon"
        );
        let default = ExtendedFabCell::default();
        assert_eq!(
            default.test_id(),
            "extended-fab-primary-container-small-expanded"
        );
        let ids: HashSet<String> = ExtendedFabCell::all()
            .map(ExtendedFabCell::test_id)
            .collect();
        assert!(ids.contains("extended-fab-primary-container-medium-collapsed"));
        assert!(ids.contains("extended-fab-secondary-container-small-expanded"));
    }

    /// Every extended FAB is at its place on the grid, inside a view with its test id, as high as its size says, as wide
    /// as it is high when collapsed and wider when it is not, within its column; it takes presses: it is a button for
    /// screen readers, named by its label, never disabled.
    #[test]
    fn every_extended_fab_is_where_the_grid_says() {
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
        for cell in ExtendedFabCell::all() {
            let id = ids[&cell.test_id()];
            let (x, y) = cell.origin();
            assert_eq!(
                (frames[&id].origin.x, frames[&id].origin.y),
                (x, y),
                "{cell:?}"
            );
            let button = child_of(id);
            let size = frames[&button].size;
            assert_eq!(size.height, cell.height(), "{cell:?}");
            if cell.kind.is_collapsed() {
                assert_eq!(size.width, cell.height(), "{cell:?}");
            } else {
                // Wider than the square, by at least what its paddings and its icon take, and inside its column with
                // the room for the shadows and the surface read beside the next FAB.
                let icon = if cell.kind.has_icon() {
                    icon_size(cell.size).0
                } else {
                    0.0
                };
                let content = leading_space(cell.size).0 + icon + trailing_space(cell.size).0;
                assert!(size.width.0 > content, "{cell:?} is {size:?}");
                assert!(size.width.0 > cell.height().0, "{cell:?} is {size:?}");
                assert!(
                    size.width.0 <= COLUMN_WIDTH.0 - 2.0 * 22.0,
                    "{cell:?} is {size:?}"
                );
            }
            let (disabled, label) = &buttons[&button];
            assert!(!disabled, "{cell:?}");
            assert_eq!(label.as_deref(), Some(LABEL), "{cell:?}");
        }
    }
}
