//! Checks the known pixels of the catalog's screenshots (ADR 0013): `cargo run -p visual-checks -- <directory>`.
//!
//! `nsh test examples/catalog --target <host>` opens the catalog on one of NSH's hosts and saves its screenshots in
//! `target/visual` (`examples/catalog/button.test.tsx`, `switch.test.tsx` and `checkbox.test.tsx`). This reads
//! each of them and checks, for every button and icon button of the catalog, the pixels whose color the tokens
//! give: the container's color inside it, the outline's on its edge, the surface outside its top left corner,
//! where a rounded corner leaves the background, and an icon button's icon color in its middle; the switches' are
//! in [`switch`] and every checkbox's at the places [`checkbox`] gives; and for every divider, the line's color from
//! end to end and across its whole thickness, and the surface one point past each of its ends and sides. Where the
//! antialiasing of an edge, of the label or of the icon's edge makes a color unknown, nothing is checked.
//!
//! The linear progress indicators have screenshots of their own, taken with reduced motion on so that they hold
//! still: for each, the colors of its line, of the gap after it, of its track and of the stop at the end of the track,
//! at the places the geometry gives in the middle of each (see [`progress_probes`]).
//!
//! It is not a `cargo xtask` task: `xtask` must build when the library does not (it regenerates the library's
//! tokens), so it cannot depend on it, and these checks take every expected color from the library's tokens.

mod checkbox;

use std::fmt;
use std::fs;
use std::io::{self, Cursor, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use catalog::checkbox::CheckboxCell;
use catalog::grid::{self, Availability, Cell, DividerCell, IconCell};
use catalog::progress::{ProgressCell, Reading};
use catalog::switch::SwitchCell;
use nsh_core::geometry::Points;
use nsh_m3e::button::{ButtonShape, ButtonSize, ButtonStyle};
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::divider::DividerOrientation;
use nsh_m3e::icon_button::{IconButtonShape, IconButtonSize, IconButtonStyle, IconButtonWidth};
use nsh_m3e::progress::ProgressShape;
use nsh_m3e::tokens::Shape;
use nsh_m3e::tokens::androidx::{
    divider_tokens, filled_button_tokens, filled_icon_button_tokens, filled_tonal_button_tokens,
    filled_tonal_icon_button_tokens, large_icon_button_tokens, linear_progress_indicator_tokens,
    medium_icon_button_tokens, outlined_button_tokens, outlined_icon_button_tokens,
    progress_indicator_tokens, small_icon_button_tokens, standard_icon_button_tokens, state_tokens,
    xlarge_icon_button_tokens, xsmall_icon_button_tokens,
};

mod switch;

const USAGE: &str = "usage: cargo run -p visual-checks -- <directory of screenshots>";

/// How far inside the container's left edge, at mid height, its color is read: past the antialiased edge, and
/// before the label, which starts 12 points or more from the edge (the extra small button's side space, androidx
/// `Button.kt` `ExtraSmallContentPadding`). Not a design value: where the check looks.
const CONTAINER_INSET: Points = Points(4.0);

/// How long a straight edge must be for its outline's color to be read in its middle: as far from either end as the
/// container's color is read from the edge ([`CONTAINER_INSET`] on each side), past the antialiasing of the corners'
/// arcs. Not a design value: where the check looks.
const STRAIGHT_EDGE: Points = Points(8.0);

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

/// A component of the catalog: a button, an icon button, a switch, a checkbox, a divider or a linear progress
/// indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    Button(Cell),
    IconButton(IconCell),
    Switch(SwitchCell),
    Checkbox(CheckboxCell),
    Divider(DividerCell),
    Progress(ProgressCell),
}

impl Target {
    /// The test id of the view around it.
    fn test_id(self) -> String {
        match self {
            Self::Button(cell) => cell.test_id(),
            Self::IconButton(cell) => cell.test_id(),
            Self::Switch(cell) => cell.test_id(),
            Self::Checkbox(cell) => cell.test_id(),
            Self::Divider(cell) => cell.test_id(),
            Self::Progress(cell) => cell.test_id(),
        }
    }
}

/// One screenshot the test scripts take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Shot {
    appearance: Appearance,
    /// The component the mouse is over, if any.
    hovered: Option<Target>,
}

impl Shot {
    /// Every screenshot `examples/catalog/button.test.tsx` takes, in each appearance: the catalog at rest, then the
    /// mouse over the small round enabled button of each style, then over the small uniform round enabled icon
    /// button of each style; then those `examples/catalog/switch.test.tsx` takes, in each appearance: the mouse over
    /// each of [`switch::HOVERED`].
    fn all() -> Vec<Self> {
        Appearance::ALL
            .into_iter()
            .flat_map(|appearance| {
                let shot = move |hovered| Self {
                    appearance,
                    hovered,
                };
                std::iter::once(shot(None))
                    .chain(grid::STYLES.into_iter().map(move |style| {
                        shot(Some(Target::Button(Cell::new(
                            style,
                            ButtonShape::Round,
                            ButtonSize::Small,
                            Availability::Enabled,
                        ))))
                    }))
                    .chain(grid::ICON_STYLES.into_iter().map(move |style| {
                        shot(Some(Target::IconButton(IconCell::new(
                            style,
                            IconButtonWidth::Uniform,
                            IconButtonShape::Round,
                            IconButtonSize::Small,
                            Availability::Enabled,
                        ))))
                    }))
            })
            .chain(Appearance::ALL.into_iter().flat_map(|appearance| {
                switch::HOVERED.into_iter().map(move |cell| Self {
                    appearance,
                    hovered: Some(Target::Switch(cell)),
                })
            }))
            .collect()
    }

    /// Its file name, as the test script saves it.
    fn file_name(self) -> String {
        let appearance = self.appearance.name();
        match self.hovered {
            None => format!("{appearance}-rest.png"),
            Some(Target::Button(cell)) => {
                format!("{appearance}-hover-{}.png", cell.label().to_lowercase())
            }
            Some(Target::IconButton(cell)) => {
                format!(
                    "{appearance}-hover-icon-{}.png",
                    cell.label().to_lowercase()
                )
            }
            // The test id already starts with `switch-`.
            Some(Target::Switch(cell)) => format!("{appearance}-hover-{}.png", cell.test_id()),
            // The buttons' script never hovers a checkbox; the checkbox's own shots are `checkbox::Shot`.
            Some(Target::Checkbox(cell)) => format!("{appearance}-hover-{}.png", cell.test_id()),
            // No screenshot has the mouse over a divider: it has no hover state, and the shots do not hover it.
            Some(Target::Divider(cell)) => {
                format!("{appearance}-hover-{}.png", cell.test_id())
            }
            // No script hovers an indicator: they have shots of their own ([`ProgressShot`]). This is the name one
            // would have.
            Some(Target::Progress(cell)) => format!("{appearance}-hover-{}.png", cell.test_id()),
        }
    }

    /// Whether the mouse is over `target` in it.
    fn hovers(self, target: Target) -> bool {
        self.hovered == Some(target)
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
    target: Target,
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
    let target = Target::Button(cell);
    let mut probes = vec![
        Probe {
            target,
            what: "container",
            x: Points(x.0 + CONTAINER_INSET.0),
            y: middle,
            expected,
            tolerance,
        },
        // The top left pixel of the container's box: outside the arc of every shape's corner (12 points or more), so
        // the surface. A corner drawn square would fill it.
        Probe {
            target,
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
            target,
            what: "outline",
            x: Points(x.0 + grid::height(cell.size).0 / 2.0 + CONTAINER_INSET.0),
            y,
            expected: outline,
            tolerance,
        });
    }
    probes
}

/// The icon's color of an enabled icon button of `style`, which its state layer takes too.
fn icon_content_role(style: IconButtonStyle) -> ColorRole {
    match style {
        IconButtonStyle::Filled => filled_icon_button_tokens::COLOR,
        IconButtonStyle::Tonal => filled_tonal_icon_button_tokens::COLOR,
        IconButtonStyle::Outlined => outlined_icon_button_tokens::COLOR,
        IconButtonStyle::Standard => standard_icon_button_tokens::COLOR,
        // `IconButtonStyle` is non-exhaustive. A style the catalog does not show is expected in the surface color,
        // which no icon is drawn in, so its checks fail until they know it. Not another style's color: the outlined
        // and standard icons are both on surface variant, and falling back to one would hide the other's arm.
        _ => ColorRole::Surface,
    }
}

/// The container's color of the icon button `cell` and its tolerance, over `surface`, with the mouse over it or not.
fn icon_container(cell: IconCell, scheme: &ColorScheme, hovered: bool) -> (Rgb, u8) {
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let disabled = cell.availability == Availability::Disabled;
    let (rest, tolerance) = match (cell.style, disabled) {
        (IconButtonStyle::Filled, false) => (
            Rgb::of(scheme, filled_icon_button_tokens::CONTAINER_COLOR),
            0,
        ),
        (IconButtonStyle::Tonal, false) => (
            Rgb::of(scheme, filled_tonal_icon_button_tokens::CONTAINER_COLOR),
            0,
        ),
        (IconButtonStyle::Filled, true) => faded(
            scheme,
            filled_icon_button_tokens::DISABLED_CONTAINER_COLOR,
            filled_icon_button_tokens::DISABLED_CONTAINER_OPACITY,
            surface,
        ),
        (IconButtonStyle::Tonal, true) => faded(
            scheme,
            filled_tonal_icon_button_tokens::DISABLED_CONTAINER_COLOR,
            filled_tonal_icon_button_tokens::DISABLED_CONTAINER_OPACITY,
            surface,
        ),
        // The outlined and standard icon buttons have no container: the surface shows through.
        _ => (surface, 0),
    };
    if hovered && !disabled {
        faded(
            scheme,
            icon_content_role(cell.style),
            state_tokens::HOVER_STATE_LAYER_OPACITY,
            rest,
        )
    } else {
        (rest, tolerance)
    }
}

/// The icon's color of the icon button `cell` and its tolerance. The icon is drawn over the state layer, so the mouse
/// over it changes nothing; a disabled icon is faded over the disabled container, a blend over a blend, whose two
/// roundings add up.
fn icon_color(cell: IconCell, scheme: &ColorScheme) -> (Rgb, u8) {
    if cell.availability == Availability::Enabled {
        return (Rgb::of(scheme, icon_content_role(cell.style)), 0);
    }
    // Every style's disabled icon is on surface at 0.38, so no style may fall back on another's: one would hide the
    // other's arm. A style the catalog does not show has none here, and is expected as if enabled.
    let disabled = match cell.style {
        IconButtonStyle::Filled => Some((
            filled_icon_button_tokens::DISABLED_COLOR,
            filled_icon_button_tokens::DISABLED_OPACITY,
        )),
        IconButtonStyle::Tonal => Some((
            filled_tonal_icon_button_tokens::DISABLED_COLOR,
            filled_tonal_icon_button_tokens::DISABLED_OPACITY,
        )),
        IconButtonStyle::Outlined => Some((
            outlined_icon_button_tokens::DISABLED_COLOR,
            outlined_icon_button_tokens::DISABLED_OPACITY,
        )),
        IconButtonStyle::Standard => Some((
            standard_icon_button_tokens::DISABLED_COLOR,
            standard_icon_button_tokens::DISABLED_OPACITY,
        )),
        _ => None,
    };
    let Some((disabled_color, opacity)) = disabled else {
        return (Rgb::of(scheme, icon_content_role(cell.style)), 0);
    };
    let (container, container_tolerance) = icon_container(cell, scheme, false);
    let (color, tolerance) = faded(scheme, disabled_color, opacity, container);
    (color, tolerance + container_tolerance)
}

/// The radius of the icon button `cell`'s corners at rest: its size's `ContainerShapeRound` or
/// `ContainerShapeSquare`, at most half its shorter side, as the icon button draws it.
fn icon_corner_radius(cell: IconCell) -> Points {
    use IconButtonShape::{Round, Square};
    use IconButtonSize::{ExtraLarge, ExtraSmall, Large, Medium, Small};
    let (width, height) = cell.frame();
    let half = width.0.min(height.0) / 2.0;
    // Every size's round shape is the full one, and two sizes share each square one, so no size may fall back on
    // another's: one would hide the other's arm. A size or shape the catalog does not show has none here.
    let shape = match (cell.size, cell.shape) {
        (ExtraSmall, Round) => Some(xsmall_icon_button_tokens::CONTAINER_SHAPE_ROUND),
        (ExtraSmall, Square) => Some(xsmall_icon_button_tokens::CONTAINER_SHAPE_SQUARE),
        (Small, Round) => Some(small_icon_button_tokens::CONTAINER_SHAPE_ROUND),
        (Small, Square) => Some(small_icon_button_tokens::CONTAINER_SHAPE_SQUARE),
        (Medium, Round) => Some(medium_icon_button_tokens::CONTAINER_SHAPE_ROUND),
        (Medium, Square) => Some(medium_icon_button_tokens::CONTAINER_SHAPE_SQUARE),
        (Large, Round) => Some(large_icon_button_tokens::CONTAINER_SHAPE_ROUND),
        (Large, Square) => Some(large_icon_button_tokens::CONTAINER_SHAPE_SQUARE),
        (ExtraLarge, Round) => Some(xlarge_icon_button_tokens::CONTAINER_SHAPE_ROUND),
        (ExtraLarge, Square) => Some(xlarge_icon_button_tokens::CONTAINER_SHAPE_SQUARE),
        _ => None,
    };
    match shape {
        Some(Shape::Circle) => Points(half),
        Some(Shape::Rounded(corners)) => Points(corners.top_start.0.min(half)),
        // Square corners: a rectangle, or no shape.
        _ => Points(0.0),
    }
}

/// The pixels the checks know for the icon button `cell` in a screenshot in `scheme`, with the mouse over it or not.
fn icon_probes(cell: IconCell, scheme: &ColorScheme, hovered: bool) -> Vec<Probe> {
    let target = Target::IconButton(cell);
    let (x, y) = cell.origin();
    let (width, height) = cell.frame();
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let (expected, tolerance) = icon_container(cell, scheme, hovered);
    let (icon, icon_tolerance) = icon_color(cell, scheme);
    let middle = (Points(x.0 + width.0 / 2.0), Points(y.0 + height.0 / 2.0));
    let mut probes = vec![
        // Halfway between the left edge and the icon's box, at mid height: past the antialiased edge and the
        // outline (2 points or more from the edge, the extra small narrow icon button's half side space), and before
        // the icon.
        Probe {
            target,
            what: "container",
            x: Points(x.0 + (width.0 - grid::icon_size(cell.size).0) / 4.0),
            y: middle.1,
            expected,
            tolerance,
        },
        // The top left pixel of the container's box: outside the arc of every shape's corner (12 points or more).
        Probe {
            target,
            what: "outside the corner",
            x,
            y,
            expected: surface,
            tolerance: 0,
        },
        // The middle of the icon, `catalog::ICON`, which is filled there.
        Probe {
            target,
            what: "icon",
            x: middle.0,
            y: middle.1,
            expected: icon,
            tolerance: icon_tolerance,
        },
    ];
    if cell.style == IconButtonStyle::Outlined {
        // The outline's first pixel row or column, in the middle of a straight edge at least 8 points long, as for the
        // buttons: the top edge if it has one, else the left. A round uniform icon button is a circle, and has none.
        // The outline is 1 point wide at every size (`SmallIconButtonTokens.OutlinedOutlineWidth`, ADR 0012),
        // faded by the icon's disabled opacity when disabled (androidx `outlinedIconButtonVibrantBorder`), and the
        // state layer lies inside it.
        let (outline, tolerance) = if cell.availability == Availability::Disabled {
            faded(
                scheme,
                outlined_icon_button_tokens::OUTLINE_COLOR,
                outlined_icon_button_tokens::DISABLED_OPACITY,
                surface,
            )
        } else {
            (
                Rgb::of(scheme, outlined_icon_button_tokens::OUTLINE_COLOR),
                0,
            )
        };
        let straight = |side: Points| side.0 - 2.0 * icon_corner_radius(cell).0 >= STRAIGHT_EDGE.0;
        let place = if straight(width) {
            Some((middle.0, y))
        } else if straight(height) {
            Some((x, middle.1))
        } else {
            None
        };
        if let Some((x, y)) = place {
            probes.push(Probe {
                target,
                what: "outline",
                x,
                y,
                expected: outline,
                tolerance,
            });
        }
    }
    probes
}

/// The pixels the checks know for the divider `cell` in a screenshot in `scheme`: the line in the token's color along
/// its whole length and across its whole thickness, and the surface one point past each end and each side of it.
///
/// A divider is one fill of a box whose edges are on whole points, so at a whole scale no pixel of it is antialiased,
/// and every probe is exact. It has no hover state, so the mouse over it or not changes nothing.
fn divider_probes(cell: DividerCell, scheme: &ColorScheme) -> Vec<Probe> {
    let target = Target::Divider(cell);
    let (x, y) = cell.origin();
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let line = Rgb::of(scheme, divider_tokens::COLOR);
    let (length, thickness) = (grid::DIVIDER_SPACE.0, cell.thickness.0);
    // The middle of the line's length, and the pixel row or column nearest the middle of its thickness: a whole number
    // of points from its start and its first side, both of which are whole numbers.
    let middle = (length / 2.0).floor();
    let center = (thickness / 2.0).floor();
    // What each probe reads: where along the line and across it, in points from its start and from its first side.
    let places = [
        ("line's middle", middle, center, line),
        ("line's first side", middle, 0.0, line),
        ("line's last side", middle, thickness - 1.0, line),
        ("surface before the line", middle, -1.0, surface),
        ("surface after the line", middle, thickness, surface),
        ("line's start", 0.0, center, line),
        ("line's end", length - 1.0, center, line),
        ("surface before the line's start", -1.0, center, surface),
        ("surface past the line's end", length, center, surface),
    ];
    places
        .into_iter()
        .map(|(what, along, across, expected)| {
            // A horizontal line runs along x and is thick along y; a vertical one is the other way round.
            let (dx, dy) = match cell.orientation {
                DividerOrientation::Vertical => (across, along),
                _ => (along, across),
            };
            Probe {
                target,
                what,
                x: Points(x.0 + dx),
                y: Points(y.0 + dy),
                expected,
                tolerance: 0,
            }
        })
        .collect()
}

/// The pixels the checks know in `shot`, for every button, icon button, switch and divider of the catalog.
fn shot_probes(shot: Shot) -> Vec<Probe> {
    let scheme = shot.appearance.scheme();
    Cell::all()
        .flat_map(|cell| probes(cell, &scheme, shot.hovers(Target::Button(cell))))
        .chain(
            IconCell::all()
                .flat_map(|cell| icon_probes(cell, &scheme, shot.hovers(Target::IconButton(cell)))),
        )
        .chain(
            SwitchCell::all()
                .flat_map(|cell| switch::probes(cell, &scheme, shot.hovers(Target::Switch(cell)))),
        )
        .chain(DividerCell::all().flat_map(|cell| divider_probes(cell, &scheme)))
        .collect()
}

/// A screenshot of the linear progress indicators, which the test script takes with reduced motion on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProgressShot {
    appearance: Appearance,
}

impl ProgressShot {
    /// Every screenshot of the indicators `examples/catalog/button.test.tsx` takes: one in each appearance.
    fn all() -> Vec<Self> {
        Appearance::ALL
            .into_iter()
            .map(|appearance| Self { appearance })
            .collect()
    }

    /// Its file name, as the test script saves it.
    fn file_name(self) -> String {
        format!("progress-{}.png", self.appearance.name())
    }
}

/// How far from each end of a line or of the track a pixel is read: past the antialiasing of a round end, which is
/// 2 points deep (the stroke is 4 points thick), and then some. Not a design value: where the check looks.
const END_INSET: f32 = 4.0;

/// The thresholds of a wavy indicator's wave, in per cent: it has no height at or under the first and at or over the
/// second, and its full height between them (androidx `WavyProgressIndicator.kt`, lines 493 to 500).
const WAVE_AFTER_PERCENT: u8 = 10;
const WAVE_BEFORE_PERCENT: u8 = 95;

/// Where, as a fraction of the width, an indeterminate indicator that holds still has the middle of its first line and
/// the middle of the track between its lines. The pose is the one at 1000 ms of the cycle (`STILL_POSE_AT_MS`): the
/// first line goes from 0.41 of the width to the end, the second from the start to a tenth of it, and the track
/// between them from 0.17 to 0.34 (the second line's end, the first line's start, and a gap of 8 points on a track
/// 120 wide on each side). Not design values: where the check looks, inside each. The first line's is not a multiple of
/// the wave's wavelength, so that the half wavelength it is rounded up to is a number the checks test.
const STILL_LINE_AT: f32 = 0.55;
const STILL_TRACK_AT: f32 = 0.25;

/// Where to read the color of a line or a track that goes from `from` to `to` points: its middle, or, for a wave, the
/// first place at least [`END_INSET`] from its start where the wave crosses its middle line (every `half` points: the
/// wave is flat there, so the line is under the pixel whatever the amplitude), and none where there is no such place
/// at least [`END_INSET`] from its end.
fn read_at(from: f32, to: f32, crossing: Option<f32>) -> Option<f32> {
    let first = from + END_INSET;
    let last = to - END_INSET;
    let at = match crossing {
        Some(half) => (first / half).ceil() * half,
        None => f32::midpoint(from, to).floor(),
    };
    (first <= at && at <= last).then_some(at)
}

/// The pixels the checks know in the indicator `cell`, in a screenshot in `scheme` with reduced motion on: the wave of
/// a wavy indicator is at its start, so it crosses its middle line every half wavelength from the left, and an
/// indeterminate indicator holds the pose of 1000 ms. All of them are solid colors, read on the row just under the
/// middle line, inside a 4-point stroke.
///
/// A determinate indicator has its line from the start to the progress, a gap of 8 points between the centers of the
/// line's and the track's round ends (the track's 4 points and the stroke's 4), its track to the end, and the stop,
/// in the line's color, at the end of the track. Read: inside the line (or the track's color at its start where the
/// line has no length, which draws nothing, not a dot), the surface in the middle of the gap, inside the track where it
/// is long enough, and the stop's center (the line's own end where it has reached it).
fn progress_probes(cell: ProgressCell, scheme: &ColorScheme) -> Vec<Probe> {
    let (x, y) = cell.origin();
    let (width, height) = cell.frame();
    let target = Target::Progress(cell);
    let row = Points(y.0 + (height.0 / 2.0).floor());
    let probe = |what: &'static str, at: f32, expected: Rgb| Probe {
        target,
        what,
        x: Points(x.0 + at),
        y: row,
        expected,
        tolerance: 0,
    };
    let line = Rgb::of(scheme, progress_indicator_tokens::ACTIVE_INDICATOR_COLOR);
    let track = Rgb::of(scheme, progress_indicator_tokens::TRACK_COLOR);
    let stop = Rgb::of(scheme, progress_indicator_tokens::STOP_COLOR);
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let tokens = (
        linear_progress_indicator_tokens::ACTIVE_THICKNESS.0,
        linear_progress_indicator_tokens::TRACK_ACTIVE_SPACE.0,
        linear_progress_indicator_tokens::ACTIVE_WAVE_WAVELENGTH.0,
        linear_progress_indicator_tokens::INDETERMINATE_ACTIVE_WAVE_WAVELENGTH.0,
    );
    let (thickness, space, wavelength, indeterminate_wavelength) = tokens;
    let wavy = cell.shape == ProgressShape::Wavy;
    // The line's round end reaches half the stroke past its end, so the ends are held that far inside the indicator.
    let cap = thickness / 2.0;
    let end = width.0 - cap;
    let Reading::Determinate(percent) = cell.reading else {
        // The pose of an indeterminate indicator that holds still: the first line, which a wave crosses the middle line
        // along, and the track between the lines. There is no stop.
        let crossing = wavy.then_some(indeterminate_wavelength / 2.0);
        let line_at = width.0 * STILL_LINE_AT;
        let line_at =
            crossing.map_or_else(|| line_at.floor(), |half| (line_at / half).ceil() * half);
        return vec![
            probe("first line", line_at, line),
            probe(
                "track between the lines",
                (width.0 * STILL_TRACK_AT).floor(),
                track,
            ),
        ];
    };
    let mut found = Vec::new();
    let done = width.0 * f32::from(percent) / 100.0;
    let head = done.max(cap).min(end);
    // The track starts a gap past the line's end: the stroke's thickness and the space, but no more than the line is
    // long (androidx's `gapSizeFraction` and `min(barHead - cap, gap)`).
    let spacing = space + thickness;
    let track_start = done + done.min(spacing);
    let track_from = track_start.max(cap).min(end);
    let waves = percent > WAVE_AFTER_PERCENT && percent < WAVE_BEFORE_PERCENT;
    if percent == 0 {
        found.push(probe("no line", cap, track));
    } else {
        let crossing = (wavy && waves).then_some(wavelength / 2.0);
        if let Some(at) = read_at(cap, head, crossing) {
            found.push(probe("line", at, line));
        }
        // The gap runs from the line's cap to the track's cap, or to the stop's edge where the track has no length; it
        // has no width where those meet, as at 95 % of the catalog's indicators.
        let (gap_from, gap_to) = (head + cap, track_start.min(end) - cap);
        if gap_from < gap_to {
            found.push(probe("gap", f32::midpoint(gap_from, gap_to), surface));
        }
    }
    // Where the track has no length (it starts at the stop), `read_at` finds no place inside it.
    if let Some(at) = read_at(track_from, end, None) {
        found.push(probe("track", at, track));
    }
    found.push(probe("stop", end, stop));
    found
}

/// The pixels the checks know in `shot`, for every indicator of the catalog.
fn progress_shot_probes(shot: ProgressShot) -> Vec<Probe> {
    let scheme = shot.appearance.scheme();
    ProgressCell::all()
        .flat_map(|cell| progress_probes(cell, &scheme))
        .collect()
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
        self.scale_in(grid::viewport())
    }

    /// How many pixels a point is, when the image is a viewport of `width` and `height` points at a whole scale.
    fn scale_in(&self, (width, height): (Points, Points)) -> Result<u32, String> {
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
            probe.target.test_id(),
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
    check_probes(image, shot_probes(shot))
}

/// Every probe of `shot` that `image` does not match.
fn check_progress(image: &Image, shot: ProgressShot) -> Result<Vec<Mismatch>, String> {
    check_probes(image, progress_shot_probes(shot))
}

/// Every one of `probes` that `image` does not match.
fn check_probes(image: &Image, probes: Vec<Probe>) -> Result<Vec<Mismatch>, String> {
    let scale = image.scale()?;
    Ok(probes
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

/// Reads `directory`'s screenshot `name` and checks it with `check`, writing what it finds to `out`; how many pixels
/// were not what was expected.
fn check_file(
    directory: &Path,
    name: &str,
    check: impl FnOnce(&Image) -> Result<Vec<Mismatch>, String>,
    out: &mut impl io::Write,
) -> Result<usize, Error> {
    let path = directory.join(name);
    let bytes = fs::read(&path).map_err(|source| Error::Read {
        path: path.clone(),
        source,
    })?;
    let image = Image::decode(bytes).map_err(|message| Error::Image {
        path: path.clone(),
        message,
    })?;
    let found = check(&image).map_err(|message| Error::Image {
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
    Ok(found.len())
}

/// Success where no pixel was wrong, and the error that counts them where some were.
fn finish(mismatches: usize) -> Result<(), Error> {
    if mismatches == 0 {
        Ok(())
    } else {
        Err(Error::Mismatches(mismatches))
    }
}

fn run(directory: &Path, out: &mut impl io::Write) -> Result<(), Error> {
    let mut mismatches = 0;
    for shot in Shot::all() {
        mismatches += check_file(
            directory,
            &shot.file_name(),
            |image| check(image, shot),
            out,
        )?;
    }
    for shot in checkbox::Shot::all() {
        mismatches += check_file(
            directory,
            &shot.file_name(),
            |image| checkbox::check(image, shot),
            out,
        )?;
    }
    finish(mismatches)
}

/// Checks the screenshots of the linear progress indicators in `directory`.
fn run_progress(directory: &Path, out: &mut impl io::Write) -> Result<(), Error> {
    let mut mismatches = 0;
    for shot in ProgressShot::all() {
        mismatches += check_file(
            directory,
            &shot.file_name(),
            |image| check_progress(image, shot),
            out,
        )?;
    }
    finish(mismatches)
}

/// Both kinds of checks have passed or neither: the first failure, if there is one.
fn both(buttons: Result<(), Error>, progress: Result<(), Error>) -> Result<(), Error> {
    buttons.and(progress)
}

/// Runs the checks on the one directory `args` names, writing what they find to `out`.
fn run_with(args: &[String], out: &mut impl io::Write) -> Result<(), Error> {
    match args {
        [directory] => {
            let directory = Path::new(directory);
            // Both are run, so that a failure of one does not hide the other's.
            let buttons = run(directory, out);
            let progress = run_progress(directory, out);
            both(buttons, progress)
        }
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
    use std::ops::Deref;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;

    /// A directory of the system's temporary one, created empty, that goes away when this is dropped, a failed
    /// assertion included: the screenshots a test writes in it weigh about 870 MB, and `cargo mutants` runs a test that
    /// fails once for each mutant it catches.
    struct ScratchDir(PathBuf);

    impl ScratchDir {
        /// An empty directory named for `name` and for this process, so that two tests do not share one.
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
            // One a run that was killed left behind is not this test's.
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Deref for ScratchDir {
        type Target = Path;

        fn deref(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for ScratchDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;

    fn cell(style: ButtonStyle, availability: Availability) -> Cell {
        Cell::new(style, ButtonShape::Round, ButtonSize::Small, availability)
    }

    fn icon_cell(style: IconButtonStyle, availability: Availability) -> IconCell {
        IconCell::new(
            style,
            IconButtonWidth::Uniform,
            IconButtonShape::Round,
            IconButtonSize::Small,
            availability,
        )
    }

    /// The catalog painted as the tokens say, at `scale`: every probe of `shot` holds its expected color, and the
    /// rest a color no probe expects.
    fn painted(shot: Shot, scale: u32) -> Image {
        let (width, height) = grid::viewport();
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

    /// `image` as an RGBA PNG without compression. Compressed by the unoptimized test build, the catalog's 18
    /// screenshots made `run_reports_missing_and_wrong_screenshots` take 36 seconds in the development container, and
    /// 9 uncompressed (one run each).
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
    fn icon_button_containers_are_the_style_tokens_colors() {
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let primary = Rgb::of(&LIGHT, ColorRole::Primary);
        let filled = icon_cell(IconButtonStyle::Filled, Availability::Enabled);
        assert_eq!(icon_container(filled, &LIGHT, false), (primary, 0));
        assert_eq!(
            icon_container(
                icon_cell(IconButtonStyle::Tonal, Availability::Enabled),
                &LIGHT,
                false
            ),
            (Rgb::of(&LIGHT, ColorRole::SecondaryContainer), 0)
        );
        for style in [IconButtonStyle::Outlined, IconButtonStyle::Standard] {
            for availability in grid::AVAILABILITIES {
                assert_eq!(
                    icon_container(icon_cell(style, availability), &LIGHT, false),
                    (surface, 0)
                );
            }
        }
        // Disabled: on surface at 0.1 over the surface, for both.
        let on_surface = Rgb::of(&LIGHT, ColorRole::OnSurface);
        for style in [IconButtonStyle::Filled, IconButtonStyle::Tonal] {
            assert_eq!(
                icon_container(icon_cell(style, Availability::Disabled), &LIGHT, false),
                (on_surface.over(0.1, surface), BLEND_TOLERANCE)
            );
        }
        // Hovered: the icon's color at 0.08 over the container; a disabled icon button shows no hover.
        let on_primary = Rgb::of(&LIGHT, ColorRole::OnPrimary);
        assert_eq!(
            icon_container(filled, &LIGHT, true),
            (on_primary.over(0.08, primary), BLEND_TOLERANCE)
        );
        let on_surface_variant = Rgb::of(&LIGHT, ColorRole::OnSurfaceVariant);
        assert_eq!(
            icon_container(
                icon_cell(IconButtonStyle::Standard, Availability::Enabled),
                &LIGHT,
                true
            ),
            (on_surface_variant.over(0.08, surface), BLEND_TOLERANCE)
        );
        let disabled = icon_cell(IconButtonStyle::Filled, Availability::Disabled);
        assert_eq!(
            icon_container(disabled, &LIGHT, true),
            icon_container(disabled, &LIGHT, false)
        );
    }

    #[test]
    fn the_icon_takes_each_style_color_and_fades_over_the_container_when_disabled() {
        let roles: Vec<ColorRole> = grid::ICON_STYLES
            .into_iter()
            .map(icon_content_role)
            .collect();
        assert_eq!(
            roles,
            [
                ColorRole::OnPrimary,
                ColorRole::OnSecondaryContainer,
                ColorRole::OnSurfaceVariant,
                ColorRole::OnSurfaceVariant,
            ]
        );
        for style in grid::ICON_STYLES {
            assert_eq!(
                icon_color(icon_cell(style, Availability::Enabled), &LIGHT),
                (Rgb::of(&LIGHT, icon_content_role(style)), 0)
            );
        }
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let on_surface = Rgb::of(&LIGHT, ColorRole::OnSurface);
        // Over the surface: one blend.
        for style in [IconButtonStyle::Outlined, IconButtonStyle::Standard] {
            assert_eq!(
                icon_color(icon_cell(style, Availability::Disabled), &LIGHT),
                (on_surface.over(0.38, surface), BLEND_TOLERANCE)
            );
        }
        // Over the disabled container: a blend over a blend.
        let container = on_surface.over(0.1, surface);
        for style in [IconButtonStyle::Filled, IconButtonStyle::Tonal] {
            assert_eq!(
                icon_color(icon_cell(style, Availability::Disabled), &LIGHT),
                (on_surface.over(0.38, container), 2 * BLEND_TOLERANCE)
            );
        }
    }

    #[test]
    fn corners_are_the_size_tokens_radius_at_most_half_the_shorter_side() {
        use IconButtonShape::{Round, Square};
        use IconButtonWidth::{Narrow, Uniform, Wide};
        let radius = |width, shape, size| {
            icon_corner_radius(IconCell::new(
                IconButtonStyle::Outlined,
                width,
                shape,
                size,
                Availability::Enabled,
            ))
            .0
        };
        let squares: Vec<f32> = grid::ICON_SIZES
            .into_iter()
            .map(|size| radius(Wide, Square, size))
            .collect();
        assert_eq!(squares, [12.0, 12.0, 16.0, 28.0, 28.0]);
        // Round: half the shorter side, the height when wide and the width when narrow.
        let rounds: Vec<f32> = grid::ICON_SIZES
            .into_iter()
            .map(|size| radius(Wide, Round, size))
            .collect();
        assert_eq!(rounds, [16.0, 20.0, 28.0, 48.0, 68.0]);
        assert_eq!(radius(Narrow, Round, IconButtonSize::ExtraSmall), 14.0);
        assert_eq!(radius(Uniform, Round, IconButtonSize::Large), 48.0);
    }

    #[test]
    fn icon_probes_read_the_container_the_corner_the_icon_and_a_straight_outline() {
        use IconButtonShape::{Round, Square};
        use IconButtonWidth::{Narrow, Uniform, Wide};
        let filled = icon_cell(IconButtonStyle::Filled, Availability::Enabled);
        let (x, y) = filled.origin();
        let found = icon_probes(filled, &LIGHT, false);
        assert_eq!(found.len(), 3);
        // A 24-point icon in 40 points: 8 points of side space, halved.
        assert_eq!(
            (found[0].x, found[0].y),
            (Points(x.0 + 4.0), Points(y.0 + 20.0))
        );
        assert_eq!(found[0].what, "container");
        assert_eq!((found[1].x, found[1].y), (x, y));
        assert_eq!(found[1].expected, Rgb::of(&LIGHT, ColorRole::Surface));
        assert_eq!(
            (found[2].x, found[2].y),
            (Points(x.0 + 20.0), Points(y.0 + 20.0))
        );
        assert_eq!(
            (found[2].expected, found[2].tolerance),
            (Rgb::of(&LIGHT, ColorRole::OnPrimary), 0)
        );
        assert_eq!(found[2].target, Target::IconButton(filled));

        let outlined = |width, shape, size, availability| {
            let cell = IconCell::new(IconButtonStyle::Outlined, width, shape, size, availability);
            (cell, icon_probes(cell, &LIGHT, false))
        };
        // Wide: the middle of the top edge. 40 by 32 points, round: 8 points straight.
        let (cell, found) = outlined(
            Wide,
            Round,
            IconButtonSize::ExtraSmall,
            Availability::Enabled,
        );
        let (x, y) = cell.origin();
        assert_eq!(found.len(), 4);
        assert_eq!((found[3].x, found[3].y), (Points(x.0 + 20.0), y));
        assert_eq!(
            (found[3].expected, found[3].tolerance),
            (Rgb::of(&LIGHT, ColorRole::OutlineVariant), 0)
        );
        // Narrow and square, 28 by 32 points with 12-point corners: 4 points straight on top, 8 on the left.
        let (cell, found) = outlined(
            Narrow,
            Square,
            IconButtonSize::ExtraSmall,
            Availability::Disabled,
        );
        let (x, y) = cell.origin();
        assert_eq!((found[3].x, found[3].y), (x, Points(y.0 + 16.0)));
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        assert_eq!(
            (found[3].expected, found[3].tolerance),
            (
                Rgb::of(&LIGHT, ColorRole::OutlineVariant).over(0.38, surface),
                BLEND_TOLERANCE
            )
        );
        // The container is read halfway into the 4-point side space.
        assert_eq!(found[0].x, Points(x.0 + 2.0));
        // Narrow and round, 28 by 32: 4 points straight on the left, none on top.
        assert_eq!(
            outlined(
                Narrow,
                Round,
                IconButtonSize::ExtraSmall,
                Availability::Enabled
            )
            .1
            .len(),
            3
        );
        // Narrow and round, 32 by 40: 8 points straight on the left.
        assert_eq!(
            outlined(Narrow, Round, IconButtonSize::Small, Availability::Enabled)
                .1
                .len(),
            4
        );
        // Uniform and round: a circle, no straight edge at any size.
        for size in grid::ICON_SIZES {
            assert_eq!(
                outlined(Uniform, Round, size, Availability::Enabled)
                    .1
                    .len(),
                3,
                "{size:?}"
            );
        }
        // Uniform and square, 32 by 32 with 12-point corners: 8 points straight on top.
        let (cell, found) = outlined(
            Uniform,
            Square,
            IconButtonSize::ExtraSmall,
            Availability::Enabled,
        );
        let (x, y) = cell.origin();
        assert_eq!((found[3].x, found[3].y), (Points(x.0 + 16.0), y));
        // Hovering changes the container, not the corner, the icon or the outline.
        let (cell, rest) = outlined(Wide, Round, IconButtonSize::Small, Availability::Enabled);
        let hovered = icon_probes(cell, &LIGHT, true);
        assert_ne!(hovered[0], rest[0]);
        assert_eq!(hovered[1..], rest[1..]);
    }

    #[test]
    fn every_shot_reads_every_component_and_the_hovered_one_hovered() {
        let shot = Shot::all()[5];
        let found = shot_probes(shot);
        let rest = shot_probes(Shot::all()[0]);
        assert_eq!(found.len(), rest.len());
        let changed: Vec<&Probe> = found
            .iter()
            .zip(&rest)
            .filter(|(a, b)| a != b)
            .map(|(a, _)| a)
            .collect();
        // Only the hovered icon button's container.
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].target, shot.hovered.unwrap());
        assert_eq!(changed[0].what, "container");
        let count = |of: fn(&Target) -> bool| rest.iter().filter(|p| of(&p.target)).count();
        let buttons = count(|target| matches!(target, Target::Button(_)));
        let icon_buttons = count(|target| matches!(target, Target::IconButton(_)));
        let switches = count(|target| matches!(target, Target::Switch(_)));
        let dividers = count(|target| matches!(target, Target::Divider(_)));
        assert_eq!(rest.len(), buttons + icon_buttons + switches + dividers);
        assert_eq!(
            buttons,
            Cell::all()
                .map(|c| probes(c, &LIGHT, false).len())
                .sum::<usize>()
        );
        assert_eq!(
            dividers,
            DividerCell::all()
                .map(|c| divider_probes(c, &LIGHT).len())
                .sum::<usize>()
        );
        assert_eq!(
            icon_buttons,
            IconCell::all()
                .map(|c| icon_probes(c, &LIGHT, false).len())
                .sum::<usize>()
        );
        assert!(icon_buttons >= IconCell::all().count() * 3);
    }

    /// The divider `orientation`, `thickness` points thick, of the catalog.
    fn divider_cell(orientation: DividerOrientation, thickness: f32) -> DividerCell {
        DividerCell::new(orientation, Points(thickness))
    }

    /// What `divider_probes` reads, as (what, x, y, whether the line is expected there), in points.
    fn read(cell: DividerCell, scheme: &ColorScheme) -> Vec<(&'static str, f32, f32, bool)> {
        let line = Rgb::of(scheme, ColorRole::OutlineVariant);
        divider_probes(cell, scheme)
            .into_iter()
            .map(|probe| (probe.what, probe.x.0, probe.y.0, probe.expected == line))
            .collect()
    }

    #[test]
    fn divider_probes_read_the_line_from_end_to_end_and_the_surface_past_it() {
        use DividerOrientation::{Horizontal, Vertical};
        // The one-point horizontal line at (2440, 904), 96 points long: its one row is its middle, first and last.
        assert_eq!(
            read(divider_cell(Horizontal, 1.0), &LIGHT),
            [
                ("line's middle", 2488.0, 904.0, true),
                ("line's first side", 2488.0, 904.0, true),
                ("line's last side", 2488.0, 904.0, true),
                ("surface before the line", 2488.0, 903.0, false),
                ("surface after the line", 2488.0, 905.0, false),
                ("line's start", 2440.0, 904.0, true),
                ("line's end", 2535.0, 904.0, true),
                ("surface before the line's start", 2439.0, 904.0, false),
                ("surface past the line's end", 2536.0, 904.0, false),
            ]
        );
        // The two-point one at (2440, 1016): its middle is the second row.
        assert_eq!(
            read(divider_cell(Horizontal, 2.0), &LIGHT),
            [
                ("line's middle", 2488.0, 1017.0, true),
                ("line's first side", 2488.0, 1016.0, true),
                ("line's last side", 2488.0, 1017.0, true),
                ("surface before the line", 2488.0, 1015.0, false),
                ("surface after the line", 2488.0, 1018.0, false),
                ("line's start", 2440.0, 1017.0, true),
                ("line's end", 2535.0, 1017.0, true),
                ("surface before the line's start", 2439.0, 1017.0, false),
                ("surface past the line's end", 2536.0, 1017.0, false),
            ]
        );
        // The four-point vertical one at (2440, 1464): the same, turned. Its middle is the third column.
        assert_eq!(
            read(divider_cell(Vertical, 4.0), &LIGHT),
            [
                ("line's middle", 2442.0, 1512.0, true),
                ("line's first side", 2440.0, 1512.0, true),
                ("line's last side", 2443.0, 1512.0, true),
                ("surface before the line", 2439.0, 1512.0, false),
                ("surface after the line", 2444.0, 1512.0, false),
                ("line's start", 2442.0, 1464.0, true),
                ("line's end", 2442.0, 1559.0, true),
                ("surface before the line's start", 2442.0, 1463.0, false),
                ("surface past the line's end", 2442.0, 1560.0, false),
            ]
        );
    }

    #[test]
    fn a_probe_expects_the_line_exactly_where_the_line_is() {
        for cell in DividerCell::all() {
            let (x, y) = cell.origin();
            let (width, height) = cell.frame();
            let found = divider_probes(cell, &LIGHT);
            assert_eq!(found.len(), 9, "{cell:?}");
            let mut lines = 0;
            for probe in &found {
                // The line is the box of its frame at its origin; a probe is inside it or one point outside.
                let inside = (x.0..x.0 + width.0).contains(&probe.x.0)
                    && (y.0..y.0 + height.0).contains(&probe.y.0);
                let expected = if inside {
                    lines += 1;
                    Rgb::of(&LIGHT, ColorRole::OutlineVariant)
                } else {
                    Rgb::of(&LIGHT, ColorRole::Surface)
                };
                assert_eq!(probe.expected, expected, "{cell:?} {}", probe.what);
                assert_eq!(probe.tolerance, 0, "{cell:?} {}", probe.what);
                assert_eq!(probe.target, Target::Divider(cell));
                // Whole points, so that a probe reads one pixel of a whole scale and no edge between two.
                assert!(probe.x.0.fract() == 0.0 && probe.x.0 > 0.0, "{probe:?}");
                assert!(probe.y.0.fract() == 0.0 && probe.y.0 > 0.0, "{probe:?}");
            }
            assert_eq!(lines, 5, "{cell:?}");
            // Each side of the line and each end is read, one point outside.
            let reads =
                |at: fn(&Probe) -> f32, edge: f32| found.iter().any(|p| (at(p) - edge).abs() < 0.5);
            assert!(reads(|p| p.x.0, x.0 - 1.0), "{cell:?}");
            assert!(reads(|p| p.x.0, x.0 + width.0), "{cell:?}");
            assert!(reads(|p| p.y.0, y.0 - 1.0), "{cell:?}");
            assert!(reads(|p| p.y.0, y.0 + height.0), "{cell:?}");
            // And the line's last point on each of them.
            assert!(reads(|p| p.x.0, x.0 + width.0 - 1.0), "{cell:?}");
            assert!(reads(|p| p.y.0, y.0 + height.0 - 1.0), "{cell:?}");
        }
    }

    #[test]
    fn the_line_is_the_outline_variant_of_each_appearance_and_no_mouse_changes_it() {
        let dark = ColorScheme::BASELINE_DARK;
        let cell = DividerCell::default();
        let light_line = divider_probes(cell, &LIGHT)[0].expected;
        let dark_line = divider_probes(cell, &dark)[0].expected;
        assert_eq!(light_line, Rgb::of(&LIGHT, ColorRole::OutlineVariant));
        assert_eq!(dark_line, Rgb::of(&dark, ColorRole::OutlineVariant));
        assert_ne!(light_line, dark_line);
        assert_eq!(
            divider_probes(cell, &dark)[3].expected,
            Rgb::of(&dark, ColorRole::Surface)
        );
        // A divider has no hover state: whatever the mouse is over, its probes are the same.
        let of_dividers = |shot: Shot| -> Vec<Probe> {
            shot_probes(shot)
                .into_iter()
                .filter(|probe| matches!(probe.target, Target::Divider(_)))
                .collect()
        };
        let shots = Shot::all();
        let rest = of_dividers(shots[0]);
        assert_eq!(rest.len(), 6 * 9);
        for shot in &shots[1..9] {
            assert_eq!(of_dividers(*shot), rest, "{shot:?}");
        }
        assert_ne!(of_dividers(shots[9]), rest);
        // No screenshot has the mouse over a divider.
        assert!(
            shots
                .iter()
                .all(|shot| !matches!(shot.hovered, Some(Target::Divider(_))))
        );
    }

    #[test]
    fn a_line_one_point_short_or_long_is_found_and_named() {
        let shot = Shot::all()[0];
        let cell = divider_cell(DividerOrientation::Horizontal, 1.0);
        let probes = divider_probes(cell, &LIGHT);
        let [end, past] = [probes[6], probes[8]];
        assert_eq!(
            (end.what, past.what),
            ("line's end", "surface past the line's end")
        );
        for (probe, wrong) in [
            (end, Rgb::of(&LIGHT, ColorRole::Surface)),
            (past, Rgb::of(&LIGHT, ColorRole::OutlineVariant)),
        ] {
            let mut image = painted(shot, 1);
            let (x, y) = pixel_of(&probe, 1);
            let start = ((y * image.width + x) * 4) as usize;
            image.rgba[start..start + 3].copy_from_slice(&wrong.0);
            let found = check(&image, shot).unwrap();
            assert_eq!(found.len(), 1);
            assert_eq!(found[0].probe, probe);
            assert_eq!(found[0].found, Some(wrong));
            assert_eq!(
                found[0].to_string(),
                format!(
                    "divider-horizontal-1 {} at ({}, {}) points: expected {} (± 0), found {wrong}",
                    probe.what, probe.x.0, probe.y.0, probe.expected
                )
            );
        }
    }

    #[test]
    fn a_target_is_named_by_its_test_id() {
        let button = cell(ButtonStyle::Text, Availability::Disabled);
        assert_eq!(Target::Button(button).test_id(), button.test_id());
        let icon_button = icon_cell(IconButtonStyle::Tonal, Availability::Enabled);
        assert_eq!(
            Target::IconButton(icon_button).test_id(),
            "icon-tonal-small-uniform-round-enabled"
        );
        assert_eq!(
            Target::Divider(DividerCell::default()).test_id(),
            "divider-horizontal-1"
        );
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
                "light-hover-icon-filled.png",
                "light-hover-icon-tonal.png",
                "light-hover-icon-outlined.png",
                "light-hover-icon-standard.png",
                "dark-rest.png",
                "dark-hover-filled.png",
                "dark-hover-tonal.png",
                "dark-hover-outlined.png",
                "dark-hover-text.png",
                "dark-hover-icon-filled.png",
                "dark-hover-icon-tonal.png",
                "dark-hover-icon-outlined.png",
                "dark-hover-icon-standard.png",
                "light-hover-switch-off-plain-enabled.png",
                "light-hover-switch-on-icon-enabled.png",
                "dark-hover-switch-off-plain-enabled.png",
                "dark-hover-switch-on-icon-enabled.png",
            ]
        );
        assert_eq!(Shot::all()[0].appearance.scheme(), LIGHT);
        assert_eq!(
            Shot::all()[9].appearance.scheme(),
            ColorScheme::BASELINE_DARK
        );
        // The icon buttons hovered are the small uniform round enabled ones.
        assert_eq!(
            Shot::all()[5].hovered,
            Some(Target::IconButton(icon_cell(
                IconButtonStyle::Filled,
                Availability::Enabled
            )))
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
        let Some(Target::Button(hovered)) = shot.hovered else {
            panic!("the second shot hovers the filled button");
        };
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
        assert_ne!(check(&image, Shot::all()[10]).unwrap(), Vec::new());
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
        let directory = ScratchDir::new("visual-checks");
        let mut out = Vec::new();
        assert!(matches!(run(&directory, &mut out), Err(Error::Read { .. })));
        for shot in Shot::all() {
            let image = painted(shot, 1);
            fs::write(
                directory.join(shot.file_name()),
                encode_uncompressed(&image),
            )
            .unwrap();
        }
        // Without the checkbox's screenshots, the run reads the first one it lacks.
        let mut out = Vec::new();
        assert!(matches!(run(&directory, &mut out), Err(Error::Read { .. })));
        for shot in checkbox::Shot::all() {
            let image = checkbox::painted(shot, 1);
            fs::write(
                directory.join(shot.file_name()),
                encode_uncompressed(&image),
            )
            .unwrap();
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
        // The catalog's screenshot is right again, and a checkbox's is wrong: the light one saved as the dark one.
        let dark_rest = Shot::all()
            .into_iter()
            .find(|shot| shot.file_name() == "dark-rest.png")
            .unwrap();
        fs::write(
            directory.join(dark_rest.file_name()),
            encode_uncompressed(&painted(dark_rest, 1)),
        )
        .unwrap();
        let rests: Vec<checkbox::Shot> = checkbox::Shot::all()
            .into_iter()
            .filter(|shot| shot.hovered.is_none())
            .collect();
        let [light, dark] = rests[..] else {
            panic!("one checkbox shot at rest in each appearance: {rests:?}");
        };
        fs::copy(
            directory.join(light.file_name()),
            directory.join(dark.file_name()),
        )
        .unwrap();
        let mut out = Vec::new();
        let counted = match run(&directory, &mut out) {
            Err(Error::Mismatches(count)) => count,
            other => panic!("the checkbox's wrong pixels are counted: {other:?}"),
        };
        let out = String::from_utf8(out).unwrap();
        assert!(counted > 0);
        assert!(
            out.contains(&format!("{}: FAILED", dark.file_name())),
            "{out}"
        );
        assert!(out.contains("dark-rest.png: ok"), "{out}");
        // Every pixel the run counted is named: two spaces start the line of each.
        assert_eq!(
            out.lines().filter(|line| line.starts_with("  ")).count(),
            counted
        );
    }

    // https://github.com/Vehicoule/nsh-m3e/issues/79
    #[test]
    fn regression_79_a_failed_test_leaves_nothing_in_the_temporary_directory() {
        let path =
            std::env::temp_dir().join(format!("visual-checks-scratch-{}", std::process::id()));
        // While the test is running the directory is there, and what is written in it with it.
        let failed = catch_unwind(AssertUnwindSafe(|| {
            let directory = ScratchDir::new("visual-checks-scratch");
            fs::write(directory.join("light-rest.png"), b"a screenshot").unwrap();
            assert!(path.join("light-rest.png").exists());
            panic!("an assertion that fails");
        }));
        assert!(failed.is_err());
        assert!(!path.exists(), "the directory was left behind");
        // A test that ends well leaves nothing either, and a directory a killed run left is not kept.
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("old.png"), b"left by a run that was killed").unwrap();
        {
            let directory = ScratchDir::new("visual-checks-scratch");
            assert!(!directory.join("old.png").exists());
        }
        assert!(!path.exists(), "the directory was left behind");
    }
    // ---- the linear progress indicators ----------------------------------------------------------------------------

    fn progress_cell(shape: ProgressShape, reading: Reading) -> ProgressCell {
        ProgressCell::new(shape, reading)
    }

    /// What `cell`'s probes read, relative to the cell's top left corner: what, how far along, how far down, and the
    /// color expected.
    fn relative(cell: ProgressCell, scheme: &ColorScheme) -> Vec<(&'static str, f32, f32, Rgb)> {
        let (x, y) = cell.origin();
        progress_probes(cell, scheme)
            .into_iter()
            .map(|p| (p.what, p.x.0 - x.0, p.y.0 - y.0, p.expected))
            .collect()
    }

    #[test]
    fn a_flat_determinate_indicator_is_read_in_its_line_its_gap_its_track_and_its_stop() {
        use ProgressShape::Flat;
        let primary = Rgb::of(&LIGHT, ColorRole::Primary);
        let track = Rgb::of(&LIGHT, ColorRole::SecondaryContainer);
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let at = |percent| relative(progress_cell(Flat, Reading::Determinate(percent)), &LIGHT);
        // 120 points wide and 4 tall: the row under the middle is 2 down, the line's end reaches the stop at 118.
        // Nothing done: no line at all, so no dot at its start, where the track is; the stop at the end of the track.
        assert_eq!(
            at(0),
            [
                ("no line", 2.0, 2.0, track),
                ("track", 60.0, 2.0, track),
                ("stop", 118.0, 2.0, primary),
            ]
        );
        // 12 points done: the line is 2 to 12, its middle 7; the gap 14 to 18 (8 between the round ends' centers, 12 and
        // 20), read at 16; the track 20 to 118, read at 69.
        assert_eq!(
            at(10),
            [
                ("line", 7.0, 2.0, primary),
                ("gap", 16.0, 2.0, surface),
                ("track", 69.0, 2.0, track),
                ("stop", 118.0, 2.0, primary),
            ]
        );
        assert_eq!(
            at(50),
            [
                ("line", 31.0, 2.0, primary),
                ("gap", 64.0, 2.0, surface),
                ("track", 93.0, 2.0, track),
                ("stop", 118.0, 2.0, primary),
            ]
        );
        // 108 done: the track is 116 to 118, too short to read, and the gap is 110 to 114.
        assert_eq!(
            at(90),
            [
                ("line", 55.0, 2.0, primary),
                ("gap", 112.0, 2.0, surface),
                ("stop", 118.0, 2.0, primary),
            ]
        );
        // 114 done and a gap would reach past the end: no track and no gap to read.
        assert_eq!(
            at(95),
            [("line", 58.0, 2.0, primary), ("stop", 118.0, 2.0, primary)]
        );
        assert_eq!(
            at(100),
            [("line", 60.0, 2.0, primary), ("stop", 118.0, 2.0, primary)]
        );
    }

    #[test]
    fn a_wavy_determinate_indicator_is_read_where_its_wave_crosses_the_middle() {
        use ProgressShape::Wavy;
        let primary = Rgb::of(&LIGHT, ColorRole::Primary);
        let track = Rgb::of(&LIGHT, ColorRole::SecondaryContainer);
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let at = |percent| relative(progress_cell(Wavy, Reading::Determinate(percent)), &LIGHT);
        // 10 points tall: the row under the middle is 5 down. Where the wave has a height (over a tenth, under 95 %) the
        // line is read where the wave crosses the middle, every 20 points (half its 40-point wavelength): at 20, where
        // the flat one is read at its middle.
        assert_eq!(
            at(0),
            [
                ("no line", 2.0, 5.0, track),
                ("track", 60.0, 5.0, track),
                ("stop", 118.0, 5.0, primary),
            ]
        );
        // A tenth: flat, so the middle, 7.
        assert_eq!(
            at(10),
            [
                ("line", 7.0, 5.0, primary),
                ("gap", 16.0, 5.0, surface),
                ("track", 69.0, 5.0, track),
                ("stop", 118.0, 5.0, primary),
            ]
        );
        assert_eq!(
            at(50),
            [
                ("line", 20.0, 5.0, primary),
                ("gap", 64.0, 5.0, surface),
                ("track", 93.0, 5.0, track),
                ("stop", 118.0, 5.0, primary),
            ]
        );
        assert_eq!(
            at(90),
            [
                ("line", 20.0, 5.0, primary),
                ("gap", 112.0, 5.0, surface),
                ("stop", 118.0, 5.0, primary),
            ]
        );
        // 95 %: flat again, so the middle, 58.
        assert_eq!(
            at(95),
            [("line", 58.0, 5.0, primary), ("stop", 118.0, 5.0, primary)]
        );
        assert_eq!(
            at(100),
            [("line", 60.0, 5.0, primary), ("stop", 118.0, 5.0, primary)]
        );
        // Just over a tenth the wave has its height, and a line that short has no crossing to read it at; just under
        // 95 % it has, and is read at 20.
        assert!(!at(11).iter().any(|probe| probe.0 == "line"));
        assert_eq!(at(94)[0], ("line", 20.0, 5.0, primary));
        assert_eq!(at(20)[0], ("line", 20.0, 5.0, primary));
    }

    #[test]
    fn an_indeterminate_indicator_is_read_in_its_first_line_and_in_the_track_between_its_lines() {
        let primary = Rgb::of(&LIGHT, ColorRole::Primary);
        let track = Rgb::of(&LIGHT, ColorRole::SecondaryContainer);
        // At 0.55 of the 120 points, inside the first line (0.41 to the end), and at a quarter, inside the track between
        // the lines (0.17 to 0.34). A wave crosses the middle every 10 points (half its 20-point wavelength): at 70.
        assert_eq!(
            relative(
                progress_cell(ProgressShape::Flat, Reading::Indeterminate),
                &LIGHT
            ),
            [
                ("first line", 66.0, 2.0, primary),
                ("track between the lines", 30.0, 2.0, track),
            ]
        );
        assert_eq!(
            relative(
                progress_cell(ProgressShape::Wavy, Reading::Indeterminate),
                &LIGHT
            ),
            [
                ("first line", 70.0, 5.0, primary),
                ("track between the lines", 30.0, 5.0, track),
            ]
        );
    }

    #[test]
    fn the_indicators_colors_are_the_tokens_in_each_appearance() {
        let dark = ColorScheme::BASELINE_DARK;
        let cell = progress_cell(ProgressShape::Flat, Reading::Determinate(50));
        let light = progress_probes(cell, &LIGHT);
        let darker = progress_probes(cell, &dark);
        assert_eq!(light.len(), darker.len());
        for (light, dark_probe) in light.iter().zip(&darker) {
            assert_eq!(
                (light.what, light.x, light.y, light.tolerance),
                (dark_probe.what, dark_probe.x, dark_probe.y, 0)
            );
        }
        // The line and the stop are primary, the track the secondary container, the gap the surface.
        let roles = [
            ColorRole::Primary,
            ColorRole::Surface,
            ColorRole::SecondaryContainer,
            ColorRole::Primary,
        ];
        for (probe, role) in darker.iter().zip(roles) {
            assert_eq!(probe.expected, Rgb::of(&dark, role), "{}", probe.what);
            assert_ne!(probe.expected, Rgb::of(&LIGHT, role), "{}", probe.what);
        }
        // Every probe is of this indicator, and in the strip beside the icon buttons.
        let (x, y) = cell.origin();
        for probe in &light {
            assert_eq!(probe.target, Target::Progress(cell));
            assert!(probe.x.0 >= x.0 && probe.x.0 < x.0 + 120.0);
            assert!(probe.y.0 >= y.0 && probe.y.0 < y.0 + 4.0);
        }
    }

    #[test]
    fn a_line_or_track_is_read_inside_it_and_not_where_it_is_too_short() {
        // The middle, at least 4 points from each end: 8 points is the least.
        assert_eq!(read_at(2.0, 12.0, None), Some(7.0));
        assert_eq!(read_at(0.0, 8.0, None), Some(4.0));
        assert_eq!(read_at(0.0, 7.0, None), None);
        assert_eq!(read_at(2.0, 9.0, None), None);
        // The middle rounds down to a pixel.
        assert_eq!(read_at(0.0, 9.0, None), Some(4.0));
        // A wave: the first crossing 4 or more points from the start, if it is 4 or more from the end.
        assert_eq!(read_at(2.0, 60.0, Some(20.0)), Some(20.0));
        assert_eq!(read_at(16.0, 100.0, Some(20.0)), Some(20.0));
        assert_eq!(read_at(17.0, 100.0, Some(20.0)), Some(40.0));
        assert_eq!(read_at(2.0, 24.0, Some(20.0)), Some(20.0));
        assert_eq!(read_at(2.0, 23.0, Some(20.0)), None);
    }

    #[test]
    fn the_progress_shots_are_the_test_script_files() {
        let shots = ProgressShot::all();
        let names: Vec<String> = shots.iter().map(|shot| shot.file_name()).collect();
        assert_eq!(names, ["progress-light.png", "progress-dark.png"]);
        assert_eq!(shots[0].appearance.scheme(), LIGHT);
        assert_eq!(shots[1].appearance.scheme(), ColorScheme::BASELINE_DARK);
        // No script hovers an indicator, but the name of a shot that did is its own.
        let cell = progress_cell(ProgressShape::Wavy, Reading::Determinate(50));
        let hovered = Shot {
            appearance: Appearance::Dark,
            hovered: Some(Target::Progress(cell)),
        };
        assert_eq!(
            hovered.file_name(),
            "dark-hover-linear-wavy-determinate-50.png"
        );
        assert_eq!(
            Target::Progress(cell).test_id(),
            "linear-wavy-determinate-50"
        );
    }

    #[test]
    fn a_progress_shot_reads_every_indicator() {
        let shot = ProgressShot::all()[0];
        let found = progress_shot_probes(shot);
        let expected: usize = ProgressCell::all()
            .map(|cell| progress_probes(cell, &LIGHT).len())
            .sum();
        assert_eq!(found.len(), expected);
        // Seven flat and seven wavy indicators: 3, 4, 4, 3, 2, 2 probes of the determinate ones and 2 of the
        // indeterminate one, twice.
        assert_eq!(expected, 2 * (3 + 4 + 4 + 3 + 2 + 2 + 2));
        let targets: std::collections::HashSet<String> =
            found.iter().map(|probe| probe.target.test_id()).collect();
        assert_eq!(targets.len(), ProgressCell::all().count());
        // The dark shot reads the same pixels for other colors.
        let dark = progress_shot_probes(ProgressShot::all()[1]);
        assert_eq!(dark.len(), found.len());
        assert!(
            found
                .iter()
                .zip(&dark)
                .all(|(light, dark)| (light.x, light.y) == (dark.x, dark.y)
                    && light.expected != dark.expected)
        );
    }

    /// The catalog's indicators painted as the tokens say, at `scale`: every probe of `shot` holds its expected color,
    /// and the rest a color no probe expects.
    fn painted_progress(shot: ProgressShot, scale: u32) -> Image {
        let (width, height) = grid::viewport();
        let (width, height) = (points_to_u32(width) * scale, points_to_u32(height) * scale);
        let mut rgba = [1, 2, 3, 255].repeat((width * height) as usize);
        for probe in progress_shot_probes(shot) {
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

    #[test]
    fn indicators_drawn_as_the_tokens_say_pass_at_any_whole_scale() {
        for shot in ProgressShot::all() {
            for scale in [1, 2] {
                assert_eq!(
                    check_progress(&painted_progress(shot, scale), shot),
                    Ok(Vec::new())
                );
            }
        }
    }

    #[test]
    fn one_wrong_indicator_pixel_fails_and_is_named() {
        let shot = ProgressShot::all()[0];
        let mut image = painted_progress(shot, 1);
        let cell = progress_cell(ProgressShape::Flat, Reading::Determinate(50));
        let probe = *progress_probes(cell, &LIGHT)
            .iter()
            .find(|probe| probe.what == "gap")
            .unwrap();
        let (x, y) = pixel_of(&probe, 1);
        let start = ((y * image.width + x) * 4) as usize;
        // The track's color where the gap is: the line and the track run together.
        image.rgba[start..start + 3]
            .copy_from_slice(&Rgb::of(&LIGHT, ColorRole::SecondaryContainer).0);
        let found = check_progress(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe, probe);
        assert_eq!(
            found[0].to_string(),
            format!(
                "linear-flat-determinate-50 gap at ({}, {}) points: expected {} (± 0), found {}",
                probe.x.0,
                probe.y.0,
                Rgb::of(&LIGHT, ColorRole::Surface),
                Rgb::of(&LIGHT, ColorRole::SecondaryContainer)
            )
        );
        // The same image is not the dark shot.
        assert_ne!(
            check_progress(&image, ProgressShot::all()[1]).unwrap(),
            Vec::new()
        );
        // The catalog's size is checked as for the buttons.
        let mut small = painted_progress(shot, 1);
        small.height -= 1;
        assert!(check_progress(&small, shot).is_err());
    }

    #[test]
    fn both_kinds_of_checks_must_pass() {
        assert!(both(Ok(()), Ok(())).is_ok());
        assert!(matches!(
            both(Err(Error::Mismatches(1)), Ok(())),
            Err(Error::Mismatches(1))
        ));
        assert!(matches!(
            both(Ok(()), Err(Error::Mismatches(2))),
            Err(Error::Mismatches(2))
        ));
        // The first failure is the one reported.
        assert!(matches!(
            both(Err(Error::Mismatches(1)), Err(Error::Mismatches(2))),
            Err(Error::Mismatches(1))
        ));
        assert!(finish(0).is_ok());
        assert!(matches!(finish(3), Err(Error::Mismatches(3))));
    }

    #[test]
    fn run_progress_reports_missing_and_wrong_screenshots() {
        let directory = ScratchDir::new("visual-checks-progress");
        let mut out = Vec::new();
        assert!(matches!(
            run_progress(&directory, &mut out),
            Err(Error::Read { .. })
        ));
        for shot in ProgressShot::all() {
            fs::write(
                directory.join(shot.file_name()),
                encode_uncompressed(&painted_progress(shot, 1)),
            )
            .unwrap();
        }
        let mut out = Vec::new();
        assert!(run_progress(&directory, &mut out).is_ok());
        let report = String::from_utf8(out).unwrap();
        assert!(report.contains("progress-light.png: ok"));
        assert!(report.contains("progress-dark.png: ok"));
        // The light screenshot saved as the dark one.
        fs::copy(
            directory.join("progress-light.png"),
            directory.join("progress-dark.png"),
        )
        .unwrap();
        let mut out = Vec::new();
        assert!(matches!(
            run_progress(&directory, &mut out),
            Err(Error::Mismatches(n)) if n > 0
        ));
        let report = String::from_utf8(out).unwrap();
        assert!(report.contains("progress-dark.png: FAILED"));
        assert!(report.contains("linear-flat-determinate-0 no line"));
    }

    #[test]
    fn the_checks_of_the_buttons_and_of_the_progress_indicators_both_run_on_the_directory() {
        let directory = ScratchDir::new("visual-checks-both");
        let args = [directory.to_str().unwrap().to_owned()];
        // `run` checks the buttons' and the checkboxes' screenshots, which `button.test.tsx` and `checkbox.test.tsx` take.
        let write_buttons = || {
            for shot in Shot::all() {
                fs::write(
                    directory.join(shot.file_name()),
                    encode_uncompressed(&painted(shot, 1)),
                )
                .unwrap();
            }
            for shot in checkbox::Shot::all() {
                fs::write(
                    directory.join(shot.file_name()),
                    encode_uncompressed(&checkbox::painted(shot, 1)),
                )
                .unwrap();
            }
        };
        let write_progress = || {
            for shot in ProgressShot::all() {
                fs::write(
                    directory.join(shot.file_name()),
                    encode_uncompressed(&painted_progress(shot, 1)),
                )
                .unwrap();
            }
        };
        // Nothing there: the buttons' screenshots are the first missing, and are what is reported.
        let mut out = Vec::new();
        assert!(matches!(run_with(&args, &mut out), Err(Error::Read { .. })));

        // Both right: both are named in what is written, and it passes.
        write_buttons();
        write_progress();
        let mut out = Vec::new();
        assert!(run_with(&args, &mut out).is_ok());
        let report = String::from_utf8(out).unwrap();
        assert!(report.contains("light-rest.png: ok"));
        assert!(report.contains("progress-light.png: ok"));

        // Only the progress indicators wrong (the light shot saved as the dark one): it fails, and says which.
        fs::copy(
            directory.join("progress-light.png"),
            directory.join("progress-dark.png"),
        )
        .unwrap();
        let mut out = Vec::new();
        assert!(matches!(
            run_with(&args, &mut out),
            Err(Error::Mismatches(n)) if n > 0
        ));
        let report = String::from_utf8(out).unwrap();
        assert!(report.contains("dark-rest.png: ok"));
        assert!(report.contains("progress-dark.png: FAILED"));

        // Only the buttons wrong: it fails, and the progress indicators were checked all the same.
        write_progress();
        fs::copy(
            directory.join("light-rest.png"),
            directory.join("dark-rest.png"),
        )
        .unwrap();
        let mut out = Vec::new();
        assert!(matches!(
            run_with(&args, &mut out),
            Err(Error::Mismatches(n)) if n > 0
        ));
        let report = String::from_utf8(out).unwrap();
        assert!(report.contains("dark-rest.png: FAILED"));
        assert!(report.contains("progress-dark.png: ok"));

        // The buttons' screenshots gone and the progress indicators' wrong: the missing file is what is reported, and
        // the wrong pixels are listed all the same.
        fs::remove_file(directory.join("light-rest.png")).unwrap();
        fs::copy(
            directory.join("progress-light.png"),
            directory.join("progress-dark.png"),
        )
        .unwrap();
        let mut out = Vec::new();
        assert!(matches!(run_with(&args, &mut out), Err(Error::Read { .. })));
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("progress-dark.png: FAILED")
        );
    }
}
