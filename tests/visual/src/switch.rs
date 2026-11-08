//! The pixels the checks know for the catalog's switches: the track's color beside the handle, the outline's on the
//! top edge of an unchecked track (and the plain track's on a checked one, which has none), the handle's, the icon's
//! in the middle of the handle, the surface outside the track's top left corner, and the state layer above the track.
//!
//! Every expected color comes from the switch's tokens, by the rule `Switch.kt` gives each of them (androidx at the
//! pinned commit, as ADR 0012 says), written again here and not taken from the library: a check that asked the library
//! for its colors could not tell it was wrong. Where the antialiasing of an edge, of the handle's, or of the icon's
//! makes a color unknown, nothing is checked.

use catalog::grid::Availability;
use catalog::switch::{Content, HEIGHT, Selection, SwitchCell, WIDTH};
use nsh_core::geometry::Points;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::tokens::androidx::{state_tokens, switch_tokens};

use super::{Probe, Rgb, Target, faded};

/// The switches the test script hovers (`examples/catalog/switch.test.tsx`): the off one with a plain handle and the
/// on one with an icon, which are the two handles' two ends, and the two places the state layer is read.
pub(crate) const HOVERED: [SwitchCell; 2] = [
    SwitchCell::new(Selection::Off, Content::Plain, Availability::Enabled),
    SwitchCell::new(Selection::On, Content::Icon, Availability::Enabled),
];

/// How far inside the handle's edge its color is read: past the antialiased edge, and past the icon, which spans 8
/// points on each side of the handle's center, in a handle that has 12 or 8. Not a design value: where the check looks.
const HANDLE_INSET: Points = Points(2.0);

/// How far from the end of the track's straight top edge its color is read, at the end away from the handle: past the
/// antialiased arc, and out of the state layer, which is centered on the handle and reaches 20 points from its center.
/// Not a design value: where the check looks.
const EDGE_INSET: Points = Points(4.0);

/// A color of a switch: a role of the scheme, at an opacity.
#[derive(Debug, Clone, Copy)]
struct Shade {
    role: ColorRole,
    opacity: f32,
}

impl Shade {
    const fn solid(role: ColorRole) -> Self {
        Self { role, opacity: 1.0 }
    }

    const fn faded(role: ColorRole, opacity: f32) -> Self {
        Self { role, opacity }
    }
}

/// The colors of a switch's parts.
#[derive(Debug, Clone, Copy)]
struct Colors {
    track: Shade,
    /// The track's outline: none where the track has none.
    outline: Option<Shade>,
    handle: Shade,
    icon: Shade,
}

/// The colors of `cell`'s parts: androidx's `SwitchDefaults.defaultSwitchColors` (`Switch.kt` lines 426-469). The
/// handle's, the track's and the icon's are those of the token file's plain `Selected` and `Unselected` names, and the
/// outline's the focus token's (`UnselectedFocusTrackOutlineColor`, lines 436-437): the hover, focus and pressed
/// colors are not read by the component (ADR 0012).
fn colors(cell: SwitchCell) -> Colors {
    use switch_tokens as t;
    let on = cell.selection == Selection::On;
    if cell.availability == Availability::Disabled {
        // Each at its token's opacity, composited over the surface (`compositeOver(surface)`).
        if on {
            Colors {
                track: Shade::faded(t::DISABLED_SELECTED_TRACK_COLOR, t::DISABLED_TRACK_OPACITY),
                outline: None,
                handle: Shade::faded(
                    t::DISABLED_SELECTED_HANDLE_COLOR,
                    t::DISABLED_SELECTED_HANDLE_OPACITY,
                ),
                icon: Shade::faded(
                    t::DISABLED_SELECTED_ICON_COLOR,
                    t::DISABLED_SELECTED_ICON_OPACITY,
                ),
            }
        } else {
            Colors {
                track: Shade::faded(
                    t::DISABLED_UNSELECTED_TRACK_COLOR,
                    t::DISABLED_TRACK_OPACITY,
                ),
                outline: Some(Shade::faded(
                    t::DISABLED_UNSELECTED_TRACK_OUTLINE_COLOR,
                    t::DISABLED_TRACK_OPACITY,
                )),
                handle: Shade::faded(
                    t::DISABLED_UNSELECTED_HANDLE_COLOR,
                    t::DISABLED_UNSELECTED_HANDLE_OPACITY,
                ),
                icon: Shade::faded(
                    t::DISABLED_UNSELECTED_ICON_COLOR,
                    t::DISABLED_UNSELECTED_ICON_OPACITY,
                ),
            }
        }
    } else if on {
        Colors {
            track: Shade::solid(t::SELECTED_TRACK_COLOR),
            outline: None,
            handle: Shade::solid(t::SELECTED_HANDLE_COLOR),
            icon: Shade::solid(t::SELECTED_ICON_COLOR),
        }
    } else {
        Colors {
            track: Shade::solid(t::UNSELECTED_TRACK_COLOR),
            outline: Some(Shade::solid(t::UNSELECTED_FOCUS_TRACK_OUTLINE_COLOR)),
            handle: Shade::solid(t::UNSELECTED_HANDLE_COLOR),
            icon: Shade::solid(t::UNSELECTED_ICON_COLOR),
        }
    }
}

/// The color a part of `cell` shows, and the tolerance it is read with. An enabled switch's colors are the roles' as
/// they are, exact; a disabled one's are laid over the surface at an opacity, as a blend is read (`Switch.kt` lines
/// 426-467).
fn paint(scheme: &ColorScheme, shade: Shade, cell: SwitchCell) -> (Rgb, u8) {
    if cell.availability == Availability::Disabled {
        faded(
            scheme,
            shade.role,
            shade.opacity,
            Rgb::of(scheme, ColorRole::Surface),
        )
    } else {
        (Rgb::of(scheme, shade.role), 0)
    }
}

/// `color` with the state layer over it: the hover opacity of `OnSurface`, the content color of the surface androidx's
/// ripple takes when it is given none (`Ripple.kt` lines 80-84), over what is below. The color is read as a blend.
fn under_layer(scheme: &ColorScheme, color: (Rgb, u8), layer: bool) -> (Rgb, u8) {
    if layer {
        faded(
            scheme,
            ColorRole::OnSurface,
            state_tokens::HOVER_STATE_LAYER_OPACITY,
            color.0,
        )
    } else {
        color
    }
}

/// The handle's diameter when the switch is not pressed: `UnselectedThumbDiameter` for an off, plain handle, and
/// `ThumbDiameter` for any other (`Switch.kt` lines 289-290).
fn diameter(cell: SwitchCell) -> f32 {
    if cell.selection == Selection::Off && cell.content == Content::Plain {
        switch_tokens::UNSELECTED_HANDLE_WIDTH.0
    } else {
        switch_tokens::SELECTED_HANDLE_WIDTH.0
    }
}

/// The distance from the track's start to the handle's center: the handle's left edge, which is `(SwitchHeight -
/// size) / 2` when off and `(SwitchWidth - ThumbDiameter) - ThumbPadding` when on (`Switch.kt` lines 295-298 and
/// 303-304, `ThumbPadding` at line 670), and half its diameter.
fn center(cell: SwitchCell) -> f32 {
    let thumb = switch_tokens::SELECTED_HANDLE_WIDTH.0;
    let padding = (HEIGHT.0 - thumb) / 2.0;
    let left = if cell.selection == Selection::On {
        (WIDTH.0 - thumb) - padding
    } else {
        (HEIGHT.0 - diameter(cell)) / 2.0
    };
    left + diameter(cell) / 2.0
}

/// The pixels the checks know for the switch `cell` in a screenshot in `scheme`, with the mouse over it or not.
pub(crate) fn probes(cell: SwitchCell, scheme: &ColorScheme, hovered: bool) -> Vec<Probe> {
    let target = Target::Switch(cell);
    let (x, y) = cell.origin();
    let at = |dx: f32, dy: f32| (Points(x.0 + dx), Points(y.0 + dy));
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let colors = colors(cell);
    let center = center(cell);
    let middle = HEIGHT.0 / 2.0;
    let widest = switch_tokens::SELECTED_HANDLE_WIDTH.0;
    // A disabled switch has no state layer, hovered or not.
    let layer = hovered && cell.availability == Availability::Enabled;
    let on = cell.selection == Selection::On;

    // Halfway between the track's end and the widest handle's near edge, at mid height: on the track, inside its
    // outline, and out of the state layer.
    let (track_x, track_y) = at(
        if on {
            (center - widest / 2.0) / 2.0
        } else {
            (center + widest / 2.0 + WIDTH.0) / 2.0
        },
        middle,
    );
    // The top edge's first pixel row, near the end of the straight part away from the handle. An off track has its
    // outline there; an on track has none, and its color reaches the edge.
    let (edge_x, edge_y) = at(
        if on {
            middle + EDGE_INSET.0
        } else {
            WIDTH.0 - middle - EDGE_INSET.0
        },
        0.0,
    );
    let (edge, edge_tolerance, edge_what) = if let Some(outline) = colors.outline {
        let (expected, tolerance) = paint(scheme, outline, cell);
        (expected, tolerance, "outline")
    } else {
        let (expected, tolerance) = paint(scheme, colors.track, cell);
        (expected, tolerance, "top edge, which has no outline")
    };
    let (track, track_tolerance) = paint(scheme, colors.track, cell);
    let (handle_x, handle_y) = at(center + diameter(cell) / 2.0 - HANDLE_INSET.0, middle);
    let (handle, handle_tolerance) = under_layer(scheme, paint(scheme, colors.handle, cell), layer);
    let (corner_x, corner_y) = at(0.0, 0.0);
    // Above the track, halfway between its top and the layer's: the layer reaches `(STATE_LAYER_SIZE - SwitchHeight) / 2`
    // past the track, centered on the handle.
    let (layer_x, layer_y) = at(
        center,
        -(switch_tokens::STATE_LAYER_SIZE.0 - HEIGHT.0) / 4.0,
    );
    let (above, above_tolerance) = under_layer(scheme, (surface, 0), layer);

    let mut probes = vec![
        Probe {
            target,
            what: "track",
            x: track_x,
            y: track_y,
            expected: track,
            tolerance: track_tolerance,
        },
        Probe {
            target,
            what: edge_what,
            x: edge_x,
            y: edge_y,
            expected: edge,
            tolerance: edge_tolerance,
        },
        Probe {
            target,
            what: "handle",
            x: handle_x,
            y: handle_y,
            expected: handle,
            tolerance: handle_tolerance,
        },
    ];
    if cell.content == Content::Icon {
        // The middle of the icon, `catalog::ICON`, which is filled there, at the handle's center.
        let (x, y) = at(center, middle);
        let (expected, tolerance) = under_layer(scheme, paint(scheme, colors.icon, cell), layer);
        probes.push(Probe {
            target,
            what: "icon",
            x,
            y,
            expected,
            tolerance,
        });
    }
    probes.push(Probe {
        target,
        what: "outside the corner",
        x: corner_x,
        y: corner_y,
        expected: surface,
        tolerance: 0,
    });
    probes.push(Probe {
        target,
        what: "state layer above the track",
        x: layer_x,
        y: layer_y,
        expected: above,
        tolerance: above_tolerance,
    });
    probes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Appearance, Shot, shot_probes};

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;
    const DARK: ColorScheme = ColorScheme::BASELINE_DARK;

    fn cell(selection: Selection, content: Content, availability: Availability) -> SwitchCell {
        SwitchCell::new(selection, content, availability)
    }

    /// The probe of `what` among `probes`.
    fn probe(probes: &[Probe], what: &str) -> Probe {
        let found: Vec<&Probe> = probes.iter().filter(|p| p.what == what).collect();
        assert_eq!(found.len(), 1, "one probe reads {what}: {probes:?}");
        *found[0]
    }

    /// The distance between two points.
    fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
        (a.0 - b.0).hypot(a.1 - b.1)
    }

    /// How far `point` is from the middle line of the track, whose ends are the centers of its round ends.
    fn from_the_middle_line(point: (f32, f32)) -> f32 {
        let radius = HEIGHT.0 / 2.0;
        let x = point.0.clamp(radius, WIDTH.0 - radius);
        distance(point, (x, radius))
    }

    #[test]
    fn the_handle_is_where_androidx_puts_it() {
        let centers: Vec<(f32, f32)> = [
            (Selection::Off, Content::Plain),
            (Selection::Off, Content::Icon),
            (Selection::On, Content::Plain),
            (Selection::On, Content::Icon),
        ]
        .into_iter()
        .map(|(selection, content)| {
            let cell = cell(selection, content, Availability::Enabled);
            (center(cell), diameter(cell))
        })
        .collect();
        // Off: 16 points from the start, 16 across with a plain handle and 24 with an icon; on: 16 from the end
        // (`SwitchWidth - ThumbPadding - ThumbDiameter / 2`, a 4-point padding), 24 across.
        assert_eq!(
            centers,
            [(16.0, 16.0), (16.0, 24.0), (36.0, 24.0), (36.0, 24.0)]
        );
    }

    #[test]
    fn the_colors_are_each_states_tokens_in_each_appearance() {
        let rgb = |scheme: &ColorScheme, role| Rgb::of(scheme, role);
        let off = colors(cell(Selection::Off, Content::Plain, Availability::Enabled));
        assert_eq!(off.track.role, ColorRole::SurfaceContainerHighest);
        assert_eq!(off.outline.map(|o| o.role), Some(ColorRole::Outline));
        assert_eq!(off.handle.role, ColorRole::Outline);
        assert_eq!(off.icon.role, ColorRole::SurfaceContainerHighest);
        let on = colors(cell(Selection::On, Content::Plain, Availability::Enabled));
        assert_eq!(on.track.role, ColorRole::Primary);
        assert!(on.outline.is_none());
        assert_eq!(on.handle.role, ColorRole::OnPrimary);
        assert_eq!(on.icon.role, ColorRole::OnPrimaryContainer);
        // The enabled ones are exact, in the light and the dark scheme.
        for scheme in [&LIGHT, &DARK] {
            for (selection, track, handle) in [
                (
                    Selection::Off,
                    ColorRole::SurfaceContainerHighest,
                    ColorRole::Outline,
                ),
                (Selection::On, ColorRole::Primary, ColorRole::OnPrimary),
            ] {
                let cell = cell(selection, Content::Plain, Availability::Enabled);
                let probes = probes(cell, scheme, false);
                assert_eq!(
                    (
                        probe(&probes, "track").expected,
                        probe(&probes, "track").tolerance
                    ),
                    (rgb(scheme, track), 0)
                );
                assert_eq!(
                    (
                        probe(&probes, "handle").expected,
                        probe(&probes, "handle").tolerance
                    ),
                    (rgb(scheme, handle), 0)
                );
            }
        }
        // The light baseline's values, for the two handles' colors: Material's baseline outline and on primary.
        assert_eq!(rgb(&LIGHT, ColorRole::Outline), Rgb([0x79, 0x74, 0x7e]));
        assert_eq!(rgb(&LIGHT, ColorRole::OnPrimary), Rgb([0xff, 0xff, 0xff]));
    }

    #[test]
    fn a_disabled_switch_is_each_part_at_its_opacity_over_the_surface() {
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        for (selection, track, handle, icon) in [
            (
                Selection::Off,
                faded(
                    &LIGHT,
                    ColorRole::SurfaceContainerHighest,
                    switch_tokens::DISABLED_TRACK_OPACITY,
                    surface,
                ),
                faded(
                    &LIGHT,
                    ColorRole::OnSurface,
                    switch_tokens::DISABLED_UNSELECTED_HANDLE_OPACITY,
                    surface,
                ),
                faded(
                    &LIGHT,
                    ColorRole::SurfaceContainerHighest,
                    switch_tokens::DISABLED_UNSELECTED_ICON_OPACITY,
                    surface,
                ),
            ),
            (
                Selection::On,
                faded(
                    &LIGHT,
                    ColorRole::OnSurface,
                    switch_tokens::DISABLED_TRACK_OPACITY,
                    surface,
                ),
                faded(
                    &LIGHT,
                    ColorRole::Surface,
                    switch_tokens::DISABLED_SELECTED_HANDLE_OPACITY,
                    surface,
                ),
                faded(
                    &LIGHT,
                    ColorRole::OnSurface,
                    switch_tokens::DISABLED_SELECTED_ICON_OPACITY,
                    surface,
                ),
            ),
        ] {
            let cell = cell(selection, Content::Icon, Availability::Disabled);
            let probes = probes(cell, &LIGHT, false);
            let read = |what| {
                let p = probe(&probes, what);
                (p.expected, p.tolerance)
            };
            assert_eq!(read("track"), track, "{selection:?}");
            assert_eq!(read("handle"), handle, "{selection:?}");
            assert_eq!(read("icon"), icon, "{selection:?}");
            // Read as a blend, with a tolerance, and not as a solid color.
            assert_eq!(track.1, crate::BLEND_TOLERANCE);
        }
        // The disabled off switch's outline is faded as its track is; the disabled on switch has none, and its top edge is
        // its track's color.
        let off = probes(
            cell(Selection::Off, Content::Plain, Availability::Disabled),
            &LIGHT,
            false,
        );
        assert_eq!(
            probe(&off, "outline").expected,
            faded(
                &LIGHT,
                ColorRole::OnSurface,
                switch_tokens::DISABLED_TRACK_OPACITY,
                surface
            )
            .0
        );
        let on = probes(
            cell(Selection::On, Content::Plain, Availability::Disabled),
            &LIGHT,
            false,
        );
        let edge = probe(&on, "top edge, which has no outline");
        assert_eq!(edge.expected, probe(&on, "track").expected);
        // The disabled handle is opaque: a disabled on switch's handle is the surface's color, over a track that is not.
        assert_eq!(probe(&on, "handle").expected, surface);
        assert_ne!(probe(&on, "track").expected, surface);
    }

    #[test]
    fn the_probes_sit_where_the_parts_are() {
        let first = SwitchCell::default();
        let probes = probes(first, &LIGHT, false);
        let places: Vec<(&str, Points, Points)> =
            probes.iter().map(|p| (p.what, p.x, p.y)).collect();
        // The off switch at (24, 2664): the track at 40 points from its start, 4 points inside the straight edge's end
        // away from the handle, 2 points inside the 16-point handle's edge (22), the corner, and above the track at the
        // handle's center.
        assert_eq!(
            places,
            [
                ("track", Points(64.0), Points(2680.0)),
                ("outline", Points(56.0), Points(2664.0)),
                ("handle", Points(46.0), Points(2680.0)),
                ("outside the corner", Points(24.0), Points(2664.0)),
                ("state layer above the track", Points(40.0), Points(2662.0)),
            ]
        );
        let on = probes_of(cell(Selection::On, Content::Icon, Availability::Enabled));
        let places: Vec<(&str, Points, Points)> = on.iter().map(|p| (p.what, p.x, p.y)).collect();
        // The on switch with an icon at (312, 2664): the handle's center is 36 from its start.
        assert_eq!(
            places,
            [
                ("track", Points(324.0), Points(2680.0)),
                (
                    "top edge, which has no outline",
                    Points(332.0),
                    Points(2664.0)
                ),
                ("handle", Points(358.0), Points(2680.0)),
                ("icon", Points(348.0), Points(2680.0)),
                ("outside the corner", Points(312.0), Points(2664.0)),
                ("state layer above the track", Points(348.0), Points(2662.0)),
            ]
        );
    }

    /// The probes of `cell` at rest in the light scheme.
    fn probes_of(cell: SwitchCell) -> Vec<Probe> {
        probes(cell, &LIGHT, false)
    }

    /// Every probe reads a pixel whose color the tokens give: well inside the part it names, and out of the others.
    #[test]
    fn each_probe_reads_a_pixel_inside_its_part_and_outside_the_others() {
        let round = HEIGHT.0 / 2.0;
        let outline = switch_tokens::TRACK_OUTLINE_WIDTH.0;
        let layer_radius = switch_tokens::STATE_LAYER_SIZE.0 / 2.0;
        let icon_half = switch_tokens::SELECTED_ICON_SIZE.0 / 2.0;
        for cell in SwitchCell::all() {
            let (origin_x, origin_y) = cell.origin();
            let handle = (center(cell), round);
            let radius = diameter(cell) / 2.0;
            for hovered in [false, true] {
                for p in probes(cell, &LIGHT, hovered) {
                    // The pixel the probe reads, in points from the track's top left corner.
                    let (px, py) = (p.x.0 - origin_x.0, p.y.0 - origin_y.0);
                    let corners = [
                        (px, py),
                        (px + 1.0, py),
                        (px, py + 1.0),
                        (px + 1.0, py + 1.0),
                    ];
                    let name = format!("{cell:?} {} hovered {hovered}", p.what);
                    match p.what {
                        "track" => {
                            for corner in corners {
                                // Inside the outline, out of the widest handle and out of the state layer.
                                assert!(from_the_middle_line(corner) <= round - outline, "{name}");
                                assert!(
                                    distance(corner, handle)
                                        >= switch_tokens::SELECTED_HANDLE_WIDTH.0 / 2.0,
                                    "{name}"
                                );
                                assert!(distance(corner, handle) >= layer_radius, "{name}");
                            }
                        }
                        "outline" | "top edge, which has no outline" => {
                            // On the straight part, in the first row: in the outline's band, out of the state layer.
                            assert!(px >= round && px + 1.0 <= WIDTH.0 - round, "{name}");
                            assert!(py.abs() < f32::EPSILON, "{name}");
                            for corner in corners {
                                let from_the_line = from_the_middle_line(corner);
                                assert!(from_the_line >= round - outline, "{name}");
                                assert!(from_the_line <= round, "{name}");
                                assert!(distance(corner, handle) >= layer_radius, "{name}");
                            }
                        }
                        "handle" => {
                            for corner in corners {
                                assert!(distance(corner, handle) <= radius, "{name}");
                                // Out of the icon's box, which is as wide as the icon, where there is an icon.
                                assert!(
                                    cell.content == Content::Plain
                                        || (corner.0 - handle.0).abs() >= icon_half,
                                    "{name}"
                                );
                            }
                        }
                        "icon" => {
                            // The handle's center is the icon's.
                            assert!(
                                (px - handle.0).abs() <= 1.0 && (py - handle.1).abs() <= 1.0,
                                "{name}"
                            );
                        }
                        "outside the corner" => {
                            for corner in corners {
                                assert!(from_the_middle_line(corner) > round, "{name}");
                            }
                        }
                        "state layer above the track" => {
                            for corner in corners {
                                // Above the track, inside the layer.
                                assert!(corner.1 <= 0.0, "{name}");
                                assert!(distance(corner, handle) <= layer_radius, "{name}");
                            }
                        }
                        other => panic!("a probe nobody checks the place of: {other}"),
                    }
                }
            }
        }
    }

    #[test]
    fn hovering_an_enabled_switch_changes_only_what_the_state_layer_covers() {
        for cell in HOVERED {
            let rest = probes(cell, &LIGHT, false);
            let hovered = probes(cell, &LIGHT, true);
            assert_eq!(rest.len(), hovered.len());
            for (a, b) in rest.iter().zip(&hovered) {
                let covered = matches!(a.what, "handle" | "icon" | "state layer above the track");
                assert_eq!(a != b, covered, "{} of {cell:?}", a.what);
            }
            // The layer is `OnSurface` at the hover opacity over what is below: the surface above the track, the
            // handle's color on the handle.
            let surface = Rgb::of(&LIGHT, ColorRole::Surface);
            assert_eq!(
                probe(&hovered, "state layer above the track").expected,
                Rgb::of(&LIGHT, ColorRole::OnSurface).over(0.08, surface)
            );
            let below = probe(&rest, "handle").expected;
            assert_eq!(
                probe(&hovered, "handle").expected,
                Rgb::of(&LIGHT, ColorRole::OnSurface).over(0.08, below)
            );
            assert_eq!(probe(&hovered, "handle").tolerance, 2);
            // At rest, nothing shows above the track.
            let above = probe(&rest, "state layer above the track");
            assert_eq!((above.expected, above.tolerance), (surface, 0));
        }
        // The icon is under the layer too.
        let on_icon = HOVERED[1];
        let below = probe(&probes(on_icon, &LIGHT, false), "icon").expected;
        assert_eq!(
            probe(&probes(on_icon, &LIGHT, true), "icon").expected,
            Rgb::of(&LIGHT, ColorRole::OnSurface).over(0.08, below)
        );
    }

    #[test]
    fn a_disabled_switch_shows_no_state_layer_even_under_the_mouse() {
        for selection in [Selection::Off, Selection::On] {
            let cell = cell(selection, Content::Icon, Availability::Disabled);
            assert_eq!(probes(cell, &LIGHT, true), probes(cell, &LIGHT, false));
            assert_eq!(probes(cell, &DARK, true), probes(cell, &DARK, false));
        }
    }

    #[test]
    fn the_appearance_changes_the_expected_colors() {
        for cell in SwitchCell::all() {
            let light = probes(cell, &LIGHT, false);
            let dark = probes(cell, &DARK, false);
            assert_eq!(light.len(), dark.len());
            let changed: Vec<&str> = light
                .iter()
                .zip(&dark)
                .filter(|(l, d)| l.expected != d.expected)
                .map(|(l, _)| l.what)
                .collect();
            // Every color the checks expect is a role of the scheme, or one laid over the surface, which changes too.
            assert_eq!(changed.len(), light.len(), "{cell:?}");
            let corner = probe(&dark, "outside the corner");
            assert_eq!(corner.expected, Rgb::of(&DARK, ColorRole::Surface));
        }
    }

    #[test]
    fn the_switch_shots_follow_the_button_shots_and_hover_each_hovered_switch() {
        let shots = Shot::all();
        // The buttons', icon buttons', navigation bars' and tabs' 17 shots in each appearance, then the switches' 2.
        assert_eq!(shots.len(), 2 * 17 + 2 * 2);
        let hovered: Vec<(Appearance, Option<Target>)> = shots[34..]
            .iter()
            .map(|shot| (shot.appearance, shot.hovered))
            .collect();
        assert_eq!(
            hovered,
            [
                (Appearance::Light, Some(Target::Switch(HOVERED[0]))),
                (Appearance::Light, Some(Target::Switch(HOVERED[1]))),
                (Appearance::Dark, Some(Target::Switch(HOVERED[0]))),
                (Appearance::Dark, Some(Target::Switch(HOVERED[1]))),
            ]
        );
        // Each file is named by what the mouse is over, as `switch.test.tsx` names it.
        let names: Vec<String> = shots[34..].iter().map(|shot| shot.file_name()).collect();
        assert_eq!(
            names,
            [
                "light-hover-switch-off-plain-enabled.png",
                "light-hover-switch-on-icon-enabled.png",
                "dark-hover-switch-off-plain-enabled.png",
                "dark-hover-switch-on-icon-enabled.png",
            ]
        );
        assert_eq!(
            Target::Switch(HOVERED[1]).test_id(),
            "switch-on-icon-enabled"
        );
    }

    #[test]
    fn every_shot_reads_every_switch_and_only_the_hovered_one_changes() {
        let rest = shot_probes(Shot::all()[0]);
        let switches = |probes: &[Probe]| {
            probes
                .iter()
                .filter(|p| matches!(p.target, Target::Switch(_)))
                .count()
        };
        // Five pixels of a plain handle's switch, six with an icon: four of each.
        assert_eq!(switches(&rest), 4 * 5 + 4 * 6);
        for (index, changed) in [(34, 2), (35, 3), (36, 2), (37, 3)] {
            let shot = Shot::all()[index];
            let found = shot_probes(shot);
            let base = shot_probes(Shot {
                hovered: None,
                ..shot
            });
            assert_eq!(found.len(), base.len());
            let differing: Vec<&Probe> = found
                .iter()
                .zip(&base)
                .filter(|(a, b)| a != b)
                .map(|(a, _)| a)
                .collect();
            // The handle, the icon where there is one, and the state layer above the track: nothing else, and nothing
            // of any other switch, button or icon button.
            assert_eq!(differing.len(), changed, "{shot:?}");
            assert!(
                differing.iter().all(|p| Some(p.target) == shot.hovered),
                "{shot:?}"
            );
        }
    }

    #[test]
    fn the_hovered_switches_are_enabled_and_each_end_of_the_track() {
        assert_eq!(
            HOVERED.map(SwitchCell::test_id),
            ["switch-off-plain-enabled", "switch-on-icon-enabled"]
        );
        assert!(
            HOVERED
                .iter()
                .all(|c| c.availability == Availability::Enabled)
        );
        assert_eq!(
            HOVERED.map(|c| c.selection),
            [Selection::Off, Selection::On]
        );
        assert_eq!(HOVERED.map(|c| c.content), [Content::Plain, Content::Icon]);
    }
}
