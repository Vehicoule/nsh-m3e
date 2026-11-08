//! The navigation bars' known pixels: the bar's edges against the surface, the container's color inside it, and for
//! each item the pill (or the container where there is none), the icon's color and the state layer of the mouse over
//! it. The labels are not read: the antialiasing of their glyphs makes every pixel of them unknown.

use catalog::navigation::{ITEMS, ItemState, NavCell};
use nsh_core::geometry::Points;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::navigation_item::{DISABLED_OPACITY, NavigationItemIconPosition, NavigationItemLabel};
use nsh_m3e::tokens::androidx::{
    navigation_bar_horizontal_item_tokens as horizontal, navigation_bar_tokens as bar,
    navigation_bar_vertical_item_tokens as vertical, state_tokens,
};

use super::{Probe, Rgb, Target, faded};

/// The color of the state layer: the bar's content color, `onSurface`, as androidx's ripple takes it
/// (`Ripple.kt`, lines 82-83, `ShortNavigationBar.kt`, `ShortNavigationBarDefaults.contentColor`). Written here, not
/// taken from the library, so that the check is the source's word and not the library's own.
const STATE_LAYER: ColorRole = ColorRole::OnSurface;

/// The pixels the checks know for the bar `cell` in a screenshot in `scheme`, with the mouse over the item in the
/// state `hovered`, if any.
pub(super) fn probes(
    cell: NavCell,
    scheme: &ColorScheme,
    hovered: Option<ItemState>,
) -> Vec<Probe> {
    let (x, y) = cell.origin();
    let (width, height) = (cell.width().0, cell.height().0);
    let middle = y.0 + height / 2.0;
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let container = Rgb::of(scheme, bar::CONTAINER_COLOR);
    let target = Target::NavigationBar(cell);
    let edge = |what, x: f32, y: f32, expected| Probe {
        target,
        what,
        x: Points(x),
        y: Points(y),
        expected,
        tolerance: 0,
    };
    // The first pixel inside each side of the bar, and the first outside it: the container fills the bar's box
    // exactly, in the middle of its height and, in its first item's column, along its top and bottom.
    let mut probes = vec![
        edge("surface left of the bar", x.0 - 1.0, middle, surface),
        edge("container at the left edge", x.0, middle, container),
        edge(
            "container at the right edge",
            x.0 + width - 1.0,
            middle,
            container,
        ),
        edge("surface right of the bar", x.0 + width, middle, surface),
        edge("surface above the bar", x.0 + 2.0, y.0 - 1.0, surface),
        edge("container at the top", x.0 + 2.0, y.0, container),
        edge(
            "container at the bottom",
            x.0 + 2.0,
            y.0 + height - 1.0,
            container,
        ),
        edge("surface below the bar", x.0 + 2.0, y.0 + height, surface),
    ];
    for state in ITEMS {
        probes.extend(item_probes(cell, state, scheme, hovered == Some(state)));
    }
    probes
}

/// The pixels the checks know for the item of `cell` in `state`, with the mouse over it or not.
fn item_probes(cell: NavCell, state: ItemState, scheme: &ColorScheme, hovered: bool) -> Vec<Probe> {
    let target = Target::NavigationItem(cell, state);
    let (center_x, middle) = cell.item_center(state);
    let (_, top) = cell.origin();
    let container = Rgb::of(scheme, bar::CONTAINER_COLOR);
    // The state layer lies over the pill and the icon, and a disabled item shows none.
    let over = |below: Rgb| {
        if hovered && state != ItemState::Disabled {
            faded(
                scheme,
                STATE_LAYER,
                state_tokens::HOVER_STATE_LAYER_OPACITY,
                below,
            )
        } else {
            (below, 0)
        }
    };
    let pill = if state == ItemState::Selected {
        Rgb::of(scheme, bar::ITEM_ACTIVE_INDICATOR_COLOR)
    } else {
        // No pill: the container shows through.
        container
    };
    let (pill_at, icon_at) = places(cell, center_x.0, middle.0, top.0);
    let (pill_color, pill_tolerance) = over(pill);
    let (icon_color, icon_tolerance) = match state {
        ItemState::Selected => over(Rgb::of(scheme, bar::ITEM_ACTIVE_ICON_COLOR)),
        ItemState::Disabled => faded(
            scheme,
            bar::ITEM_INACTIVE_ICON_COLOR,
            DISABLED_OPACITY,
            container,
        ),
        _ => over(Rgb::of(scheme, bar::ITEM_INACTIVE_ICON_COLOR)),
    };
    let probe = |what, (x, y): (f32, f32), expected, tolerance| Probe {
        target,
        what,
        x: Points(x),
        y: Points(y),
        expected,
        tolerance,
    };
    let mut probes = vec![
        // Above every item's pill, whatever its shape: the pill of an item with its icon on top and a label starts
        // 6 points below the bar's top, and the state layer lies inside it.
        probe("above the pill", (center_x.0, top.0 + 2.0), container, 0),
        probe("pill", pill_at, pill_color, pill_tolerance),
    ];
    if let Some(at) = icon_at {
        probes.push(probe("icon", at, icon_color, icon_tolerance));
    }
    probes
}

/// Where the pill's color is read, in the item of `cell` whose middle is `center_x` and `middle`, in a bar whose top
/// is `top`, and where its icon's middle is, when that is known.
///
/// With the icon on top, the pill is 56 × 32 points around the icon: read halfway between the icon's side and the
/// pill's, at the pill's mid height. With the icon at the start, the pill is 40 points tall around the icon and the
/// label, as wide as the label makes it: the item's middle is the pill's middle, but where the icon is depends on the
/// label's width, which the font gives, so the icon is read only when there is no label; the pill is read halfway
/// between the icon's top and the pill's, above the label too. Not design values: where the check looks.
fn places(cell: NavCell, center_x: f32, middle: f32, top: f32) -> ((f32, f32), Option<(f32, f32)>) {
    match (cell.icon_position, cell.label) {
        (NavigationItemIconPosition::Start, NavigationItemLabel::Hidden) => {
            let inset = (horizontal::ICON_SIZE.0 + horizontal::ACTIVE_INDICATOR_HEIGHT.0) / 4.0;
            ((center_x, middle - inset), Some((center_x, middle)))
        }
        (NavigationItemIconPosition::Start, _) => {
            let inset = (horizontal::ICON_SIZE.0 + horizontal::ACTIVE_INDICATOR_HEIGHT.0) / 4.0;
            ((center_x, middle - inset), None)
        }
        (_, label) => {
            // With a label the content hangs from the top, `ContainerBetweenSpace` below it; without, it is centered.
            let pill_middle = if label == NavigationItemLabel::Hidden {
                middle
            } else {
                top + vertical::CONTAINER_BETWEEN_SPACE.0
                    + vertical::ACTIVE_INDICATOR_HEIGHT.0 / 2.0
            };
            let inset = (vertical::ICON_SIZE.0 + vertical::ACTIVE_INDICATOR_WIDTH.0) / 4.0;
            (
                (center_x - inset, pill_middle),
                Some((center_x, pill_middle)),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use catalog::navigation::{ARRANGEMENTS, ICON_POSITIONS, LABELS};
    use nsh_m3e::navigation_bar::NavigationBarArrangement;

    use super::*;

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;
    const DARK: ColorScheme = ColorScheme::BASELINE_DARK;

    fn cell(position: NavigationItemIconPosition, label: NavigationItemLabel) -> NavCell {
        NavCell::new(NavigationBarArrangement::EqualWeight, position, label)
    }

    fn named<'a>(probes: &'a [Probe], target: Target, what: &str) -> &'a Probe {
        let found: Vec<&Probe> = probes
            .iter()
            .filter(|p| p.target == target && p.what == what)
            .collect();
        let [probe] = found[..] else {
            panic!("{target:?} {what}: one probe, not {found:?}");
        };
        probe
    }

    #[test]
    fn the_bar_is_read_at_its_edges_against_the_surface() {
        let cell = NavCell::default();
        let probes = probes(cell, &LIGHT, None);
        let target = Target::NavigationBar(cell);
        let (x, y) = cell.origin();
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let container = Rgb::of(&LIGHT, bar::CONTAINER_COLOR);
        // The color of a bar's container, which is not the page's.
        assert_ne!(surface, container);
        let at = |what: &str, px: f32, py: f32, expected: Rgb| {
            let probe = named(&probes, target, what);
            assert_eq!(
                (probe.x, probe.y, probe.expected, probe.tolerance),
                (Points(px), Points(py), expected, 0),
                "{what}"
            );
        };
        // A bar of 480 points at 24, 2,664 and 64 points tall: its middle is 2,696.
        assert_eq!((x, y), (Points(24.0), Points(2760.0)));
        at("surface left of the bar", 23.0, 2792.0, surface);
        at("container at the left edge", 24.0, 2792.0, container);
        at("container at the right edge", 503.0, 2792.0, container);
        at("surface right of the bar", 504.0, 2792.0, surface);
        at("surface above the bar", 26.0, 2759.0, surface);
        at("container at the top", 26.0, 2760.0, container);
        at("container at the bottom", 26.0, 2823.0, container);
        at("surface below the bar", 26.0, 2824.0, surface);
    }

    #[test]
    fn every_item_is_read_above_its_pill_in_the_containers_color() {
        let cell = NavCell::default();
        for scheme in [LIGHT, DARK] {
            let probes = probes(cell, &scheme, None);
            let container = Rgb::of(&scheme, bar::CONTAINER_COLOR);
            for (state, center) in [
                (ItemState::Selected, 104.0),
                (ItemState::Unselected, 264.0),
                (ItemState::Disabled, 424.0),
            ] {
                let probe = named(
                    &probes,
                    Target::NavigationItem(cell, state),
                    "above the pill",
                );
                assert_eq!(
                    (probe.x, probe.y, probe.expected, probe.tolerance),
                    (Points(center), Points(2762.0), container, 0),
                    "{state:?}"
                );
            }
        }
    }

    #[test]
    fn only_the_selected_item_has_a_pill() {
        let cell = NavCell::default();
        let probes = probes(cell, &LIGHT, None);
        let pill = Rgb::of(&LIGHT, bar::ITEM_ACTIVE_INDICATOR_COLOR);
        let container = Rgb::of(&LIGHT, bar::CONTAINER_COLOR);
        assert_ne!(pill, container);
        // The icon on top, with a label: the pill is 56 × 32 points, 6 below the bar's top (2,664), so its middle is
        // 2,686; read 20 points left of the item's middle, halfway between the icon's side and the pill's.
        let at = |state, expected: Rgb, x: f32| {
            let probe = named(&probes, Target::NavigationItem(cell, state), "pill");
            assert_eq!(
                (probe.x, probe.y, probe.expected, probe.tolerance),
                (Points(x), Points(2782.0), expected, 0),
                "{state:?}"
            );
        };
        at(ItemState::Selected, pill, 84.0);
        at(ItemState::Unselected, container, 244.0);
        at(ItemState::Disabled, container, 404.0);
    }

    #[test]
    fn the_icon_takes_the_selected_unselected_and_disabled_colors() {
        let cell = NavCell::default();
        for scheme in [LIGHT, DARK] {
            let probes = probes(cell, &scheme, None);
            let container = Rgb::of(&scheme, bar::CONTAINER_COLOR);
            let icon = |state| {
                let probe = named(&probes, Target::NavigationItem(cell, state), "icon");
                // The icon's middle: the pill's, 2,686 points down.
                assert_eq!(probe.y, Points(2782.0));
                (probe.x, probe.expected, probe.tolerance)
            };
            let selected = Rgb::of(&scheme, bar::ITEM_ACTIVE_ICON_COLOR);
            let unselected = Rgb::of(&scheme, bar::ITEM_INACTIVE_ICON_COLOR);
            // Two colors of the scheme that differ, so that no item can pass for another.
            assert_ne!(selected, unselected);
            assert_eq!(icon(ItemState::Selected), (Points(104.0), selected, 0));
            assert_eq!(icon(ItemState::Unselected), (Points(264.0), unselected, 0));
            // Disabled: the unselected icon color at 0.38 over the container, a blend.
            let (faded_icon, tolerance) = faded(
                &scheme,
                bar::ITEM_INACTIVE_ICON_COLOR,
                DISABLED_OPACITY,
                container,
            );
            assert_ne!(faded_icon, unselected);
            assert_ne!(faded_icon, container);
            assert_eq!(
                icon(ItemState::Disabled),
                (Points(424.0), faded_icon, tolerance)
            );
        }
    }

    #[test]
    fn the_state_layer_lies_over_the_pill_and_the_icon_of_the_hovered_item_only() {
        let cell = NavCell::default();
        let surface_layer = |below: Rgb| {
            faded(
                &LIGHT,
                ColorRole::OnSurface,
                state_tokens::HOVER_STATE_LAYER_OPACITY,
                below,
            )
        };
        let container = Rgb::of(&LIGHT, bar::CONTAINER_COLOR);
        let pill = Rgb::of(&LIGHT, bar::ITEM_ACTIVE_INDICATOR_COLOR);
        let selected_icon = Rgb::of(&LIGHT, bar::ITEM_ACTIVE_ICON_COLOR);
        let unselected_icon = Rgb::of(&LIGHT, bar::ITEM_INACTIVE_ICON_COLOR);
        let read = |hovered: Option<ItemState>, state, what| {
            let probes = probes(cell, &LIGHT, hovered);
            let probe = named(&probes, Target::NavigationItem(cell, state), what);
            (probe.expected, probe.tolerance)
        };
        // The mouse over the selected item: its pill and its icon are tinted, the others' are not.
        let hovered = Some(ItemState::Selected);
        assert_eq!(
            read(hovered, ItemState::Selected, "pill"),
            surface_layer(pill)
        );
        assert_eq!(
            read(hovered, ItemState::Selected, "icon"),
            surface_layer(selected_icon)
        );
        assert_eq!(read(hovered, ItemState::Unselected, "pill"), (container, 0));
        assert_eq!(
            read(hovered, ItemState::Unselected, "icon"),
            (unselected_icon, 0)
        );
        // Over the unselected one: the state layer over the container, where the pill would be.
        let hovered = Some(ItemState::Unselected);
        assert_eq!(
            read(hovered, ItemState::Unselected, "pill"),
            surface_layer(container)
        );
        assert_eq!(
            read(hovered, ItemState::Unselected, "icon"),
            surface_layer(unselected_icon)
        );
        assert_eq!(read(hovered, ItemState::Selected, "pill"), (pill, 0));
        // The tint is a visible one: no state can pass for another.
        assert_ne!(surface_layer(pill).0, pill);
        assert_ne!(surface_layer(container).0, container);
        // Over a disabled one, no state layer: as at rest.
        let hovered = Some(ItemState::Disabled);
        assert_eq!(
            read(hovered, ItemState::Disabled, "pill"),
            read(None, ItemState::Disabled, "pill")
        );
        assert_eq!(
            read(hovered, ItemState::Disabled, "icon"),
            read(None, ItemState::Disabled, "icon")
        );
        // The rest of the bar does not change when the mouse is over an item.
        assert_eq!(
            read(hovered, ItemState::Selected, "above the pill"),
            (container, 0)
        );
    }

    #[test]
    fn the_places_follow_the_icon_position_and_the_label() {
        // Top, shown: the pill hangs 6 points from the top (2,664): middle 2,686. Top, hidden: centered, 2,696.
        let (pill, icon) = places(
            cell(NavigationItemIconPosition::Top, NavigationItemLabel::Shown),
            104.0,
            2792.0,
            2760.0,
        );
        assert_eq!((pill, icon), ((84.0, 2782.0), Some((104.0, 2782.0))));
        let (pill, icon) = places(
            cell(NavigationItemIconPosition::Top, NavigationItemLabel::Hidden),
            104.0,
            2792.0,
            2760.0,
        );
        assert_eq!((pill, icon), ((84.0, 2792.0), Some((104.0, 2792.0))));
        // Start: the pill is read 16 points above the middle; the icon only where there is no label.
        let (pill, icon) = places(
            cell(
                NavigationItemIconPosition::Start,
                NavigationItemLabel::Hidden,
            ),
            104.0,
            2792.0,
            2760.0,
        );
        assert_eq!((pill, icon), ((104.0, 2776.0), Some((104.0, 2792.0))));
        let (pill, icon) = places(
            cell(
                NavigationItemIconPosition::Start,
                NavigationItemLabel::Shown,
            ),
            104.0,
            2792.0,
            2760.0,
        );
        assert_eq!((pill, icon), ((104.0, 2776.0), None));
    }

    #[test]
    fn every_bar_is_read_and_every_probe_is_inside_the_catalog() {
        let (width, height) = catalog::grid::viewport();
        let mut count = 0;
        for arrangement in ARRANGEMENTS {
            for position in ICON_POSITIONS {
                for label in LABELS {
                    let cell = NavCell::new(arrangement, position, label);
                    let probes = probes(cell, &LIGHT, None);
                    // The bar's 8 edges, and for each of the 3 items: above the pill and the pill, and the icon unless
                    // the label's width decides where it is.
                    let icons = match (position, label) {
                        (NavigationItemIconPosition::Start, NavigationItemLabel::Shown) => 0,
                        _ => ITEMS.len(),
                    };
                    assert_eq!(probes.len(), 8 + 2 * ITEMS.len() + icons, "{cell:?}");
                    for probe in &probes {
                        assert!(probe.x.0 >= 0.0 && probe.x.0 < width.0, "{probe:?}");
                        assert!(probe.y.0 >= 0.0 && probe.y.0 < height.0, "{probe:?}");
                    }
                    count += 1;
                }
            }
        }
        assert_eq!(count, NavCell::all().count());
    }
}
