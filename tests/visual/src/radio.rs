//! The radio buttons' known pixels in the catalog's screenshots (`examples/catalog/radio.test.tsx`), checked as the
//! buttons' are: every expected color comes from the library's tokens.
//!
//! For every radio button of the catalog, in each appearance and with the mouse over it or not, a screenshot holds
//! these pixels, each read wholly inside the region it names, past the antialiasing of every edge:
//!
//! - the ring, at its four sides and at 45 degrees: the mark's color (`RadioButtonTokens`' icon colors, selected or
//!   not, and the disabled ones at their opacity over the surface);
//! - the dot, at the middle: the mark's color when selected, and nothing when not (a dot of no radius leaves no
//!   artifact);
//! - between the dot and the ring, just outside the ring and inside the state layer: what is behind the radio button,
//!   which is the surface, or, with the mouse over an enabled one, its state layer (the hover color at
//!   `StateTokens.HoverStateLayerOpacity` over the surface);
//! - just outside the state layer's edge, and in the corner of its box, which a circle leaves empty: the surface,
//!   hovered or not.
//!
//! The ring's width is bounded by the pixels just inside and just outside it, and the state layer's diameter by those
//! just inside and just outside its edge.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use catalog::grid::Availability;
use catalog::radio::{self, Cell, Selection};
use nsh_core::geometry::Points;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::radio::{FRAME, STROKE_WIDTH};
use nsh_m3e::tokens::androidx::{radio_button_tokens, state_tokens};

use super::{Appearance, Error, Image, Rgb, faded, points_to_u32};

/// The radio buttons the mouse goes over in a screenshot, one each: the unselected enabled, the selected enabled, and
/// the unselected disabled one, which shows no state layer.
const HOVERED: [Cell; 3] = [
    Cell::new(Selection::Unselected, Availability::Enabled),
    Cell::new(Selection::Selected, Availability::Enabled),
    Cell::new(Selection::Unselected, Availability::Disabled),
];

/// How far from the middle, in whole points on each axis, the ring's pixel at 45 degrees is read: the one whole point
/// whose pixel lies wholly inside the ring's band (its corners are 8.5 and 9.9 points from the middle, where the band
/// is 8 to 10). Not a design value: where the check looks.
const DIAGONAL: Points = Points(6.0);

/// How far inside the state layer's edge its color is read, and how far in from its box's corner the corner is read:
/// a pixel wholly inside the circle, and one wholly outside it and inside its box. Not design values: where the check
/// looks.
const LAYER_INSET: Points = Points(2.0);
const LAYER_CORNER_INSET: Points = Points(4.0);

/// One screenshot the test script takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Shot {
    appearance: Appearance,
    /// The radio button the mouse is over, if any.
    hovered: Option<Cell>,
}

impl Shot {
    /// Every screenshot `examples/catalog/radio.test.tsx` takes, in each appearance: the catalog at rest, then the mouse
    /// over each of the [`HOVERED`] radio buttons.
    fn all() -> Vec<Self> {
        Appearance::ALL
            .into_iter()
            .flat_map(|appearance| {
                std::iter::once(Self {
                    appearance,
                    hovered: None,
                })
                .chain(HOVERED.into_iter().map(move |cell| Self {
                    appearance,
                    hovered: Some(cell),
                }))
            })
            .collect()
    }

    /// Its file name, as the test script saves it.
    fn file_name(self) -> String {
        let appearance = self.appearance.name();
        match self.hovered {
            None => format!("radio-{appearance}-rest.png"),
            Some(cell) => format!("radio-{appearance}-hover-{}.png", cell.slug()),
        }
    }

    /// Whether the mouse is over `cell` in it.
    fn hovers(self, cell: Cell) -> bool {
        self.hovered == Some(cell)
    }
}

/// A pixel the checks know, in the catalog's points.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Probe {
    cell: Cell,
    what: &'static str,
    x: Points,
    y: Points,
    expected: Rgb,
    tolerance: u8,
}

/// The ring's and the dot's color in `cell` and its tolerance: the icon color of its selection, and when disabled that
/// color's role at its opacity over the surface.
fn mark(cell: Cell, scheme: &ColorScheme) -> (Rgb, u8) {
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let selected = cell.selection == Selection::Selected;
    match (selected, cell.availability == Availability::Disabled) {
        (true, false) => (Rgb::of(scheme, radio_button_tokens::SELECTED_ICON_COLOR), 0),
        (false, false) => (
            Rgb::of(scheme, radio_button_tokens::UNSELECTED_ICON_COLOR),
            0,
        ),
        (true, true) => faded(
            scheme,
            radio_button_tokens::DISABLED_SELECTED_ICON_COLOR,
            radio_button_tokens::DISABLED_SELECTED_ICON_OPACITY,
            surface,
        ),
        (false, true) => faded(
            scheme,
            radio_button_tokens::DISABLED_UNSELECTED_ICON_COLOR,
            radio_button_tokens::DISABLED_UNSELECTED_ICON_OPACITY,
            surface,
        ),
    }
}

/// What the page shows behind `cell`'s ring and dot, and its tolerance: the surface, and, with the mouse over an enabled
/// radio button, its state layer: the layer's color at the hover opacity over the surface. A disabled radio button
/// shows no state.
fn behind(cell: Cell, scheme: &ColorScheme, hovered: bool) -> (Rgb, u8) {
    let surface = Rgb::of(scheme, ColorRole::Surface);
    if hovered && cell.availability == Availability::Enabled {
        let role = if cell.selection == Selection::Selected {
            radio_button_tokens::SELECTED_HOVER_ICON_COLOR
        } else {
            radio_button_tokens::UNSELECTED_HOVER_ICON_COLOR
        };
        faded(
            scheme,
            role,
            state_tokens::HOVER_STATE_LAYER_OPACITY,
            surface,
        )
    } else {
        (surface, 0)
    }
}

/// The pixels the checks know for `cell` in a screenshot in `scheme`, with the mouse over it or not.
fn probes(cell: Cell, scheme: &ColorScheme, hovered: bool) -> Vec<Probe> {
    let (x, y) = cell.origin();
    let middle = (x.0 + FRAME.0 / 2.0, y.0 + FRAME.0 / 2.0);
    let outer = radio_button_tokens::ICON_SIZE.0 / 2.0;
    let inner = outer - STROKE_WIDTH.0;
    let layer = radio_button_tokens::STATE_LAYER_SIZE.0 / 2.0;
    let surface = (Rgb::of(scheme, ColorRole::Surface), 0);
    let ring = mark(cell, scheme);
    let behind = behind(cell, scheme, hovered);
    // The pixel whose top left corner is `dx`, `dy` points from the middle.
    let at = |what, dx: f32, dy: f32, (expected, tolerance)| Probe {
        cell,
        what,
        x: Points(middle.0 + dx),
        y: Points(middle.1 + dy),
        expected,
        tolerance,
    };
    // Whether the dot is there: where it is not, the radio button shows what is behind it.
    let dot = if cell.selection == Selection::Selected {
        ring
    } else {
        behind
    };
    vec![
        // The ring's band is `inner` to `outer` from the middle: the first pixel of it on each side, which is wholly
        // inside it (a pixel at `outer - 1` would reach past the band's curve at its far corner).
        at("ring, right", inner, 0.0, ring),
        at("ring, left", -inner - 1.0, 0.0, ring),
        at("ring, bottom", 0.0, inner, ring),
        at("ring, top", 0.0, -inner - 1.0, ring),
        at("ring, 45 degrees", DIAGONAL.0, DIAGONAL.0, ring),
        at("dot", 0.0, 0.0, dot),
        // Between the dot (5 points) and the ring (from 8), and just outside the ring (from 10): not the mark.
        at("between the dot and the ring", inner - 2.0, 0.0, behind),
        at("outside the ring", outer, 0.0, behind),
        // In the state layer, away from the ring, and just inside its edge.
        at("in the state layer", outer + 4.0, 0.0, behind),
        at(
            "just inside the state layer's edge",
            layer - LAYER_INSET.0,
            0.0,
            behind,
        ),
        // Just outside the state layer's edge, and in its box's corner, which a circle leaves empty: the surface,
        // hovered or not.
        at("outside the state layer", layer, 0.0, surface),
        at(
            "outside the state layer's corner",
            layer - LAYER_CORNER_INSET.0,
            layer - LAYER_CORNER_INSET.0,
            surface,
        ),
    ]
}

/// The pixels the checks know in `shot`, for every radio button of the catalog.
fn shot_probes(shot: Shot) -> Vec<Probe> {
    let scheme = shot.appearance.scheme();
    Cell::all()
        .flat_map(|cell| probes(cell, &scheme, shot.hovers(cell)))
        .collect()
}

/// How many pixels a point is: the image is the radio buttons' viewport at a whole scale (1 on a desktop, 2 on a
/// high-density screen).
fn scale(image: &Image) -> Result<u32, String> {
    let (width, height) = radio::viewport();
    let scale = image.width / points_to_u32(width).max(1);
    if scale == 0
        || image.width != scale * points_to_u32(width)
        || image.height != scale * points_to_u32(height)
    {
        return Err(format!(
            "the image is {}x{} pixels, not the radio buttons' {}x{} points at a whole scale",
            image.width, image.height, width.0, height.0
        ));
    }
    Ok(scale)
}

/// The pixel a probe reads: the top left one of the point it names.
fn pixel_of(probe: &Probe, scale: u32) -> (u32, u32) {
    (
        points_to_u32(probe.x) * scale,
        points_to_u32(probe.y) * scale,
    )
}

/// What a probe found when it was not what was expected.
#[derive(Debug, Clone, PartialEq)]
struct Mismatch {
    probe: Probe,
    found: Option<Rgb>,
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let probe = &self.probe;
        write!(
            f,
            "{} {} at ({}, {}) points: expected {} (± {}), found ",
            probe.cell.test_id(),
            probe.what,
            probe.x.0,
            probe.y.0,
            probe.expected,
            probe.tolerance
        )?;
        match self.found {
            Some(found) => write!(f, "{found}"),
            None => write!(f, "no pixel"),
        }
    }
}

/// Every probe of `shot` that `image` does not match.
fn check(image: &Image, shot: Shot) -> Result<Vec<Mismatch>, String> {
    let scale = scale(image)?;
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

/// Checks the radio buttons' screenshots in `directory`, as the test script saves them, and returns an error for the
/// first one it cannot read or the pixels that are not the tokens' colors.
pub(crate) fn run(directory: &Path, out: &mut impl io::Write) -> Result<(), Error> {
    run_shots(directory, &Shot::all(), out)
}

fn run_shots(directory: &Path, shots: &[Shot], out: &mut impl io::Write) -> Result<(), Error> {
    let mut mismatches = 0;
    for shot in shots {
        let path: PathBuf = directory.join(shot.file_name());
        let bytes = fs::read(&path).map_err(|source| Error::Read {
            path: path.clone(),
            source,
        })?;
        let image = Image::decode(bytes).map_err(|message| Error::Image {
            path: path.clone(),
            message,
        })?;
        let found = check(&image, *shot).map_err(|message| Error::Image {
            path: path.clone(),
            message,
        })?;
        let _ = writeln!(
            out,
            "{}: {}",
            path.display(),
            if found.is_empty() { "ok" } else { "FAILED" }
        );
        for mismatch in &found {
            let _ = writeln!(out, "  {mismatch}");
        }
        mismatches += found.len();
    }
    if mismatches == 0 {
        Ok(())
    } else {
        Err(Error::Mismatches(mismatches))
    }
}

/// The result `before` of the buttons' checks in `directory`, joined with the radio buttons': both run, so that a
/// failure of one does not hide the other's, and their mismatches add up. Another failure (a screenshot that cannot
/// be read) is the first one's.
pub(crate) fn also(
    before: Result<(), Error>,
    directory: &Path,
    out: &mut impl io::Write,
) -> Result<(), Error> {
    match (before, run(directory, out)) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(Error::Mismatches(a)), Err(Error::Mismatches(b))) => {
            Err(Error::Mismatches(a.saturating_add(b)))
        }
        (Err(error), _) | (Ok(()), Err(error)) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use nsh_m3e::radio::DOT_SIZE;

    use super::super::BLEND_TOLERANCE;
    use super::*;

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;
    const DARK: ColorScheme = ColorScheme::BASELINE_DARK;

    fn cell(selection: Selection, availability: Availability) -> Cell {
        Cell::new(selection, availability)
    }

    /// The catalog painted as the tokens say, at `scale`: every probe of `shot` holds its expected color, and the rest
    /// a color no probe expects.
    fn painted(shot: Shot, scale: u32) -> Image {
        let (width, height) = radio::viewport();
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

    /// `image` as an RGBA PNG without compression, which the unoptimized test build writes quickly.
    fn encode_uncompressed(image: &Image) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, image.width, image.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::NoCompression);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&image.rgba).unwrap();
        writer.finish().unwrap();
        bytes
    }

    fn probe(cell: Cell, scheme: &ColorScheme, hovered: bool, what: &str) -> Probe {
        probes(cell, scheme, hovered)
            .into_iter()
            .find(|probe| probe.what == what)
            .unwrap_or_else(|| panic!("no probe named {what}"))
    }

    #[test]
    fn a_probe_reads_the_top_left_pixel_of_its_point_at_the_scale() {
        let probe = probes(
            cell(Selection::Selected, Availability::Enabled),
            &LIGHT,
            false,
        )[0];
        let (x, y) = (points_to_u32(probe.x), points_to_u32(probe.y));
        // Neither is 0 or 1, which scaling and shifting by the scale would not tell apart.
        assert!(x > 1 && y > 1, "{probe:?}");
        for scale in 1..=3 {
            assert_eq!(pixel_of(&probe, scale), (x * scale, y * scale));
        }
        // Whole points, so the point is the pixel's top left corner and not a fraction of one.
        assert_eq!(probe.x.0.fract().to_bits(), 0.0_f32.to_bits());
        assert_eq!(probe.y.0.fract().to_bits(), 0.0_f32.to_bits());
    }

    #[test]
    fn the_ring_and_the_dot_take_the_icon_colors_of_the_tokens() {
        let rgb = |scheme: &ColorScheme, role| Rgb::of(scheme, role);
        for scheme in [LIGHT, DARK] {
            let surface = rgb(&scheme, ColorRole::Surface);
            // Enabled: primary selected, on surface variant not, exactly.
            let selected = mark(cell(Selection::Selected, Availability::Enabled), &scheme);
            assert_eq!(
                (selected.0, selected.1),
                (rgb(&scheme, ColorRole::Primary), 0)
            );
            let unselected = mark(cell(Selection::Unselected, Availability::Enabled), &scheme);
            assert_eq!(
                (unselected.0, unselected.1),
                (rgb(&scheme, ColorRole::OnSurfaceVariant), 0)
            );
            // Disabled: on surface at 38 % over the surface, selected or not, with a blend's tolerance.
            for selection in [Selection::Selected, Selection::Unselected] {
                assert_eq!(
                    mark(cell(selection, Availability::Disabled), &scheme),
                    (
                        rgb(&scheme, ColorRole::OnSurface).over(0.38, surface),
                        BLEND_TOLERANCE
                    )
                );
            }
        }
        // The two appearances differ.
        assert_ne!(
            mark(cell(Selection::Selected, Availability::Enabled), &LIGHT),
            mark(cell(Selection::Selected, Availability::Enabled), &DARK)
        );
    }

    #[test]
    fn what_is_behind_a_radio_button_is_the_surface_or_its_state_layer() {
        for scheme in [LIGHT, DARK] {
            let surface = Rgb::of(&scheme, ColorRole::Surface);
            let rest = |cell| behind(cell, &scheme, false);
            let hover = |cell| behind(cell, &scheme, true);
            for selection in [Selection::Unselected, Selection::Selected] {
                for availability in [Availability::Enabled, Availability::Disabled] {
                    assert_eq!(rest(cell(selection, availability)), (surface, 0));
                }
            }
            // Hovered, an unselected one's layer is on surface and a selected one's primary, at 8 % over the surface.
            assert_eq!(
                hover(cell(Selection::Unselected, Availability::Enabled)),
                (
                    Rgb::of(&scheme, ColorRole::OnSurface).over(0.08, surface),
                    BLEND_TOLERANCE
                )
            );
            assert_eq!(
                hover(cell(Selection::Selected, Availability::Enabled)),
                (
                    Rgb::of(&scheme, ColorRole::Primary).over(0.08, surface),
                    BLEND_TOLERANCE
                )
            );
            // A disabled one shows no state, hovered or not.
            for selection in [Selection::Unselected, Selection::Selected] {
                assert_eq!(hover(cell(selection, Availability::Disabled)), (surface, 0));
            }
        }
    }

    #[test]
    fn the_dot_is_the_mark_when_selected_and_what_is_behind_when_not() {
        for scheme in [LIGHT, DARK] {
            let selected = cell(Selection::Selected, Availability::Enabled);
            let unselected = cell(Selection::Unselected, Availability::Enabled);
            let disabled = cell(Selection::Selected, Availability::Disabled);
            assert_eq!(
                probe(selected, &scheme, false, "dot").expected,
                Rgb::of(&scheme, ColorRole::Primary)
            );
            assert_eq!(
                probe(unselected, &scheme, false, "dot").expected,
                Rgb::of(&scheme, ColorRole::Surface)
            );
            // Hovered, an unselected radio button's middle shows its state layer; a selected one's is still the dot.
            assert_eq!(
                probe(unselected, &scheme, true, "dot").expected,
                behind(unselected, &scheme, true).0
            );
            assert_ne!(
                probe(unselected, &scheme, true, "dot").expected,
                Rgb::of(&scheme, ColorRole::Surface)
            );
            assert_eq!(
                probe(selected, &scheme, true, "dot").expected,
                Rgb::of(&scheme, ColorRole::Primary)
            );
            assert_eq!(
                probe(disabled, &scheme, true, "dot").expected,
                mark(disabled, &scheme).0
            );
        }
    }

    /// The smallest and the largest distance from `middle` to the pixel whose top left corner is `corner`, and the
    /// largest distance along an axis.
    fn distances(corner: (f32, f32), middle: (f32, f32)) -> (f32, f32, f32) {
        let xs = [corner.0 - middle.0, corner.0 + 1.0 - middle.0];
        let ys = [corner.1 - middle.1, corner.1 + 1.0 - middle.1];
        let nearest = |values: [f32; 2]| {
            if values[0] <= 0.0 && values[1] >= 0.0 {
                0.0
            } else {
                values[0].abs().min(values[1].abs())
            }
        };
        let farthest = |values: [f32; 2]| values[0].abs().max(values[1].abs());
        (
            nearest(xs).hypot(nearest(ys)),
            farthest(xs).hypot(farthest(ys)),
            farthest(xs).max(farthest(ys)),
        )
    }

    #[test]
    fn every_probe_lies_wholly_in_the_region_it_names() {
        let outer = radio_button_tokens::ICON_SIZE.0 / 2.0;
        let inner = outer - STROKE_WIDTH.0;
        // The dot is drawn one half stroke inside its radius (`RadioButton.kt`, line 188 and 189).
        let dot = DOT_SIZE.0 / 2.0 - STROKE_WIDTH.0 / 2.0;
        let layer = radio_button_tokens::STATE_LAYER_SIZE.0 / 2.0;
        for cell in Cell::all() {
            let (x, y) = cell.origin();
            let middle = (x.0 + FRAME.0 / 2.0, y.0 + FRAME.0 / 2.0);
            for probe in probes(cell, &LIGHT, false) {
                let (near, far, along) = distances((probe.x.0, probe.y.0), middle);
                let what = probe.what;
                let inside = match what {
                    w if w.starts_with("ring") => near >= inner && far <= outer,
                    "dot" => far <= dot,
                    "between the dot and the ring" => near >= dot && far <= inner,
                    // Past the ring and within the state layer's circle.
                    "outside the ring"
                    | "in the state layer"
                    | "just inside the state layer's edge" => near >= outer && far <= layer,
                    "outside the state layer" => near >= layer,
                    // Outside the circle, inside its box.
                    "outside the state layer's corner" => near >= layer && along <= layer,
                    other => panic!("a probe the test does not know: {other}"),
                };
                assert!(
                    inside,
                    "{cell:?} {what}: {near}..{far} points from the middle, {along} along an axis"
                );
            }
        }
        // The ring's pixels are the band's: the ones just inside and just outside it are not probed as the ring.
        assert_eq!((inner, outer, dot, layer), (8.0, 10.0, 5.0, 20.0));
    }

    #[test]
    fn the_probes_read_each_radio_button_in_twelve_places_that_are_all_different() {
        for shot in Shot::all() {
            let found = shot_probes(shot);
            assert_eq!(found.len(), Cell::all().count() * 12);
            for (i, a) in found.iter().enumerate() {
                for b in &found[i + 1..] {
                    assert!(
                        (a.x, a.y) != (b.x, b.y),
                        "{} {} and {} {}",
                        a.cell.test_id(),
                        a.what,
                        b.cell.test_id(),
                        b.what
                    );
                }
            }
        }
    }

    #[test]
    fn hovering_changes_only_what_is_behind_the_hovered_radio_button() {
        let appearance = Appearance::Light;
        let rest = shot_probes(Shot {
            appearance,
            hovered: None,
        });
        for hovered in HOVERED {
            let shot = Shot {
                appearance,
                hovered: Some(hovered),
            };
            let found = shot_probes(shot);
            assert_eq!(found.len(), rest.len());
            let changed: Vec<&Probe> = found
                .iter()
                .zip(&rest)
                .filter(|(a, b)| a != b)
                .map(|(a, _)| a)
                .collect();
            for probe in &changed {
                assert_eq!(probe.cell, hovered, "{probe:?}");
            }
            if hovered.availability == Availability::Disabled {
                // No state layer: nothing changes at all.
                assert_eq!(changed, Vec::<&Probe>::new());
            } else {
                // The probes that read what is behind it, and not the ring, the surface outside the layer or the dot
                // of a selected one.
                let names: Vec<&str> = changed.iter().map(|probe| probe.what).collect();
                let mut expected = vec![
                    "between the dot and the ring",
                    "outside the ring",
                    "in the state layer",
                    "just inside the state layer's edge",
                ];
                if hovered.selection == Selection::Unselected {
                    expected.insert(0, "dot");
                }
                assert_eq!(names, expected, "{hovered:?}");
            }
        }
    }

    #[test]
    fn the_shots_are_the_test_script_files() {
        let names: Vec<String> = Shot::all().into_iter().map(Shot::file_name).collect();
        assert_eq!(
            names,
            [
                "radio-light-rest.png",
                "radio-light-hover-unselected-enabled.png",
                "radio-light-hover-selected-enabled.png",
                "radio-light-hover-unselected-disabled.png",
                "radio-dark-rest.png",
                "radio-dark-hover-unselected-enabled.png",
                "radio-dark-hover-selected-enabled.png",
                "radio-dark-hover-unselected-disabled.png",
            ]
        );
        assert_eq!(Shot::all()[0].appearance.scheme(), LIGHT);
        assert_eq!(Shot::all()[4].appearance.scheme(), DARK);
        // The script saves these files, over these radio buttons, in this viewport.
        let script = include_str!("../../../examples/catalog/radio.test.tsx");
        let (width, height) = radio::viewport();
        assert!(
            script.contains(&format!(
                "const VIEWPORT = {{ width: {}, height: {} }};",
                width.0, height.0
            )),
            "the script's viewport is not {}x{}",
            width.0,
            height.0
        );
        let slugs: Vec<String> = HOVERED
            .iter()
            .map(|cell| format!("\"{}\"", cell.slug()))
            .collect();
        assert!(script.contains(&format!("const HOVERED = [{}];", slugs.join(", "))));
        assert!(script.contains("`target/visual/radio-${scheme}-rest.png`"));
        assert!(script.contains("`target/visual/radio-${scheme}-hover-${radio}.png`"));
        assert!(script.contains("`id:radio-${radio}`"));
        assert!(script.contains("[\"light\", \"dark\"]"));
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
        let shot = Shot::all()[1];
        let mut image = painted(shot, 1);
        let hovered = shot.hovered.unwrap();
        let wanted = probe(hovered, &LIGHT, true, "in the state layer");
        let (x, y) = pixel_of(&wanted, 1);
        let start = ((y * image.width + x) * 4) as usize;
        // The surface where the mouse is over the radio button: its state layer is missing.
        image.rgba[start..start + 3].copy_from_slice(&Rgb::of(&LIGHT, ColorRole::Surface).0);
        let found = check(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe, wanted);
        assert_eq!(found[0].found, Some(Rgb::of(&LIGHT, ColorRole::Surface)));
        // The first radio button's box starts at the grid's margin (24) and the block at 2920; its middle is 12 points in,
        // and the probe is 14 points right of the middle (the ring's 10, then 4).
        assert_eq!(
            found[0].to_string(),
            format!(
                "radio-unselected-enabled in the state layer at (50, 2932) points: expected {} (± 2), found {}",
                wanted.expected,
                Rgb::of(&LIGHT, ColorRole::Surface)
            )
        );
        // The same image is not the dark shot.
        assert_ne!(check(&image, Shot::all()[5]).unwrap(), Vec::new());
    }

    #[test]
    fn a_state_layer_drawn_square_or_missing_or_on_a_disabled_one_fails() {
        // The hovered unselected radio button, with its layer drawn as a square: the box's corner is not the surface.
        let shot = Shot::all()[1];
        let hovered = shot.hovered.unwrap();
        let mut image = painted(shot, 1);
        let corner = probe(hovered, &LIGHT, true, "outside the state layer's corner");
        let layer = probe(hovered, &LIGHT, true, "in the state layer");
        let (x, y) = pixel_of(&corner, 1);
        let start = ((y * image.width + x) * 4) as usize;
        image.rgba[start..start + 3].copy_from_slice(&layer.expected.0);
        let found = check(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe.what, "outside the state layer's corner");

        // The disabled radio button hovered, with a state layer: what is behind it is not the surface.
        let shot = Shot::all()[3];
        let hovered = shot.hovered.unwrap();
        let mut image = painted(shot, 1);
        let behind = probe(hovered, &LIGHT, true, "in the state layer");
        let (x, y) = pixel_of(&behind, 1);
        let start = ((y * image.width + x) * 4) as usize;
        image.rgba[start..start + 3].copy_from_slice(
            &Rgb::of(&LIGHT, ColorRole::OnSurface)
                .over(0.08, Rgb::of(&LIGHT, ColorRole::Surface))
                .0,
        );
        assert_eq!(check(&image, shot).unwrap().len(), 1);
    }

    #[test]
    fn an_image_of_another_size_is_refused() {
        // The buttons' viewport is shorter than the radio buttons': a screenshot of it holds no radio button.
        let (width, height) = catalog::grid::viewport();
        let image = Image {
            width: points_to_u32(width),
            height: points_to_u32(height),
            rgba: vec![0; (points_to_u32(width) * points_to_u32(height) * 4) as usize],
        };
        assert!(check(&image, Shot::all()[0]).is_err());
        assert!(scale(&image).is_err());
        let mut image = painted(Shot::all()[0], 1);
        image.height -= 1;
        assert!(check(&image, Shot::all()[0]).is_err());
        let mut image = painted(Shot::all()[0], 2);
        image.width += 2;
        assert!(check(&image, Shot::all()[0]).is_err());
        let image = Image {
            width: 10,
            height: 10,
            rgba: vec![0; 400],
        };
        assert!(scale(&image).is_err());
    }

    fn directory(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("visual-checks-radio-{name}-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[test]
    fn run_reports_missing_and_wrong_screenshots() {
        let directory = directory("run");
        let shots = &Shot::all()[..2];
        let mut out = Vec::new();
        assert!(matches!(
            run_shots(&directory, shots, &mut out),
            Err(Error::Read { .. })
        ));
        for shot in shots {
            fs::write(
                directory.join(shot.file_name()),
                encode_uncompressed(&painted(*shot, 1)),
            )
            .unwrap();
        }
        let mut out = Vec::new();
        assert!(run_shots(&directory, shots, &mut out).is_ok());
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("radio-light-rest.png: ok")
        );
        // The rest screenshot saved as the hovered one: the state layer is missing.
        fs::copy(
            directory.join("radio-light-rest.png"),
            directory.join("radio-light-hover-unselected-enabled.png"),
        )
        .unwrap();
        let mut out = Vec::new();
        assert!(
            matches!(run_shots(&directory, shots, &mut out), Err(Error::Mismatches(n)) if n > 0)
        );
        let out = String::from_utf8(out).unwrap();
        assert!(out.contains("radio-light-hover-unselected-enabled.png: FAILED"));
        assert!(out.contains("radio-unselected-enabled in the state layer"));
        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn the_radio_buttons_checks_join_the_buttons_without_hiding_either() {
        // An empty directory: no screenshot to read, so the radio buttons' checks fail on reading.
        let directory = directory("also");
        let mut out = Vec::new();
        assert!(matches!(
            also(Ok(()), &directory, &mut out),
            Err(Error::Read { .. })
        ));
        // The buttons' own failure comes first, and the radio buttons' reading failure does not replace it.
        assert!(matches!(
            also(Err(Error::Mismatches(3)), &directory, &mut out),
            Err(Error::Mismatches(3))
        ));
        assert!(matches!(
            also(Err(Error::Usage), &directory, &mut out),
            Err(Error::Usage)
        ));
        // Both with wrong pixels: the counts add up. Every screenshot is a blank page of the right size.
        for shot in Shot::all() {
            let (width, height) = radio::viewport();
            let image = Image {
                width: points_to_u32(width),
                height: points_to_u32(height),
                rgba: vec![255; (points_to_u32(width) * points_to_u32(height) * 4) as usize],
            };
            fs::write(
                directory.join(shot.file_name()),
                encode_uncompressed(&image),
            )
            .unwrap();
        }
        let mut out = Vec::new();
        let Err(Error::Mismatches(radio_only)) = also(Ok(()), &directory, &mut out) else {
            panic!("blank pages are not the tokens' colors");
        };
        assert!(radio_only > 0);
        let mut out = Vec::new();
        assert!(matches!(
            also(Err(Error::Mismatches(7)), &directory, &mut out),
            Err(Error::Mismatches(n)) if n == radio_only + 7
        ));
        fs::remove_dir_all(&directory).unwrap();
    }
}
