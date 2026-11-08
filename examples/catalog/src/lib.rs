//! The catalog: every component of `nsh-m3e` in each of its states, for the visual checks of ADR 0013: the buttons,
//! the icon buttons and the list items.
//!
//! `nsh test` opens it on NSH's hosts (`button.test.tsx`, beside its `nsh.json`) and saves screenshots of it; `visual-checks`
//! reads the pixels it knows in them, at the places [`grid`] gives. The catalog is a Rust app (`nsh.json` names its
//! root, [`Catalog`]); nothing in it runs JavaScript.

pub mod grid;
pub mod list_item;

use nsh_core::element::AdaptiveColor;
use nsh_core::prelude::*;
use nsh_m3e::button::button;
use nsh_m3e::color::ColorScheme;
use nsh_m3e::icon::Icon;
use nsh_m3e::icon_button::icon_button;
use nsh_m3e::material_symbols::outlined;

use crate::grid::{Availability, Cell, IconCell};
use crate::list_item::ListCell;

/// The icon every icon button of the catalog shows: Material Symbols' `favorite`, filled, whose middle is filled, so
/// that the visual checks read the icon's color there.
pub const ICON: Icon = outlined::filled::FAVORITE;

#[native_module("catalog/native")]
fn native(registry: &mut Registry) {
    registry.register::<Catalog>();
}

/// Every button, icon button and list item of the catalog at its place on the [`grid`] and the [`list_item`] grid, on
/// the scheme's surface color.
#[component]
pub fn Catalog() -> impl IntoElement {
    let surface = AdaptiveColor::rgb(
        ColorScheme::BASELINE_LIGHT.surface.to_hex(),
        ColorScheme::BASELINE_DARK.surface.to_hex(),
    );
    let page = Cell::all().fold(view().relative().size_full().bg(surface), |page, cell| {
        let (x, y) = cell.origin();
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .test_id(&cell.test_id())
                .child(
                    button()
                        .label(cell.label())
                        .style(cell.style)
                        .shape(cell.shape)
                        .size(cell.size)
                        .disabled(cell.availability == Availability::Disabled),
                ),
        )
    });
    let page = IconCell::all().fold(page, |page, cell| {
        let (x, y) = cell.origin();
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .test_id(&cell.test_id())
                .child(
                    icon_button()
                        .icon(ICON)
                        .label(cell.label())
                        .style(cell.style)
                        .width(cell.width)
                        .shape(cell.shape)
                        .size(cell.size)
                        .disabled(cell.availability == Availability::Disabled),
                ),
        )
    });
    ListCell::all().fold(page, |page, cell| {
        let (x, y) = cell.origin();
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .w(list_item::WIDTH)
                .test_id(&cell.test_id())
                .child(list_item::item(cell)),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, PoisonError};

    use nsh_core::geometry::{Bounds, Points};
    use nsh_core::{AccessibilityRole, MountItem, NodeId};
    use nsh_engine::{EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::list_item::Kind;

    #[test]
    fn the_app_registers_its_root() {
        let registry = registry();
        assert_eq!(registry.specifier(), Some("catalog/native"));
        let exports: Vec<String> = registry.specs().iter().map(|s| s.export.clone()).collect();
        assert_eq!(exports, ["Catalog"]);
    }

    /// Everything the catalog mounts on its first screen, in the order the engine sent it.
    fn mounted_items() -> Vec<MountItem> {
        let mut app = Headless::start(EngineConfig::new(Root::of::<Catalog>())).unwrap();
        let items: Arc<Mutex<Vec<MountItem>>> = Arc::default();
        let sink = Arc::clone(&items);
        app.observe(move |_, batch| {
            sink.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .extend(batch.items.iter().cloned());
        });
        app.wait_for("Filled").unwrap();
        let mounted = items.lock().unwrap_or_else(PoisonError::into_inner);
        mounted.clone()
    }

    /// Every button and icon button is at its place on the grid, inside a view with its test id, enabled or disabled
    /// as its cell says; every icon button has the size the grid gives it.
    #[test]
    fn every_button_is_where_the_grid_says() {
        let items = mounted_items();

        let mut ids = HashMap::new();
        let mut frames: HashMap<NodeId, Bounds<Points>> = HashMap::new();
        let mut disabled = HashMap::new();
        let mut parents = HashMap::new();
        for item in &items {
            match item {
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        ids.insert(test_id.to_string(), *id);
                    }
                    if props.accessibility.role == AccessibilityRole::Button {
                        disabled.insert(*id, props.accessibility.state.disabled);
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
        // The clickable and the disabled list items have the button role too; the selected ones are radios.
        let list_buttons = ListCell::all()
            .filter(|cell| matches!(cell.kind, Kind::Clickable | Kind::Disabled))
            .count();
        assert_eq!(
            disabled.len(),
            Cell::all().count() + IconCell::all().count() + list_buttons
        );
        let child_of = |id: NodeId| {
            parents
                .iter()
                .find(|(_, parent)| **parent == id)
                .map(|(child, _)| *child)
                .unwrap()
        };
        for cell in Cell::all() {
            let id = ids[&cell.test_id()];
            let (x, y) = cell.origin();
            assert_eq!(
                (frames[&id].origin.x, frames[&id].origin.y),
                (x, y),
                "{cell:?}"
            );
            assert_eq!(
                disabled[&child_of(id)],
                cell.availability == Availability::Disabled,
                "{cell:?}"
            );
        }
        for cell in IconCell::all() {
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
            assert_eq!(
                disabled[&button],
                cell.availability == Availability::Disabled,
                "{cell:?}"
            );
        }
    }

    /// Every list item is at its place on the list items' grid, inside a view with its test id and the width the
    /// catalog gives it; the item has its kind's role and state, the height of its lines, and the texts its lines
    /// show, and no others.
    #[test]
    fn every_list_item_is_where_the_list_grid_says() {
        let items = mounted_items();

        let mut ids = HashMap::new();
        let mut frames: HashMap<NodeId, Bounds<Points>> = HashMap::new();
        let mut roles = HashMap::new();
        let mut parents = HashMap::new();
        let mut texts = Vec::new();
        for item in &items {
            match item {
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        ids.insert(test_id.to_string(), *id);
                    }
                    roles.insert(
                        *id,
                        (props.accessibility.role, props.accessibility.state.disabled),
                    );
                }
                MountItem::Frame { id, frame } => {
                    frames.insert(*id, *frame);
                }
                MountItem::Insert { parent, child, .. } => {
                    parents.insert(*child, *parent);
                }
                MountItem::Text { id, text, .. } => {
                    texts.push((*id, text.text.clone()));
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
        // The wrapper a node is inside of, if it is inside one of the list items'.
        let wrapper_of = |mut id: NodeId| {
            let wrappers: HashMap<NodeId, ListCell> = ListCell::all()
                .map(|cell| (ids[&cell.test_id()], cell))
                .collect();
            for _ in 0..parents.len() {
                if let Some(cell) = wrappers.get(&id) {
                    return Some(*cell);
                }
                id = *parents.get(&id)?;
            }
            None
        };
        let radios = ListCell::all()
            .filter(|cell| matches!(cell.kind, Kind::Selected))
            .count();
        assert_eq!(
            roles
                .values()
                .filter(|(role, _)| *role == AccessibilityRole::Radio)
                .count(),
            radios
        );
        for cell in ListCell::all() {
            let id = ids[&cell.test_id()];
            let (x, y) = cell.origin();
            let frame = frames[&id];
            assert_eq!((frame.origin.x, frame.origin.y), (x, y), "{cell:?}");
            assert_eq!(frame.size.width, list_item::WIDTH, "{cell:?}");
            let item = child_of(id);
            assert_eq!(frames[&item].size.height, cell.height(), "{cell:?}");
            assert_eq!(frames[&item].size.width, list_item::WIDTH, "{cell:?}");
            // The views of the item with a role: the plain item has none, the others have one.
            let with_role: Vec<(AccessibilityRole, bool)> = roles
                .iter()
                .filter(|(node, (role, _))| {
                    *role != AccessibilityRole::default() && wrapper_of(**node) == Some(cell)
                })
                .map(|(_, role)| *role)
                .collect();
            let expected = match cell.kind {
                Kind::Clickable => vec![(AccessibilityRole::Button, false)],
                Kind::Disabled => vec![(AccessibilityRole::Button, true)],
                Kind::Selected => vec![(AccessibilityRole::Radio, false)],
                _ => Vec::new(),
            };
            assert_eq!(with_role, expected, "{cell:?}");
            let mut shown: Vec<&str> = texts
                .iter()
                .filter(|(text, _)| wrapper_of(*text) == Some(cell))
                .map(|(_, text)| text.as_str())
                .collect();
            shown.sort_unstable();
            let mut expected = vec![cell.label()];
            if cell.has_overline() {
                expected.push("Overline");
            }
            if cell.has_supporting() {
                expected.push("Supporting text");
            }
            expected.sort_unstable();
            assert_eq!(shown, expected, "{cell:?}");
        }
    }
}
