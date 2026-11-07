//! The catalog: every component of `nsh-m3e` in each of its states, for the visual checks of ADR 0013: the buttons,
//! the icon buttons and the badges.
//!
//! `nsh test` opens it on NSH's hosts (`button.test.tsx`, beside its `nsh.json`) and saves screenshots of it; `visual-checks`
//! reads the pixels it knows in them, at the places [`grid`] gives. The catalog is a Rust app (`nsh.json` names its
//! root, [`Catalog`]); nothing in it runs JavaScript.

pub mod badge;
pub mod grid;

use nsh_core::element::AdaptiveColor;
use nsh_core::prelude::*;
use nsh_m3e::badge::badged_box;
use nsh_m3e::button::button;
use nsh_m3e::color::ColorScheme;
use nsh_m3e::icon::Icon;
use nsh_m3e::icon_button::{IconButtonSize, IconButtonStyle, IconButtonWidth, icon_button};
use nsh_m3e::material_symbols::outlined;

use crate::badge::BadgeCell;
use crate::grid::{Availability, Cell, IconCell};

/// The icon every icon button of the catalog shows: Material Symbols' `favorite`, filled, whose middle is filled, so
/// that the visual checks read the icon's color there.
pub const ICON: Icon = outlined::filled::FAVORITE;

#[native_module("catalog/native")]
fn native(registry: &mut Registry) {
    registry.register::<Catalog>();
}

/// Every button, icon button and badge of the catalog at its place on the [`grid`], on the scheme's surface color.
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
    BadgeCell::all().fold(page, |page, cell| {
        let (x, y) = cell.origin();
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .test_id(cell.test_id())
                .child(
                    badged_box()
                        .label(cell.label().map(SharedString::from))
                        // The anchor: the small standard icon button, which `badge::anchor` sizes.
                        .child(
                            icon_button()
                                .icon(ICON)
                                .label("Favorite")
                                .style(IconButtonStyle::Standard)
                                .width(IconButtonWidth::Uniform)
                                .size(IconButtonSize::Small),
                        ),
                ),
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

    #[test]
    fn the_app_registers_its_root() {
        let registry = registry();
        assert_eq!(registry.specifier(), Some("catalog/native"));
        let exports: Vec<String> = registry.specs().iter().map(|s| s.export.clone()).collect();
        assert_eq!(exports, ["Catalog"]);
    }

    /// Every button and icon button is at its place on the grid, inside a view with its test id, enabled or disabled
    /// as its cell says; every icon button has the size the grid gives it.
    #[test]
    fn every_button_is_where_the_grid_says() {
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
        // The buttons, the icon buttons, and the icon button each badge sits on.
        assert_eq!(
            disabled.len(),
            Cell::all().count() + IconCell::all().count() + BadgeCell::all().count()
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

    /// Every badge is where the grid says, on an anchor of the size the grid gives, and the badge is where androidx's
    /// `BadgedBox` puts it: the places the visual checks read.
    #[test]
    fn every_badge_is_where_the_grid_says() {
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
        let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for item in &items {
            match item {
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        ids.insert(test_id.to_string(), *id);
                    }
                }
                MountItem::Frame { id, frame } => {
                    frames.insert(*id, *frame);
                }
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
                _ => {}
            }
        }
        let (anchor_width, anchor_height) = badge::anchor();
        for cell in BadgeCell::all() {
            let wrapper = ids[cell.test_id()];
            let (x, y) = cell.origin();
            assert_eq!(
                (frames[&wrapper].origin.x, frames[&wrapper].origin.y),
                (x, y),
                "{cell:?}"
            );
            // The badged box, its anchor and its corner; the badge alone in the corner.
            let [container] = children[&wrapper][..] else {
                panic!(
                    "the badged box alone in {cell:?}'s view: {:?}",
                    children[&wrapper]
                );
            };
            let [anchor, corner] = children[&container][..] else {
                panic!(
                    "an anchor and a corner in {cell:?}: {:?}",
                    children[&container]
                );
            };
            let [badge] = children[&corner][..] else {
                panic!("a badge in {cell:?}'s corner: {:?}", children[&corner]);
            };
            // The anchor is the size the grid says, and the badged box is its size.
            assert_eq!(
                (frames[&anchor].size.width, frames[&anchor].size.height),
                (anchor_width, anchor_height),
                "{cell:?}"
            );
            assert_eq!(frames[&container].size, frames[&anchor].size, "{cell:?}");
            // The badge, in the viewport: its corner view's place and its own in it.
            let (badge_x, badge_y) = cell.badge_origin();
            assert_eq!(
                (
                    x.0 + frames[&container].origin.x.0
                        + frames[&corner].origin.x.0
                        + frames[&badge].origin.x.0,
                    y.0 + frames[&container].origin.y.0
                        + frames[&corner].origin.y.0
                        + frames[&badge].origin.y.0,
                ),
                (badge_x.0, badge_y.0),
                "{cell:?}"
            );
            assert_eq!(frames[&badge].size.height, cell.badge_height(), "{cell:?}");
        }
    }
}
