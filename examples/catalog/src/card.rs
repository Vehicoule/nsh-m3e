//! Where the catalog puts each card, so that the visual checks know where to read its pixels.
//!
//! The cards sit in a grid below the FABs' row (see [`crate::fab`]), the last block of the page: one row per style, and
//! in a row a card that is only a container, a card that takes presses, and one that is disabled. Every card holds
//! the same box, [`WIDTH`] by [`HEIGHT`], with its name in the top left corner, so that all nine are as large as each
//! other. A card casts a shadow (ADR 0018), so the cards are [`GAP`] apart, which leaves room for the shadows and for
//! the surface beyond them, and [`viewport`] is the page's size with this grid.
//!
//! The grid's top is where the FABs' row ends, a gap below it: `button.test.tsx`, `switch.test.tsx`,
//! `checkbox.test.tsx` and `fab.test.tsx` open the page at viewports that stop above it; `card.test.tsx` opens it at
//! [`viewport`].

use nsh_core::element::AdaptiveColor;
use nsh_core::geometry::Points;
use nsh_core::prelude::*;
use nsh_m3e::card::{CardStyle, card, clickable_card};
use nsh_m3e::color::ColorScheme;

use crate::fab;
use crate::grid::{self, MARGIN};

/// Every style, in the order of the rows.
pub const STYLES: [CardStyle; 3] = [CardStyle::Filled, CardStyle::Elevated, CardStyle::Outlined];

/// Every kind, in the order of the columns.
pub const KINDS: [CardKind; 3] = [CardKind::Static, CardKind::Clickable, CardKind::Disabled];

/// The width of the box every card holds, in points. A layout value of the catalog, not of Material.
pub const WIDTH: Points = Points(160.0);

/// The height of the box every card holds, in points. A layout value of the catalog, not of Material.
pub const HEIGHT: Points = Points(96.0);

/// The space between two cards, to the side and below: far enough that the surface is the same color between two
/// cards, past the reach of both their shadows, and that the shadow below a card ends before the next row (the highest
/// level a card sits at, level 2, reaches 8 points to the sides and 10 below, ADR 0018). A layout value of the
/// catalog, not of Material.
pub const GAP: Points = Points(48.0);

/// The padding between the box's edge and the card's name.
const PADDING: Points = Points(16.0);

/// What a card of the catalog does.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum CardKind {
    /// A container with no behavior (`card()`).
    #[default]
    Static,
    /// A card that takes presses (`clickable_card()`).
    Clickable,
    /// A card that takes presses, disabled (`clickable_card().disabled(true)`).
    Disabled,
}

/// The width of a column: a card and the gap after it.
#[must_use]
pub fn column_width() -> Points {
    Points(WIDTH.0 + GAP.0)
}

/// The distance between the tops of two rows: a card and the gap below it.
#[must_use]
pub fn row_pitch() -> Points {
    Points(HEIGHT.0 + GAP.0)
}

/// The top of the cards' grid: where the FABs' row ends, a gap below it (the bottom margin of [`fab::viewport`]
/// starts where the row and its row gap end).
#[must_use]
pub fn top() -> Points {
    Points(fab::viewport().1.0 - MARGIN.0 + GAP.0)
}

/// The height of the grid and the gap below its last row, which holds the last shadows.
#[must_use]
pub fn block_height() -> Points {
    let mut height = 0.0_f32;
    for _ in STYLES {
        height += row_pitch().0;
    }
    Points(height)
}

/// The size of the viewport the catalog is opened in to show the cards, width and height: the page's width, as the
/// grid is narrower than the buttons', and the height of the page with the grid and the margin below it.
#[must_use]
pub fn viewport() -> (Points, Points) {
    (
        grid::viewport().0,
        Points(top().0 + block_height().0 + MARGIN.0),
    )
}

/// One card of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CardCell {
    /// Its style.
    pub style: CardStyle,
    /// What it does.
    pub kind: CardKind,
}

impl CardCell {
    /// The card of this style and kind.
    #[must_use]
    pub const fn new(style: CardStyle, kind: CardKind) -> Self {
        Self { style, kind }
    }

    /// Every card of the catalog, row by row.
    pub fn all() -> impl Iterator<Item = Self> {
        STYLES
            .into_iter()
            .flat_map(|style| KINDS.into_iter().map(move |kind| Self::new(style, kind)))
    }

    /// The name of its style in a test id and a screenshot's file name.
    #[must_use]
    pub fn style_name(self) -> &'static str {
        match self.style {
            CardStyle::Elevated => "elevated",
            CardStyle::Outlined => "outlined",
            // `CardStyle` is non-exhaustive; the default style is the filled one.
            _ => "filled",
        }
    }

    /// The name of its kind in a test id.
    #[must_use]
    pub fn kind_name(self) -> &'static str {
        match self.kind {
            CardKind::Clickable => "clickable",
            CardKind::Disabled => "disabled",
            // `CardKind` is non-exhaustive; the default kind is the static one.
            _ => "static",
        }
    }

    /// Its name, which it shows in its top left corner and which screen readers say for the cards that take presses.
    /// No two are alike, and none is the label of a button of the catalog.
    #[must_use]
    pub fn label(self) -> String {
        let style = match self.style {
            CardStyle::Elevated => "Elevated",
            CardStyle::Outlined => "Outlined",
            _ => "Filled",
        };
        match self.kind {
            CardKind::Clickable => format!("{style} card, clickable"),
            CardKind::Disabled => format!("{style} card, disabled"),
            _ => format!("{style} card"),
        }
    }

    /// The test id of the view around it, which the test scripts name (`id:card-filled-clickable`).
    #[must_use]
    pub fn test_id(self) -> String {
        format!("card-{}-{}", self.style_name(), self.kind_name())
    }

    /// Its width and height, in points: the box it holds.
    #[must_use]
    pub fn frame(self) -> (Points, Points) {
        (WIDTH, HEIGHT)
    }

    /// The top left corner of its container in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let column = grid::position(&KINDS, self.kind);
        let row = grid::position(&STYLES, self.style);
        let mut x = MARGIN.0;
        for _ in 0..column {
            x += column_width().0;
        }
        let mut y = top().0;
        for _ in 0..row {
            y += row_pitch().0;
        }
        (Points(x), Points(y))
    }
}

/// The color of the text of a card's name: the scheme's on surface, which is the content color of all three
/// containers (`ColorScheme.contentColorFor`). Written out here because the catalog has no token module of its own.
fn name_color() -> AdaptiveColor {
    AdaptiveColor::rgb(
        ColorScheme::BASELINE_LIGHT.on_surface.to_hex(),
        ColorScheme::BASELINE_DARK.on_surface.to_hex(),
    )
}

/// The card `cell`: the box its content fills, with its name in the top left corner.
fn element(cell: CardCell) -> AnyElement {
    let content = view()
        .w(WIDTH)
        .h(HEIGHT)
        .p(PADDING)
        .child(text(cell.label()).text_color(name_color()));
    match cell.kind {
        CardKind::Clickable => clickable_card()
            .style(cell.style)
            .child(content)
            .into_element(),
        CardKind::Disabled => clickable_card()
            .style(cell.style)
            .disabled(true)
            .child(content)
            .into_element(),
        _ => card().style(cell.style).child(content).into_element(),
    }
}

/// `page` with every card of the catalog at its place, each in a view with its test id.
#[must_use]
pub fn place(page: View) -> View {
    CardCell::all().fold(page, |page, cell| {
        let (x, y) = cell.origin();
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .test_id(&cell.test_id())
                .child(element(cell)),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex, PoisonError};

    use nsh_core::geometry::Bounds;
    use nsh_core::platform::{ColorScheme as Appearance, Environment};
    use nsh_core::{AccessibilityRole, Color, MountItem, NodeId};
    use nsh_engine::{EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::Catalog;

    /// The card test script, `card.test.tsx`, which opens the catalog at [`viewport`] and takes the screenshots
    /// `visual-checks` reads.
    const SCRIPT: &str = include_str!("../card.test.tsx");

    #[test]
    fn every_card_is_once_in_the_catalog() {
        let cells: Vec<CardCell> = CardCell::all().collect();
        assert_eq!(cells.len(), STYLES.len() * KINDS.len());
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
        assert_eq!(
            cells
                .iter()
                .map(|c| c.label())
                .collect::<HashSet<_>>()
                .len(),
            cells.len()
        );
    }

    #[test]
    fn the_cards_are_in_a_grid_below_the_fabs() {
        // The FABs' row ends at the margin before the FABs' viewport does; the cards start a gap below it.
        assert_eq!(top(), Points(fab::viewport().1.0 - 24.0 + 48.0));
        assert_eq!(top(), Points(3032.0));
        let first = CardCell::new(CardStyle::Filled, CardKind::Static);
        assert_eq!(first.origin(), (Points(24.0), top()));
        // The kinds are the columns of a row, 208 points apart; the styles are the rows, 144 points apart.
        let clickable = CardCell {
            kind: CardKind::Clickable,
            ..first
        };
        assert_eq!(clickable.origin(), (Points(232.0), top()));
        let disabled = CardCell {
            kind: CardKind::Disabled,
            ..first
        };
        assert_eq!(disabled.origin(), (Points(440.0), top()));
        let elevated = CardCell {
            style: CardStyle::Elevated,
            ..first
        };
        assert_eq!(elevated.origin(), (Points(24.0), Points(3176.0)));
        let last = CardCell::new(CardStyle::Outlined, CardKind::Disabled);
        assert_eq!(last.origin(), (Points(440.0), Points(3320.0)));
    }

    #[test]
    fn the_viewport_holds_the_cards_and_the_grids_above() {
        let (width, height) = viewport();
        // The page is as wide as the buttons' grid; the cards' grid is three rows of a card and a gap, and the margin
        // below it.
        assert_eq!((width, height), (Points(2608.0), Points(3488.0)));
        assert_eq!(width, grid::viewport().0);
        assert_eq!(block_height(), Points(432.0));
        assert_eq!(height.0, top().0 + block_height().0 + MARGIN.0);
        let mut bottom = Points(0.0);
        for cell in CardCell::all() {
            let (x, y) = cell.origin();
            let (cell_width, cell_height) = cell.frame();
            assert!(x.0 >= MARGIN.0, "{cell:?}");
            assert!(x.0 + cell_width.0 <= width.0 - MARGIN.0, "{cell:?}");
            assert!(y.0 >= top().0, "{cell:?}");
            assert!(y.0 + cell_height.0 <= height.0 - MARGIN.0, "{cell:?}");
            bottom = Points(bottom.0.max(y.0 + cell_height.0));
        }
        // A gap below the last row, then the margin.
        assert_eq!(bottom.0 + GAP.0, height.0 - MARGIN.0);
        // No card takes the place of a FAB, a checkbox, a switch, a button or an icon button.
        // A gap or more below every FAB: the highest FAB's shadow reaches 20 points below it (ADR 0018).
        for cell in fab::FabCell::all() {
            assert!(cell.origin().1.0 + cell.frame().1.0 + GAP.0 <= top().0);
        }
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
            "await app.screenshot(`target/visual/card-${scheme}-rest.png`);",
            "await app.hover(`id:card-${style}-clickable`);",
            "`target/visual/card-${scheme}-hover-${style}.png`",
            "for (const scheme of [\"light\", \"dark\"] as const) {",
        ] {
            assert!(SCRIPT.contains(line), "the script lacks: {line}");
        }
        // The cards it looks for are the grid's, in the grid's names.
        let ids: Vec<String> = CardCell::all().map(CardCell::test_id).collect();
        assert_eq!(
            ids,
            [
                "card-filled-static",
                "card-filled-clickable",
                "card-filled-disabled",
                "card-elevated-static",
                "card-elevated-clickable",
                "card-elevated-disabled",
                "card-outlined-static",
                "card-outlined-clickable",
                "card-outlined-disabled",
            ]
        );
        for name in ["filled", "elevated", "outlined"] {
            assert!(SCRIPT.contains(&format!("\"{name}\"")), "{name}");
        }
        for name in ["static", "clickable", "disabled"] {
            assert!(SCRIPT.contains(&format!("\"{name}\"")), "{name}");
        }
    }

    #[test]
    fn labels_and_ids_name_the_card() {
        let cell = CardCell::new(CardStyle::Elevated, CardKind::Clickable);
        assert_eq!(cell.label(), "Elevated card, clickable");
        assert_eq!(cell.style_name(), "elevated");
        assert_eq!(cell.kind_name(), "clickable");
        assert_eq!(cell.test_id(), "card-elevated-clickable");
        let labels: Vec<String> = KINDS
            .into_iter()
            .map(|kind| CardCell { kind, ..cell }.label())
            .collect();
        assert_eq!(
            labels,
            [
                "Elevated card",
                "Elevated card, clickable",
                "Elevated card, disabled"
            ]
        );
        let styles: Vec<String> = STYLES
            .into_iter()
            .map(|style| CardCell { style, ..cell }.test_id())
            .collect();
        assert_eq!(
            styles,
            [
                "card-filled-clickable",
                "card-elevated-clickable",
                "card-outlined-clickable",
            ]
        );
        assert_eq!(cell.frame(), (Points(160.0), Points(96.0)));
    }

    /// Every card is at its place on the grid, inside a view with its test id, as large as the box it holds; the
    /// cards that take presses are buttons for screen readers, disabled or not as their kind says, and the static
    /// one is not.
    #[test]
    fn every_card_is_where_the_grid_says() {
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
                        buttons.insert(*id, props.accessibility.state.disabled);
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
        for cell in CardCell::all() {
            let id = ids[&cell.test_id()];
            let (x, y) = cell.origin();
            assert_eq!(
                (frames[&id].origin.x, frames[&id].origin.y),
                (x, y),
                "{cell:?}"
            );
            let container = child_of(id);
            let (width, height) = cell.frame();
            assert_eq!(
                (
                    frames[&container].size.width,
                    frames[&container].size.height
                ),
                (width, height),
                "{cell:?}"
            );
            let button = buttons.get(&container).copied();
            assert_eq!(
                button,
                match cell.kind {
                    CardKind::Clickable => Some(false),
                    CardKind::Disabled => Some(true),
                    _ => None,
                },
                "{cell:?}"
            );
        }
    }

    /// Every card shows its name in the on-surface color of the appearance, the content color of all three
    /// containers, so that it reads on each of them.
    #[test]
    fn every_card_names_itself_in_the_on_surface_color_of_each_appearance() {
        for (appearance, scheme) in [
            (Appearance::Light, ColorScheme::BASELINE_LIGHT),
            (Appearance::Dark, ColorScheme::BASELINE_DARK),
        ] {
            let mut app = Headless::start(EngineConfig {
                environment: Environment {
                    color_scheme: appearance,
                    ..Environment::default()
                },
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
            app.wait_for("Filled").unwrap();
            let items = items.lock().unwrap_or_else(PoisonError::into_inner).clone();
            let names: HashMap<String, Color> = items
                .iter()
                .filter_map(|item| match item {
                    MountItem::Text { text, .. } => text
                        .spans
                        .first()
                        .map(|span| (text.text.clone(), span.style.color)),
                    _ => None,
                })
                .collect();
            let on_surface = Color((scheme.on_surface.to_hex() << 8) | 0xff);
            for cell in CardCell::all() {
                assert_eq!(
                    names.get(&cell.label()),
                    Some(&on_surface),
                    "{cell:?} in {appearance:?}"
                );
            }
        }
    }
}
