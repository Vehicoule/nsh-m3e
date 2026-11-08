//! The text fields' known pixels (ADR 0013): what `text_field.test.tsx` screenshots show of each text field of the
//! catalog (`catalog::text_field`), at rest and with the focus in one of them.
//!
//! The pixels the checks know in them: the filled container's color inside it and the surface
//! outside its top left corner, the indicator's color and thickness along its bottom edge (its rows, and the one above
//! them), the outlined field's outline on its top edge (its rows, and the one below them) and the notch the floating
//! label leaves in it, and the icons' colors in their middle. Where the antialiasing of an edge, of the label or of the
//! text makes a color unknown, nothing is checked: the label, the text and the supporting text are not read.
//!
//! Every color comes from the text field tokens (`FilledTextFieldTokens`, `OutlinedTextFieldTokens`), by the look the
//! field has: resting, focused, in error or disabled, in the order androidx `TextFieldColors` takes them. Two of
//! androidx's choices differ from what the tokens suggest, as ADR 0012 says: the container is the same color in every
//! look (`DisabledContainerColor` is not used), and a field in error that has the focus is as thick as a focused one,
//! with the error color.

use catalog::text_field::{Cell, Content, FIELD_WIDTH};
use nsh_core::geometry::Points;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::text_field::{TextFieldStatus, TextFieldStyle};
use nsh_m3e::tokens::androidx::{
    filled_text_field_tokens as filled, outlined_text_field_tokens as outlined, typography_tokens,
};

use super::{Appearance, CONTAINER_INSET, Image, Mismatch, Probe, Rgb, Target, faded, pixel_of};

/// One screenshot `examples/catalog/text_field.test.tsx` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Shot {
    pub(crate) appearance: Appearance,
    /// The text field the focus is in, if any.
    pub(crate) focused: Option<Cell>,
}

impl Shot {
    /// Every screenshot the script takes, in each appearance: the catalog at rest, then the focus in each of the
    /// fields [`focused`] names, one after the other.
    pub(crate) fn all() -> Vec<Self> {
        Appearance::ALL
            .into_iter()
            .flat_map(|appearance| {
                let shot = move |focused| Self {
                    appearance,
                    focused,
                };
                std::iter::once(shot(None))
                    .chain(focused().into_iter().map(move |cell| shot(Some(cell))))
            })
            .collect()
    }

    /// Its file name, as the test script saves it.
    pub(crate) fn file_name(self) -> String {
        let appearance = self.appearance.name();
        match self.focused {
            None => format!("text-field-{appearance}-rest.png"),
            Some(cell) => format!("text-field-{appearance}-focus-{}.png", cell.name()),
        }
    }
}

/// How far from the container's left edge the outlined field's top edge is read: past the corner's arc (the extra small
/// corner is 4 points), where the edge is straight. Not a design value: where the check looks.
const EDGE_INSET: Points = Points(8.0);

/// How far from the container's left edge the notch is read: inside it, which begins 12 points from the edge, and
/// before the floating label, which begins at 16 (androidx `OutlinedTextField.kt` `OutlinedTextFieldInnerPadding` and
/// `TextFieldImpl.kt` `TextFieldPadding`). Not a design value: where the check looks.
const NOTCH_INSET: Points = Points(14.0);

/// The width of an icon's box, which the icon is centered in: the touch target's 48 points (androidx
/// `minimumInteractiveComponentSize`), which the text field gives its leading and trailing icons. Not a token.
const ICON_BOX_WIDTH: Points = Points(48.0);

/// The height of the container, which both styles have: `OutlinedTextFieldTokens.ContainerHeight`, the same 56 points
/// as androidx `TextFieldDefaults.MinHeight`, which the filled tokens do not give.
const CONTAINER_HEIGHT: Points = outlined::CONTAINER_HEIGHT;

/// The text fields the test script gives the focus, one at a time, for a screenshot of each: the plain enabled one and
/// the rich one in error, of each style. The first shows the focused look, the second that an error stays an error
/// with the focus, and as thick as a focused field.
fn focused() -> [Cell; 4] {
    let cell = Cell::new;
    [
        cell(
            TextFieldStyle::Filled,
            Content::Plain,
            TextFieldStatus::Enabled,
        ),
        cell(
            TextFieldStyle::Filled,
            Content::Rich,
            TextFieldStatus::Error,
        ),
        cell(
            TextFieldStyle::Outlined,
            Content::Plain,
            TextFieldStatus::Enabled,
        ),
        cell(
            TextFieldStyle::Outlined,
            Content::Rich,
            TextFieldStatus::Error,
        ),
    ]
}

/// How a text field looks, in the order androidx `TextFieldColors` takes it: disabled first, then in error, then
/// focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Look {
    Resting,
    Focused,
    Error,
    Disabled,
}

impl Look {
    fn of(status: TextFieldStatus, focused: bool) -> Self {
        match status {
            TextFieldStatus::Disabled => Self::Disabled,
            TextFieldStatus::Error | TextFieldStatus::ReadOnlyError => Self::Error,
            _ if focused => Self::Focused,
            _ => Self::Resting,
        }
    }
}

/// How far the outlined field's container is below the top of its cell: the floating label sticks out half of its line
/// above the outline. The filled field has no overhang.
fn overhang(style: TextFieldStyle) -> Points {
    match style {
        TextFieldStyle::Outlined => Points(typography_tokens::BODY_SMALL.line_height.0 / 2.0),
        _ => Points(0.0),
    }
}

/// How thick the indicator or the outline is, in points. A disabled field takes no focus, and a field in error that has
/// it is as thick as a focused one. A style the catalog does not show has no line, of no thickness: its checks read
/// where no line is, and fail until they know it.
fn thickness(style: TextFieldStyle, disabled: bool, focused: bool) -> Points {
    match (style, disabled, focused) {
        (TextFieldStyle::Outlined, true, _) => outlined::DISABLED_OUTLINE_WIDTH,
        (TextFieldStyle::Outlined, false, true) => outlined::FOCUS_OUTLINE_WIDTH,
        (TextFieldStyle::Outlined, false, false) => outlined::OUTLINE_WIDTH,
        (TextFieldStyle::Filled, true, _) => filled::DISABLED_ACTIVE_INDICATOR_HEIGHT,
        (TextFieldStyle::Filled, false, true) => filled::FOCUS_ACTIVE_INDICATOR_HEIGHT,
        (TextFieldStyle::Filled, false, false) => filled::ACTIVE_INDICATOR_HEIGHT,
        _ => Points(0.0),
    }
}

/// The color of the indicator (filled) or of the outline, and its tolerance, over `behind`, which a disabled one is
/// faded over. A style the catalog does not show is expected in the surface color, which no line is drawn in, so its
/// checks fail until they know it. Not the filled one's: the error colors of both styles are the same, and falling
/// back to one would hide the other's arm.
fn line_color(style: TextFieldStyle, look: Look, scheme: &ColorScheme, behind: Rgb) -> (Rgb, u8) {
    let (role, opacity) = match (style, look) {
        (TextFieldStyle::Outlined, Look::Resting) => (outlined::OUTLINE_COLOR, None),
        (TextFieldStyle::Outlined, Look::Focused) => (outlined::FOCUS_OUTLINE_COLOR, None),
        (TextFieldStyle::Outlined, Look::Error) => (outlined::ERROR_OUTLINE_COLOR, None),
        (TextFieldStyle::Outlined, Look::Disabled) => (
            outlined::DISABLED_OUTLINE_COLOR,
            Some(outlined::DISABLED_OUTLINE_OPACITY),
        ),
        (TextFieldStyle::Filled, Look::Resting) => (filled::ACTIVE_INDICATOR_COLOR, None),
        (TextFieldStyle::Filled, Look::Focused) => (filled::FOCUS_ACTIVE_INDICATOR_COLOR, None),
        (TextFieldStyle::Filled, Look::Error) => (filled::ERROR_ACTIVE_INDICATOR_COLOR, None),
        (TextFieldStyle::Filled, Look::Disabled) => (
            filled::DISABLED_ACTIVE_INDICATOR_COLOR,
            Some(filled::DISABLED_ACTIVE_INDICATOR_OPACITY),
        ),
        _ => (ColorRole::Surface, None),
    };
    match opacity {
        Some(opacity) => faded(scheme, role, opacity, behind),
        None => (Rgb::of(scheme, role), 0),
    }
}

/// The colors of an icon in each look, and the opacity of the disabled one.
struct IconRoles {
    resting: ColorRole,
    focused: ColorRole,
    error: ColorRole,
    disabled: ColorRole,
    disabled_opacity: f32,
}

/// The tokens of the leading icon (`trailing` false) or of the trailing icon, in `style`. A style the catalog does not
/// show has icons in the surface color, which no icon is drawn in, so its checks fail until they know it: not the
/// filled one's, whose colors are the outlined one's.
fn icon_roles(style: TextFieldStyle, trailing: bool) -> IconRoles {
    match (style, trailing) {
        (TextFieldStyle::Outlined, false) => IconRoles {
            resting: outlined::LEADING_ICON_COLOR,
            focused: outlined::FOCUS_LEADING_ICON_COLOR,
            error: outlined::ERROR_LEADING_ICON_COLOR,
            disabled: outlined::DISABLED_LEADING_ICON_COLOR,
            disabled_opacity: outlined::DISABLED_LEADING_ICON_OPACITY,
        },
        (TextFieldStyle::Outlined, true) => IconRoles {
            resting: outlined::TRAILING_ICON_COLOR,
            focused: outlined::FOCUS_TRAILING_ICON_COLOR,
            error: outlined::ERROR_TRAILING_ICON_COLOR,
            disabled: outlined::DISABLED_TRAILING_ICON_COLOR,
            disabled_opacity: outlined::DISABLED_TRAILING_ICON_OPACITY,
        },
        (TextFieldStyle::Filled, false) => IconRoles {
            resting: filled::LEADING_ICON_COLOR,
            focused: filled::FOCUS_LEADING_ICON_COLOR,
            error: filled::ERROR_LEADING_ICON_COLOR,
            disabled: filled::DISABLED_LEADING_ICON_COLOR,
            disabled_opacity: filled::DISABLED_LEADING_ICON_OPACITY,
        },
        (TextFieldStyle::Filled, true) => IconRoles {
            resting: filled::TRAILING_ICON_COLOR,
            focused: filled::FOCUS_TRAILING_ICON_COLOR,
            error: filled::ERROR_TRAILING_ICON_COLOR,
            disabled: filled::DISABLED_TRAILING_ICON_COLOR,
            disabled_opacity: filled::DISABLED_TRAILING_ICON_OPACITY,
        },
        _ => IconRoles {
            resting: ColorRole::Surface,
            focused: ColorRole::Surface,
            error: ColorRole::Surface,
            disabled: ColorRole::Surface,
            disabled_opacity: 1.0,
        },
    }
}

/// The color of an icon, and its tolerance, over `behind`, which a disabled one is faded over.
fn icon_color(
    style: TextFieldStyle,
    trailing: bool,
    look: Look,
    scheme: &ColorScheme,
    behind: Rgb,
) -> (Rgb, u8) {
    let roles = icon_roles(style, trailing);
    match look {
        Look::Resting => (Rgb::of(scheme, roles.resting), 0),
        Look::Focused => (Rgb::of(scheme, roles.focused), 0),
        Look::Error => (Rgb::of(scheme, roles.error), 0),
        Look::Disabled => faded(scheme, roles.disabled, roles.disabled_opacity, behind),
    }
}

/// The pixels the checks know for the text field `cell` in a screenshot in `scheme`, with the focus in it or not.
fn probes(cell: Cell, scheme: &ColorScheme, focused: bool) -> Vec<Probe> {
    let (x, y) = cell.origin();
    let outlined_field = cell.style == TextFieldStyle::Outlined;
    let disabled = cell.status == TextFieldStatus::Disabled;
    // A disabled field takes no focus, whatever the screenshot's script does.
    let focused = focused && !disabled;
    let look = Look::of(cell.status, focused);
    let surface = Rgb::of(scheme, ColorRole::Surface);
    // What lies behind the line and the icons: the filled field's container, in every look, or the surface.
    let behind = if outlined_field {
        surface
    } else {
        Rgb::of(scheme, filled::CONTAINER_COLOR)
    };
    let line = line_color(cell.style, look, scheme, behind);
    let thickness = thickness(cell.style, disabled, focused).0;
    let top = y.0 + overhang(cell.style).0;
    let probe = |what: &'static str, dx: f32, dy: f32, (expected, tolerance): (Rgb, u8)| Probe {
        target: Target::TextField(cell),
        what,
        x: Points(x.0 + dx),
        y: Points(top + dy),
        expected,
        tolerance,
    };
    let middle = CONTAINER_HEIGHT.0 / 2.0;
    let mut probes = vec![
        // The top left pixel of the container's box: outside the arc of its corner (4 points), so the surface. A corner
        // drawn square would fill it.
        probe("outside the corner", 0.0, 0.0, (surface, 0)),
    ];
    if outlined_field {
        // On the straight part of the top edge: the outline's first pixel row, its last (the same one when it is a
        // pixel thick), and the row after it, inside the outline, where the surface shows.
        probes.push(probe("outline", EDGE_INSET.0, 0.0, line));
        probes.push(probe(
            "outline's last row",
            EDGE_INSET.0,
            thickness - 1.0,
            line,
        ));
        probes.push(probe(
            "inside the outline",
            EDGE_INSET.0,
            thickness,
            (surface, 0),
        ));
        // The notch: where the floating label leaves the outline open, only while the label floats, which it does with
        // the focus or a value.
        let floating = focused || cell.content == Content::Rich;
        probes.push(probe(
            "notch",
            NOTCH_INSET.0,
            0.0,
            if floating { (surface, 0) } else { line },
        ));
    } else {
        // The container, to the left of the label and of the icon; and along the bottom edge the indicator's last pixel
        // row, its first (the same one when it is a pixel thick), and the row before it, inside the container.
        let middle_x = FIELD_WIDTH.0 / 2.0;
        let bottom = CONTAINER_HEIGHT.0 - 1.0;
        probes.push(probe("container", CONTAINER_INSET.0, middle, (behind, 0)));
        probes.push(probe("indicator", middle_x, bottom, line));
        probes.push(probe(
            "indicator's first row",
            middle_x,
            bottom - (thickness - 1.0),
            line,
        ));
        probes.push(probe(
            "above the indicator",
            middle_x,
            bottom - thickness,
            (behind, 0),
        ));
    }
    if cell.content == Content::Rich {
        // The middle of each icon, `catalog::ICON`, which is filled there, in the middle of its box.
        probes.push(probe(
            "leading icon",
            ICON_BOX_WIDTH.0 / 2.0,
            middle,
            icon_color(cell.style, false, look, scheme, behind),
        ));
        probes.push(probe(
            "trailing icon",
            FIELD_WIDTH.0 - ICON_BOX_WIDTH.0 / 2.0,
            middle,
            icon_color(cell.style, true, look, scheme, behind),
        ));
    }
    probes
}

/// The pixels the checks know in `shot`, for every text field of the catalog.
fn shot_probes(shot: Shot) -> Vec<Probe> {
    let scheme = shot.appearance.scheme();
    Cell::all()
        .flat_map(|cell| probes(cell, &scheme, shot.focused == Some(cell)))
        .collect()
}

/// Every probe of `shot` that `image` does not match; an error if the image is not the catalog's text field viewport
/// at a whole scale.
pub(crate) fn check(image: &Image, shot: Shot) -> Result<Vec<Mismatch>, String> {
    let scale = image.scale_in(catalog::text_field::viewport())?;
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
    let (width, height) = catalog::text_field::viewport();
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
    use std::collections::HashMap;

    use super::super::{BLEND_TOLERANCE, points_to_u32};
    use super::*;

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;
    const DARK: ColorScheme = ColorScheme::BASELINE_DARK;

    fn cell(style: TextFieldStyle, content: Content, status: TextFieldStatus) -> Cell {
        Cell::new(style, content, status)
    }

    fn rgb(scheme: &ColorScheme, role: ColorRole) -> Rgb {
        Rgb::of(scheme, role)
    }

    /// The one probe of `found` that reads `what`.
    fn named(found: &[Probe], what: &str) -> Probe {
        let mut with_name = found.iter().filter(|probe| probe.what == what);
        let probe = with_name.next().copied().expect("a probe of that name");
        assert!(with_name.next().is_none(), "one probe named {what}");
        probe
    }

    fn at(cell: Cell, what: &'static str, x: f32, y: f32, expected: Rgb, tolerance: u8) -> Probe {
        Probe {
            target: Target::TextField(cell),
            what,
            x: Points(x),
            y: Points(y),
            expected,
            tolerance,
        }
    }

    #[test]
    fn a_resting_filled_field_is_read_at_its_corner_container_and_indicator() {
        let cell = cell(
            TextFieldStyle::Filled,
            Content::Plain,
            TextFieldStatus::Enabled,
        );
        let surface = rgb(&LIGHT, ColorRole::Surface);
        let container = rgb(&LIGHT, ColorRole::SurfaceContainerHighest);
        let line = rgb(&LIGHT, ColorRole::OnSurfaceVariant);
        // The cell's corner is (24, 3032); the container is 56 points high, and the indicator a point of it.
        assert_eq!(
            probes(cell, &LIGHT, false),
            [
                at(cell, "outside the corner", 24.0, 3032.0, surface, 0),
                at(cell, "container", 28.0, 3060.0, container, 0),
                at(cell, "indicator", 164.0, 3087.0, line, 0),
                at(cell, "indicator's first row", 164.0, 3087.0, line, 0),
                at(cell, "above the indicator", 164.0, 3086.0, container, 0),
            ]
        );
    }

    #[test]
    fn a_focused_filled_field_has_a_two_point_indicator_in_the_primary_color() {
        let cell = cell(
            TextFieldStyle::Filled,
            Content::Plain,
            TextFieldStatus::Enabled,
        );
        let container = rgb(&LIGHT, ColorRole::SurfaceContainerHighest);
        let line = rgb(&LIGHT, ColorRole::Primary);
        let found = probes(cell, &LIGHT, true);
        assert_eq!(found.len(), 5);
        assert_eq!(
            found[2..],
            [
                at(cell, "indicator", 164.0, 3087.0, line, 0),
                at(cell, "indicator's first row", 164.0, 3086.0, line, 0),
                at(cell, "above the indicator", 164.0, 3085.0, container, 0),
            ]
        );
        // The container and the corner are the same with the focus.
        assert_eq!(found[..2], probes(cell, &LIGHT, false)[..2]);
    }

    #[test]
    fn an_outlined_field_is_read_at_its_corner_outline_and_notch() {
        let cell = cell(
            TextFieldStyle::Outlined,
            Content::Plain,
            TextFieldStatus::Enabled,
        );
        let surface = rgb(&LIGHT, ColorRole::Surface);
        let outline = rgb(&LIGHT, ColorRole::Outline);
        // The cell's corner is (24, 3132); the container is below the 8 points the label sticks out above it.
        assert_eq!(
            probes(cell, &LIGHT, false),
            [
                at(cell, "outside the corner", 24.0, 3140.0, surface, 0),
                at(cell, "outline", 32.0, 3140.0, outline, 0),
                at(cell, "outline's last row", 32.0, 3140.0, outline, 0),
                at(cell, "inside the outline", 32.0, 3141.0, surface, 0),
                // The label rests, so the outline is not open.
                at(cell, "notch", 38.0, 3140.0, outline, 0),
            ]
        );
        // With the focus the outline is two points thick, in the primary color, and open where the label floats.
        let primary = rgb(&LIGHT, ColorRole::Primary);
        assert_eq!(
            probes(cell, &LIGHT, true),
            [
                at(cell, "outside the corner", 24.0, 3140.0, surface, 0),
                at(cell, "outline", 32.0, 3140.0, primary, 0),
                at(cell, "outline's last row", 32.0, 3141.0, primary, 0),
                at(cell, "inside the outline", 32.0, 3142.0, surface, 0),
                at(cell, "notch", 38.0, 3140.0, surface, 0),
            ]
        );
    }

    #[test]
    fn the_label_floats_with_a_value_and_opens_the_notch_without_the_focus() {
        let rich = cell(
            TextFieldStyle::Outlined,
            Content::Rich,
            TextFieldStatus::Enabled,
        );
        let notch = named(&probes(rich, &LIGHT, false), "notch");
        assert_eq!(
            (notch.x, notch.y, notch.expected),
            (
                Points(984.0 + 14.0),
                Points(3140.0),
                rgb(&LIGHT, ColorRole::Surface)
            )
        );
        // Plain, unfocused: the label rests, and the outline is whole there, for every status.
        for status in [
            TextFieldStatus::Enabled,
            TextFieldStatus::Error,
            TextFieldStatus::Disabled,
        ] {
            let plain = cell(TextFieldStyle::Outlined, Content::Plain, status);
            let found = probes(plain, &LIGHT, false);
            assert_eq!(
                named(&found, "notch").expected,
                named(&found, "outline").expected,
                "{status:?}"
            );
            let found = probes(
                cell(TextFieldStyle::Outlined, Content::Rich, status),
                &LIGHT,
                false,
            );
            assert_eq!(
                named(&found, "notch").expected,
                rgb(&LIGHT, ColorRole::Surface),
                "{status:?}"
            );
        }
    }

    /// A line's color: a role, and when it is faded the opacity.
    type Line = (ColorRole, Option<f32>);

    /// Every look of a field, as `(status, focused)`, with the indicator or outline it has: `[resting, focused, in error,
    /// in error with the focus, disabled, disabled with the focus]`, and how thick it is.
    fn looks(
        resting: Line,
        focused: Line,
        error: Line,
        disabled: Line,
    ) -> [(TextFieldStatus, bool, Line, f32); 6] {
        [
            (TextFieldStatus::Enabled, false, resting, 1.0),
            (TextFieldStatus::Enabled, true, focused, 2.0),
            (TextFieldStatus::Error, false, error, 1.0),
            (TextFieldStatus::Error, true, error, 2.0),
            (TextFieldStatus::Disabled, false, disabled, 1.0),
            // A disabled field takes no focus: it looks as it does without.
            (TextFieldStatus::Disabled, true, disabled, 1.0),
        ]
    }

    #[test]
    fn the_indicator_is_the_tokens_color_and_thickness_in_every_look() {
        let container = rgb(&LIGHT, ColorRole::SurfaceContainerHighest);
        let table = looks(
            (ColorRole::OnSurfaceVariant, None),
            (ColorRole::Primary, None),
            (ColorRole::Error, None),
            (ColorRole::OnSurface, Some(0.38)),
        );
        for (status, focused, (role, opacity), thickness) in table {
            let cell = cell(TextFieldStyle::Filled, Content::Plain, status);
            let what = format!("{status:?} {focused}");
            let found = probes(cell, &LIGHT, focused);
            // Halfway along the 280 points of the field, which is in the column of its status.
            let middle = cell.origin().0.0 + 140.0;
            let (line, tolerance) = match opacity {
                Some(opacity) => (rgb(&LIGHT, role).over(opacity, container), BLEND_TOLERANCE),
                None => (rgb(&LIGHT, role), 0),
            };
            assert_eq!(
                named(&found, "indicator"),
                at(cell, "indicator", middle, 3087.0, line, tolerance),
                "{what}"
            );
            assert_eq!(
                named(&found, "indicator's first row"),
                at(
                    cell,
                    "indicator's first row",
                    middle,
                    3087.0 - (thickness - 1.0),
                    line,
                    tolerance
                ),
                "{what}"
            );
            assert_eq!(
                named(&found, "above the indicator"),
                at(
                    cell,
                    "above the indicator",
                    middle,
                    3087.0 - thickness,
                    container,
                    0
                ),
                "{what}"
            );
            // The container is the same in every look.
            assert_eq!(named(&found, "container").expected, container, "{what}");
        }
    }

    #[test]
    fn the_outline_is_the_tokens_color_and_thickness_in_every_look() {
        let surface = rgb(&LIGHT, ColorRole::Surface);
        let table = looks(
            (ColorRole::Outline, None),
            (ColorRole::Primary, None),
            (ColorRole::Error, None),
            (ColorRole::OnSurface, Some(0.12)),
        );
        for (status, focused, (role, opacity), thickness) in table {
            let cell = cell(TextFieldStyle::Outlined, Content::Plain, status);
            let what = format!("{status:?} {focused}");
            let found = probes(cell, &LIGHT, focused);
            // 8 points into the cell, which is in the column of its status.
            let edge = cell.origin().0.0 + 8.0;
            let (line, tolerance) = match opacity {
                Some(opacity) => (rgb(&LIGHT, role).over(opacity, surface), BLEND_TOLERANCE),
                None => (rgb(&LIGHT, role), 0),
            };
            assert_eq!(
                named(&found, "outline"),
                at(cell, "outline", edge, 3140.0, line, tolerance),
                "{what}"
            );
            assert_eq!(
                named(&found, "outline's last row"),
                at(
                    cell,
                    "outline's last row",
                    edge,
                    3140.0 + thickness - 1.0,
                    line,
                    tolerance
                ),
                "{what}"
            );
            assert_eq!(
                named(&found, "inside the outline"),
                at(
                    cell,
                    "inside the outline",
                    edge,
                    3140.0 + thickness,
                    surface,
                    0
                ),
                "{what}"
            );
        }
    }

    #[test]
    fn a_disabled_field_takes_no_focus() {
        for style in [TextFieldStyle::Filled, TextFieldStyle::Outlined] {
            for content in [Content::Plain, Content::Rich] {
                let cell = cell(style, content, TextFieldStatus::Disabled);
                assert_eq!(
                    probes(cell, &LIGHT, true),
                    probes(cell, &LIGHT, false),
                    "{cell:?}"
                );
            }
        }
    }

    #[test]
    fn the_icons_are_read_in_the_middle_of_their_boxes_in_the_looks_colors() {
        let table = [
            // (status, focused, the leading icon's role, the trailing icon's, the opacity when faded)
            (
                TextFieldStatus::Enabled,
                false,
                ColorRole::OnSurfaceVariant,
                ColorRole::OnSurfaceVariant,
                None,
            ),
            (
                TextFieldStatus::Enabled,
                true,
                ColorRole::OnSurfaceVariant,
                ColorRole::OnSurfaceVariant,
                None,
            ),
            (
                TextFieldStatus::Error,
                false,
                ColorRole::OnSurfaceVariant,
                ColorRole::Error,
                None,
            ),
            (
                TextFieldStatus::Error,
                true,
                ColorRole::OnSurfaceVariant,
                ColorRole::Error,
                None,
            ),
            (
                TextFieldStatus::Disabled,
                false,
                ColorRole::OnSurface,
                ColorRole::OnSurface,
                Some(0.38),
            ),
        ];
        for style in [TextFieldStyle::Filled, TextFieldStyle::Outlined] {
            let behind = match style {
                TextFieldStyle::Outlined => rgb(&LIGHT, ColorRole::Surface),
                _ => rgb(&LIGHT, ColorRole::SurfaceContainerHighest),
            };
            // The container's top: 8 points into the cell for the outlined field, which the label sticks out above.
            let top = match style {
                TextFieldStyle::Outlined => 3132.0 + 8.0,
                _ => 3032.0,
            };
            for (status, focused, leading, trailing, opacity) in table {
                let cell = cell(style, Content::Rich, status);
                let what = format!("{style:?} {status:?} {focused}");
                let found = probes(cell, &LIGHT, focused);
                // The rich fields start at the fourth column, and the status takes one of the three after it.
                let x = cell.origin().0.0;
                let expected = |role| match opacity {
                    Some(opacity) => (rgb(&LIGHT, role).over(opacity, behind), BLEND_TOLERANCE),
                    None => (rgb(&LIGHT, role), 0),
                };
                let (color, tolerance) = expected(leading);
                assert_eq!(
                    named(&found, "leading icon"),
                    at(cell, "leading icon", x + 24.0, top + 28.0, color, tolerance),
                    "{what}"
                );
                let (color, tolerance) = expected(trailing);
                assert_eq!(
                    named(&found, "trailing icon"),
                    at(
                        cell,
                        "trailing icon",
                        x + 256.0,
                        top + 28.0,
                        color,
                        tolerance
                    ),
                    "{what}"
                );
            }
            // A plain field has no icon to read.
            let plain = probes(
                Cell::new(style, Content::Plain, TextFieldStatus::Enabled),
                &LIGHT,
                false,
            );
            assert!(plain.iter().all(|probe| !probe.what.ends_with("icon")));
        }
    }

    #[test]
    fn the_colors_follow_the_scheme() {
        let cell = cell(
            TextFieldStyle::Filled,
            Content::Rich,
            TextFieldStatus::Enabled,
        );
        let light = probes(cell, &LIGHT, true);
        let dark = probes(cell, &DARK, true);
        assert_eq!(
            named(&dark, "container").expected,
            rgb(&DARK, ColorRole::SurfaceContainerHighest)
        );
        assert_eq!(
            named(&dark, "indicator").expected,
            rgb(&DARK, ColorRole::Primary)
        );
        assert_eq!(
            named(&dark, "outside the corner").expected,
            rgb(&DARK, ColorRole::Surface)
        );
        assert_eq!(
            named(&dark, "leading icon").expected,
            rgb(&DARK, ColorRole::OnSurfaceVariant)
        );
        assert_ne!(
            named(&dark, "container").expected,
            named(&light, "container").expected
        );
    }

    #[test]
    fn a_look_is_taken_in_the_order_of_text_field_colors() {
        use TextFieldStatus::{Disabled, Enabled, Error, ReadOnly, ReadOnlyError};
        for (status, focused, look) in [
            (Enabled, false, Look::Resting),
            (Enabled, true, Look::Focused),
            (ReadOnly, false, Look::Resting),
            (ReadOnly, true, Look::Focused),
            (Error, false, Look::Error),
            (Error, true, Look::Error),
            (ReadOnlyError, false, Look::Error),
            (ReadOnlyError, true, Look::Error),
            (Disabled, false, Look::Disabled),
            (Disabled, true, Look::Disabled),
        ] {
            assert_eq!(Look::of(status, focused), look, "{status:?} {focused}");
        }
    }

    #[test]
    fn the_thickness_is_the_tokens() {
        for style in [TextFieldStyle::Filled, TextFieldStyle::Outlined] {
            assert_eq!(thickness(style, false, false), Points(1.0));
            assert_eq!(thickness(style, false, true), Points(2.0));
            assert_eq!(thickness(style, true, false), Points(1.0));
            assert_eq!(thickness(style, true, true), Points(1.0));
        }
    }

    #[test]
    fn the_outlined_label_sticks_out_half_its_line_above_the_container() {
        assert_eq!(overhang(TextFieldStyle::Outlined), Points(8.0));
        assert_eq!(overhang(TextFieldStyle::Filled), Points(0.0));
        assert_eq!(CONTAINER_HEIGHT, Points(56.0));
    }

    #[test]
    fn four_text_fields_take_the_focus() {
        let ids: Vec<String> = focused().into_iter().map(Cell::test_id).collect();
        assert_eq!(
            ids,
            [
                "text-field-filled-plain-enabled",
                "text-field-filled-rich-error",
                "text-field-outlined-plain-enabled",
                "text-field-outlined-rich-error",
            ]
        );
    }

    #[test]
    fn the_shots_are_the_rest_and_the_four_focuses_in_each_appearance() {
        let shots = Shot::all();
        assert_eq!(shots.len(), 10);
        let names: Vec<String> = shots.iter().map(|shot| shot.file_name()).collect();
        assert_eq!(
            names,
            [
                "text-field-light-rest.png",
                "text-field-light-focus-filled-plain-enabled.png",
                "text-field-light-focus-filled-rich-error.png",
                "text-field-light-focus-outlined-plain-enabled.png",
                "text-field-light-focus-outlined-rich-error.png",
                "text-field-dark-rest.png",
                "text-field-dark-focus-filled-plain-enabled.png",
                "text-field-dark-focus-filled-rich-error.png",
                "text-field-dark-focus-outlined-plain-enabled.png",
                "text-field-dark-focus-outlined-rich-error.png",
            ]
        );
        assert_eq!(shots[0].appearance, Appearance::Light);
        assert_eq!(shots[5].appearance, Appearance::Dark);
        assert!(shots[0].focused.is_none() && shots[5].focused.is_none());
        // The script's names are the shots' (`text_field.test.tsx`).
        let script = include_str!("../../../examples/catalog/text_field.test.tsx");
        assert!(script.contains("`target/visual/text-field-${scheme}-rest.png`"));
        assert!(
            script.contains("`target/visual/text-field-${scheme}-focus-${style}-${field}.png`")
        );
        for cell in focused() {
            let name = cell.name();
            let (style, field) = name.split_once('-').expect("a style and a field");
            assert!(script.contains(&format!("\"{style}\"")), "{name}");
            assert!(script.contains(&format!("\"{field}\"")), "{name}");
        }
    }

    #[test]
    fn a_shot_with_the_focus_in_a_field_reads_that_field_focused_and_the_rest_as_they_are() {
        let shots = Shot::all();
        let focus: Vec<&Shot> = shots.iter().filter(|shot| shot.focused.is_some()).collect();
        assert_eq!(focus.len(), 8);
        for shot in focus {
            let cell = shot.focused.expect("a focused field");
            let at_rest = shot_probes(Shot {
                focused: None,
                ..*shot
            });
            let found = shot_probes(*shot);
            assert_eq!(found.len(), at_rest.len());
            let changed: Vec<&Probe> = found
                .iter()
                .zip(&at_rest)
                .filter(|(a, b)| a != b)
                .map(|(a, _)| a)
                .collect();
            assert!(!changed.is_empty(), "{cell:?}");
            // Only that field's probes differ.
            assert!(
                changed
                    .iter()
                    .all(|probe| probe.target == Target::TextField(cell)),
                "{cell:?}"
            );
        }
    }

    #[test]
    fn a_text_field_is_a_target_named_by_its_test_id() {
        let cell = cell(
            TextFieldStyle::Outlined,
            Content::Rich,
            TextFieldStatus::Error,
        );
        assert_eq!(
            Target::TextField(cell).test_id(),
            "text-field-outlined-rich-error"
        );
    }

    #[test]
    fn no_two_probes_of_a_shot_expect_different_colors_of_one_pixel() {
        for shot in Shot::all() {
            let mut seen: HashMap<(u32, u32), Probe> = HashMap::new();
            for probe in shot_probes(shot) {
                let key = (probe.x.0.to_bits(), probe.y.0.to_bits());
                if let Some(before) = seen.insert(key, probe) {
                    assert_eq!(
                        (before.expected, before.tolerance),
                        (probe.expected, probe.tolerance),
                        "{before:?} and {probe:?}"
                    );
                }
            }
        }
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
        // The focused filled field: the indicator two points thick, in the primary color.
        let shot = Shot::all()[1];
        let focused = shot.focused.expect("a focused field");
        let mut image = painted(shot, 1);
        let wanted = probes(focused, &LIGHT, true)
            .into_iter()
            .find(|probe| probe.what == "indicator")
            .expect("an indicator probe");
        let (x, y) = pixel_of(&wanted, 1);
        let start = ((y * image.width + x) * 4) as usize;
        // The indicator's place showing the resting indicator's color, as a field that took no focus would.
        let resting = Rgb::of(&LIGHT, filled::ACTIVE_INDICATOR_COLOR);
        image.rgba[start..start + 3].copy_from_slice(&resting.0);
        let found = check(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe, wanted);
        assert_eq!(found[0].found, Some(resting));
        assert_eq!(
            found[0].to_string(),
            format!(
                "text-field-filled-plain-enabled indicator at (164, 3087) points: expected {} (± 0), found {resting}",
                wanted.expected
            )
        );
        // The same image is not the dark shot, nor the one with the focus elsewhere.
        assert_ne!(check(&image, Shot::all()[6]).unwrap(), Vec::new());
        assert_ne!(check(&image, Shot::all()[2]).unwrap(), Vec::new());
    }

    #[test]
    fn a_screenshot_of_another_viewport_is_refused() {
        // `checkbox.test.tsx` opens the catalog at the viewport above the text fields.
        let (width, height) = catalog::checkbox::viewport();
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
