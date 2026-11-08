//! The checkbox's known pixels (ADR 0013): what `checkbox.test.tsx` screenshots show of each checkbox of the catalog
//! (`catalog::checkbox`), enabled and disabled, at rest and with the mouse over it.
//!
//! Every place is a point of the checkbox's 40-point layout, where the antialiasing of no edge makes the color
//! unknown:
//!
//! - the box's fill, inside its top left corner (`CheckboxTokens.SelectedContainerColor` when checked, none when not);
//! - the outline, in the middle of the box's left edge, a straight edge 2 points wide in every state (androidx
//!   `CheckboxDefaults.StrokeWidth`, `Checkbox.kt` line 482, ADR 0012);
//! - the mark, on its second stroke for the check and on the middle line for the dash, and, for an unchecked box, the
//!   place its zero-length mark would leave a dot (a path of no length drawn with square ends is a dot on some
//!   canvases): nothing there;
//! - the state layer, between the box and the edge of the layer, which hover tints, and the corner of the layer's
//!   square, which a circle never reaches and no state tints.

use catalog::checkbox::{CheckboxCell, STATES};
use catalog::grid::{AVAILABILITIES, Availability};
use nsh_core::geometry::Points;
use nsh_m3e::checkbox::CheckboxState;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::tokens::androidx::{checkbox_tokens, state_tokens};

use super::{Appearance, BLEND_TOLERANCE, Image, Mismatch, Probe, Rgb, Target, faded, pixel_of};

/// One screenshot `examples/catalog/checkbox.test.tsx` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Shot {
    pub(crate) appearance: Appearance,
    /// The checkbox the mouse is over, if any.
    pub(crate) hovered: Option<CheckboxCell>,
}

impl Shot {
    /// Every screenshot the script takes, in each appearance: the catalog at rest, then the mouse over each checkbox,
    /// the enabled row and then the disabled one, a column per state.
    pub(crate) fn all() -> Vec<Self> {
        Appearance::ALL
            .into_iter()
            .flat_map(|appearance| {
                let shot = move |hovered| Self {
                    appearance,
                    hovered,
                };
                std::iter::once(shot(None)).chain(AVAILABILITIES.into_iter().flat_map(
                    move |availability| {
                        STATES
                            .into_iter()
                            .map(move |state| shot(Some(CheckboxCell::new(state, availability))))
                    },
                ))
            })
            .collect()
    }

    /// Its file name, as the test script saves it.
    pub(crate) fn file_name(self) -> String {
        let appearance = self.appearance.name();
        match self.hovered {
            None => format!("checkbox-{appearance}-rest.png"),
            Some(cell) => {
                let id = cell.test_id();
                let name = id.strip_prefix("checkbox-").unwrap_or_default();
                format!("checkbox-{appearance}-hover-{name}.png")
            }
        }
    }
}

/// The side of the checkbox's layout: its state layer's size, `CheckboxTokens.StateLayerSize`.
const SIDE: f32 = checkbox_tokens::STATE_LAYER_SIZE.0;

/// The side of the box, `CheckboxTokens.ContainerSize`, and the offset of its top left corner in the layout.
const BOX: f32 = checkbox_tokens::CONTAINER_SIZE.0;
const BOX_OFFSET: f32 = (SIDE - BOX) / 2.0;

/// How far inside the box's top left corner the fill is read: past the 2-point outline and the arc of a 2-point
/// corner, and away from the mark, which starts at a quarter of the side. Not a design value: where the check looks.
const FILL_INSET: f32 = 3.0;

/// Where the mark is read, in the box's own points. The check's second stroke runs from (0.4, 0.65) to (0.75, 0.3) of
/// the side (`Checkbox.kt` lines 713 to 718, the branch of the flag the checkbox follows); its middle, (10.35, 8.55)
/// in an 18-point box, is in the pixel (10, 8), which the 2-point stroke covers whole (its corners are 0.64 and 0.78
/// points from the stroke's axis). The dash is on the middle line, 2 points wide, from 4.5 to 13.5 across: the pixel
/// (9, 9) is inside it. The start of the mark, (4.5, 9), is where an unchecked box's zero-length mark would put its
/// dot: the pixel (4, 8).
const CHECK_MARK: (f32, f32) = (10.0, 8.0);
const DASH_MARK: (f32, f32) = (9.0, 9.0);
const MARK_START: (f32, f32) = (4.0, 8.0);

/// How far from the layout's left edge the state layer is read between the box and the edge: inside the circle of the
/// layer (its middle is `SIDE / 2` from the edge), outside the box (which starts at `BOX_OFFSET`).
const LAYER_INSET: f32 = 2.0;

/// The colors of a checkbox's box, as the tokens and `Checkbox.kt` give them, over the page's surface: fill and
/// outline, and the mark if it shows one, each with its tolerance.
struct Colors {
    fill: (Rgb, u8),
    outline: (Rgb, u8),
    mark: Option<(Rgb, u8)>,
}

/// The colors of `cell`'s box at rest in `scheme`: `CheckboxColors` as `defaultCheckboxColors` builds them
/// (`Checkbox.kt` lines 446 to 476), where transparent is the surface showing through.
fn colors(cell: CheckboxCell, scheme: &ColorScheme) -> Colors {
    let surface = (Rgb::of(scheme, ColorRole::Surface), 0);
    let solid = |role| (Rgb::of(scheme, role), 0);
    let checked = cell.state != CheckboxState::Unchecked;
    match (checked, cell.availability) {
        (false, Availability::Disabled) => Colors {
            fill: surface,
            outline: faded(
                scheme,
                checkbox_tokens::UNSELECTED_DISABLED_OUTLINE_COLOR,
                checkbox_tokens::UNSELECTED_DISABLED_CONTAINER_OPACITY,
                surface.0,
            ),
            mark: None,
        },
        (false, _) => Colors {
            fill: surface,
            outline: solid(checkbox_tokens::UNSELECTED_OUTLINE_COLOR),
            mark: None,
        },
        (true, Availability::Disabled) => {
            // The box and its border are faded; the mark is the disabled icon color, whole (lines 452 and 453).
            let faded_box = faded(
                scheme,
                checkbox_tokens::SELECTED_DISABLED_CONTAINER_COLOR,
                checkbox_tokens::SELECTED_DISABLED_CONTAINER_OPACITY,
                surface.0,
            );
            Colors {
                fill: faded_box,
                outline: faded_box,
                mark: Some(solid(checkbox_tokens::SELECTED_DISABLED_ICON_COLOR)),
            }
        }
        (true, _) => Colors {
            fill: solid(checkbox_tokens::SELECTED_CONTAINER_COLOR),
            outline: solid(checkbox_tokens::SELECTED_CONTAINER_COLOR),
            mark: Some(solid(checkbox_tokens::SELECTED_ICON_COLOR)),
        },
    }
}

/// The role of the state layer over `cell`, which hover shows: the outline's role for an unchecked box and the
/// container's for a checked one (`UnselectedHoverOutlineColor`, `SelectedHoverContainerColor`).
fn layer_role(cell: CheckboxCell) -> ColorRole {
    if cell.state == CheckboxState::Unchecked {
        checkbox_tokens::UNSELECTED_HOVER_OUTLINE_COLOR
    } else {
        checkbox_tokens::SELECTED_HOVER_CONTAINER_COLOR
    }
}

/// The pixels the checks know in `cell`, as the state layer shows or not.
fn probes(cell: CheckboxCell, scheme: &ColorScheme, hovered: bool) -> Vec<Probe> {
    let target = Target::Checkbox(cell);
    let (x, y) = cell.origin();
    let at = |dx: f32, dy: f32| (Points(x.0 + dx), Points(y.0 + dy));
    let in_box = |dx: f32, dy: f32| at(BOX_OFFSET + dx, BOX_OFFSET + dy);
    let colors = colors(cell, scheme);
    // A disabled checkbox shows no state layer, and an enabled one shows it while the mouse is over it.
    let layer = (hovered && cell.availability == Availability::Enabled)
        .then(|| (layer_role(cell), state_tokens::HOVER_STATE_LAYER_OPACITY));
    // What a place shows when the state layer lies over `base`: the layer's color at its opacity, over what is there.
    let under_layer = |(base, tolerance): (Rgb, u8)| match layer {
        Some((role, opacity)) => (
            Rgb::of(scheme, role).over(opacity, base),
            tolerance.max(BLEND_TOLERANCE),
        ),
        None => (base, tolerance),
    };
    let surface = (Rgb::of(scheme, ColorRole::Surface), 0);
    let mut places = vec![
        ("fill", in_box(FILL_INSET, FILL_INSET), colors.fill),
        ("outline", in_box(0.0, BOX / 2.0), colors.outline),
        // Between the box and the layer's edge: the surface, which hover tints, and nothing else.
        ("state layer", at(LAYER_INSET, SIDE / 2.0), surface),
    ];
    match (cell.state, colors.mark) {
        (CheckboxState::Indeterminate, Some(mark)) => {
            places.push(("mark", in_box(DASH_MARK.0, DASH_MARK.1), mark));
        }
        (_, Some(mark)) => places.push(("mark", in_box(CHECK_MARK.0, CHECK_MARK.1), mark)),
        // Where an unchecked box's mark would start: no dot.
        (_, None) => places.push(("no mark", in_box(MARK_START.0, MARK_START.1), colors.fill)),
    }
    let mut probes: Vec<Probe> = places
        .into_iter()
        .map(|(what, (x, y), expected)| {
            let (expected, tolerance) = under_layer(expected);
            Probe {
                target,
                what,
                x,
                y,
                expected,
                tolerance,
            }
        })
        .collect();
    // The corner of the layout's square, outside the layer's circle: the surface in every state.
    let (x, y) = at(0.0, 0.0);
    probes.push(Probe {
        target,
        what: "outside the state layer",
        x,
        y,
        expected: surface.0,
        tolerance: surface.1,
    });
    probes
}

/// The pixels the checks know in `shot`, for every checkbox of the catalog.
fn shot_probes(shot: Shot) -> Vec<Probe> {
    let scheme = shot.appearance.scheme();
    CheckboxCell::all()
        .flat_map(|cell| probes(cell, &scheme, shot.hovered == Some(cell)))
        .collect()
}

/// Every probe of `shot` that `image` does not match; an error if the image is not the catalog's checkbox viewport at
/// a whole scale.
pub(crate) fn check(image: &Image, shot: Shot) -> Result<Vec<Mismatch>, String> {
    let scale = image.scale_in(catalog::checkbox::viewport())?;
    Ok(shot_probes(shot)
        .into_iter()
        .filter_map(|probe| {
            let (x, y) = pixel_of(&probe, scale);
            let found = image.pixel(x, y);
            match found {
                Some(found) if found.near(probe.expected, probe.tolerance) => None,
                _ => Some(Mismatch { probe, found }),
            }
        })
        .collect())
}

/// The catalog painted as the tokens say, at `scale`: every probe of `shot` holds its expected color, and the rest a
/// color no probe expects.
#[cfg(test)]
pub(crate) fn painted(shot: Shot, scale: u32) -> Image {
    use super::points_to_u32;
    let (width, height) = catalog::checkbox::viewport();
    let (width, height) = (points_to_u32(width) * scale, points_to_u32(height) * scale);
    let mut rgba = [1, 2, 3, 255].repeat((width * height) as usize);
    for probe in shot_probes(shot) {
        let (x, y) = pixel_of(&probe, scale);
        let start = ((y * width + x) * 4) as usize;
        rgba[start..start + 3].copy_from_slice(&probe.expected.0);
    }
    Image {
        width,
        height,
        rgba,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::points_to_u32;

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;

    fn cell(state: CheckboxState, availability: Availability) -> CheckboxCell {
        CheckboxCell::new(state, availability)
    }

    /// The probe of `cell` named `what`.
    fn probe(cell: CheckboxCell, scheme: &ColorScheme, hovered: bool, what: &str) -> Probe {
        probes(cell, scheme, hovered)
            .into_iter()
            .find(|probe| probe.what == what)
            .unwrap_or_else(|| panic!("{cell:?} has no probe {what}"))
    }

    fn hex(rgb: Rgb) -> String {
        rgb.to_string()
    }

    #[test]
    fn the_boxes_are_the_token_colors_in_each_state() {
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let primary = Rgb::of(&LIGHT, ColorRole::Primary);
        // The baseline light scheme's colors, written out: primary #6750a4, on primary white, on surface variant
        // #49454f, surface #fef7ff.
        assert_eq!(hex(primary), "#6750a4");
        assert_eq!(hex(Rgb::of(&LIGHT, ColorRole::OnPrimary)), "#ffffff");
        assert_eq!(hex(Rgb::of(&LIGHT, ColorRole::OnSurfaceVariant)), "#49454f");
        assert_eq!(hex(surface), "#fef7ff");
        let enabled = |state| cell(state, Availability::Enabled);
        // Unchecked: the surface shows through the box, the outline is on-surface-variant, and nothing is marked.
        let (fill, outline) = (
            probe(enabled(CheckboxState::Unchecked), &LIGHT, false, "fill"),
            probe(enabled(CheckboxState::Unchecked), &LIGHT, false, "outline"),
        );
        assert_eq!((fill.expected, fill.tolerance), (surface, 0));
        assert_eq!(hex(outline.expected), "#49454f");
        assert_eq!(outline.tolerance, 0);
        // Checked and indeterminate alike: the primary box and outline, the on-primary mark.
        for state in [CheckboxState::Checked, CheckboxState::Indeterminate] {
            for what in ["fill", "outline"] {
                let found = probe(enabled(state), &LIGHT, false, what);
                assert_eq!(
                    (found.expected, found.tolerance),
                    (primary, 0),
                    "{state:?} {what}"
                );
            }
            let mark = probe(enabled(state), &LIGHT, false, "mark");
            assert_eq!(hex(mark.expected), "#ffffff", "{state:?}");
        }
        // Disabled: the box and outline are on-surface at 38 % over the surface, the mark is the surface itself, and
        // an unchecked outline is on-surface at 38 %.
        let faded_box = Rgb::of(&LIGHT, ColorRole::OnSurface).over(0.38, surface);
        for state in [CheckboxState::Checked, CheckboxState::Indeterminate] {
            let disabled = cell(state, Availability::Disabled);
            for what in ["fill", "outline"] {
                let found = probe(disabled, &LIGHT, false, what);
                assert_eq!(found.expected, faded_box, "{state:?} {what}");
                assert_eq!(found.tolerance, BLEND_TOLERANCE);
            }
            let mark = probe(disabled, &LIGHT, false, "mark");
            assert_eq!((mark.expected, mark.tolerance), (surface, 0), "{state:?}");
        }
        let unchecked = cell(CheckboxState::Unchecked, Availability::Disabled);
        assert_eq!(
            probe(unchecked, &LIGHT, false, "outline").expected,
            faded_box
        );
        assert_eq!(probe(unchecked, &LIGHT, false, "fill").expected, surface);
        // The faded colors are not the enabled ones.
        assert_ne!(faded_box, primary);
    }

    #[test]
    fn an_unchecked_box_has_no_mark_and_the_others_have_one() {
        for (state, availability) in [
            (CheckboxState::Unchecked, Availability::Enabled),
            (CheckboxState::Unchecked, Availability::Disabled),
        ] {
            let names: Vec<&str> = probes(cell(state, availability), &LIGHT, false)
                .iter()
                .map(|probe| probe.what)
                .collect();
            assert!(names.contains(&"no mark"), "{state:?}");
            assert!(!names.contains(&"mark"), "{state:?}");
        }
        for state in [CheckboxState::Checked, CheckboxState::Indeterminate] {
            let names: Vec<&str> = probes(cell(state, Availability::Enabled), &LIGHT, false)
                .iter()
                .map(|probe| probe.what)
                .collect();
            assert!(
                names.contains(&"mark") && !names.contains(&"no mark"),
                "{state:?}"
            );
        }
        // The check and the dash are read in different places, and the empty box where the mark would start.
        let at = |state| {
            let found = probe(cell(state, Availability::Enabled), &LIGHT, false, "mark");
            (found.x, found.y)
        };
        assert_ne!(at(CheckboxState::Checked), at(CheckboxState::Indeterminate));
        let none = probe(
            cell(CheckboxState::Unchecked, Availability::Enabled),
            &LIGHT,
            false,
            "no mark",
        );
        assert_eq!(
            (none.x, none.y),
            (Points(24.0 + 11.0 + 4.0), Points(2664.0 + 11.0 + 8.0))
        );
    }

    #[test]
    fn the_places_are_inside_the_box_the_layer_and_the_checkbox() {
        for cell in CheckboxCell::all() {
            let (x, y) = cell.origin();
            let places = probes(cell, &LIGHT, false);
            assert_eq!(places.len(), 5, "{cell:?}");
            for probe in &places {
                let (dx, dy) = (probe.x.0 - x.0, probe.y.0 - y.0);
                assert!(
                    (0.0..SIDE).contains(&dx) && (0.0..SIDE).contains(&dy),
                    "{probe:?}"
                );
                // Whole points, as the probes' pixels are.
                assert!(dx.fract().abs() < f32::EPSILON && dy.fract().abs() < f32::EPSILON);
                let in_box = |d: f32| (BOX_OFFSET..BOX_OFFSET + BOX).contains(&d);
                match probe.what {
                    "fill" | "outline" | "mark" | "no mark" => {
                        assert!(in_box(dx) && in_box(dy), "{probe:?}");
                    }
                    "state layer" => {
                        // Left of the box, inside the circle: even the pixel's farthest corner is.
                        assert!(dx < BOX_OFFSET, "{probe:?}");
                        assert!((dy - SIDE / 2.0).abs() < f32::EPSILON, "{probe:?}");
                        let farthest = (SIDE / 2.0 - dx).abs().max((SIDE / 2.0 - dx - 1.0).abs());
                        assert!(farthest.hypot(1.0) < SIDE / 2.0, "{probe:?}");
                    }
                    "outside the state layer" => {
                        // The pixel's nearest point to the layer's middle is beyond its radius.
                        let nearest = (SIDE / 2.0 - dx - 1.0).hypot(SIDE / 2.0 - dy - 1.0);
                        assert!(nearest > SIDE / 2.0, "{probe:?}");
                    }
                    other => panic!("{other}"),
                }
            }
            // The outline is read in the middle of the box's left edge, 2 points wide.
            let outline = places.iter().find(|p| p.what == "outline").unwrap();
            assert_eq!(outline.x, Points(x.0 + BOX_OFFSET));
            assert_eq!(checkbox_tokens::UNSELECTED_OUTLINE_WIDTH, Points(2.0));
            // The fill is read past it.
            let fill = places.iter().find(|p| p.what == "fill").unwrap();
            assert!(fill.x.0 - x.0 - BOX_OFFSET >= checkbox_tokens::UNSELECTED_OUTLINE_WIDTH.0);
        }
    }

    #[test]
    fn hover_tints_the_enabled_checkbox_under_the_mouse_with_its_layer_color() {
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let opacity = state_tokens::HOVER_STATE_LAYER_OPACITY;
        for cell in CheckboxCell::all() {
            let rest = probes(cell, &LIGHT, false);
            let hovered = probes(cell, &LIGHT, true);
            assert_eq!(rest.len(), hovered.len());
            let enabled = cell.availability == Availability::Enabled;
            for (rest, hovered) in rest.iter().zip(&hovered) {
                if hovered.what == "outside the state layer" || !enabled {
                    // A circle never reaches the corner, and a disabled checkbox shows no state.
                    assert_eq!(rest, hovered, "{cell:?} {}", hovered.what);
                } else {
                    let role = if cell.state == CheckboxState::Unchecked {
                        ColorRole::OnSurface
                    } else {
                        ColorRole::Primary
                    };
                    assert_eq!(
                        hovered.expected,
                        Rgb::of(&LIGHT, role).over(opacity, rest.expected),
                        "{cell:?} {}",
                        hovered.what
                    );
                    assert_eq!(hovered.tolerance, BLEND_TOLERANCE);
                }
            }
        }
        // Over the surface, the layer of an unchecked box is on-surface at 8 %.
        let ring = probe(
            cell(CheckboxState::Unchecked, Availability::Enabled),
            &LIGHT,
            true,
            "state layer",
        );
        assert_eq!(
            ring.expected,
            Rgb::of(&LIGHT, ColorRole::OnSurface).over(0.08, surface)
        );
        assert_ne!(ring.expected, surface);
        // The layer of a checked box is primary: invisible over the primary box, but over the mark and the surface.
        let checked = cell(CheckboxState::Checked, Availability::Enabled);
        let fill = probe(checked, &LIGHT, true, "fill");
        assert!(fill.expected.near(Rgb::of(&LIGHT, ColorRole::Primary), 1));
        let ring = probe(checked, &LIGHT, true, "state layer");
        assert_eq!(
            ring.expected,
            Rgb::of(&LIGHT, ColorRole::Primary).over(0.08, surface)
        );
    }

    #[test]
    fn the_shots_are_the_test_script_files() {
        let names: Vec<String> = Shot::all().into_iter().map(Shot::file_name).collect();
        assert_eq!(
            names,
            [
                "checkbox-light-rest.png",
                "checkbox-light-hover-unchecked-enabled.png",
                "checkbox-light-hover-checked-enabled.png",
                "checkbox-light-hover-indeterminate-enabled.png",
                "checkbox-light-hover-unchecked-disabled.png",
                "checkbox-light-hover-checked-disabled.png",
                "checkbox-light-hover-indeterminate-disabled.png",
                "checkbox-dark-rest.png",
                "checkbox-dark-hover-unchecked-enabled.png",
                "checkbox-dark-hover-checked-enabled.png",
                "checkbox-dark-hover-indeterminate-enabled.png",
                "checkbox-dark-hover-unchecked-disabled.png",
                "checkbox-dark-hover-checked-disabled.png",
                "checkbox-dark-hover-indeterminate-disabled.png",
            ]
        );
        assert_eq!(Shot::all()[0].appearance.scheme(), LIGHT);
        assert_eq!(
            Shot::all()[7].appearance.scheme(),
            ColorScheme::BASELINE_DARK
        );
        assert_eq!(
            Shot::all()[2].hovered,
            Some(cell(CheckboxState::Checked, Availability::Enabled))
        );
        // The script's names are the shots' (`checkbox.test.tsx`).
        let script = include_str!("../../../examples/catalog/checkbox.test.tsx");
        assert!(script.contains("`target/visual/checkbox-${scheme}-rest.png`"));
        assert!(
            script
                .contains("`target/visual/checkbox-${scheme}-hover-${state}-${availability}.png`")
        );
    }

    #[test]
    fn a_catalog_drawn_as_the_tokens_say_passes_at_any_whole_scale() {
        for shot in Shot::all() {
            for scale in [1, 2] {
                assert_eq!(check(&painted(shot, scale), shot), Ok(Vec::new()));
            }
        }
    }

    #[test]
    fn one_wrong_pixel_fails_and_is_named() {
        let shot = Shot::all()[2];
        let hovered = shot.hovered.unwrap();
        let mut image = painted(shot, 1);
        let wanted = probe(hovered, &LIGHT, true, "mark");
        let (x, y) = pixel_of(&wanted, 1);
        let start = ((y * image.width + x) * 4) as usize;
        // The mark's place showing the box's color, as a box whose check was never drawn would.
        let primary = Rgb::of(&LIGHT, ColorRole::Primary);
        image.rgba[start..start + 3].copy_from_slice(&primary.0);
        let found = check(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe, wanted);
        assert_eq!(found[0].found, Some(primary));
        assert_eq!(
            found[0].to_string(),
            format!(
                "checkbox-checked-enabled mark at (109, 2683) points: expected {} (± 2), found #6750a4",
                wanted.expected
            )
        );
        // The same image is not the dark shot, nor the one with the mouse elsewhere.
        assert_ne!(check(&image, Shot::all()[9]).unwrap(), Vec::new());
        assert_ne!(check(&image, Shot::all()[1]).unwrap(), Vec::new());
    }

    #[test]
    fn a_dot_where_the_unchecked_mark_would_start_is_found() {
        let shot = Shot::all()[0];
        let mut image = painted(shot, 1);
        let wanted = probe(
            cell(CheckboxState::Unchecked, Availability::Enabled),
            &LIGHT,
            false,
            "no mark",
        );
        let (x, y) = pixel_of(&wanted, 1);
        let start = ((y * image.width + x) * 4) as usize;
        // The mark's color, there, as a dot of a zero-length path would leave.
        image.rgba[start..start + 3].copy_from_slice(&Rgb::of(&LIGHT, ColorRole::OnPrimary).0);
        let found = check(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe.what, "no mark");
    }

    #[test]
    fn a_square_layer_is_found_by_the_corner_and_a_missing_one_by_the_ring() {
        let shot = Shot::all()[1];
        let hovered = shot.hovered.unwrap();
        // Corner of the square tinted: the layer is not a circle.
        let mut image = painted(shot, 1);
        let corner = probe(hovered, &LIGHT, true, "outside the state layer");
        let (x, y) = pixel_of(&corner, 1);
        let start = ((y * image.width + x) * 4) as usize;
        let tinted = Rgb::of(&LIGHT, ColorRole::OnSurface).over(0.08, corner.expected);
        image.rgba[start..start + 3].copy_from_slice(&tinted.0);
        assert_eq!(check(&image, shot).unwrap().len(), 1);
        // The layer missing between the box and the edge: the ring at rest.
        let mut image = painted(shot, 1);
        let ring = probe(hovered, &LIGHT, true, "state layer");
        let (x, y) = pixel_of(&ring, 1);
        let start = ((y * image.width + x) * 4) as usize;
        image.rgba[start..start + 3].copy_from_slice(&Rgb::of(&LIGHT, ColorRole::Surface).0);
        let found = check(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe, ring);
    }

    #[test]
    fn a_screenshot_that_stops_above_the_checkboxes_is_refused() {
        // A viewport that ends where the last band starts, with the margin below the icon buttons' rows: the page
        // without the checkboxes and the FABs, which stand in that band (the FABs add no height to it, so the buttons'
        // viewport is the checkboxes' too).
        let (width, _) = catalog::grid::viewport();
        let height = Points(catalog::grid::band_top().0 + catalog::grid::MARGIN.0);
        let (width, height) = (points_to_u32(width), points_to_u32(height));
        let image = Image {
            width,
            height,
            rgba: vec![0; 4 * (width * height) as usize],
        };
        assert!(check(&image, Shot::all()[0]).is_err());
        let mut image = painted(Shot::all()[0], 1);
        image.height += 1;
        assert!(check(&image, Shot::all()[0]).is_err());
    }
}
