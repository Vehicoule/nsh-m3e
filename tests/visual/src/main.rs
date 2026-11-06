//! Checks the known pixels of the catalog's screenshots (ADR 0013): `cargo run -p visual-checks -- <directory>`.
//!
//! `nsh test examples/catalog --target <host>` opens the catalog on one of NSH's hosts and saves its screenshots in
//! `target/visual` (`examples/catalog/button.test.tsx`). This reads each of them and checks, for every button of the catalog, the pixels whose
//! color the tokens give: the container's color inside it, the outline's on its edge, and the surface outside its
//! top left corner, where a rounded corner leaves the background. Where the antialiasing of an edge or of the label
//! makes a color unknown, nothing is checked.
//!
//! It is not a `cargo xtask` task: `xtask` must build when the library does not (it regenerates the library's
//! tokens), so it cannot depend on it, and these checks take every expected color from the library's tokens.

use std::fmt;
use std::fs;
use std::io::{self, Cursor, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use catalog::grid::{self, Availability, Cell};
use nsh_core::geometry::Points;
use nsh_m3e::button::{ButtonShape, ButtonSize, ButtonStyle};
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::tokens::androidx::{
    filled_button_tokens, filled_tonal_button_tokens, outlined_button_tokens, state_tokens,
};

const USAGE: &str = "usage: cargo run -p visual-checks -- <directory of screenshots>";

/// How far inside the container's left edge, at mid height, its color is read: past the antialiased edge, and
/// before the label, which starts 12 points or more from the edge (the extra small button's side space, androidx
/// `Button.kt` `ExtraSmallContentPadding`). Not a design value: where the check looks.
const CONTAINER_INSET: Points = Points(4.0);

/// How far a channel may be from the expected value where a color is laid over another at an opacity: hosts round
/// the opacity and the blend to 8 bits in their own ways. A solid color must be exact. Not a design value: the
/// rounding of two 8-bit steps, one for the opacity and one for the blend.
const BLEND_TOLERANCE: u8 = 2;

/// The system appearance a screenshot was taken in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Appearance {
    Light,
    Dark,
}

impl Appearance {
    const ALL: [Self; 2] = [Self::Light, Self::Dark];

    fn name(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    fn scheme(self) -> ColorScheme {
        match self {
            Self::Light => ColorScheme::BASELINE_LIGHT,
            Self::Dark => ColorScheme::BASELINE_DARK,
        }
    }
}

/// One screenshot the test scripts take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Shot {
    appearance: Appearance,
    /// The button the mouse is over, if any.
    hovered: Option<Cell>,
}

impl Shot {
    /// Every screenshot `examples/catalog/button.test.tsx` takes: the catalog at rest, then the mouse over the small
    /// round enabled button of each style, in each appearance.
    fn all() -> Vec<Self> {
        Appearance::ALL
            .into_iter()
            .flat_map(|appearance| {
                std::iter::once(Self {
                    appearance,
                    hovered: None,
                })
                .chain(grid::STYLES.into_iter().map(move |style| Self {
                    appearance,
                    hovered: Some(Cell::new(
                        style,
                        ButtonShape::Round,
                        ButtonSize::Small,
                        Availability::Enabled,
                    )),
                }))
            })
            .collect()
    }

    /// Its file name, as the test script saves it.
    fn file_name(self) -> String {
        match self.hovered {
            None => format!("{}-rest.png", self.appearance.name()),
            Some(cell) => format!(
                "{}-hover-{}.png",
                self.appearance.name(),
                cell.label().to_lowercase()
            ),
        }
    }
}

/// An 8-bit sRGB color, as a screenshot holds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rgb([u8; 3]);

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [r, g, b] = self.0;
        write!(f, "#{r:02x}{g:02x}{b:02x}")
    }
}

impl Rgb {
    fn of(scheme: &ColorScheme, role: ColorRole) -> Self {
        let color = scheme.get(role);
        Self([color.red, color.green, color.blue])
    }

    /// `self` laid over `below` at `opacity`, as the source-over blend gives it.
    fn over(self, opacity: f32, below: Self) -> Self {
        let mut out = [0; 3];
        for ((out, top), below) in out.iter_mut().zip(self.0).zip(below.0) {
            let value = f32::from(top) * opacity + f32::from(below) * (1.0 - opacity);
            *out = nearest_u8(value);
        }
        Self(out)
    }

    /// Whether every channel is within `tolerance` of `other`'s.
    fn near(self, other: Self, tolerance: u8) -> bool {
        self.0
            .iter()
            .zip(other.0)
            .all(|(a, b)| a.abs_diff(b) <= tolerance)
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

/// The color of `role` laid over `below` at `opacity`, with the tolerance a blend needs.
fn faded(scheme: &ColorScheme, role: ColorRole, opacity: f32, below: Rgb) -> (Rgb, u8) {
    (Rgb::of(scheme, role).over(opacity, below), BLEND_TOLERANCE)
}

/// The label's color of an enabled button of `style`, which the state layer takes.
fn content_role(style: ButtonStyle) -> ColorRole {
    match style {
        ButtonStyle::Tonal => filled_tonal_button_tokens::LABEL_TEXT_COLOR,
        ButtonStyle::Outlined => outlined_button_tokens::LABEL_TEXT_COLOR,
        // androidx `Button.kt`, `ButtonDefaults.defaultTextButtonColors` (line 1423): the primary color, where
        // `TextButtonTokens.LabelColor` says on surface variant (ADR 0012), as the button takes it.
        ButtonStyle::Text => ColorRole::Primary,
        _ => filled_button_tokens::LABEL_TEXT_COLOR,
    }
}

/// The container's color of `cell` and its tolerance, over `surface`, with the mouse over it or not.
fn container(cell: Cell, scheme: &ColorScheme, hovered: bool) -> (Rgb, u8) {
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let disabled = cell.availability == Availability::Disabled;
    let (rest, tolerance) = match (cell.style, disabled) {
        (ButtonStyle::Filled, false) => (Rgb::of(scheme, filled_button_tokens::CONTAINER_COLOR), 0),
        (ButtonStyle::Tonal, false) => (
            Rgb::of(scheme, filled_tonal_button_tokens::CONTAINER_COLOR),
            0,
        ),
        (ButtonStyle::Filled, true) => faded(
            scheme,
            filled_button_tokens::DISABLED_CONTAINER_COLOR,
            filled_button_tokens::DISABLED_CONTAINER_OPACITY,
            surface,
        ),
        (ButtonStyle::Tonal, true) => faded(
            scheme,
            filled_tonal_button_tokens::DISABLED_CONTAINER_COLOR,
            filled_tonal_button_tokens::DISABLED_CONTAINER_OPACITY,
            surface,
        ),
        // The outlined and text buttons have no container: the surface shows through.
        _ => (surface, 0),
    };
    if hovered && !disabled {
        faded(
            scheme,
            content_role(cell.style),
            state_tokens::HOVER_STATE_LAYER_OPACITY,
            rest,
        )
    } else {
        (rest, tolerance)
    }
}

/// The pixels the checks know for `cell` in a screenshot in `scheme`, with the mouse over it or not.
fn probes(cell: Cell, scheme: &ColorScheme, hovered: bool) -> Vec<Probe> {
    let (x, y) = cell.origin();
    let middle = Points(y.0 + grid::height(cell.size).0 / 2.0);
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let (expected, tolerance) = container(cell, scheme, hovered);
    let mut probes = vec![
        Probe {
            cell,
            what: "container",
            x: Points(x.0 + CONTAINER_INSET.0),
            y: middle,
            expected,
            tolerance,
        },
        // The top left pixel of the container's box: outside the arc of every shape's corner (12 points or more), so
        // the surface. A corner drawn square would fill it.
        Probe {
            cell,
            what: "outside the corner",
            x,
            y,
            expected: surface,
            tolerance: 0,
        },
    ];
    if cell.style == ButtonStyle::Outlined {
        // The outline's first pixel row, past the end of the top left corner's arc (half the height at most, for the
        // round shape), where the edge is straight: a browser antialiases a 1-point line along an arc, even where
        // the arc is nearly straight. Every button of the catalog is more than 8 points wider than it is tall (its
        // side spaces and its label), so the top edge is straight there whatever the shape. The outline is 1 point wide
        // (`ButtonSmallTokens.OutlinedOutlineWidth` at every size), faded by the container's disabled opacity when
        // disabled (androidx `outlinedButtonBorder`, ADR 0012). The state layer covers the container inside the
        // outline (an absolute view at inset 0, which starts inside the border), so hovering leaves the outline as it is.
        let (outline, tolerance) = if cell.availability == Availability::Disabled {
            faded(
                scheme,
                outlined_button_tokens::OUTLINE_COLOR,
                outlined_button_tokens::DISABLED_CONTAINER_OPACITY,
                surface,
            )
        } else {
            (Rgb::of(scheme, outlined_button_tokens::OUTLINE_COLOR), 0)
        };
        probes.push(Probe {
            cell,
            what: "outline",
            x: Points(x.0 + grid::height(cell.size).0 / 2.0 + CONTAINER_INSET.0),
            y,
            expected: outline,
            tolerance,
        });
    }
    probes
}

/// A decoded screenshot: 8-bit RGBA, row by row.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Image {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl Image {
    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let mut decoder = png::Decoder::new(Cursor::new(bytes));
        // Palettes and low bit depths expanded, 16-bit channels cut to 8, an alpha channel added: RGBA8 or gray
        // with alpha. The flags are distinct bits, so their union is written with `union`, where `|` and `^` would
        // give the same result and no test could tell them apart.
        decoder.set_transformations(
            png::Transformations::EXPAND
                .union(png::Transformations::STRIP_16)
                .union(png::Transformations::ALPHA),
        );
        let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
        let size = reader
            .output_buffer_size()
            .ok_or("the image is too large to decode")?;
        let mut buffer = vec![0; size];
        let info = reader.next_frame(&mut buffer).map_err(|e| e.to_string())?;
        buffer.truncate(info.buffer_size());
        let rgba = match info.color_type {
            png::ColorType::Rgba => buffer,
            png::ColorType::GrayscaleAlpha => buffer
                .as_chunks::<2>()
                .0
                .iter()
                .flat_map(|&[g, a]| [g, g, g, a])
                .collect(),
            other => return Err(format!("unexpected color type {other:?}")),
        };
        Ok(Self {
            width: info.width,
            height: info.height,
            rgba,
        })
    }

    /// The color of the pixel at `x`, `y`, or none outside the image. Screenshots are opaque: alpha is not read.
    fn pixel(&self, x: u32, y: u32) -> Option<Rgb> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let start =
            usize::try_from((u64::from(y) * u64::from(self.width) + u64::from(x)) * 4).ok()?;
        let rgb = self.rgba.get(start..start + 3)?;
        Some(Rgb([*rgb.first()?, *rgb.get(1)?, *rgb.get(2)?]))
    }

    /// How many pixels a point is: the image is the catalog's viewport at a whole scale (1 on a desktop, 2 on a
    /// high-density screen).
    fn scale(&self) -> Result<u32, String> {
        let (width, height) = grid::viewport();
        let scale = self.width / points_to_u32(width).max(1);
        if scale == 0
            || self.width != scale * points_to_u32(width)
            || self.height != scale * points_to_u32(height)
        {
            return Err(format!(
                "the image is {}x{} pixels, not the catalog's {}x{} points at a whole scale",
                self.width, self.height, width.0, height.0
            ));
        }
        Ok(scale)
    }
}

/// The 8-bit value nearest to `value`, clamped to 0..=255.
///
/// Found among the 256 values, because Rust has no conversion from a float to an integer that clippy's casting lints
/// accept; the casts would need an `#[expect]`.
fn nearest_u8(value: f32) -> u8 {
    (0..=u8::MAX)
        .min_by(|a, b| {
            (f32::from(*a) - value)
                .abs()
                .total_cmp(&(f32::from(*b) - value).abs())
        })
        .unwrap_or_default()
}

/// A length of the catalog in points as a whole number: the catalog's lengths are whole numbers of points, from 0 to
/// a few thousand. Found by search for the same reason as [`nearest_u8`]; a length that is not a whole number below
/// 65536 gives 0, which no check accepts (the image would not be the catalog's size).
fn points_to_u32(points: Points) -> u32 {
    (0..=u16::MAX)
        .find(|n| (f32::from(*n) - points.0).abs() < 0.5)
        .map_or(0, u32::from)
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
    let scale = image.scale()?;
    let scheme = shot.appearance.scheme();
    Ok(Cell::all()
        .flat_map(|cell| probes(cell, &scheme, shot.hovered == Some(cell)))
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

#[derive(Debug)]
enum Error {
    Usage,
    Read { path: PathBuf, source: io::Error },
    Image { path: PathBuf, message: String },
    Mismatches(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => write!(f, "{USAGE}"),
            Self::Read { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Image { path, message } => write!(f, "{}: {message}", path.display()),
            Self::Mismatches(count) => write!(f, "{count} pixels are not the tokens' colors"),
        }
    }
}

impl std::error::Error for Error {}

fn run(directory: &Path, out: &mut impl io::Write) -> Result<(), Error> {
    let mut mismatches = 0;
    for shot in Shot::all() {
        let path = directory.join(shot.file_name());
        let bytes = fs::read(&path).map_err(|source| Error::Read {
            path: path.clone(),
            source,
        })?;
        let image = Image::decode(bytes).map_err(|message| Error::Image {
            path: path.clone(),
            message,
        })?;
        let found = check(&image, shot).map_err(|message| Error::Image {
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

/// Runs the checks on the one directory `args` names, writing what they find to `out`.
fn run_with(args: &[String], out: &mut impl io::Write) -> Result<(), Error> {
    match args {
        [directory] => run(Path::new(directory), out),
        _ => Err(Error::Usage),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    run_with(&args, &mut io::stdout()).map_or_else(
        |error| {
            let _ = writeln!(io::stderr(), "visual-checks: {error}");
            ExitCode::FAILURE
        },
        |()| ExitCode::SUCCESS,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;

    fn cell(style: ButtonStyle, availability: Availability) -> Cell {
        Cell::new(style, ButtonShape::Round, ButtonSize::Small, availability)
    }

    /// The catalog painted as the tokens say, at `scale`: every probe of `shot` holds its expected color, and the
    /// rest a color no probe expects.
    fn painted(shot: Shot, scale: u32) -> Image {
        let (width, height) = grid::viewport();
        let (width, height) = (points_to_u32(width) * scale, points_to_u32(height) * scale);
        let mut rgba = [1, 2, 3, 255].repeat((width * height) as usize);
        let scheme = shot.appearance.scheme();
        for cell in Cell::all() {
            for probe in probes(cell, &scheme, shot.hovered == Some(cell)) {
                let (x, y) = pixel_of(&probe, scale);
                let start = ((y * width + x) * 4) as usize;
                rgba[start..start + 3].copy_from_slice(&probe.expected.0);
            }
        }
        Image {
            width,
            height,
            rgba,
        }
    }

    fn encode(width: u32, height: u32, color: png::ColorType, data: &[u8]) -> Vec<u8> {
        encode_with(width, height, color, png::BitDepth::Eight, None, data)
    }

    fn encode_with(
        width: u32,
        height: u32,
        color: png::ColorType,
        depth: png::BitDepth,
        palette: Option<Vec<u8>>,
        data: &[u8],
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(color);
        encoder.set_depth(depth);
        if let Some(palette) = palette {
            encoder.set_palette(palette);
        }
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(data).unwrap();
        writer.finish().unwrap();
        bytes
    }

    #[test]
    fn a_blend_is_the_source_over_formula_rounded() {
        let white = Rgb([255, 255, 255]);
        let black = Rgb([0, 0, 0]);
        assert_eq!(black.over(0.2, white), Rgb([204, 204, 204]));
        assert_eq!(white.over(0.08, black), Rgb([20, 20, 20]));
        assert_eq!(
            Rgb([100, 0, 200]).over(0.5, Rgb([0, 100, 0])),
            Rgb([50, 50, 100])
        );
        assert_eq!(white.over(1.0, black), white);
        assert_eq!(white.over(0.0, black), black);
    }

    #[test]
    fn near_allows_the_tolerance_and_no_more() {
        let color = Rgb([10, 20, 30]);
        assert!(color.near(Rgb([12, 18, 30]), 2));
        assert!(!color.near(Rgb([13, 20, 30]), 2));
        assert!(!color.near(Rgb([10, 20, 27]), 2));
        assert!(color.near(color, 0));
        assert!(!color.near(Rgb([10, 21, 30]), 0));
    }

    #[test]
    fn containers_are_the_style_tokens_colors() {
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let primary = Rgb::of(&LIGHT, ColorRole::Primary);
        let filled = cell(ButtonStyle::Filled, Availability::Enabled);
        assert_eq!(
            container(filled, &LIGHT, false),
            (Rgb([0x67, 0x50, 0xa4]), 0)
        );
        assert_eq!(
            container(
                cell(ButtonStyle::Tonal, Availability::Enabled),
                &LIGHT,
                false
            ),
            (Rgb([0xe8, 0xde, 0xf8]), 0)
        );
        for style in [ButtonStyle::Outlined, ButtonStyle::Text] {
            assert_eq!(
                container(cell(style, Availability::Enabled), &LIGHT, false),
                (surface, 0)
            );
            assert_eq!(
                container(cell(style, Availability::Disabled), &LIGHT, false),
                (surface, 0)
            );
        }
        // Disabled: on surface at 0.1 (filled) and 0.12 (tonal) over the surface.
        let on_surface = Rgb::of(&LIGHT, ColorRole::OnSurface);
        for (style, opacity) in [(ButtonStyle::Filled, 0.1), (ButtonStyle::Tonal, 0.12)] {
            assert_eq!(
                container(cell(style, Availability::Disabled), &LIGHT, false),
                (on_surface.over(opacity, surface), BLEND_TOLERANCE)
            );
        }
        // Hovered: the label's color at 0.08 over the container; a disabled button shows no hover.
        let on_primary = Rgb::of(&LIGHT, ColorRole::OnPrimary);
        assert_eq!(
            container(filled, &LIGHT, true),
            (on_primary.over(0.08, primary), BLEND_TOLERANCE)
        );
        assert_eq!(
            container(cell(ButtonStyle::Text, Availability::Enabled), &LIGHT, true),
            (primary.over(0.08, surface), BLEND_TOLERANCE)
        );
        assert_eq!(
            container(
                cell(ButtonStyle::Filled, Availability::Disabled),
                &LIGHT,
                true
            ),
            container(
                cell(ButtonStyle::Filled, Availability::Disabled),
                &LIGHT,
                false
            )
        );
    }

    #[test]
    fn the_state_layer_takes_each_style_label_color() {
        assert_eq!(content_role(ButtonStyle::Filled), ColorRole::OnPrimary);
        assert_eq!(
            content_role(ButtonStyle::Tonal),
            ColorRole::OnSecondaryContainer
        );
        assert_eq!(
            content_role(ButtonStyle::Outlined),
            ColorRole::OnSurfaceVariant
        );
        assert_eq!(content_role(ButtonStyle::Text), ColorRole::Primary);
        // The text button's label is `Button.kt`'s primary, not its token's.
        assert_ne!(
            nsh_m3e::tokens::androidx::text_button_tokens::LABEL_COLOR,
            ColorRole::Primary
        );
    }

    #[test]
    fn probes_read_inside_the_container_outside_its_corner_and_on_the_outline() {
        let filled = cell(ButtonStyle::Filled, Availability::Enabled);
        let (x, y) = filled.origin();
        let found = probes(filled, &LIGHT, false);
        assert_eq!(found.len(), 2);
        assert_eq!(
            (found[0].x, found[0].y),
            (Points(x.0 + 4.0), Points(y.0 + 20.0))
        );
        assert_eq!((found[1].x, found[1].y), (x, y));
        assert_eq!(found[1].expected, Rgb::of(&LIGHT, ColorRole::Surface));

        let outlined = cell(ButtonStyle::Outlined, Availability::Enabled);
        let (x, y) = outlined.origin();
        let found = probes(outlined, &LIGHT, false);
        assert_eq!(found.len(), 3);
        assert_eq!((found[2].x, found[2].y), (Points(x.0 + 24.0), y));
        assert_eq!(
            found[2].expected,
            Rgb::of(&LIGHT, ColorRole::OutlineVariant)
        );
        assert_eq!(found[2].tolerance, 0);

        let disabled = probes(
            cell(ButtonStyle::Outlined, Availability::Disabled),
            &LIGHT,
            false,
        );
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        assert_eq!(
            (disabled[2].expected, disabled[2].tolerance),
            (
                Rgb::of(&LIGHT, ColorRole::OutlineVariant).over(0.1, surface),
                BLEND_TOLERANCE
            )
        );
        // Hovering lays the state layer inside the outline, not over it.
        let hovered = probes(outlined, &LIGHT, true);
        assert_eq!(hovered[2], found[2]);
        assert_ne!(hovered[0], found[0]);
        // Mid height follows the size.
        let large = Cell::new(
            ButtonStyle::Filled,
            ButtonShape::Square,
            ButtonSize::Large,
            Availability::Enabled,
        );
        let (_, y) = large.origin();
        assert_eq!(probes(large, &LIGHT, false)[0].y, Points(y.0 + 48.0));
    }

    #[test]
    fn the_shots_are_the_test_script_files() {
        let names: Vec<String> = Shot::all().into_iter().map(Shot::file_name).collect();
        assert_eq!(
            names,
            [
                "light-rest.png",
                "light-hover-filled.png",
                "light-hover-tonal.png",
                "light-hover-outlined.png",
                "light-hover-text.png",
                "dark-rest.png",
                "dark-hover-filled.png",
                "dark-hover-tonal.png",
                "dark-hover-outlined.png",
                "dark-hover-text.png",
            ]
        );
        assert_eq!(Shot::all()[0].appearance.scheme(), LIGHT);
        assert_eq!(
            Shot::all()[5].appearance.scheme(),
            ColorScheme::BASELINE_DARK
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
        let shot = Shot::all()[1];
        let mut image = painted(shot, 1);
        let hovered = shot.hovered.unwrap();
        let probe = probes(hovered, &LIGHT, true)[0];
        let (x, y) = pixel_of(&probe, 1);
        let start = ((y * image.width + x) * 4) as usize;
        // The container at rest where the mouse is over it.
        image.rgba[start..start + 3].copy_from_slice(&Rgb::of(&LIGHT, ColorRole::Primary).0);
        let found = check(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe, probe);
        assert_eq!(found[0].found, Some(Rgb([0x67, 0x50, 0xa4])));
        assert_eq!(
            found[0].to_string(),
            format!(
                "filled-small-round-enabled container at (28, 92) points: expected {} (± 2), found #6750a4",
                probe.expected
            )
        );
        // The same image is not the dark shot.
        assert_ne!(check(&image, Shot::all()[6]).unwrap(), Vec::new());
    }

    #[test]
    fn an_image_of_another_size_is_refused() {
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
        assert!(image.scale().is_err());
    }

    #[test]
    fn pixels_outside_the_image_are_none() {
        let image = Image {
            width: 2,
            height: 1,
            rgba: vec![1, 2, 3, 255, 4, 5, 6, 255],
        };
        assert_eq!(image.pixel(0, 0), Some(Rgb([1, 2, 3])));
        assert_eq!(image.pixel(1, 0), Some(Rgb([4, 5, 6])));
        assert_eq!(image.pixel(2, 0), None);
        assert_eq!(image.pixel(0, 1), None);
        // Past the end of a row, though still in the buffer: the next row's pixel is not this one.
        let tall = Image {
            width: 1,
            height: 2,
            rgba: vec![1, 2, 3, 255, 4, 5, 6, 255],
        };
        assert_eq!(tall.pixel(0, 1), Some(Rgb([4, 5, 6])));
        assert_eq!(tall.pixel(1, 0), None);
    }

    #[test]
    fn rgba_and_gray_pngs_decode_to_rgba() {
        let rgba = [10, 20, 30, 255, 40, 50, 60, 128];
        let image = Image::decode(encode(2, 1, png::ColorType::Rgba, &rgba)).unwrap();
        assert_eq!((image.width, image.height), (2, 1));
        assert_eq!(image.rgba, rgba);
        let image = Image::decode(encode(1, 2, png::ColorType::Rgb, &[1, 2, 3, 4, 5, 6])).unwrap();
        assert_eq!(image.rgba, [1, 2, 3, 255, 4, 5, 6, 255]);
        let image = Image::decode(encode(2, 1, png::ColorType::Grayscale, &[7, 9])).unwrap();
        assert_eq!(image.rgba, [7, 7, 7, 255, 9, 9, 9, 255]);
        assert!(Image::decode(b"not a png".to_vec()).is_err());
    }

    #[test]
    fn sixteen_bit_and_palette_pngs_decode_to_8_bit_rgba() {
        // 16 bits per channel: the high byte of each.
        let data = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc];
        let image = Image::decode(encode_with(
            1,
            1,
            png::ColorType::Rgb,
            png::BitDepth::Sixteen,
            None,
            &data,
        ))
        .unwrap();
        assert_eq!(image.rgba, [0x12, 0x56, 0x9a, 255]);
        // A palette: each index becomes its color.
        let image = Image::decode(encode_with(
            2,
            1,
            png::ColorType::Indexed,
            png::BitDepth::Eight,
            Some(vec![1, 2, 3, 4, 5, 6]),
            &[1, 0],
        ))
        .unwrap();
        assert_eq!(image.rgba, [4, 5, 6, 255, 1, 2, 3, 255]);
        // 16-bit gray with alpha.
        let image = Image::decode(encode_with(
            1,
            1,
            png::ColorType::GrayscaleAlpha,
            png::BitDepth::Sixteen,
            None,
            &[0x40, 0, 0x80, 0],
        ))
        .unwrap();
        assert_eq!(image.rgba, [0x40, 0x40, 0x40, 0x80]);
    }

    #[test]
    fn whole_points_convert_and_others_do_not() {
        assert_eq!(points_to_u32(Points(0.0)), 0);
        assert_eq!(points_to_u32(Points(7.0)), 7);
        assert_eq!(points_to_u32(Points(2608.0)), 2608);
        assert_eq!(points_to_u32(Points(7.2)), 7);
        // Halfway between two whole numbers is neither.
        assert_eq!(points_to_u32(Points(2.5)), 0);
        assert_eq!(points_to_u32(Points(-3.0)), 0);
        assert_eq!(nearest_u8(229.4), 229);
        assert_eq!(nearest_u8(229.6), 230);
        assert_eq!(nearest_u8(-4.0), 0);
        assert_eq!(nearest_u8(300.0), 255);
    }

    #[test]
    fn a_probe_reads_the_top_left_pixel_of_its_point_at_the_scale() {
        let probe = probes(
            cell(ButtonStyle::Filled, Availability::Enabled),
            &LIGHT,
            false,
        )[0];
        assert_eq!((probe.x, probe.y), (Points(28.0), Points(92.0)));
        assert_eq!(pixel_of(&probe, 1), (28, 92));
        assert_eq!(pixel_of(&probe, 2), (56, 184));
        assert_eq!(pixel_of(&probe, 3), (84, 276));
    }

    #[test]
    fn errors_say_what_failed() {
        assert_eq!(Error::Usage.to_string(), USAGE);
        assert_eq!(
            Error::Mismatches(3).to_string(),
            "3 pixels are not the tokens' colors"
        );
        assert_eq!(
            Error::Image {
                path: PathBuf::from("a.png"),
                message: "bad".to_owned()
            }
            .to_string(),
            "a.png: bad"
        );
        assert_eq!(
            Error::Read {
                path: PathBuf::from("b.png"),
                source: io::Error::other("gone")
            }
            .to_string(),
            "b.png: gone"
        );
    }

    #[test]
    fn the_arguments_name_one_directory() {
        let mut out = Vec::new();
        assert!(matches!(run_with(&[], &mut out), Err(Error::Usage)));
        let two = ["a".to_owned(), "b".to_owned()];
        assert!(matches!(run_with(&two, &mut out), Err(Error::Usage)));
        let missing = ["/nonexistent/visual-checks".to_owned()];
        assert!(matches!(
            run_with(&missing, &mut out),
            Err(Error::Read { .. })
        ));
    }

    #[test]
    fn run_reports_missing_and_wrong_screenshots() {
        let directory = std::env::temp_dir().join(format!("visual-checks-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let mut out = Vec::new();
        assert!(matches!(run(&directory, &mut out), Err(Error::Read { .. })));
        for shot in Shot::all() {
            let image = painted(shot, 1);
            let bytes = encode(image.width, image.height, png::ColorType::Rgba, &image.rgba);
            fs::write(directory.join(shot.file_name()), bytes).unwrap();
        }
        let mut out = Vec::new();
        assert!(run(&directory, &mut out).is_ok());
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("light-rest.png: ok")
        );
        // The light screenshot saved as the dark one.
        fs::copy(
            directory.join("light-rest.png"),
            directory.join("dark-rest.png"),
        )
        .unwrap();
        let mut out = Vec::new();
        assert!(matches!(run(&directory, &mut out), Err(Error::Mismatches(n)) if n > 0));
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("dark-rest.png: FAILED")
        );
        fs::remove_dir_all(&directory).unwrap();
    }
}
