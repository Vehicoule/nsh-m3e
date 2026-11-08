//! Where the catalog puts each switch, and how it builds one, so that the visual checks know where to read its pixels.
//!
//! The switches sit on a third grid, below the icon buttons': one column per value and handle (off and on, each with
//! a plain handle and with an icon in it), one row per availability, enabled above disabled. Each switch's top left
//! corner is at a fixed place.

use nsh_core::geometry::Points;
use nsh_core::prelude::*;
use nsh_m3e::switch::switch;
use nsh_m3e::tokens::androidx::switch_tokens;

use crate::ICON;
use crate::grid::{self, AVAILABILITIES, Availability, MARGIN, ROW_GAP};

/// Whether a switch of the catalog is on.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Selection {
    /// It is off: the handle at the start of the track.
    #[default]
    Off,
    /// It is on: the handle at the end of the track.
    On,
}

/// Every value, in the order of the columns.
pub const SELECTIONS: [Selection; 2] = [Selection::Off, Selection::On];

/// What the handle of a switch of the catalog holds.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Content {
    /// Nothing: a small handle when the switch is off.
    #[default]
    Plain,
    /// [`ICON`], whatever the value: a catalog shows the same filled shape in both, so that the visual
    /// checks find its color in the middle of the handle (the check mark and the cross of Material's samples are
    /// thin).
    Icon,
}

/// Every handle content, in the order of the columns within a value.
pub const CONTENTS: [Content; 2] = [Content::Plain, Content::Icon];

/// The width of a column: more than a switch (52 points) and the state layer around its handle, which reaches 4 points
/// past each side of the track when the handle is at an end. A layout value of the catalog, not of Material.
pub const COLUMN_WIDTH: Points = Points(96.0);

/// The width of a switch of the catalog: the track's, `SwitchTokens.TrackWidth`.
pub const WIDTH: Points = switch_tokens::TRACK_WIDTH;

/// The height of a switch of the catalog: the track's, `SwitchTokens.TrackHeight`.
pub const HEIGHT: Points = switch_tokens::TRACK_HEIGHT;

/// The name screen readers say for every switch of the catalog. The test ids tell the switches apart.
pub const LABEL: &str = "Switch";

/// The top of the switches' grid: below the icon buttons' rows, each followed by its gap.
#[must_use]
pub fn top() -> Points {
    let mut top = grid::icon_top().0;
    for _ in AVAILABILITIES {
        for _ in grid::ICON_SHAPES {
            for size in grid::ICON_SIZES {
                top += grid::icon_height(size).0 + ROW_GAP.0;
            }
        }
    }
    Points(top)
}

/// The height of the switches' grid: each row and its gap. The gap is more than the 4 points the state layer reaches
/// past the track, so that the layers of two rows do not touch.
#[must_use]
pub fn block_height() -> Points {
    let mut height = 0.0;
    for _ in AVAILABILITIES {
        height += HEIGHT.0 + ROW_GAP.0;
    }
    Points(height)
}

/// One switch of the catalog.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SwitchCell {
    /// Whether it is on.
    pub selection: Selection,
    /// What its handle holds.
    pub content: Content,
    /// Whether it takes presses.
    pub availability: Availability,
}

impl SwitchCell {
    /// The switch with this value, handle content and availability.
    #[must_use]
    pub const fn new(selection: Selection, content: Content, availability: Availability) -> Self {
        Self {
            selection,
            content,
            availability,
        }
    }

    /// Every switch of the catalog, column by column.
    pub fn all() -> impl Iterator<Item = Self> {
        SELECTIONS.into_iter().flat_map(|selection| {
            CONTENTS.into_iter().flat_map(move |content| {
                AVAILABILITIES
                    .into_iter()
                    .map(move |availability| Self::new(selection, content, availability))
            })
        })
    }

    /// The test id of the view around it, which the test scripts name (`id:switch-off-plain-enabled`).
    #[must_use]
    pub fn test_id(self) -> String {
        let selection = match self.selection {
            Selection::On => "on",
            _ => "off",
        };
        let content = match self.content {
            Content::Icon => "icon",
            _ => "plain",
        };
        let availability = match self.availability {
            Availability::Disabled => "disabled",
            Availability::Enabled => "enabled",
        };
        format!("switch-{selection}-{content}-{availability}")
    }

    /// The top left corner of its container in the viewport, in points.
    #[must_use]
    pub fn origin(self) -> (Points, Points) {
        let column = grid::position(&SELECTIONS, self.selection) * CONTENTS.len()
            + grid::position(&CONTENTS, self.content);
        let mut x = MARGIN.0;
        for _ in 0..column {
            x += COLUMN_WIDTH.0;
        }
        let mut y = top().0;
        for availability in AVAILABILITIES {
            if availability == self.availability {
                return (Points(x), Points(y));
            }
            y += HEIGHT.0 + ROW_GAP.0;
        }
        (Points(x), Points(y))
    }
}

/// The switch of `cell` at its place on the grid, inside a view with its test id.
///
/// The switch is controlled by nothing: the catalog is a page to look at, and a press on a switch changes nothing.
#[must_use]
pub fn element(cell: SwitchCell) -> impl IntoElement {
    let (x, y) = cell.origin();
    let icon = match cell.content {
        Content::Icon => Some(ICON),
        _ => None,
    };
    view()
        .absolute()
        .left(x)
        .top(y)
        .test_id(&cell.test_id())
        .child(
            switch()
                .label(LABEL)
                .checked(cell.selection == Selection::On)
                .icon(icon)
                .disabled(cell.availability == Availability::Disabled),
        )
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex, PoisonError};

    use nsh_core::geometry::{Bounds, Size};
    use nsh_core::platform::{Environment, Os};
    use nsh_core::{AccessibilityRole, Checked, MountItem, NodeId, ViewKind};
    use nsh_engine::{EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::Catalog;

    #[test]
    fn every_switch_is_once_in_the_catalog() {
        let cells: Vec<SwitchCell> = SwitchCell::all().collect();
        assert_eq!(
            cells.len(),
            SELECTIONS.len() * CONTENTS.len() * AVAILABILITIES.len()
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
        let first = SwitchCell::new(Selection::Off, Content::Plain, Availability::Enabled);
        // Below the icon buttons: 24 points of margin, 880 of buttons' rows and gaps, 1,760 of icon buttons'.
        assert_eq!(top(), Points(2664.0));
        assert_eq!(first.origin(), (Points(24.0), Points(2664.0)));
        // The icon is the second column of a value; on the second value.
        let icon = SwitchCell {
            content: Content::Icon,
            ..first
        };
        assert_eq!(icon.origin(), (Points(120.0), Points(2664.0)));
        let on = SwitchCell {
            selection: Selection::On,
            ..first
        };
        assert_eq!(on.origin(), (Points(216.0), Points(2664.0)));
        // Below the enabled row: 32 points and the gap.
        let disabled = SwitchCell {
            availability: Availability::Disabled,
            ..first
        };
        assert_eq!(disabled.origin(), (Points(24.0), Points(2712.0)));
        let last = SwitchCell::new(Selection::On, Content::Icon, Availability::Disabled);
        assert_eq!(last.origin(), (Points(312.0), Points(2712.0)));
    }

    #[test]
    fn the_grid_ends_at_the_viewports_margin_and_leaves_room_for_the_state_layer() {
        let (width, height) = grid::viewport();
        assert_eq!(block_height(), Points(96.0));
        assert_eq!(top().0 + block_height().0 + MARGIN.0, height.0);
        // The state layer reaches this far past the track, above and below (and on each side when the handle is at an
        // end): it must stay inside the switch's column and between two rows without touching theirs.
        let reach = (switch_tokens::STATE_LAYER_SIZE.0 - HEIGHT.0) / 2.0;
        assert!(COLUMN_WIDTH.0 >= WIDTH.0 + 2.0 * reach);
        assert!(ROW_GAP.0 > 2.0 * reach);
        let mut bottom = Points(0.0);
        for cell in SwitchCell::all() {
            let (x, y) = cell.origin();
            // Above the first row, the icon buttons' last row ends a gap above the grid; the layer stays below it.
            assert!(y.0 - reach > top().0 - ROW_GAP.0, "{cell:?}");
            assert!(x.0 + COLUMN_WIDTH.0 <= width.0 - MARGIN.0, "{cell:?}");
            bottom = Points(bottom.0.max(y.0 + HEIGHT.0));
        }
        // The last row and its gap end at the margin.
        assert_eq!(bottom.0 + ROW_GAP.0, height.0 - MARGIN.0);
    }

    #[test]
    fn labels_and_ids_name_the_switch() {
        let cell = SwitchCell::new(Selection::On, Content::Icon, Availability::Disabled);
        assert_eq!(cell.test_id(), "switch-on-icon-disabled");
        assert_eq!(
            SwitchCell::default().test_id(),
            "switch-off-plain-enabled",
            "the default cell is the first one"
        );
        let ids: Vec<String> = SwitchCell::all().map(SwitchCell::test_id).collect();
        assert_eq!(
            ids,
            [
                "switch-off-plain-enabled",
                "switch-off-plain-disabled",
                "switch-off-icon-enabled",
                "switch-off-icon-disabled",
                "switch-on-plain-enabled",
                "switch-on-plain-disabled",
                "switch-on-icon-enabled",
                "switch-on-icon-disabled",
            ]
        );
        assert_eq!(LABEL, "Switch");
    }

    /// What `switch.test.tsx` does, as headless input does it: the mouse over the middle of the view with a switch's test
    /// id lays the state layer over that switch's handle, and over no other switch's.
    #[test]
    fn the_mouse_over_a_switchs_view_shows_its_state_layer_and_no_other() {
        // The test script's window, on a host with a mouse.
        let (width, height) = grid::viewport();
        let viewport = Size::new(width, height);
        let mut app = Headless::start(EngineConfig {
            viewport,
            environment: Environment {
                os: Os::Web,
                window: viewport,
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
        let snapshot = |items: &Mutex<Vec<MountItem>>| {
            items.lock().unwrap_or_else(PoisonError::into_inner).clone()
        };
        let mounted = snapshot(&items);

        // Each switch's roots and layers: a switch's children are its track, handle, icon and state layer, in this
        // order.
        let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        let mut ids = HashMap::new();
        let mut roots = Vec::new();
        for item in &mounted {
            match item {
                MountItem::Insert {
                    parent,
                    child,
                    before,
                } => {
                    let siblings = children.entry(*parent).or_default();
                    let at = before
                        .and_then(|before| siblings.iter().position(|id| *id == before))
                        .unwrap_or(siblings.len());
                    siblings.insert(at, *child);
                }
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        ids.insert(test_id.to_string(), *id);
                    }
                    if props.accessibility.role == AccessibilityRole::Switch {
                        roots.push(*id);
                    }
                }
                _ => {}
            }
        }
        let layer_of = |root: NodeId| children[&root][3];
        let layers: Vec<NodeId> = roots.iter().map(|root| layer_of(*root)).collect();
        assert_eq!(layers.len(), SwitchCell::all().count());

        for hovered in [
            SwitchCell::default(),
            SwitchCell::new(Selection::On, Content::Icon, Availability::Enabled),
        ] {
            let container = ids[&hovered.test_id()];
            let [root] = children[&container][..] else {
                panic!("one switch in {hovered:?}'s view");
            };
            let before = snapshot(&items).len();
            // The selector `switch.test.tsx` hovers with: the switch itself, not the view around it, which headless
            // input takes as the whole chain of views it hovers (a real host hit tests the pointer's place).
            app.hover(&format!("id:{} >> role:switch", hovered.test_id()))
                .unwrap();
            let sent: Vec<(NodeId, f32)> = snapshot(&items)[before..]
                .iter()
                .filter_map(|item| match item {
                    MountItem::ViewStyle { id, style } if layers.contains(id) => {
                        Some((*id, style.opacity))
                    }
                    _ => None,
                })
                .collect();
            // Only the hovered switch's layer, at the hover state layer opacity; the one before it, which the mouse
            // left, goes clear in the same step.
            assert!(
                sent.contains(&(layer_of(root), 0.08)),
                "{hovered:?}: {sent:?}"
            );
            assert!(
                sent.iter()
                    .all(|(id, opacity)| *id == layer_of(root) || opacity.abs() < f32::EPSILON),
                "{hovered:?}: {sent:?}"
            );
        }
    }

    /// Every switch is at its place on the grid, inside a view with its test id, with the value, the availability and
    /// the size its cell says, and an icon in its handle where its cell has one.
    #[test]
    fn every_switch_is_where_the_grid_says() {
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
        let mut kinds = HashMap::new();
        let mut frames: HashMap<NodeId, Bounds<Points>> = HashMap::new();
        let mut parents = HashMap::new();
        // The switches' roots, with the value and availability their accessibility state says.
        let mut switches = HashMap::new();
        for item in &items {
            match item {
                MountItem::Create { id, kind } => {
                    kinds.insert(*id, *kind);
                }
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        ids.insert(test_id.to_string(), *id);
                    }
                    if props.accessibility.role == AccessibilityRole::Switch {
                        switches.insert(
                            *id,
                            (
                                props.accessibility.state.checked,
                                props.accessibility.state.disabled,
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
        assert_eq!(switches.len(), SwitchCell::all().count());
        let children_of = |id: NodeId| -> Vec<NodeId> {
            parents
                .iter()
                .filter(|(_, parent)| **parent == id)
                .map(|(child, _)| *child)
                .collect()
        };
        for cell in SwitchCell::all() {
            let container = ids[&cell.test_id()];
            let (x, y) = cell.origin();
            assert_eq!(
                (frames[&container].origin.x, frames[&container].origin.y),
                (x, y),
                "{cell:?}"
            );
            let [root] = children_of(container)[..] else {
                panic!("one switch in {cell:?}'s view");
            };
            assert_eq!(
                switches.get(&root),
                Some(&(
                    Some(if cell.selection == Selection::On {
                        Checked::True
                    } else {
                        Checked::False
                    }),
                    cell.availability == Availability::Disabled
                )),
                "{cell:?}"
            );
            assert_eq!(
                (frames[&root].size.width, frames[&root].size.height),
                (WIDTH, HEIGHT),
                "{cell:?}"
            );
            // The icon's canvas is laid out at its size where the handle holds one, and hidden where it does not.
            let canvases: Vec<NodeId> = children_of(root)
                .into_iter()
                .filter(|id| kinds.get(id) == Some(&ViewKind::Canvas))
                .collect();
            let [canvas] = canvases[..] else {
                panic!("one icon canvas in {cell:?}");
            };
            let expected = match cell.content {
                Content::Icon => (Points(16.0), Points(16.0)),
                _ => (Points(0.0), Points(0.0)),
            };
            assert_eq!(
                (frames[&canvas].size.width, frames[&canvas].size.height),
                expected,
                "{cell:?}"
            );
        }
    }
}
