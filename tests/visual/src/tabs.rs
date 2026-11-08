//! The tab rows' known pixels: the row's edges against the surface, the divider under the tabs that are not selected,
//! and for each tab the container, the indicator bar (or what shows where it would be), the icon's color and the state
//! layer of the mouse over it. The labels are not read: the antialiasing of their glyphs makes every pixel of them
//! unknown, and so is where an icon is next to a label.
//!
//! A row's container is `Surface`, the color of the page it sits on, so its box cannot be read at its edges as a
//! navigation bar's can: its bottom line is, where the divider is, and the container is read in each tab.

use catalog::tabs::{Content, RowKind, TABS, TabCell, TabState};
use nsh_core::geometry::Points;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::navigation_item::DISABLED_OPACITY;
use nsh_m3e::tokens::androidx::{
    divider_tokens as divider, primary_navigation_tab_tokens as primary,
    secondary_navigation_tab_tokens as secondary, state_tokens,
};

use super::{Probe, Rgb, Target, faded};

/// How far inside or outside the end of the indicator bar its color is read: past the antialiasing of the capsule's
/// ends. Not a design value: where the check looks.
const END_INSET: f32 = 4.0;

/// The color of a row's container: `PrimaryNavigationTabTokens.ContainerColor` and
/// `SecondaryNavigationTabTokens.ContainerColor`, `Surface` for both rows, which a test pins (so no row has a function of
/// its own to choose it, where a mutant that picks the other token would read the same color).
const CONTAINER: ColorRole = primary::CONTAINER_COLOR;

/// The color of the state layer: the row's content color, which androidx's ripple takes (`Tab.kt`, lines 166 and 239:
/// `ripple(bounded = true, color = selectedContentColor)`; `TabRow.kt`, lines 1016-1021: `primaryContentColor` is
/// `Primary`, `secondaryContentColor` is `OnSurface`). Written here, not taken from the library, so that the check is
/// the source's word and not the library's own.
fn state_layer(kind: RowKind) -> ColorRole {
    match kind {
        RowKind::Secondary => ColorRole::OnSurface,
        _ => ColorRole::Primary,
    }
}

/// The colors of a tab's icon: the selected tab's, and the other tabs', at rest and with the mouse over the tab. Under
/// the mouse an unselected tab's icon goes from `OnSurfaceVariant` to `OnSurface` (ADR 0021); the secondary tokens name
/// one color for a state, for both.
fn icon_roles(kind: RowKind, hovered: bool) -> (ColorRole, ColorRole) {
    match (kind, hovered) {
        (RowKind::Secondary, false) => {
            (secondary::ACTIVE_ICON_COLOR, secondary::INACTIVE_ICON_COLOR)
        }
        (RowKind::Secondary, true) => (secondary::HOVER_ICON_COLOR, secondary::HOVER_ICON_COLOR),
        (_, false) => (primary::ACTIVE_ICON_COLOR, primary::INACTIVE_ICON_COLOR),
        (_, true) => (
            primary::ACTIVE_HOVER_ICON_COLOR,
            primary::INACTIVE_HOVER_ICON_COLOR,
        ),
    }
}

/// The pixels the checks know for the row `cell` in a screenshot in `scheme`, with the mouse over the tab in the state
/// `hovered`, if any.
pub(super) fn probes(cell: TabCell, scheme: &ColorScheme, hovered: Option<TabState>) -> Vec<Probe> {
    let mut probes = row_probes(cell, scheme);
    for state in TABS {
        probes.extend(tab_probes(cell, state, scheme, hovered == Some(state)));
    }
    probes
}

/// The pixels of the row itself: the surface around it, and the divider's last pixel at its right end, which is under
/// the disabled tab, where no indicator is.
fn row_probes(cell: TabCell, scheme: &ColorScheme) -> Vec<Probe> {
    let (x, y) = cell.origin();
    let (width, height) = (cell.width().0, cell.height().0);
    let line = y.0 + height - divider::THICKNESS.0;
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let target = Target::TabRow(cell);
    let edge = |what, x: f32, y: f32, expected| Probe {
        target,
        what,
        x: Points(x),
        y: Points(y),
        expected,
        tolerance: 0,
    };
    vec![
        edge("surface left of the row", x.0 - 1.0, line, surface),
        edge("surface right of the row", x.0 + width, line, surface),
        edge("surface above the row", x.0 + 2.0, y.0 - 1.0, surface),
        edge("surface below the row", x.0 + 2.0, y.0 + height, surface),
        edge(
            "divider at the right edge",
            x.0 + width - 1.0,
            line,
            Rgb::of(scheme, divider::COLOR),
        ),
    ]
}

/// The pixels the checks know for the tab of `cell` in `state`, with the mouse over it or not.
fn tab_probes(cell: TabCell, state: TabState, scheme: &ColorScheme, hovered: bool) -> Vec<Probe> {
    let target = Target::Tab(cell, state);
    let kind = cell.kind;
    let (center_x, middle) = cell.tab_center(state);
    let left = cell.tab_left(state).0;
    let width = cell.tab_width().0;
    let (_, top) = cell.origin();
    let bottom = top.0 + cell.height().0;
    // The indicator's band is the bar's height at the bottom of the row, with the divider's line as its last row. The
    // check reads its first row, and the row between that and the divider's line, which is neither.
    let band_top = bottom - primary::ACTIVE_INDICATOR_HEIGHT.0;
    let band_middle = band_top + 1.0;
    let line = bottom - divider::THICKNESS.0;
    let selected = state == TabState::Selected;
    let container = Rgb::of(scheme, CONTAINER);
    let indicator = Rgb::of(scheme, primary::ACTIVE_INDICATOR_COLOR);
    // A disabled tab shows no state, whatever the mouse does.
    let hovered = hovered && state != TabState::Disabled;
    let (active_icon, inactive_icon) = icon_roles(kind, hovered);
    // The state layer lies over the container and the icon of an enabled tab, and under the divider and the bar, which
    // the row draws after its tabs.
    let over = |below: Rgb| {
        if hovered {
            faded(
                scheme,
                state_layer(kind),
                state_tokens::HOVER_STATE_LAYER_OPACITY,
                below,
            )
        } else {
            (below, 0)
        }
    };
    let solid = |color: Rgb| (color, 0);
    // The secondary row's bar is as wide as its tab: it reaches both of the tab's edges.
    let edge_color = if selected && kind == RowKind::Secondary {
        solid(indicator)
    } else {
        over(container)
    };
    let (top_color, bottom_color) = if selected {
        (solid(indicator), solid(indicator))
    } else {
        (over(container), solid(Rgb::of(scheme, divider::COLOR)))
    };
    let probe = |what, x: f32, y: f32, (expected, tolerance): (Rgb, u8)| Probe {
        target,
        what,
        x: Points(x),
        y: Points(y),
        expected,
        tolerance,
    };
    let mut probes = vec![
        probe("container", center_x.0, top.0 + 2.0, over(container)),
        probe("above the bar", center_x.0, band_top - 1.0, over(container)),
        probe("bar top", center_x.0, band_top, top_color),
        // The bar is drawn over the divider: at its bottom line, the selected tab's is the bar's color.
        probe("bottom line", center_x.0, line, bottom_color),
        probe("bar at the left edge", left, band_middle, edge_color),
        probe(
            "bar at the right edge",
            left + width - 1.0,
            band_middle,
            edge_color,
        ),
    ];
    if selected && kind != RowKind::Secondary && cell.content == Content::Icon {
        // The primary bar is as wide as the content less its padding, at least 24 points: under an icon alone, the
        // icon's width. A label's width is the font's, so only an icon's bar has ends the check can know.
        let half = primary::ICON_SIZE.0 / 2.0;
        probes.extend([
            probe(
                "bar inside its left end",
                center_x.0 - (half - END_INSET),
                band_middle,
                solid(indicator),
            ),
            probe(
                "bar inside its right end",
                center_x.0 + (half - END_INSET),
                band_middle,
                solid(indicator),
            ),
            probe(
                "left of the bar",
                center_x.0 - (half + END_INSET),
                band_middle,
                over(container),
            ),
            probe(
                "right of the bar",
                center_x.0 + (half + END_INSET),
                band_middle,
                over(container),
            ),
        ]);
    }
    if cell.content == Content::Icon {
        // An icon alone is in the tab's middle. Next to a label, where it is depends on the label's width.
        let color = match state {
            TabState::Selected => over(Rgb::of(scheme, active_icon)),
            TabState::Disabled => faded(scheme, inactive_icon, DISABLED_OPACITY, container),
            _ => over(Rgb::of(scheme, inactive_icon)),
        };
        probes.push(probe("icon", center_x.0, middle.0, color));
    }
    probes
}

#[cfg(test)]
mod tests {
    use catalog::tabs::{CONTENTS, KINDS};

    use super::*;

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;
    const DARK: ColorScheme = ColorScheme::BASELINE_DARK;

    fn cell(kind: RowKind, content: Content) -> TabCell {
        TabCell::new(kind, content)
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

    /// What `what` of the tab in `state` reads in a row of `cell` with the mouse over `hovered`: the point, the color
    /// and the tolerance.
    fn read(
        cell: TabCell,
        scheme: &ColorScheme,
        hovered: Option<TabState>,
        state: TabState,
        what: &str,
    ) -> ((Points, Points), Rgb, u8) {
        let all = probes(cell, scheme, hovered);
        let probe = named(&all, Target::Tab(cell, state), what);
        ((probe.x, probe.y), probe.expected, probe.tolerance)
    }

    fn layer(kind: RowKind, below: Rgb) -> (Rgb, u8) {
        faded(
            &LIGHT,
            state_layer(kind),
            state_tokens::HOVER_STATE_LAYER_OPACITY,
            below,
        )
    }

    #[test]
    fn both_rows_have_the_same_container() {
        // The two tokens name one color, the page's: a row's container is read with one constant.
        assert_eq!(primary::CONTAINER_COLOR, secondary::CONTAINER_COLOR);
        assert_eq!(CONTAINER, ColorRole::Surface);
    }

    #[test]
    fn the_row_is_read_at_its_edges_against_the_surface() {
        let cell = TabCell::default();
        let all = probes(cell, &LIGHT, None);
        let target = Target::TabRow(cell);
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let divider = Rgb::of(&LIGHT, divider::COLOR);
        // The row's container is the page's color: its edges are the divider's.
        assert_eq!(Rgb::of(&LIGHT, CONTAINER), surface);
        assert_ne!(divider, surface);
        let at = |what: &str, px: f32, py: f32, expected: Rgb| {
            let probe = named(&all, target, what);
            assert_eq!(
                (probe.x, probe.y, probe.expected, probe.tolerance),
                (Points(px), Points(py), expected, 0),
                "{what}"
            );
        };
        // A row of 480 points at 24, 2,824 and 48 points tall: its divider's line is 2,871.
        assert_eq!(cell.origin(), (Points(24.0), Points(2824.0)));
        at("surface left of the row", 23.0, 2871.0, surface);
        at("surface right of the row", 504.0, 2871.0, surface);
        at("surface above the row", 26.0, 2823.0, surface);
        at("surface below the row", 26.0, 2872.0, surface);
        at("divider at the right edge", 503.0, 2871.0, divider);
        assert_eq!(all.iter().filter(|p| p.target == target).count(), 5);
    }

    #[test]
    fn every_tab_is_read_in_the_containers_color_at_rest() {
        for kind in KINDS {
            let cell = cell(kind, Content::Label);
            for scheme in [LIGHT, DARK] {
                let container = Rgb::of(&scheme, CONTAINER);
                let page = Rgb::of(&scheme, ColorRole::Surface);
                assert_eq!(container, page);
                for (state, center) in [
                    (TabState::Selected, 104.0),
                    (TabState::Unselected, 264.0),
                    (TabState::Disabled, 424.0),
                ] {
                    for what in ["container", "above the bar"] {
                        let ((x, _), expected, tolerance) = read(cell, &scheme, None, state, what);
                        assert_eq!(
                            (x, expected, tolerance),
                            (Points(center), container, 0),
                            "{kind:?} {state:?} {what}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn only_the_selected_tab_has_a_bar_over_the_divider() {
        let cell = TabCell::default();
        let indicator = Rgb::of(&LIGHT, primary::ACTIVE_INDICATOR_COLOR);
        let divider = Rgb::of(&LIGHT, divider::COLOR);
        let container = Rgb::of(&LIGHT, primary::CONTAINER_COLOR);
        // Colors of the scheme that differ, so that no pixel can pass for another.
        assert_ne!(indicator, divider);
        assert_ne!(indicator, container);
        assert_ne!(divider, container);
        // The bar is 3 points tall at the bottom of a row that ends at 2,872: its first row is 2,869, the divider's
        // line, its last, 2,871.
        let top = |state| read(cell, &LIGHT, None, state, "bar top");
        assert_eq!(
            top(TabState::Selected),
            ((Points(104.0), Points(2869.0)), indicator, 0)
        );
        assert_eq!(
            top(TabState::Unselected),
            ((Points(264.0), Points(2869.0)), container, 0)
        );
        assert_eq!(
            top(TabState::Disabled),
            ((Points(424.0), Points(2869.0)), container, 0)
        );
        let line = |state| read(cell, &LIGHT, None, state, "bottom line");
        assert_eq!(
            line(TabState::Selected),
            ((Points(104.0), Points(2871.0)), indicator, 0)
        );
        assert_eq!(
            line(TabState::Unselected),
            ((Points(264.0), Points(2871.0)), divider, 0)
        );
        assert_eq!(
            line(TabState::Disabled),
            ((Points(424.0), Points(2871.0)), divider, 0)
        );
        // The bar is no taller than 3 points: above it, the container.
        assert_eq!(
            read(cell, &LIGHT, None, TabState::Selected, "above the bar"),
            ((Points(104.0), Points(2868.0)), container, 0)
        );
    }

    #[test]
    fn the_secondary_bar_is_as_wide_as_its_tab_and_the_primary_one_is_not() {
        let primary_row = cell(RowKind::Primary, Content::Label);
        let secondary_row = cell(RowKind::Secondary, Content::Label);
        let indicator = Rgb::of(&LIGHT, primary::ACTIVE_INDICATOR_COLOR);
        let container = Rgb::of(&LIGHT, secondary::CONTAINER_COLOR);
        // The second row is at 2,888: its bar's middle row is 2,934. The selected tab spans 24 to 184.
        assert_eq!(secondary_row.origin(), (Points(24.0), Points(2888.0)));
        for (what, x) in [
            ("bar at the left edge", 24.0),
            ("bar at the right edge", 183.0),
        ] {
            assert_eq!(
                read(secondary_row, &LIGHT, None, TabState::Selected, what),
                ((Points(x), Points(2934.0)), indicator, 0),
                "{what}"
            );
            // The tabs next to it have none at their edges.
            assert_eq!(
                read(secondary_row, &LIGHT, None, TabState::Unselected, what).1,
                container,
                "{what}"
            );
            // The primary row's bar stays under the content: its tab's edges show the container.
            assert_eq!(
                read(primary_row, &LIGHT, None, TabState::Selected, what).1,
                Rgb::of(&LIGHT, primary::CONTAINER_COLOR),
                "{what}"
            );
        }
        assert_ne!(indicator, container);
    }

    #[test]
    fn an_icons_bar_is_as_wide_as_the_icon_and_read_at_four_points_from_each_end() {
        let row = cell(RowKind::Primary, Content::Icon);
        let indicator = Rgb::of(&LIGHT, primary::ACTIVE_INDICATOR_COLOR);
        let container = Rgb::of(&LIGHT, primary::CONTAINER_COLOR);
        // The icon column starts at 664: the selected tab's middle is 744, and the bar spans 732 to 756.
        let read = |what| read(row, &LIGHT, None, TabState::Selected, what);
        assert_eq!(
            read("bar inside its left end"),
            ((Points(736.0), Points(2870.0)), indicator, 0)
        );
        assert_eq!(
            read("bar inside its right end"),
            ((Points(752.0), Points(2870.0)), indicator, 0)
        );
        assert_eq!(
            read("left of the bar"),
            ((Points(728.0), Points(2870.0)), container, 0)
        );
        assert_eq!(
            read("right of the bar"),
            ((Points(760.0), Points(2870.0)), container, 0)
        );
        // Only the selected tab of a primary row with an icon alone has them: a label's width is the font's, and a
        // secondary bar has no ends inside its tab.
        for (kind, content) in [
            (RowKind::Primary, Content::Label),
            (RowKind::Primary, Content::Both),
            (RowKind::Secondary, Content::Icon),
        ] {
            let all = probes(cell(kind, content), &LIGHT, None);
            assert!(
                all.iter()
                    .all(|p| !p.what.starts_with("bar inside") && !p.what.ends_with("of the bar")),
                "{kind:?} {content:?}"
            );
        }
    }

    #[test]
    fn the_icon_takes_each_rows_selected_unselected_and_disabled_colors() {
        for kind in KINDS {
            let cell = cell(kind, Content::Icon);
            for scheme in [LIGHT, DARK] {
                let container = Rgb::of(&scheme, CONTAINER);
                let (active, inactive) = icon_roles(kind, false);
                let (selected, unselected) = (Rgb::of(&scheme, active), Rgb::of(&scheme, inactive));
                // Two colors of the scheme that differ, so that no tab can pass for another.
                assert_ne!(selected, unselected, "{kind:?}");
                // The icon's middle: the icon column's first tab's, 744, and the row's, 24 points down.
                let top = if kind == RowKind::Secondary {
                    2912.0
                } else {
                    2848.0
                };
                let icon = |state| read(cell, &scheme, None, state, "icon");
                assert_eq!(
                    icon(TabState::Selected),
                    ((Points(744.0), Points(top)), selected, 0),
                    "{kind:?}"
                );
                assert_eq!(
                    icon(TabState::Unselected),
                    ((Points(904.0), Points(top)), unselected, 0),
                    "{kind:?}"
                );
                // Disabled: the unselected color at 0.38 over the container, a blend.
                let (faded_icon, tolerance) = faded(&scheme, inactive, DISABLED_OPACITY, container);
                assert_ne!(faded_icon, unselected);
                assert_ne!(faded_icon, container);
                assert_eq!(
                    icon(TabState::Disabled),
                    ((Points(1064.0), Points(top)), faded_icon, tolerance),
                    "{kind:?}"
                );
            }
        }
        // The primary row's selected icon is the primary color, the secondary row's the content color: the rows differ.
        assert_ne!(
            Rgb::of(&LIGHT, icon_roles(RowKind::Primary, false).0),
            Rgb::of(&LIGHT, icon_roles(RowKind::Secondary, false).0)
        );
        // Rows that show no icon read none.
        for kind in KINDS {
            for content in [Content::Label, Content::Both] {
                let all = probes(cell(kind, content), &LIGHT, None);
                assert!(all.iter().all(|p| p.what != "icon"), "{kind:?} {content:?}");
            }
        }
    }

    #[test]
    fn the_state_layer_lies_over_the_container_and_the_icon_of_the_hovered_tab_only() {
        for kind in KINDS {
            let cell = cell(kind, Content::Icon);
            let container = Rgb::of(&LIGHT, CONTAINER);
            let (active, inactive) = icon_roles(kind, false);
            let (active_hovered, inactive_hovered) = icon_roles(kind, true);
            let (selected_icon, unselected_icon) =
                (Rgb::of(&LIGHT, active), Rgb::of(&LIGHT, inactive));
            // Under the mouse the selected tab's icon keeps its color and the unselected one's goes to `OnSurface`.
            let (selected_hovered, unselected_hovered) = (
                Rgb::of(&LIGHT, active_hovered),
                Rgb::of(&LIGHT, inactive_hovered),
            );
            assert_eq!(selected_hovered, selected_icon, "{kind:?}");
            assert_ne!(unselected_hovered, unselected_icon, "{kind:?}");
            let color = |hovered, state, what: &str| {
                let (_, expected, tolerance) = read(cell, &LIGHT, hovered, state, what);
                (expected, tolerance)
            };
            // The mouse over the selected tab: its container and its icon are tinted, the other tabs' are not.
            let hovered = Some(TabState::Selected);
            assert_eq!(
                color(hovered, TabState::Selected, "container"),
                layer(kind, container)
            );
            assert_eq!(
                color(hovered, TabState::Selected, "icon"),
                layer(kind, selected_hovered)
            );
            assert_eq!(
                color(hovered, TabState::Unselected, "container"),
                (container, 0)
            );
            assert_eq!(
                color(hovered, TabState::Unselected, "icon"),
                (unselected_icon, 0)
            );
            // The layer lies under the bar, which is drawn after the tabs: the bar's own pixels do not change.
            assert_eq!(
                color(hovered, TabState::Selected, "bar top"),
                color(None, TabState::Selected, "bar top")
            );
            assert_eq!(
                color(hovered, TabState::Selected, "bottom line"),
                color(None, TabState::Selected, "bottom line")
            );
            // Over the unselected one: the layer over the container, and over its icon in the color it has under the
            // mouse. The selected tab's icon is as it was.
            let hovered = Some(TabState::Unselected);
            assert_eq!(
                color(hovered, TabState::Unselected, "container"),
                layer(kind, container)
            );
            assert_eq!(
                color(hovered, TabState::Unselected, "icon"),
                layer(kind, unselected_hovered)
            );
            assert_eq!(
                color(hovered, TabState::Selected, "icon"),
                (selected_icon, 0)
            );
            // Where the bar would be, the layer over the container; the divider's line is drawn over it.
            assert_eq!(
                color(hovered, TabState::Unselected, "bar top"),
                layer(kind, container)
            );
            assert_eq!(
                color(hovered, TabState::Unselected, "bottom line"),
                color(None, TabState::Unselected, "bottom line")
            );
            // The tint is a visible one: no state can pass for another. A selected tab's icon is the layer's own color, so
            // the layer changes nothing over it and only the container shows it there; the icon of the others changes.
            assert_ne!(layer(kind, container).0, container);
            assert_eq!(layer(kind, selected_icon).0, selected_icon);
            // The primary row's layer (`Primary`) tints an unselected tab's icon under the mouse (`OnSurface`); the
            // secondary row's layer and that icon are both `OnSurface`, and the layer changes nothing over it.
            if state_layer(kind) == inactive_hovered {
                assert_eq!(layer(kind, unselected_hovered).0, unselected_hovered);
            } else {
                assert_ne!(layer(kind, unselected_hovered).0, unselected_hovered);
            }
            assert_ne!(layer(kind, unselected_hovered).0, unselected_icon);
            // Over a disabled one, no state layer: as at rest.
            let hovered = Some(TabState::Disabled);
            for what in ["container", "icon", "bar top", "bottom line"] {
                assert_eq!(
                    color(hovered, TabState::Disabled, what),
                    color(None, TabState::Disabled, what),
                    "{kind:?} {what}"
                );
            }
        }
    }

    #[test]
    fn the_state_layer_is_each_rows_content_color() {
        // The primary row's is the primary color, the secondary row's on surface: the layer over the container differs.
        let primary_layer = layer(RowKind::Primary, Rgb::of(&LIGHT, ColorRole::Surface));
        let secondary_layer = layer(RowKind::Secondary, Rgb::of(&LIGHT, ColorRole::Surface));
        assert_ne!(primary_layer.0, secondary_layer.0);
        assert_eq!(state_layer(RowKind::Primary), ColorRole::Primary);
        assert_eq!(state_layer(RowKind::Secondary), ColorRole::OnSurface);
    }

    #[test]
    fn every_row_is_read_and_every_probe_is_inside_the_catalog() {
        let (width, height) = catalog::grid::viewport();
        let mut count = 0;
        for kind in KINDS {
            for content in CONTENTS {
                let cell = cell(kind, content);
                let all = probes(cell, &LIGHT, None);
                // The row's 5 edges, and for each of the 3 tabs: 6 reads of the container and the bar, 3 icons when the
                // icon is alone, and for the selected tab of a primary row with an icon alone the 4 reads of the bar's
                // ends.
                let icons = if content == Content::Icon {
                    TABS.len()
                } else {
                    0
                };
                let ends = if content == Content::Icon && kind != RowKind::Secondary {
                    4
                } else {
                    0
                };
                assert_eq!(all.len(), 5 + 6 * TABS.len() + icons + ends, "{cell:?}");
                for probe in &all {
                    assert!(probe.x.0 >= 0.0 && probe.x.0 < width.0, "{probe:?}");
                    assert!(probe.y.0 >= 0.0 && probe.y.0 < height.0, "{probe:?}");
                }
                count += 1;
            }
        }
        assert_eq!(count, TabCell::all().count());
    }
}
