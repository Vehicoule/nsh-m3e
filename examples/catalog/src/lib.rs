//! The catalog: every component of `nsh-m3e` in each of its states, for the visual checks of ADR 0013: the buttons,
//! the icon buttons, the switches, the checkboxes, the dividers, the floating action buttons and the extended ones.
//!
//! `nsh test` opens it on NSH's hosts (`button.test.tsx`, `switch.test.tsx`, `checkbox.test.tsx`, `fab.test.tsx` and
//! `extended_fab.test.tsx`, beside its `nsh.json`) and saves screenshots of it; `visual-checks` reads the pixels it knows
//! in them, at the places [`grid`], [`switch`], [`checkbox`], [`fab`] and [`extended_fab`] give.
//! The catalog is a Rust app (`nsh.json` names its root, [`Catalog`]); nothing in it runs JavaScript.

pub mod checkbox;
pub mod extended_fab;
pub mod fab;
pub mod grid;
pub mod switch;

use nsh_core::element::AdaptiveColor;
use nsh_core::prelude::*;
use nsh_m3e::button::button;
use nsh_m3e::color::ColorScheme;
use nsh_m3e::divider::divider;
use nsh_m3e::icon::Icon;
use nsh_m3e::icon_button::icon_button;
use nsh_m3e::material_symbols::outlined;

use crate::grid::{Availability, Cell, DIVIDER_SPACE, DividerCell, IconCell};
use crate::switch::SwitchCell;

/// The icon every icon button and FAB of the catalog shows: Material Symbols' `favorite`, filled, whose middle is
/// filled, so that the visual checks read the icon's color there.
pub const ICON: Icon = outlined::filled::FAVORITE;

#[native_module("catalog/native")]
fn native(registry: &mut Registry) {
    registry.register::<Catalog>();
}

/// Every button, icon button, switch, checkbox, divider, FAB and extended FAB of the catalog at its place on the
/// [`grid`], the [`switch`] grid, the [`checkbox`] grid, the [`fab`] row and the [`extended_fab`] grid, on the scheme's
/// surface color.
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
    // A divider fills the space it is in along its length, so each one is given a square space to fill, as an app's
    // layout gives it one.
    let page = DividerCell::all().fold(page, |page, cell| {
        let (x, y) = cell.origin();
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .w(DIVIDER_SPACE)
                .h(DIVIDER_SPACE)
                .test_id(&cell.test_id())
                .child(
                    divider()
                        .orientation(cell.orientation)
                        .thickness(cell.thickness),
                ),
        )
    });
    let page = SwitchCell::all().fold(page, |page, cell| page.child(switch::element(cell)));
    extended_fab::place(fab::place(checkbox::place(page)))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, PoisonError};

    use nsh_core::geometry::{Bounds, Points, Size, point};
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
        // The FABs and the extended FABs are buttons for screen readers too.
        assert_eq!(
            disabled.len(),
            Cell::all().count()
                + IconCell::all().count()
                + fab::FabCell::all().count()
                + extended_fab::ExtendedFabCell::all().count()
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

    /// Every divider is in its space at its place on the grid, and fills the space along its length and is as thick as
    /// its cell says across it, at the start of the space; and it is the only thing in the space, hidden from
    /// assistive technologies.
    #[test]
    fn every_divider_is_where_the_grid_says() {
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
        let mut hidden = HashMap::new();
        let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for item in &items {
            match item {
                MountItem::ViewProps { id, props } => {
                    if let Some(test_id) = &props.test_id {
                        ids.insert(test_id.to_string(), *id);
                    }
                    hidden.insert(*id, props.accessibility.hidden);
                }
                MountItem::Frame { id, frame } => {
                    frames.insert(*id, *frame);
                }
                MountItem::Insert { parent, child, .. } => {
                    children.entry(*parent).or_default().push(*child);
                }
                _ => {}
            }
        }
        assert_eq!(DividerCell::all().count(), 6);
        for cell in DividerCell::all() {
            let space = ids[&cell.test_id()];
            let (x, y) = cell.origin();
            assert_eq!(
                frames[&space],
                Bounds {
                    origin: point(x, y),
                    size: Size {
                        width: DIVIDER_SPACE,
                        height: DIVIDER_SPACE,
                    },
                },
                "{cell:?}"
            );
            let [line] = children[&space][..] else {
                panic!("one divider in the space of {cell:?}");
            };
            let (width, height) = cell.frame();
            assert_eq!(
                frames[&line],
                Bounds {
                    origin: point(Points(0.0), Points(0.0)),
                    size: Size { width, height },
                },
                "{cell:?}"
            );
            assert_eq!(hidden.get(&line), Some(&true), "{cell:?}");
            assert_ne!(hidden.get(&space), Some(&true), "{cell:?}");
        }
    }
}
