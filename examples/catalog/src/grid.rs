//! Where the catalog puts each button, so that the visual checks know where to read its pixels.
//!
//! The buttons sit on a grid: one column per style and shape, one row per size, the enabled rows above the disabled
//! ones. Each button's top left corner is at a fixed place, whatever the width of its label.

use nsh_core::geometry::Points;
use nsh_m3e::button::{ButtonShape, ButtonSize, ButtonStyle};
use nsh_m3e::tokens::androidx::{
    button_large_tokens, button_medium_tokens, button_small_tokens, button_xlarge_tokens,
    button_xsmall_tokens,
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

/// The size of the viewport the catalog is opened in, width and height: the grid and its margins.
#[must_use]
pub fn viewport() -> (Points, Points) {
    let mut width = MARGIN.0 * 2.0;
    for _ in 0..STYLES.len() * SHAPES.len() {
        width += COLUMN_WIDTH.0;
    }
    let mut height = MARGIN.0 * 2.0;
    for _ in AVAILABILITIES {
        for size in SIZES {
            height += self::height(size).0 + ROW_GAP.0;
        }
    }
    (Points(width), Points(height))
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
    fn the_viewport_holds_the_grid_and_its_margins() {
        let (width, height) = viewport();
        assert_eq!((width, height), (Points(2608.0), Points(928.0)));
        for cell in Cell::all() {
            let (x, y) = cell.origin();
            assert!(x.0 + COLUMN_WIDTH.0 <= width.0 - MARGIN.0);
            assert!(y.0 + super::height(cell.size).0 <= height.0 - MARGIN.0);
        }
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
}
