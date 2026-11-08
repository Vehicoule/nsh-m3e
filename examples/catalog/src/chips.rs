//! Where the catalog puts each chip, so that the visual checks know where to read its pixels, and the chips
//! themselves.
//!
//! The chips sit in a block of their own below the grids of [`crate::grid`] (the buttons' and the icon buttons'): one
//! row per kind, selection and availability (the assist and suggestion chips are never selected), one column per
//! content (the label alone, an icon before it, an icon after it, both, an avatar, an avatar and an icon after it). A
//! kind shows only the contents it takes: the suggestion chip has one icon, before its label, and only the input chip
//! has an avatar. A chip's top left corner is at a fixed place, whatever the width of its label.
//!
//! [`crate::grid::viewport`] stays the viewport of the buttons' screenshots (`button.test.tsx`), and `chips.test.tsx`
//! opens the catalog in [`viewport`], which holds this block as well: the same cells at the same places, with the
//! block below the bottom edge of the smaller one.
//!
//! The chips are the flat ones. Elevation is decided by ADR 0018 and written by its own change: no chip here is
//! elevated.

use nsh_core::element::AdaptiveColor;
use nsh_core::geometry::Points;
use nsh_core::prelude::*;
use nsh_m3e::assist_chip::assist_chip;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::filter_chip::filter_chip;
use nsh_m3e::input_chip::input_chip;
use nsh_m3e::suggestion_chip::suggestion_chip;
use nsh_m3e::tokens::androidx::{
    assist_chip_tokens, filter_chip_tokens, input_chip_tokens, suggestion_chip_tokens,
};

use crate::ICON;
use crate::grid::{self, AVAILABILITIES, Availability, MARGIN, ROW_GAP};

/// The color role the avatar of an input chip is filled with: the tertiary color, which no chip color is, so that the
/// visual checks tell the avatar from the container and the icons.
pub const AVATAR_ROLE: ColorRole = ColorRole::Tertiary;

/// The width of a column: more than the widest chip, the suggestion chip with an icon or the input chip with an avatar
/// and an icon after it (about 110 points with the label of the catalog). A layout value of the catalog, not of
/// Material.
pub const COLUMN_WIDTH: Points = Points(160.0);

/// The space before the leading icon of every kind of chip: `ContentPadding` of `AssistChipDefaults` (androidx
/// `Chip.kt` line 1962), `FilterChipDefaults` (2269) and `SuggestionChipDefaults` (2915), and `contentPadding` of
/// `InputChipDefaults` with an icon (2683). A copy of the library's value on purpose: the catalog's test checks the
/// layout against it, and the visual checks read the icon where it says.
pub const ICON_PADDING: Points = Points(8.0);

/// The space before the avatar of an input chip: `InputChipDefaults.contentPadding` with an avatar (`Chip.kt` line
/// 2683). A copy of the library's value on purpose, as [`ICON_PADDING`] is.
pub const AVATAR_PADDING: Points = Points(4.0);

/// A kind of chip.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ChipKind {
    /// The assist chip: an action.
    #[default]
    Assist,
    /// The filter chip: a selection that filters.
    Filter,
    /// The input chip: a piece of information the user entered.
    Input,
    /// The suggestion chip: an offer.
    Suggestion,
}

/// Every kind, in the order of the row blocks.
pub const KINDS: [ChipKind; 4] = [
    ChipKind::Assist,
    ChipKind::Filter,
    ChipKind::Input,
    ChipKind::Suggestion,
];

impl ChipKind {
    /// Its label, which the chips of the catalog say: the name of the kind.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Filter => "Filter",
            Self::Input => "Input",
            Self::Suggestion => "Suggestion",
            // `ChipKind` is non-exhaustive; the default kind is the assist chip.
            _ => "Assist",
        }
    }

    /// Its height: its `ContainerHeight` token.
    #[must_use]
    pub fn height(self) -> Points {
        match self {
            Self::Filter => filter_chip_tokens::CONTAINER_HEIGHT,
            Self::Input => input_chip_tokens::CONTAINER_HEIGHT,
            Self::Suggestion => suggestion_chip_tokens::CONTAINER_HEIGHT,
            // `ChipKind` is non-exhaustive; the default kind is the assist chip.
            _ => assist_chip_tokens::CONTAINER_HEIGHT,
        }
    }

    /// The selections it shows: only the filter and input chips are ever selected.
    #[must_use]
    pub fn selections(self) -> &'static [Selection] {
        match self {
            Self::Filter | Self::Input => &SELECTIONS,
            _ => &[Selection::Unselected],
        }
    }

    /// The contents it shows, in the order of the columns.
    #[must_use]
    pub fn contents(self) -> &'static [ChipContent] {
        match self {
            Self::Assist | Self::Filter => &[
                ChipContent::Label,
                ChipContent::LeadingIcon,
                ChipContent::TrailingIcon,
                ChipContent::BothIcons,
            ],
            Self::Input => &CONTENTS,
            // The suggestion chip has one icon, before its label.
            _ => &[ChipContent::Label, ChipContent::LeadingIcon],
        }
    }

    /// Its name in a test id.
    fn name(self) -> &'static str {
        match self {
            Self::Filter => "filter",
            Self::Input => "input",
            Self::Suggestion => "suggestion",
            _ => "assist",
        }
    }
}

/// Whether a chip is selected.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Selection {
    /// It is not selected.
    #[default]
    Unselected,
    /// It is selected.
    Selected,
}

/// Every selection, in the order of the rows within a kind.
pub const SELECTIONS: [Selection; 2] = [Selection::Unselected, Selection::Selected];

/// What a chip shows besides its label.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ChipContent {
    /// Nothing.
    #[default]
    Label,
    /// An icon before the label.
    LeadingIcon,
    /// An icon after the label.
    TrailingIcon,
    /// An icon before the label and one after it.
    BothIcons,
    /// An avatar before the label.
    Avatar,
    /// An avatar before the label and an icon after it.
    AvatarAndTrailingIcon,
}

/// Every content, in the order of the columns.
pub const CONTENTS: [ChipContent; 6] = [
    ChipContent::Label,
    ChipContent::LeadingIcon,
    ChipContent::TrailingIcon,
    ChipContent::BothIcons,
    ChipContent::Avatar,
    ChipContent::AvatarAndTrailingIcon,
];

impl ChipContent {
    /// Whether the chip has an icon before its label.
    #[must_use]
    pub const fn has_leading_icon(self) -> bool {
        matches!(self, Self::LeadingIcon | Self::BothIcons)
    }

    /// Whether the chip has an icon after its label.
    #[must_use]
    pub const fn has_trailing_icon(self) -> bool {
        matches!(
            self,
            Self::TrailingIcon | Self::BothIcons | Self::AvatarAndTrailingIcon
        )
    }

    /// Whether the chip has an avatar.
    #[must_use]
    pub const fn has_avatar(self) -> bool {
        matches!(self, Self::Avatar | Self::AvatarAndTrailingIcon)
    }

    /// Its name in a test id.
    fn name(self) -> &'static str {
        match self {
            Self::LeadingIcon => "leading-icon",
            Self::TrailingIcon => "trailing-icon",
            Self::BothIcons => "both-icons",
            Self::Avatar => "avatar",
            Self::AvatarAndTrailingIcon => "avatar-trailing-icon",
            _ => "label",
        }
    }
}

/// One chip of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ChipCell {
    /// Its kind.
    pub kind: ChipKind,
    /// Whether it is selected.
    pub selection: Selection,
    /// What it shows besides its label.
    pub content: ChipContent,
    /// Whether it takes presses.
    pub availability: Availability,
}

/// The rows of the grid, top to bottom: each kind's selections, each with the enabled chips then the disabled ones.
fn rows() -> impl Iterator<Item = (ChipKind, Selection, Availability)> {
    KINDS.into_iter().flat_map(|kind| {
        kind.selections()
            .iter()
            .copied()
            .flat_map(move |selection| {
                AVAILABILITIES
                    .into_iter()
                    .map(move |availability| (kind, selection, availability))
            })
    })
}

impl ChipCell {
    /// The chip of this kind, selection, content and availability.
    #[must_use]
    pub const fn new(
        kind: ChipKind,
        selection: Selection,
        content: ChipContent,
        availability: Availability,
    ) -> Self {
        Self {
            kind,
            selection,
            content,
            availability,
        }
    }

    /// Every chip of the catalog, row by row.
    pub fn all() -> impl Iterator<Item = Self> {
        rows().flat_map(|(kind, selection, availability)| {
            kind.contents()
                .iter()
                .copied()
                .map(move |content| Self::new(kind, selection, content, availability))
        })
    }

    /// Its label: the name of its kind.
    #[must_use]
    pub fn label(self) -> &'static str {
        self.kind.label()
    }

    /// The test id of the view around it, which the test scripts name (`id:chip-filter-selected-label-enabled`).
    #[must_use]
    pub fn test_id(self) -> String {
        let selection = match self.selection {
            Selection::Selected => "selected",
            _ => "unselected",
        };
        let availability = match self.availability {
            Availability::Disabled => "disabled",
            Availability::Enabled => "enabled",
        };
        format!(
            "chip-{}-{selection}-{}-{availability}",
            self.kind.name(),
            self.content.name()
        )
    }

    /// The top left corner of its container in the block, in points.
    #[must_use]
    pub fn position(self) -> (Points, Points) {
        let mut x = MARGIN.0;
        for _ in 0..position(&CONTENTS, self.content) {
            x += COLUMN_WIDTH.0;
        }
        let mut y = 0.0;
        for (kind, selection, availability) in rows() {
            if (kind, selection, availability) == (self.kind, self.selection, self.availability) {
                break;
            }
            y += kind.height().0 + ROW_GAP.0;
        }
        (Points(x), Points(y))
    }

    /// The top left corner of its container in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let (x, y) = self.position();
        (x, Points(top().0 + y.0))
    }

    /// The side of the avatar or icon before its label, if it has one: the avatar's `AvatarSize` token, or the icon's
    /// size (the leading icon's of its kind's tokens). An avatar takes precedence over an icon, and no chip has both.
    #[must_use]
    pub fn leading_side(self) -> Option<Points> {
        if self.content.has_avatar() {
            Some(input_chip_tokens::AVATAR_SIZE)
        } else if self.content.has_leading_icon() {
            Some(match self.kind {
                ChipKind::Filter => filter_chip_tokens::ICON_SIZE,
                ChipKind::Input => input_chip_tokens::LEADING_ICON_SIZE,
                ChipKind::Suggestion => suggestion_chip_tokens::LEADING_ICON_SIZE,
                _ => assist_chip_tokens::ICON_SIZE,
            })
        } else {
            None
        }
    }

    /// The top left corner of the avatar or icon before its label in the viewport, in points: after the chip's
    /// padding, centered on its height.
    #[must_use]
    pub fn leading_origin(self) -> Option<(Points, Points)> {
        let side = self.leading_side()?;
        let padding = if self.content.has_avatar() {
            AVATAR_PADDING
        } else {
            ICON_PADDING
        };
        let (x, y) = self.origin();
        Some((
            Points(x.0 + padding.0),
            Points(y.0 + (self.kind.height().0 - side.0) / 2.0),
        ))
    }
}

/// The test id of the view that holds the block, which the tests name.
pub const BLOCK_TEST_ID: &str = "chips";

/// The top of the block: the bottom of the viewport of the buttons' screenshots, so below every button and icon
/// button and the margin under them.
#[must_use]
pub fn top() -> Points {
    grid::viewport().1
}

/// The height of the block: the rows of chips and the gaps between them.
#[must_use]
pub fn block_height() -> Points {
    let mut height = 0.0;
    for (kind, _, _) in rows() {
        height += kind.height().0 + ROW_GAP.0;
    }
    Points(height - ROW_GAP.0)
}

/// The size of the viewport the chips' screenshots are taken in, width and height: the buttons' with the block and a
/// margin below it. The buttons' grid is the wider, so the block's columns fit in its width.
#[must_use]
pub fn viewport() -> (Points, Points) {
    let (width, _) = grid::viewport();
    (width, Points(top().0 + block_height().0 + MARGIN.0))
}

/// The color of the avatar of an input chip: [`AVATAR_ROLE`] in each appearance.
fn avatar_color() -> AdaptiveColor {
    AdaptiveColor::rgb(
        ColorScheme::BASELINE_LIGHT.get(AVATAR_ROLE).to_hex(),
        ColorScheme::BASELINE_DARK.get(AVATAR_ROLE).to_hex(),
    )
}

/// The chip of `cell`.
fn chip(cell: ChipCell) -> AnyElement {
    let disabled = cell.availability == Availability::Disabled;
    let selected = cell.selection == Selection::Selected;
    let leading = cell.content.has_leading_icon().then_some(ICON);
    let trailing = cell.content.has_trailing_icon().then_some(ICON);
    match cell.kind {
        ChipKind::Filter => filter_chip()
            .label(cell.label())
            .selected(selected)
            .leading_icon(leading)
            .trailing_icon(trailing)
            .disabled(disabled)
            .into_element(),
        ChipKind::Input => {
            let chip = input_chip()
                .label(cell.label())
                .selected(selected)
                .leading_icon(leading)
                .trailing_icon(trailing)
                .disabled(disabled);
            if cell.content.has_avatar() {
                // Fills the avatar's circle: the avatar is a plain color, so that the visual checks read it.
                chip.avatar(view().size_full().bg(avatar_color()))
            } else {
                chip
            }
            .into_element()
        }
        ChipKind::Suggestion => suggestion_chip()
            .label(cell.label())
            .icon(leading)
            .disabled(disabled)
            .into_element(),
        _ => assist_chip()
            .label(cell.label())
            .leading_icon(leading)
            .trailing_icon(trailing)
            .disabled(disabled)
            .into_element(),
    }
}

/// The block of the catalog's chips, at [`top`], each at its [`ChipCell::position`] inside it: the block is as large as
/// the cells, so that the chips lie within the view that holds them (a view takes no press outside its parent's bounds
/// on every host).
#[must_use]
pub fn block() -> impl IntoElement {
    let (width, _) = viewport();
    ChipCell::all().fold(
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
                    // A row, so that the chip is as wide as its content, whatever the view around it does.
                    .child(view().flex_row().child(chip(cell))),
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
    use nsh_core::{AccessibilityRole, MountItem, NodeId, ViewKind};
    use nsh_engine::{EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::Catalog;
    use crate::grid;

    fn cell(kind: ChipKind, selection: Selection, content: ChipContent) -> ChipCell {
        ChipCell::new(kind, selection, content, Availability::Enabled)
    }

    #[test]
    fn every_chip_is_once_in_the_catalog() {
        let cells: Vec<ChipCell> = ChipCell::all().collect();
        // The assist chip: 4 contents, enabled and disabled. The filter chip: 4 contents, 2 selections, 2
        // availabilities. The input chip: 6 contents, 2 and 2. The suggestion chip: 2 contents, enabled and disabled.
        assert_eq!(cells.len(), 4 * 2 + 4 * 2 * 2 + 6 * 2 * 2 + 2 * 2);
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
    fn a_kind_shows_what_it_takes() {
        // Only the filter and input chips are selected.
        for cell in ChipCell::all() {
            if !matches!(cell.kind, ChipKind::Filter | ChipKind::Input) {
                assert_eq!(cell.selection, Selection::Unselected, "{cell:?}");
            }
            // Only the input chip has an avatar; only the suggestion chip lacks an icon after its label.
            if cell.kind != ChipKind::Input {
                assert!(!cell.content.has_avatar(), "{cell:?}");
            }
            if cell.kind == ChipKind::Suggestion {
                assert!(!cell.content.has_trailing_icon(), "{cell:?}");
            }
        }
        // And each kind shows both selections where it has them, and every content it takes.
        for kind in [ChipKind::Filter, ChipKind::Input] {
            let selections: HashSet<_> = ChipCell::all()
                .filter(|c| c.kind == kind)
                .map(|c| c.selection)
                .collect();
            assert_eq!(selections.len(), 2, "{kind:?}");
        }
        for kind in KINDS {
            let contents: Vec<_> = ChipCell::all()
                .filter(|c| c.kind == kind && c.selection == Selection::Unselected)
                .filter(|c| c.availability == Availability::Enabled)
                .map(|c| c.content)
                .collect();
            assert_eq!(contents, kind.contents(), "{kind:?}");
        }
    }

    #[test]
    fn the_rows_and_columns_are_where_the_layout_says() {
        let first = cell(ChipKind::Assist, Selection::Unselected, ChipContent::Label);
        // Below the buttons' viewport: 2,688 points.
        assert_eq!(top(), Points(2688.0));
        assert_eq!(first.position(), (Points(24.0), Points(0.0)));
        assert_eq!(first.origin(), (Points(24.0), Points(2688.0)));
        // The leading icon is the second column, both icons the fourth, the avatar the fifth.
        let leading = ChipCell {
            content: ChipContent::LeadingIcon,
            ..first
        };
        assert_eq!(leading.origin(), (Points(184.0), Points(2688.0)));
        let avatar = cell(ChipKind::Input, Selection::Unselected, ChipContent::Avatar);
        assert_eq!(avatar.origin().0, Points(24.0 + 4.0 * 160.0));
        // The disabled assist row is below the enabled one (32 points and the gap).
        let disabled = ChipCell {
            availability: Availability::Disabled,
            ..first
        };
        assert_eq!(disabled.origin(), (Points(24.0), Points(2736.0)));
        // The filter chips follow the two assist rows, unselected then selected, enabled then disabled.
        let filter = cell(ChipKind::Filter, Selection::Unselected, ChipContent::Label);
        assert_eq!(filter.origin(), (Points(24.0), Points(2784.0)));
        let selected = ChipCell {
            selection: Selection::Selected,
            ..filter
        };
        assert_eq!(selected.origin(), (Points(24.0), Points(2880.0)));
        // The last row: the disabled suggestion chips, after 11 rows of 32 points and a gap.
        let last = ChipCell::new(
            ChipKind::Suggestion,
            Selection::Unselected,
            ChipContent::LeadingIcon,
            Availability::Disabled,
        );
        assert_eq!(last.position(), (Points(184.0), Points(11.0 * 48.0)));
        assert_eq!(last.origin(), (Points(184.0), Points(2688.0 + 11.0 * 48.0)));
        // 12 rows of 32 points and the 11 gaps between them.
        assert_eq!(block_height(), Points(12.0 * 32.0 + 11.0 * 16.0));
    }

    #[test]
    fn the_viewport_holds_the_block_below_the_buttons_viewport_and_a_margin() {
        let (width, height) = viewport();
        // The buttons' width, and their 2,688 points of height with the block's 560 and a margin of 24.
        assert_eq!((width, height), (Points(2608.0), Points(3272.0)));
        assert_eq!(width, grid::viewport().0);
        // The buttons' own screenshots keep their viewport.
        assert_eq!(grid::viewport(), (Points(2608.0), Points(2688.0)));
        let mut bottom = Points(0.0);
        for cell in ChipCell::all() {
            let (x, y) = cell.origin();
            assert!(x.0 >= MARGIN.0, "{cell:?}");
            assert!(x.0 + COLUMN_WIDTH.0 <= width.0 - MARGIN.0, "{cell:?}");
            // Under the buttons' viewport, so below every button and icon button.
            assert!(y.0 >= grid::viewport().1.0, "{cell:?}");
            assert!(
                y.0 + cell.kind.height().0 <= height.0 - MARGIN.0,
                "{cell:?}"
            );
            bottom = Points(bottom.0.max(y.0 + cell.kind.height().0));
        }
        // The last row ends where the block does, a margin above the viewport's bottom.
        assert_eq!(bottom.0, top().0 + block_height().0);
        assert_eq!(bottom.0 + MARGIN.0, height.0);
        // The buttons' and icon buttons' cells all end above the block.
        for cell in crate::grid::Cell::all() {
            assert!(cell.origin().1.0 < top().0);
        }
        for cell in crate::grid::IconCell::all() {
            assert!(cell.origin().1.0 + cell.frame().1.0 <= top().0 - MARGIN.0);
        }
    }

    #[test]
    fn labels_ids_and_sizes_name_the_chip() {
        let cell = ChipCell::new(
            ChipKind::Input,
            Selection::Selected,
            ChipContent::AvatarAndTrailingIcon,
            Availability::Disabled,
        );
        assert_eq!(cell.label(), "Input");
        assert_eq!(
            cell.test_id(),
            "chip-input-selected-avatar-trailing-icon-disabled"
        );
        let labels: Vec<&str> = KINDS.into_iter().map(ChipKind::label).collect();
        assert_eq!(labels, ["Assist", "Filter", "Input", "Suggestion"]);
        let heights: Vec<f32> = KINDS.into_iter().map(|k| k.height().0).collect();
        assert_eq!(heights, [32.0; 4]);
        assert_eq!(
            ChipCell::new(
                ChipKind::Assist,
                Selection::Unselected,
                ChipContent::Label,
                Availability::Enabled
            )
            .test_id(),
            "chip-assist-unselected-label-enabled"
        );
    }

    #[test]
    fn the_leading_part_is_after_the_padding_and_centered() {
        let icon = cell(
            ChipKind::Filter,
            Selection::Unselected,
            ChipContent::BothIcons,
        );
        let (x, y) = icon.origin();
        assert_eq!(icon.leading_side(), Some(Points(18.0)));
        assert_eq!(
            icon.leading_origin(),
            Some((Points(x.0 + 8.0), Points(y.0 + 7.0)))
        );
        let avatar = cell(ChipKind::Input, Selection::Selected, ChipContent::Avatar);
        let (x, y) = avatar.origin();
        assert_eq!(avatar.leading_side(), Some(Points(24.0)));
        assert_eq!(
            avatar.leading_origin(),
            Some((Points(x.0 + 4.0), Points(y.0 + 4.0)))
        );
        // No icon, no avatar: nothing before the label. An icon after it is not before it.
        for content in [ChipContent::Label, ChipContent::TrailingIcon] {
            assert_eq!(
                cell(ChipKind::Assist, Selection::Unselected, content).leading_origin(),
                None
            );
        }
        // The icon's side is its kind's token: 18 points for every kind, and the avatar's 24.
        for kind in KINDS {
            let cell = cell(kind, Selection::Unselected, ChipContent::LeadingIcon);
            assert_eq!(cell.leading_side(), Some(Points(18.0)), "{kind:?}");
        }
    }

    /// What the engine mounts for the catalog: the views by test id, their kinds, frames and children, and whether
    /// each view with the button role is disabled.
    struct Page {
        ids: HashMap<String, NodeId>,
        kinds: HashMap<NodeId, ViewKind>,
        frames: HashMap<NodeId, Bounds<Points>>,
        disabled: HashMap<NodeId, bool>,
        children: HashMap<NodeId, Vec<NodeId>>,
    }

    fn mounted() -> Page {
        let mut app = Headless::start(EngineConfig::new(Root::of::<Catalog>())).unwrap();
        let items: Arc<Mutex<Vec<MountItem>>> = Arc::default();
        let sink = Arc::clone(&items);
        app.observe(move |_, batch| {
            sink.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .extend(batch.items.iter().cloned());
        });
        // The first column of the buttons is on the headless screen; the rest of the page is mounted with it.
        app.wait_for("Filled").unwrap();
        let items = items.lock().unwrap_or_else(PoisonError::into_inner).clone();

        let mut page = Page {
            ids: HashMap::new(),
            kinds: HashMap::new(),
            frames: HashMap::new(),
            disabled: HashMap::new(),
            children: HashMap::new(),
        };
        for item in &items {
            match item {
                MountItem::Create { id, kind } => {
                    page.kinds.insert(*id, *kind);
                }
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        page.ids.insert(test_id.to_string(), *id);
                    }
                    if props.accessibility.role == AccessibilityRole::Button {
                        page.disabled
                            .insert(*id, props.accessibility.state.disabled);
                    }
                }
                MountItem::Frame { id, frame } => {
                    page.frames.insert(*id, *frame);
                }
                MountItem::Insert {
                    parent,
                    child,
                    before,
                } => {
                    // An insert of a view that is already in a parent moves it.
                    for siblings in page.children.values_mut() {
                        siblings.retain(|id| id != child);
                    }
                    let siblings: &mut Vec<NodeId> = page.children.entry(*parent).or_default();
                    let at = before
                        .and_then(|before| siblings.iter().position(|id| *id == before))
                        .unwrap_or(siblings.len());
                    siblings.insert(at, *child);
                }
                MountItem::Remove { id } | MountItem::Delete { id } => {
                    for siblings in page.children.values_mut() {
                        siblings.retain(|child| child != id);
                    }
                }
                _ => {}
            }
        }
        page
    }

    /// What `nsh test` would draw for the chips, from the engine's mount items: every chip is at its place on the
    /// grid, 32 points tall and narrower than its column, enabled or disabled as its cell says, with its icon or avatar
    /// where the cell says and its label after it.
    #[test]
    fn every_chip_is_where_the_grid_says() {
        let Page {
            ids,
            kinds,
            frames,
            disabled,
            children,
        } = mounted();
        // The block holds every chip: it starts below the buttons' viewport, at the left edge of the page.
        let block = frames[&ids[BLOCK_TEST_ID]];
        assert_eq!((block.origin.x, block.origin.y), (Points(0.0), top()));
        assert_eq!(
            (block.size.width, block.size.height),
            (viewport().0, block_height())
        );
        for cell in ChipCell::all() {
            let wrapper = ids[&cell.test_id()];
            let (x, y) = cell.origin();
            // Each chip's place is relative to the block, which is at `top()` on the page.
            let (px, py) = cell.position();
            assert_eq!(
                (frames[&wrapper].origin.x, frames[&wrapper].origin.y),
                (px, py),
                "{cell:?}"
            );
            assert_eq!((x, y), (px, Points(top().0 + py.0)), "{cell:?}");
            assert!(
                children[&ids[BLOCK_TEST_ID]].contains(&wrapper),
                "{cell:?} is not in the block"
            );
            // The row, then the chip, which has the button role.
            let row = children[&wrapper][0];
            let chip = children[&row][0];
            assert_eq!(
                disabled[&chip],
                cell.availability == Availability::Disabled,
                "{cell:?}"
            );
            let frame = frames[&chip];
            assert_eq!(
                (frame.origin.x, frame.origin.y),
                (Points(0.0), Points(0.0)),
                "{cell:?}"
            );
            assert_eq!(frame.size.height, cell.kind.height(), "{cell:?}");
            assert!(frame.size.width.0 < COLUMN_WIDTH.0, "{cell:?}");
            // The container and the state layer come first, then the avatar or icon before the label.
            let parts: Vec<(NodeId, ViewKind)> = children[&chip]
                .iter()
                .map(|part| (*part, kinds[part]))
                .collect();
            assert!(
                matches!(parts[..2], [(_, ViewKind::Canvas), (_, ViewKind::Canvas)]),
                "{cell:?}: {parts:?}"
            );
            let leading =
                (cell.content.has_avatar() || cell.content.has_leading_icon()).then(|| parts[2]);
            match (cell.leading_origin(), cell.leading_side(), leading) {
                (Some((lx, ly)), Some(side), Some((part, kind))) => {
                    let expected_kind = if cell.content.has_avatar() {
                        ViewKind::View
                    } else {
                        ViewKind::Canvas
                    };
                    assert_eq!(kind, expected_kind, "{cell:?}");
                    let frame = frames[&part];
                    assert_eq!(
                        (frame.origin.x.0, frame.origin.y.0),
                        (lx.0 - x.0, ly.0 - y.0),
                        "{cell:?}"
                    );
                    assert_eq!(
                        (frame.size.width, frame.size.height),
                        (side, side),
                        "{cell:?}"
                    );
                }
                (None, None, None) => {
                    // The label comes right after the container and the state layer.
                    assert_eq!(parts[2].1, ViewKind::Text, "{cell:?}: {parts:?}");
                }
                other => panic!("{cell:?}: {other:?}"),
            }
        }
    }
}
