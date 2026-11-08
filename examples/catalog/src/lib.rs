//! The catalog: every component of `nsh-m3e` in each of its states, for the visual checks of ADR 0013: the buttons,
//! the icon buttons, the navigation bars and the tab rows.
//!
//! `nsh test` opens it on NSH's hosts (`button.test.tsx`, beside its `nsh.json`) and saves screenshots of it; `visual-checks`
//! reads the pixels it knows in them, at the places [`grid`] gives. The catalog is a Rust app (`nsh.json` names its
//! root, [`Catalog`]); nothing in it runs JavaScript.

pub mod grid;
pub mod navigation;
pub mod tabs;

use nsh_core::element::AdaptiveColor;
use nsh_core::prelude::*;
use nsh_m3e::button::button;
use nsh_m3e::color::ColorScheme;
use nsh_m3e::icon::Icon;
use nsh_m3e::icon_button::icon_button;
use nsh_m3e::material_symbols::outlined;
use nsh_m3e::navigation_bar::{navigation_bar, navigation_bar_item};
use nsh_m3e::tabs::{primary_tab_row, secondary_tab_row, tab};

use crate::grid::{Availability, Cell, IconCell};
use crate::navigation::{ITEMS, ItemState, NavCell};
use crate::tabs::{RowKind, TABS, TabCell, TabState};

/// The icon every icon button, navigation bar item and tab of the catalog shows: Material Symbols' `favorite`, filled, whose
/// middle is filled, so that the visual checks read the icon's color there. Material shows the outlined icon in an item
/// that is not selected; the catalog shows the filled one in all, to have a color to read.
pub const ICON: Icon = outlined::filled::FAVORITE;

#[native_module("catalog/native")]
fn native(registry: &mut Registry) {
    registry.register::<Catalog>();
}

/// Every button, icon button, navigation bar and row of tabs of the catalog at its place on the [`grid`], the
/// [`navigation`] grid and the [`tabs`] grid, on the scheme's surface color.
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
    let page = NavCell::all().fold(page, |page, cell| {
        let (x, y) = cell.origin();
        let bar = ITEMS.into_iter().fold(
            navigation_bar().arrangement(cell.arrangement),
            |bar, state| {
                bar.child(
                    navigation_bar_item()
                        .icon(ICON)
                        .label(state.label())
                        .selected(state == ItemState::Selected)
                        .disabled(state == ItemState::Disabled)
                        .icon_position(cell.icon_position)
                        .label_visibility(cell.label),
                )
            },
        );
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .w(cell.width())
                .test_id(&cell.test_id())
                .child(bar),
        )
    });
    TabCell::all().fold(page, |page, cell| {
        let (x, y) = cell.origin();
        let tabs: Vec<AnyElement> = TABS
            .into_iter()
            .map(|state| {
                tab()
                    .label(state.label())
                    .icon(cell.content.has_icon().then_some(ICON))
                    .label_visibility(cell.content.label_visibility())
                    .selected(state == TabState::Selected)
                    .disabled(state == TabState::Disabled)
                    .into_element()
            })
            .collect();
        let row = match cell.kind {
            RowKind::Secondary => secondary_tab_row().children(tabs).into_element(),
            _ => primary_tab_row().children(tabs).into_element(),
        };
        page.child(
            view()
                .absolute()
                .left(x)
                .top(y)
                .w(cell.width())
                .test_id(&cell.test_id())
                .child(row),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, PoisonError};

    use nsh_core::geometry::{Bounds, Points, Size};
    use nsh_core::{AccessibilityRole, AccessibilityState, MountItem, NodeId};
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
        assert_eq!(
            disabled.len(),
            Cell::all().count() + IconCell::all().count()
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

    /// What the catalog mounted that the navigation bars' test reads.
    #[derive(Default)]
    struct Mounted {
        /// The view of each test id.
        ids: HashMap<String, NodeId>,
        frames: HashMap<NodeId, Bounds<Points>>,
        parents: HashMap<NodeId, NodeId>,
        /// The label and the state of each view with the tab role: the navigation bars' items.
        tabs: HashMap<NodeId, (Option<Arc<str>>, AccessibilityState)>,
    }

    impl Mounted {
        /// What `items` mounted, the last props and frame of each view standing.
        fn of(items: &[MountItem]) -> Self {
            let mut mounted = Self::default();
            for item in items {
                match item {
                    MountItem::ViewProps { id, props } => {
                        if let Some(test_id) = &props.test_id {
                            mounted.ids.insert(test_id.to_string(), *id);
                        }
                        if props.accessibility.role == AccessibilityRole::Tab {
                            mounted.tabs.insert(
                                *id,
                                (props.accessibility.label.clone(), props.accessibility.state),
                            );
                        }
                    }
                    MountItem::Frame { id, frame } => {
                        mounted.frames.insert(*id, *frame);
                    }
                    MountItem::Insert { parent, child, .. } => {
                        mounted.parents.insert(*child, *parent);
                    }
                    _ => {}
                }
            }
            mounted
        }

        /// The top left corner of `id` in the viewport: a frame is relative to its parent's, so this adds them up to
        /// the root.
        fn place(&self, id: NodeId) -> (f32, f32) {
            let mut at = (0.0, 0.0);
            let mut view = Some(id);
            while let Some(id) = view {
                let frame = self.frames.get(&id).copied().unwrap_or_default();
                at = (at.0 + frame.origin.x.0, at.1 + frame.origin.y.0);
                view = self.parents.get(&id).copied();
            }
            at
        }

        /// Whether `id` is `ancestor` or below it.
        fn inside(&self, id: NodeId, ancestor: NodeId) -> bool {
            let mut view = Some(id);
            while let Some(id) = view {
                if id == ancestor {
                    return true;
                }
                view = self.parents.get(&id).copied();
            }
            false
        }

        /// The item of `bar` that says `label`: there is one.
        fn item(&self, bar: NodeId, label: &str) -> NodeId {
            let found: Vec<NodeId> = self
                .tabs
                .iter()
                .filter(|(id, (said, _))| said.as_deref() == Some(label) && self.inside(**id, bar))
                .map(|(id, _)| *id)
                .collect();
            let [item] = found[..] else {
                panic!("{label}: one item, not {found:?}");
            };
            item
        }

        /// The middle of `id` in the viewport.
        fn middle(&self, id: NodeId) -> (f32, f32) {
            let (left, top) = self.place(id);
            let size = self.frames[&id].size;
            (left + size.width.0 / 2.0, top + size.height.0 / 2.0)
        }
    }

    /// The catalog's items, mounted in a window as large as its viewport: a text below the screen's edge is not
    /// shown, and the bars are at the bottom.
    fn mounted_in_the_viewport() -> Mounted {
        let (width, height) = grid::viewport();
        let mut app = Headless::start(EngineConfig {
            viewport: Size::new(width, height),
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
        app.wait_for("Home").unwrap();
        app.wait_for("Songs").unwrap();
        let items = items.lock().unwrap_or_else(PoisonError::into_inner).clone();
        Mounted::of(&items)
    }

    /// Every navigation bar is at its place on the grid, as wide and as tall as the grid says, with its three items
    /// where the arrangement puts them, the first selected and the last disabled.
    #[test]
    fn every_navigation_bar_is_where_the_grid_says() {
        let mounted = mounted_in_the_viewport();
        // The tab role is the navigation bars' items' and the tab rows' tabs': this test reads the bars' ones.
        assert_eq!(
            mounted.tabs.len(),
            (NavCell::all().count() + TabCell::all().count()) * ITEMS.len()
        );
        for cell in NavCell::all() {
            let bar = mounted.ids[&cell.test_id()];
            let (x, y) = cell.origin();
            assert_eq!(mounted.place(bar), (x.0, y.0), "{cell:?}");
            let size = mounted.frames[&bar].size;
            assert_eq!((size.width, size.height), (cell.width(), cell.height()));
            for state in ITEMS {
                let item = mounted.item(bar, state.label());
                let (center_x, center_y) = cell.item_center(state);
                let (found_x, found_y) = mounted.middle(item);
                // Layout rounds to whole points.
                assert!(
                    (found_x - center_x.0).abs() < 0.5 && (found_y - center_y.0).abs() < 0.5,
                    "{cell:?} {state:?}: at ({found_x}, {found_y}), not ({center_x:?}, {center_y:?})"
                );
                let (_, accessibility) = &mounted.tabs[&item];
                assert_eq!(accessibility.selected, state == ItemState::Selected);
                assert_eq!(accessibility.disabled, state == ItemState::Disabled);
            }
        }
    }

    /// Every row of tabs is at its place on the grid, as wide and as tall as the grid says, with its three tabs a third
    /// of it each, the first selected and the last disabled.
    #[test]
    fn every_row_of_tabs_is_where_the_grid_says() {
        let mounted = mounted_in_the_viewport();
        for cell in TabCell::all() {
            let row = mounted.ids[&cell.test_id()];
            let (x, y) = cell.origin();
            assert_eq!(mounted.place(row), (x.0, y.0), "{cell:?}");
            let size = mounted.frames[&row].size;
            assert_eq!((size.width, size.height), (cell.width(), cell.height()));
            for state in TABS {
                let tab = mounted.item(row, state.label());
                let (center_x, center_y) = cell.tab_center(state);
                let (found_x, found_y) = mounted.middle(tab);
                // Layout rounds to whole points.
                assert!(
                    (found_x - center_x.0).abs() < 0.5 && (found_y - center_y.0).abs() < 0.5,
                    "{cell:?} {state:?}: at ({found_x}, {found_y}), not ({center_x:?}, {center_y:?})"
                );
                let (_, accessibility) = &mounted.tabs[&tab];
                assert_eq!(accessibility.selected, state == TabState::Selected);
                assert_eq!(accessibility.disabled, state == TabState::Disabled);
            }
        }
    }
}
