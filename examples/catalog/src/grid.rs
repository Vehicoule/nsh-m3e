//! Where the catalog puts each button and icon button, so that the visual checks know where to read its pixels.
//!
//! The buttons sit on a grid: one column per style and shape, one row per size, the enabled rows above the disabled
//! ones. Each button's top left corner is at a fixed place, whatever the width of its label.
//!
//! The icon buttons sit on a second grid below the first: one column per style and width, one row per size, in four
//! blocks of rows, enabled then disabled, round then square in each.
//!
//! The floating action buttons sit in one row below the second: one column per style and size, a style's four sizes
//! side by side, their tops aligned.

use nsh_core::geometry::Points;
use nsh_m3e::button::{ButtonShape, ButtonSize, ButtonStyle};
use nsh_m3e::fab::{FabSize, FabStyle};
use nsh_m3e::icon_button::{IconButtonShape, IconButtonSize, IconButtonStyle, IconButtonWidth};
use nsh_m3e::tokens::androidx::{
    button_large_tokens, button_medium_tokens, button_small_tokens, button_xlarge_tokens,
    button_xsmall_tokens, fab_baseline_tokens, fab_large_tokens, fab_medium_tokens,
    fab_small_tokens, large_icon_button_tokens, medium_icon_button_tokens,
    small_icon_button_tokens, xlarge_icon_button_tokens, xsmall_icon_button_tokens,
};

/// Every style, in the order of the columns.
pub const STYLES: [ButtonStyle; 4] = [
    ButtonStyle::Filled,
    ButtonStyle::Tonal,
    ButtonStyle::Outlined,
    ButtonStyle::Text,
];

/// Every shape, in the order of the columns within a style.
pub const SHAPES: [ButtonShape; 2] = [ButtonShape::Round, ButtonShape::Square];

/// Every size, in the order of the rows.
pub const SIZES: [ButtonSize; 5] = [
    ButtonSize::ExtraSmall,
    ButtonSize::Small,
    ButtonSize::Medium,
    ButtonSize::Large,
    ButtonSize::ExtraLarge,
];

/// Whether a button in the catalog takes presses.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Availability {
    /// It takes presses.
    #[default]
    Enabled,
    /// It is disabled.
    Disabled,
}

/// Every availability, in the order of the row blocks.
pub const AVAILABILITIES: [Availability; 2] = [Availability::Enabled, Availability::Disabled];

/// The space around the grid. A layout value of the catalog, not of Material: the catalog is a test page.
pub const MARGIN: Points = Points(24.0);

/// The width of a column: more than the widest button, the extra large outlined one (64 points on each side of a
/// 32-point label). A layout value of the catalog, not of Material.
pub const COLUMN_WIDTH: Points = Points(320.0);

/// The space between two rows. A layout value of the catalog, not of Material.
pub const ROW_GAP: Points = Points(16.0);

/// Every icon button style, in the order of the columns: the same order as the buttons', the most emphasis first.
pub const ICON_STYLES: [IconButtonStyle; 4] = [
    IconButtonStyle::Filled,
    IconButtonStyle::Tonal,
    IconButtonStyle::Outlined,
    IconButtonStyle::Standard,
];

/// Every icon button width, in the order of the columns within a style.
pub const ICON_WIDTHS: [IconButtonWidth; 3] = [
    IconButtonWidth::Narrow,
    IconButtonWidth::Uniform,
    IconButtonWidth::Wide,
];

/// Every icon button shape, in the order of the row blocks within an availability.
pub const ICON_SHAPES: [IconButtonShape; 2] = [IconButtonShape::Round, IconButtonShape::Square];

/// Every icon button size, in the order of the rows within a block.
pub const ICON_SIZES: [IconButtonSize; 5] = [
    IconButtonSize::ExtraSmall,
    IconButtonSize::Small,
    IconButtonSize::Medium,
    IconButtonSize::Large,
    IconButtonSize::ExtraLarge,
];

/// The width of an icon button column: more than the widest icon button, the extra large wide one (72 points on each
/// side of a 40-point icon). A layout value of the catalog, not of Material.
pub const ICON_COLUMN_WIDTH: Points = Points(200.0);

/// Every FAB style, in the order of the columns.
pub const FAB_STYLES: [FabStyle; 2] = [FabStyle::PrimaryContainer, FabStyle::SecondaryContainer];

/// Every FAB size, in the order of the columns within a style.
pub const FAB_SIZES: [FabSize; 4] = [
    FabSize::Small,
    FabSize::Baseline,
    FabSize::Medium,
    FabSize::Large,
];

/// The width of a FAB column: more than the largest FAB (96 points), so that two FABs have surface between them. A
/// layout value of the catalog, not of Material.
pub const FAB_COLUMN_WIDTH: Points = Points(128.0);

/// The size of the viewport the catalog is opened in, width and height: the three grids and their margins. The
/// buttons' grid is the wider: 8 columns of 320 points, where the icon buttons' grid has 12 of 200 and the FABs' 8 of
/// 128.
#[must_use]
pub fn viewport() -> (Points, Points) {
    let mut width = MARGIN.0 * 2.0;
    for _ in 0..STYLES.len() * SHAPES.len() {
        width += COLUMN_WIDTH.0;
    }
    let mut height = icon_top().0 + MARGIN.0;
    for _ in AVAILABILITIES {
        for _ in ICON_SHAPES {
            for size in ICON_SIZES {
                height += icon_height(size).0 + ROW_GAP.0;
            }
        }
    }
    // The FABs' row: its tallest FAB, and its gap.
    let mut row = 0.0_f32;
    for size in FAB_SIZES {
        row = row.max(fab_frame(size).1.0);
    }
    height += row + ROW_GAP.0;
    (Points(width), Points(height))
}

/// The top of the icon buttons' grid: below the buttons' rows, each followed by its gap.
#[must_use]
pub fn icon_top() -> Points {
    let mut top = MARGIN.0;
    for _ in AVAILABILITIES {
        for size in SIZES {
            top += height(size).0 + ROW_GAP.0;
        }
    }
    Points(top)
}

/// The top of the FABs' row: below the icon buttons' rows, each followed by its gap.
#[must_use]
pub fn fab_top() -> Points {
    let mut top = icon_top().0;
    for _ in AVAILABILITIES {
        for _ in ICON_SHAPES {
            for size in ICON_SIZES {
                top += icon_height(size).0 + ROW_GAP.0;
            }
        }
    }
    Points(top)
}

/// The width and height of a FAB of `size`: its `ContainerWidth` and `ContainerHeight` tokens, as the FAB takes them.
#[must_use]
pub fn fab_frame(size: FabSize) -> (Points, Points) {
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

/// The height of an icon button of `size`: its `ContainerHeight` token, as the icon button takes it.
#[must_use]
pub fn icon_height(size: IconButtonSize) -> Points {
    match size {
        IconButtonSize::ExtraSmall => xsmall_icon_button_tokens::CONTAINER_HEIGHT,
        IconButtonSize::Medium => medium_icon_button_tokens::CONTAINER_HEIGHT,
        IconButtonSize::Large => large_icon_button_tokens::CONTAINER_HEIGHT,
        IconButtonSize::ExtraLarge => xlarge_icon_button_tokens::CONTAINER_HEIGHT,
        // `IconButtonSize` is non-exhaustive: a size added later falls back to the default one until the catalog
        // shows it.
        _ => small_icon_button_tokens::CONTAINER_HEIGHT,
    }
}

/// The size of the icon in an icon button of `size`: its `IconSize` token.
#[must_use]
pub fn icon_size(size: IconButtonSize) -> Points {
    match size {
        IconButtonSize::ExtraSmall => xsmall_icon_button_tokens::ICON_SIZE,
        IconButtonSize::Small => small_icon_button_tokens::ICON_SIZE,
        IconButtonSize::Medium => medium_icon_button_tokens::ICON_SIZE,
        IconButtonSize::Large => large_icon_button_tokens::ICON_SIZE,
        IconButtonSize::ExtraLarge => xlarge_icon_button_tokens::ICON_SIZE,
        // `IconButtonSize` is non-exhaustive: a size added later has no icon size here until the catalog shows it.
        // Not the small size's, as for the heights: the small and medium icons are both 24 points, and falling back
        // to one would hide the other's arm from the tests.
        _ => Points(0.0),
    }
}

/// The width of an icon button of `size` and `width`: its icon and the spaces on each side, as the icon button takes
/// them from its size's tokens (the narrow, default and wide spaces; `LargeIconButtonTokens` names its default ones
/// `UniformLeadingSpace`).
#[must_use]
pub fn icon_width(size: IconButtonSize, width: IconButtonWidth) -> Points {
    use IconButtonWidth::{Narrow, Wide};
    let (leading, trailing) = match (size, width) {
        (IconButtonSize::ExtraSmall, Narrow) => (
            xsmall_icon_button_tokens::NARROW_LEADING_SPACE,
            xsmall_icon_button_tokens::NARROW_TRAILING_SPACE,
        ),
        (IconButtonSize::ExtraSmall, Wide) => (
            xsmall_icon_button_tokens::WIDE_LEADING_SPACE,
            xsmall_icon_button_tokens::WIDE_TRAILING_SPACE,
        ),
        (IconButtonSize::ExtraSmall, _) => (
            xsmall_icon_button_tokens::DEFAULT_LEADING_SPACE,
            xsmall_icon_button_tokens::DEFAULT_LEADING_SPACE,
        ),
        (IconButtonSize::Medium, Narrow) => (
            medium_icon_button_tokens::NARROW_LEADING_SPACE,
            medium_icon_button_tokens::NARROW_TRAILING_SPACE,
        ),
        (IconButtonSize::Medium, Wide) => (
            medium_icon_button_tokens::WIDE_LEADING_SPACE,
            medium_icon_button_tokens::WIDE_TRAILING_SPACE,
        ),
        (IconButtonSize::Medium, _) => (
            medium_icon_button_tokens::DEFAULT_LEADING_SPACE,
            medium_icon_button_tokens::DEFAULT_LEADING_SPACE,
        ),
        (IconButtonSize::Large, Narrow) => (
            large_icon_button_tokens::NARROW_LEADING_SPACE,
            large_icon_button_tokens::NARROW_TRAILING_SPACE,
        ),
        (IconButtonSize::Large, Wide) => (
            large_icon_button_tokens::WIDE_LEADING_SPACE,
            large_icon_button_tokens::WIDE_TRAILING_SPACE,
        ),
        (IconButtonSize::Large, _) => (
            large_icon_button_tokens::UNIFORM_LEADING_SPACE,
            large_icon_button_tokens::UNIFORM_LEADING_SPACE,
        ),
        (IconButtonSize::ExtraLarge, Narrow) => (
            xlarge_icon_button_tokens::NARROW_LEADING_SPACE,
            xlarge_icon_button_tokens::NARROW_TRAILING_SPACE,
        ),
        (IconButtonSize::ExtraLarge, Wide) => (
            xlarge_icon_button_tokens::WIDE_LEADING_SPACE,
            xlarge_icon_button_tokens::WIDE_TRAILING_SPACE,
        ),
        (IconButtonSize::ExtraLarge, _) => (
            xlarge_icon_button_tokens::DEFAULT_LEADING_SPACE,
            xlarge_icon_button_tokens::DEFAULT_LEADING_SPACE,
        ),
        (_, Narrow) => (
            small_icon_button_tokens::NARROW_LEADING_SPACE,
            small_icon_button_tokens::NARROW_TRAILING_SPACE,
        ),
        (_, Wide) => (
            small_icon_button_tokens::WIDE_LEADING_SPACE,
            small_icon_button_tokens::WIDE_TRAILING_SPACE,
        ),
        (_, _) => (
            small_icon_button_tokens::DEFAULT_LEADING_SPACE,
            small_icon_button_tokens::DEFAULT_LEADING_SPACE,
        ),
    };
    Points(icon_size(size).0 + leading.0 + trailing.0)
}

/// The height of a button of `size`: its `ContainerHeight` token, as the button takes it.
#[must_use]
pub fn height(size: ButtonSize) -> Points {
    match size {
        ButtonSize::ExtraSmall => button_xsmall_tokens::CONTAINER_HEIGHT,
        ButtonSize::Medium => button_medium_tokens::CONTAINER_HEIGHT,
        ButtonSize::Large => button_large_tokens::CONTAINER_HEIGHT,
        ButtonSize::ExtraLarge => button_xlarge_tokens::CONTAINER_HEIGHT,
        // `ButtonSize` is non-exhaustive: a size added later falls back to the default one until the catalog shows
        // it.
        _ => button_small_tokens::CONTAINER_HEIGHT,
    }
}

/// One button of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Cell {
    /// Its color style.
    pub style: ButtonStyle,
    /// Its shape.
    pub shape: ButtonShape,
    /// Its size.
    pub size: ButtonSize,
    /// Whether it takes presses.
    pub availability: Availability,
}

impl Cell {
    /// The button of this style, shape, size and availability.
    #[must_use]
    pub const fn new(
        style: ButtonStyle,
        shape: ButtonShape,
        size: ButtonSize,
        availability: Availability,
    ) -> Self {
        Self {
            style,
            shape,
            size,
            availability,
        }
    }

    /// Every button of the catalog, column by column.
    pub fn all() -> impl Iterator<Item = Self> {
        STYLES.into_iter().flat_map(|style| {
            SHAPES.into_iter().flat_map(move |shape| {
                AVAILABILITIES.into_iter().flat_map(move |availability| {
                    SIZES
                        .into_iter()
                        .map(move |size| Self::new(style, shape, size, availability))
                })
            })
        })
    }

    /// Its label: the name of its style.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self.style {
            ButtonStyle::Tonal => "Tonal",
            ButtonStyle::Outlined => "Outlined",
            ButtonStyle::Text => "Text",
            // `ButtonStyle` is non-exhaustive; the default style is the filled one.
            _ => "Filled",
        }
    }

    /// The test id of the view around it, which the test scripts name (`id:filled-small-round-enabled`).
    #[must_use]
    pub fn test_id(self) -> String {
        let size = match self.size {
            ButtonSize::ExtraSmall => "extra-small",
            ButtonSize::Medium => "medium",
            ButtonSize::Large => "large",
            ButtonSize::ExtraLarge => "extra-large",
            _ => "small",
        };
        let shape = match self.shape {
            ButtonShape::Square => "square",
            _ => "round",
        };
        let availability = match self.availability {
            Availability::Disabled => "disabled",
            Availability::Enabled => "enabled",
        };
        format!(
            "{}-{size}-{shape}-{availability}",
            self.label().to_lowercase()
        )
    }

    /// The top left corner of its container in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let column = position(&STYLES, self.style) * SHAPES.len() + position(&SHAPES, self.shape);
        let mut x = MARGIN.0;
        for _ in 0..column {
            x += COLUMN_WIDTH.0;
        }
        let mut y = MARGIN.0;
        for availability in AVAILABILITIES {
            for size in SIZES {
                if (availability, size) == (self.availability, self.size) {
                    return (Points(x), Points(y));
                }
                y += height(size).0 + ROW_GAP.0;
            }
        }
        (Points(x), Points(y))
    }
}

/// One icon button of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct IconCell {
    /// Its color style.
    pub style: IconButtonStyle,
    /// Its width.
    pub width: IconButtonWidth,
    /// Its shape.
    pub shape: IconButtonShape,
    /// Its size.
    pub size: IconButtonSize,
    /// Whether it takes presses.
    pub availability: Availability,
}

impl IconCell {
    /// The icon button of this style, width, shape, size and availability.
    #[must_use]
    pub const fn new(
        style: IconButtonStyle,
        width: IconButtonWidth,
        shape: IconButtonShape,
        size: IconButtonSize,
        availability: Availability,
    ) -> Self {
        Self {
            style,
            width,
            shape,
            size,
            availability,
        }
    }

    /// Every icon button of the catalog, column by column.
    pub fn all() -> impl Iterator<Item = Self> {
        ICON_STYLES.into_iter().flat_map(|style| {
            ICON_WIDTHS.into_iter().flat_map(move |width| {
                AVAILABILITIES.into_iter().flat_map(move |availability| {
                    ICON_SHAPES.into_iter().flat_map(move |shape| {
                        ICON_SIZES
                            .into_iter()
                            .map(move |size| Self::new(style, width, shape, size, availability))
                    })
                })
            })
        })
    }

    /// Its label, which screen readers say: the name of its style.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self.style {
            IconButtonStyle::Filled => "Filled",
            IconButtonStyle::Tonal => "Tonal",
            IconButtonStyle::Outlined => "Outlined",
            // `IconButtonStyle` is non-exhaustive; the default style is the standard one.
            _ => "Standard",
        }
    }

    /// The test id of the view around it, which the test scripts name (`id:icon-filled-small-uniform-round-enabled`).
    #[must_use]
    pub fn test_id(self) -> String {
        let size = match self.size {
            IconButtonSize::ExtraSmall => "extra-small",
            IconButtonSize::Medium => "medium",
            IconButtonSize::Large => "large",
            IconButtonSize::ExtraLarge => "extra-large",
            _ => "small",
        };
        let width = match self.width {
            IconButtonWidth::Narrow => "narrow",
            IconButtonWidth::Wide => "wide",
            _ => "uniform",
        };
        let shape = match self.shape {
            IconButtonShape::Square => "square",
            _ => "round",
        };
        let availability = match self.availability {
            Availability::Disabled => "disabled",
            Availability::Enabled => "enabled",
        };
        format!(
            "icon-{}-{size}-{width}-{shape}-{availability}",
            self.label().to_lowercase()
        )
    }

    /// Its width and height, in points.
    #[must_use]
    pub fn frame(self) -> (Points, Points) {
        (icon_width(self.size, self.width), icon_height(self.size))
    }

    /// The top left corner of its container in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let column = position(&ICON_STYLES, self.style) * ICON_WIDTHS.len()
            + position(&ICON_WIDTHS, self.width);
        let mut x = MARGIN.0;
        for _ in 0..column {
            x += ICON_COLUMN_WIDTH.0;
        }
        let mut y = icon_top().0;
        for availability in AVAILABILITIES {
            for shape in ICON_SHAPES {
                for size in ICON_SIZES {
                    if (availability, shape, size) == (self.availability, self.shape, self.size) {
                        return (Points(x), Points(y));
                    }
                    y += icon_height(size).0 + ROW_GAP.0;
                }
            }
        }
        (Points(x), Points(y))
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
        FAB_STYLES.into_iter().flat_map(|style| {
            FAB_SIZES
                .into_iter()
                .map(move |size| Self::new(style, size))
        })
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
        fab_frame(self.size)
    }

    /// The top left corner of its container in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let column =
            position(&FAB_STYLES, self.style) * FAB_SIZES.len() + position(&FAB_SIZES, self.size);
        let mut x = MARGIN.0;
        for _ in 0..column {
            x += FAB_COLUMN_WIDTH.0;
        }
        (Points(x), fab_top())
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

    #[test]
    fn every_button_is_once_in_the_catalog() {
        let cells: Vec<Cell> = Cell::all().collect();
        assert_eq!(
            cells.len(),
            STYLES.len() * SHAPES.len() * SIZES.len() * AVAILABILITIES.len()
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
    fn the_rows_and_columns_are_where_the_layout_says() {
        let first = Cell::new(
            ButtonStyle::Filled,
            ButtonShape::Round,
            ButtonSize::ExtraSmall,
            Availability::Enabled,
        );
        assert_eq!(first.origin(), (Points(24.0), Points(24.0)));
        // Square is the second column of a style; Tonal the second style.
        let square = Cell {
            shape: ButtonShape::Square,
            ..first
        };
        assert_eq!(square.origin(), (Points(344.0), Points(24.0)));
        let tonal = Cell {
            style: ButtonStyle::Tonal,
            ..first
        };
        assert_eq!(tonal.origin(), (Points(664.0), Points(24.0)));
        // Below the extra small row (32 points and the gap).
        let small = Cell {
            size: ButtonSize::Small,
            ..first
        };
        assert_eq!(small.origin(), (Points(24.0), Points(72.0)));
        // Below the five enabled rows: 360 points of buttons and five gaps.
        let disabled = Cell {
            availability: Availability::Disabled,
            ..first
        };
        assert_eq!(disabled.origin(), (Points(24.0), Points(464.0)));
        let last = Cell::new(
            ButtonStyle::Text,
            ButtonShape::Square,
            ButtonSize::ExtraLarge,
            Availability::Disabled,
        );
        assert_eq!(last.origin(), (Points(2264.0), Points(752.0)));
    }

    #[test]
    fn the_viewport_holds_the_grids_and_their_margins() {
        let (width, height) = viewport();
        // The buttons' 8 columns of 320 points are wider than the icon buttons' 12 of 200 and the FABs' 8 of 128; below
        // the buttons' 880 points of rows, the icon buttons' 1,760, and the FABs' 96 with their gap.
        assert_eq!((width, height), (Points(2608.0), Points(2800.0)));
        for cell in Cell::all() {
            let (x, y) = cell.origin();
            assert!(x.0 + COLUMN_WIDTH.0 <= width.0 - MARGIN.0);
            assert!(y.0 + super::height(cell.size).0 <= icon_top().0 - ROW_GAP.0);
        }
        let mut bottom = Points(0.0);
        for cell in IconCell::all() {
            let (x, y) = cell.origin();
            let (cell_width, cell_height) = cell.frame();
            assert!(cell_width.0 < ICON_COLUMN_WIDTH.0, "{cell:?}");
            assert!(x.0 + ICON_COLUMN_WIDTH.0 <= width.0 - MARGIN.0);
            assert!(y.0 >= icon_top().0);
            assert!(y.0 + cell_height.0 <= height.0 - MARGIN.0);
            bottom = Points(bottom.0.max(y.0 + cell_height.0));
        }
        // The icon buttons' last row and its gap end where the FABs' row starts.
        assert_eq!(bottom.0 + ROW_GAP.0, fab_top().0);
        let mut bottom = Points(0.0);
        for cell in FabCell::all() {
            let (x, y) = cell.origin();
            let (cell_width, cell_height) = cell.frame();
            assert!(cell_width.0 < FAB_COLUMN_WIDTH.0, "{cell:?}");
            assert!(x.0 + FAB_COLUMN_WIDTH.0 <= width.0 - MARGIN.0);
            assert!(y.0 >= fab_top().0);
            bottom = Points(bottom.0.max(y.0 + cell_height.0));
        }
        // The last row and its gap end at the margin.
        assert_eq!(bottom.0 + ROW_GAP.0, height.0 - MARGIN.0);
    }

    #[test]
    fn every_icon_button_is_once_in_the_catalog() {
        let cells: Vec<IconCell> = IconCell::all().collect();
        assert_eq!(
            cells.len(),
            ICON_STYLES.len()
                * ICON_WIDTHS.len()
                * ICON_SHAPES.len()
                * ICON_SIZES.len()
                * AVAILABILITIES.len()
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
    fn the_icon_buttons_rows_and_columns_are_where_the_layout_says() {
        let first = IconCell::new(
            IconButtonStyle::Filled,
            IconButtonWidth::Narrow,
            IconButtonShape::Round,
            IconButtonSize::ExtraSmall,
            Availability::Enabled,
        );
        // Below the buttons: 24 points of margin, then 880 of rows and gaps.
        assert_eq!(icon_top(), Points(904.0));
        assert_eq!(first.origin(), (Points(24.0), Points(904.0)));
        // Uniform is the second column of a style, wide the third; Tonal the second style.
        let uniform = IconCell {
            width: IconButtonWidth::Uniform,
            ..first
        };
        assert_eq!(uniform.origin(), (Points(224.0), Points(904.0)));
        let wide = IconCell {
            width: IconButtonWidth::Wide,
            ..first
        };
        assert_eq!(wide.origin(), (Points(424.0), Points(904.0)));
        let tonal = IconCell {
            style: IconButtonStyle::Tonal,
            ..first
        };
        assert_eq!(tonal.origin(), (Points(624.0), Points(904.0)));
        // Below the extra small row (32 points and the gap).
        let small = IconCell {
            size: IconButtonSize::Small,
            ..first
        };
        assert_eq!(small.origin(), (Points(24.0), Points(952.0)));
        // Below the five round rows: 360 points of icon buttons and five gaps.
        let square = IconCell {
            shape: IconButtonShape::Square,
            ..first
        };
        assert_eq!(square.origin(), (Points(24.0), Points(1344.0)));
        // Below the round and square enabled blocks.
        let disabled = IconCell {
            availability: Availability::Disabled,
            ..first
        };
        assert_eq!(disabled.origin(), (Points(24.0), Points(1784.0)));
        let last = IconCell::new(
            IconButtonStyle::Standard,
            IconButtonWidth::Wide,
            IconButtonShape::Square,
            IconButtonSize::ExtraLarge,
            Availability::Disabled,
        );
        assert_eq!(last.origin(), (Points(2224.0), Points(2512.0)));
    }

    #[test]
    fn icon_button_sizes_are_their_tokens() {
        let heights: Vec<f32> = ICON_SIZES.into_iter().map(|s| icon_height(s).0).collect();
        assert_eq!(heights, [32.0, 40.0, 56.0, 96.0, 136.0]);
        let icons: Vec<f32> = ICON_SIZES.into_iter().map(|s| icon_size(s).0).collect();
        assert_eq!(icons, [20.0, 24.0, 24.0, 32.0, 40.0]);
        // The icon and the narrow, default or wide space on each side.
        let widths: Vec<[f32; 3]> = ICON_SIZES
            .into_iter()
            .map(|size| ICON_WIDTHS.map(|width| icon_width(size, width).0))
            .collect();
        assert_eq!(
            widths,
            [
                [28.0, 32.0, 40.0],
                [32.0, 40.0, 52.0],
                [48.0, 56.0, 72.0],
                [64.0, 96.0, 128.0],
                [104.0, 136.0, 184.0],
            ]
        );
        let cell = IconCell::new(
            IconButtonStyle::Filled,
            IconButtonWidth::Wide,
            IconButtonShape::Round,
            IconButtonSize::Medium,
            Availability::Enabled,
        );
        assert_eq!(cell.frame(), (Points(72.0), Points(56.0)));
    }

    #[test]
    fn labels_and_ids_name_the_icon_button() {
        let cell = IconCell::new(
            IconButtonStyle::Outlined,
            IconButtonWidth::Wide,
            IconButtonShape::Square,
            IconButtonSize::ExtraLarge,
            Availability::Disabled,
        );
        assert_eq!(cell.label(), "Outlined");
        assert_eq!(
            cell.test_id(),
            "icon-outlined-extra-large-wide-square-disabled"
        );
        let labels: Vec<&str> = ICON_STYLES
            .into_iter()
            .map(|style| IconCell { style, ..cell }.label())
            .collect();
        assert_eq!(labels, ["Filled", "Tonal", "Outlined", "Standard"]);
        let sizes: Vec<String> = ICON_SIZES
            .into_iter()
            .map(|size| IconCell { size, ..cell }.test_id())
            .collect();
        assert_eq!(
            sizes,
            [
                "icon-outlined-extra-small-wide-square-disabled",
                "icon-outlined-small-wide-square-disabled",
                "icon-outlined-medium-wide-square-disabled",
                "icon-outlined-large-wide-square-disabled",
                "icon-outlined-extra-large-wide-square-disabled",
            ]
        );
        let widths: Vec<String> = ICON_WIDTHS
            .into_iter()
            .map(|width| IconCell { width, ..cell }.test_id())
            .collect();
        assert_eq!(
            widths,
            [
                "icon-outlined-extra-large-narrow-square-disabled",
                "icon-outlined-extra-large-uniform-square-disabled",
                "icon-outlined-extra-large-wide-square-disabled",
            ]
        );
        assert_eq!(
            IconCell {
                shape: IconButtonShape::Round,
                availability: Availability::Enabled,
                ..cell
            }
            .test_id(),
            "icon-outlined-extra-large-wide-round-enabled"
        );
    }

    #[test]
    fn heights_are_the_size_tokens() {
        let heights: Vec<f32> = SIZES.into_iter().map(|s| height(s).0).collect();
        assert_eq!(heights, [32.0, 40.0, 56.0, 96.0, 136.0]);
    }

    #[test]
    fn labels_and_ids_name_the_button() {
        let cell = Cell::new(
            ButtonStyle::Outlined,
            ButtonShape::Square,
            ButtonSize::ExtraLarge,
            Availability::Disabled,
        );
        assert_eq!(cell.label(), "Outlined");
        assert_eq!(cell.test_id(), "outlined-extra-large-square-disabled");
        let labels: Vec<&str> = STYLES
            .into_iter()
            .map(|style| Cell { style, ..cell }.label())
            .collect();
        assert_eq!(labels, ["Filled", "Tonal", "Outlined", "Text"]);
        let sizes: Vec<String> = SIZES
            .into_iter()
            .map(|size| Cell { size, ..cell }.test_id())
            .collect();
        assert_eq!(
            sizes,
            [
                "outlined-extra-small-square-disabled",
                "outlined-small-square-disabled",
                "outlined-medium-square-disabled",
                "outlined-large-square-disabled",
                "outlined-extra-large-square-disabled",
            ]
        );
        assert_eq!(
            Cell {
                shape: ButtonShape::Round,
                availability: Availability::Enabled,
                ..cell
            }
            .test_id(),
            "outlined-extra-large-round-enabled"
        );
    }

    #[test]
    fn every_fab_is_once_in_the_catalog() {
        let cells: Vec<FabCell> = FabCell::all().collect();
        assert_eq!(cells.len(), FAB_STYLES.len() * FAB_SIZES.len());
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
    fn the_fabs_are_in_one_row_below_the_icon_buttons() {
        // Below the icon buttons: their top, and 24 points of margin and 1,760 of rows and gaps.
        assert_eq!(fab_top(), Points(2664.0));
        let first = FabCell::new(FabStyle::PrimaryContainer, FabSize::Small);
        assert_eq!(first.origin(), (Points(24.0), Points(2664.0)));
        // The sizes are the columns of a style, 128 points apart; the second style starts after the fourth size.
        let baseline = FabCell {
            size: FabSize::Baseline,
            ..first
        };
        assert_eq!(baseline.origin(), (Points(152.0), Points(2664.0)));
        let secondary = FabCell {
            style: FabStyle::SecondaryContainer,
            ..first
        };
        assert_eq!(secondary.origin(), (Points(536.0), Points(2664.0)));
        let last = FabCell::new(FabStyle::SecondaryContainer, FabSize::Large);
        assert_eq!(last.origin(), (Points(920.0), Points(2664.0)));
    }

    #[test]
    fn fab_sizes_are_their_tokens() {
        let frames: Vec<(f32, f32)> = FAB_SIZES
            .into_iter()
            .map(|size| {
                let (width, height) = fab_frame(size);
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
        let labels: Vec<&str> = FAB_STYLES
            .into_iter()
            .map(|style| FabCell { style, ..cell }.label())
            .collect();
        assert_eq!(labels, ["Primary container", "Secondary container"]);
        let sizes: Vec<String> = FAB_SIZES
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
}
