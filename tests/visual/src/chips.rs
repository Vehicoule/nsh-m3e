//! The chips' known pixels in the catalog's screenshots (`examples/catalog/chips.test.tsx`), checked as the buttons'
//! are: every expected color comes from the library's tokens.
//!
//! For every chip of the catalog, in each appearance and with the mouse over it or not, a screenshot holds these pixels:
//! in each chip, the container's color or the surface where it has none, the surface outside its top left corner, the
//! outline's color on its top edge (or the container's where a selected chip has none), the color of its icon before the
//! label, and, for an input chip with an avatar, the avatar's color and the container's where the avatar's circle
//! leaves its box.
//!
//! Not read: the icon after the label, whose place depends on the width of the label in the host's font, and the
//! label. Where the antialiasing of an edge makes a color unknown, nothing is checked.
//!
//! Every color comes from the chips' tokens, and every number that is not a token is a layout value of androidx's
//! `Chip.kt` that `catalog::chips` names (ADR 0012).

use std::f32::consts::SQRT_2;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use catalog::chips::{self, AVATAR_ROLE, ChipCell, ChipContent, ChipKind, Selection};
use catalog::grid::Availability;
use nsh_core::geometry::Points;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::tokens::Shape;
use nsh_m3e::tokens::androidx::{
    assist_chip_tokens, chips_tokens, filter_chip_tokens, input_chip_tokens, state_tokens,
    suggestion_chip_tokens,
};

use super::{Appearance, CONTAINER_INSET, Error, Image, Rgb, faded, points_to_u32};

/// The chips the mouse goes over in a screenshot, one each, in the order of their files: the enabled chip that shows
/// only its label, of each kind and, for the filter and input chips, of each selection.
const HOVERED: [ChipCell; 6] = [
    ChipCell::new(
        ChipKind::Assist,
        Selection::Unselected,
        ChipContent::Label,
        Availability::Enabled,
    ),
    ChipCell::new(
        ChipKind::Filter,
        Selection::Unselected,
        ChipContent::Label,
        Availability::Enabled,
    ),
    ChipCell::new(
        ChipKind::Filter,
        Selection::Selected,
        ChipContent::Label,
        Availability::Enabled,
    ),
    ChipCell::new(
        ChipKind::Input,
        Selection::Unselected,
        ChipContent::Label,
        Availability::Enabled,
    ),
    ChipCell::new(
        ChipKind::Input,
        Selection::Selected,
        ChipContent::Label,
        Availability::Enabled,
    ),
    ChipCell::new(
        ChipKind::Suggestion,
        Selection::Unselected,
        ChipContent::Label,
        Availability::Enabled,
    ),
];

/// How far inside a chip's left edge, at mid height, its container's color is read: past the antialiased edge and the
/// outline (every chip's outline is 1 point wide: `FlatOutlineWidth`, `UnselectedOutlineWidth`), and before the avatar,
/// which starts 4 points from the edge at the nearest (`Chip.kt` line 2683). Not a design value: where the check looks.
const INSET: Points = Points(2.0);

/// One screenshot the test script takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Shot {
    appearance: Appearance,
    /// The chip the mouse is over, if any.
    hovered: Option<ChipCell>,
}

impl Shot {
    /// Every screenshot `examples/catalog/chips.test.tsx` takes, in each appearance: the catalog at rest, then the mouse
    /// over each of the [`HOVERED`] chips.
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

    /// Its file name, as the test script saves it: the kind of the hovered chip, then `-selected` for a selected one.
    fn file_name(self) -> String {
        let appearance = self.appearance.name();
        match self.hovered {
            None => format!("chips-{appearance}-rest.png"),
            Some(cell) => {
                let selected = match cell.selection {
                    Selection::Selected => "-selected",
                    _ => "",
                };
                format!(
                    "chips-{appearance}-hover-{}{selected}.png",
                    cell.kind.label().to_lowercase()
                )
            }
        }
    }

    /// Whether the mouse is over `cell` in it.
    fn hovers(self, cell: ChipCell) -> bool {
        self.hovered == Some(cell)
    }
}

/// A pixel the checks know, in the catalog's points.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Probe {
    cell: ChipCell,
    what: &'static str,
    x: Points,
    y: Points,
    expected: Rgb,
    tolerance: u8,
}

/// The color of the state layer of `cell`: its label's color, which hover lays over it at `HoverStateLayerOpacity`
/// (androidx's ripple takes the content color; the `Hover` label tokens name the same roles as the label's).
fn state_role(cell: ChipCell) -> ColorRole {
    match (cell.kind, cell.selection) {
        (ChipKind::Filter, Selection::Selected) => {
            filter_chip_tokens::SELECTED_HOVER_LABEL_TEXT_COLOR
        }
        (ChipKind::Filter, _) => filter_chip_tokens::UNSELECTED_HOVER_LABEL_TEXT_COLOR,
        (ChipKind::Input, Selection::Selected) => {
            input_chip_tokens::SELECTED_HOVER_LABEL_TEXT_COLOR
        }
        (ChipKind::Input, _) => input_chip_tokens::UNSELECTED_HOVER_LABEL_TEXT_COLOR,
        (ChipKind::Suggestion, _) => suggestion_chip_tokens::HOVER_LABEL_TEXT_COLOR,
        _ => assist_chip_tokens::HOVER_LABEL_TEXT_COLOR,
    }
}

/// The container's color of `cell` and its tolerance, over `surface`, with the mouse over it or not. A chip that is
/// not selected has none (the flat chips have a `Color.Transparent` container: `Chip.kt`, `assistChipColors`,
/// `suggestionChipColors` and the unselected colors of the filter and input chips): the surface shows through.
fn container(cell: ChipCell, scheme: &ColorScheme, hovered: bool) -> (Rgb, u8) {
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let disabled = cell.availability == Availability::Disabled;
    let (rest, tolerance) = match (cell.kind, cell.selection, disabled) {
        (ChipKind::Filter, Selection::Selected, false) => (
            Rgb::of(scheme, filter_chip_tokens::FLAT_SELECTED_CONTAINER_COLOR),
            0,
        ),
        (ChipKind::Filter, Selection::Selected, true) => faded(
            scheme,
            filter_chip_tokens::FLAT_DISABLED_SELECTED_CONTAINER_COLOR,
            filter_chip_tokens::FLAT_DISABLED_SELECTED_CONTAINER_OPACITY,
            surface,
        ),
        (ChipKind::Input, Selection::Selected, false) => (
            Rgb::of(scheme, input_chip_tokens::SELECTED_CONTAINER_COLOR),
            0,
        ),
        (ChipKind::Input, Selection::Selected, true) => faded(
            scheme,
            input_chip_tokens::DISABLED_SELECTED_CONTAINER_COLOR,
            input_chip_tokens::DISABLED_SELECTED_CONTAINER_OPACITY,
            surface,
        ),
        _ => (surface, 0),
    };
    if hovered && !disabled {
        faded(
            scheme,
            state_role(cell),
            state_tokens::HOVER_STATE_LAYER_OPACITY,
            rest,
        )
    } else {
        (rest, tolerance)
    }
}

/// The color of the icon before the label of an enabled `cell`.
fn icon_role(cell: ChipCell) -> ColorRole {
    match (cell.kind, cell.selection) {
        (ChipKind::Filter, Selection::Selected) => filter_chip_tokens::SELECTED_LEADING_ICON_COLOR,
        // androidx `Chip.kt`, `defaultTonalFilterChipColors` (line 2374): `ChipsTokens.UnselectedLeadingIconColor`, on
        // surface variant, where `FilterChipTokens.UnselectedLeadingIconColor` says the primary color (ADR 0012), as the
        // filter chip takes it.
        (ChipKind::Filter, _) => chips_tokens::UNSELECTED_LEADING_ICON_COLOR,
        (ChipKind::Input, Selection::Selected) => input_chip_tokens::SELECTED_LEADING_ICON_COLOR,
        (ChipKind::Input, _) => input_chip_tokens::UNSELECTED_LEADING_ICON_COLOR,
        (ChipKind::Suggestion, _) => suggestion_chip_tokens::LEADING_ICON_COLOR,
        _ => assist_chip_tokens::ICON_COLOR,
    }
}

/// The color and opacity of the icon before the label of a disabled `cell`.
fn disabled_icon(cell: ChipCell) -> (ColorRole, f32) {
    match cell.kind {
        ChipKind::Filter => (
            filter_chip_tokens::DISABLED_LEADING_ICON_COLOR,
            filter_chip_tokens::DISABLED_LEADING_ICON_OPACITY,
        ),
        ChipKind::Input => (
            input_chip_tokens::DISABLED_LEADING_ICON_COLOR,
            input_chip_tokens::DISABLED_LEADING_ICON_OPACITY,
        ),
        ChipKind::Suggestion => (
            suggestion_chip_tokens::DISABLED_LEADING_ICON_COLOR,
            suggestion_chip_tokens::DISABLED_LEADING_ICON_OPACITY,
        ),
        _ => (
            assist_chip_tokens::DISABLED_ICON_COLOR,
            assist_chip_tokens::DISABLED_ICON_OPACITY,
        ),
    }
}

/// The icon's color of `cell` and its tolerance. The icon is drawn over the state layer, so the mouse over it changes
/// nothing; a disabled icon is faded over the container, a blend that may be over a blend, whose two roundings add up.
fn icon_color(cell: ChipCell, scheme: &ColorScheme) -> (Rgb, u8) {
    if cell.availability == Availability::Enabled {
        return (Rgb::of(scheme, icon_role(cell)), 0);
    }
    let (role, opacity) = disabled_icon(cell);
    let (container, container_tolerance) = container(cell, scheme, false);
    let (color, tolerance) = faded(scheme, role, opacity, container);
    (color, tolerance + container_tolerance)
}

/// The avatar's color of `cell`, which the catalog fills with [`AVATAR_ROLE`], and its tolerance: faded to
/// `DisabledAvatarOpacity` over the container when the chip is disabled (`Chip.kt`, `InputChip`, lines 1323 to 1341).
fn avatar_color(cell: ChipCell, scheme: &ColorScheme) -> (Rgb, u8) {
    if cell.availability == Availability::Enabled {
        return (Rgb::of(scheme, AVATAR_ROLE), 0);
    }
    let (container, container_tolerance) = container(cell, scheme, false);
    let (color, tolerance) = faded(
        scheme,
        AVATAR_ROLE,
        input_chip_tokens::DISABLED_AVATAR_OPACITY,
        container,
    );
    (color, tolerance + container_tolerance)
}

/// The outline's color of an unselected `cell` and its tolerance, over `surface`: its `OutlineColor`, faded by the
/// disabled outline's opacity when it is disabled.
fn outline_color(cell: ChipCell, scheme: &ColorScheme) -> (Rgb, u8) {
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let disabled = cell.availability == Availability::Disabled;
    // Each kind has its own tokens: none may fall back on another's, or one would hide the other's arm.
    let (enabled, fade) = match cell.kind {
        ChipKind::Filter => (
            filter_chip_tokens::FLAT_UNSELECTED_OUTLINE_COLOR,
            (
                filter_chip_tokens::FLAT_DISABLED_UNSELECTED_OUTLINE_COLOR,
                filter_chip_tokens::FLAT_DISABLED_UNSELECTED_OUTLINE_OPACITY,
            ),
        ),
        ChipKind::Input => (
            input_chip_tokens::UNSELECTED_OUTLINE_COLOR,
            (
                input_chip_tokens::DISABLED_UNSELECTED_OUTLINE_COLOR,
                input_chip_tokens::DISABLED_UNSELECTED_OUTLINE_OPACITY,
            ),
        ),
        ChipKind::Suggestion => (
            suggestion_chip_tokens::FLAT_OUTLINE_COLOR,
            (
                suggestion_chip_tokens::FLAT_DISABLED_OUTLINE_COLOR,
                suggestion_chip_tokens::FLAT_DISABLED_OUTLINE_OPACITY,
            ),
        ),
        _ => (
            assist_chip_tokens::FLAT_OUTLINE_COLOR,
            (
                assist_chip_tokens::FLAT_DISABLED_OUTLINE_COLOR,
                assist_chip_tokens::FLAT_DISABLED_OUTLINE_OPACITY,
            ),
        ),
    };
    if disabled {
        faded(scheme, fade.0, fade.1, surface)
    } else {
        (Rgb::of(scheme, enabled), 0)
    }
}

/// The radius of `cell`'s corners at rest, at most half its height: the container's shape for the assist and suggestion
/// chips, which do not morph, and `ChipsTokens`' unselected or selected shape for the filter and input chips (the
/// expressive chips take their corners from there: `Chip.kt` `FilterChipDefaults.shapes()`, line 2590, and
/// `InputChipDefaults.shapes()`, line 2879).
fn radius(cell: ChipCell) -> Points {
    let half = cell.kind.height().0 / 2.0;
    let shape = match (cell.kind, cell.selection) {
        (ChipKind::Filter | ChipKind::Input, Selection::Selected) => chips_tokens::SELECTED_SHAPE,
        (ChipKind::Filter | ChipKind::Input, _) => chips_tokens::UNSELECTED_SHAPE,
        (ChipKind::Suggestion, _) => suggestion_chip_tokens::CONTAINER_SHAPE,
        _ => assist_chip_tokens::CONTAINER_SHAPE,
    };
    match shape {
        Shape::Circle => Points(half),
        Shape::Rounded(corners) => Points(corners.top_start.0.min(half)),
        // Square corners: a rectangle, or no shape.
        _ => Points(0.0),
    }
}

/// The largest `i` such that the pixel `i` points from a corner along its diagonal is wholly outside a corner of
/// `radius`: the arc is centered `radius` from each edge, and the pixel's corner nearest the center, `i + 1` from each
/// edge, is farther from it than the radius. A rectangle's corner has none.
fn outside_index(radius: Points) -> Option<Points> {
    let r = radius.0;
    (0_u16..64)
        .filter(|i| {
            let nearest = r - f32::from(*i) - 1.0;
            nearest > 0.0 && nearest * SQRT_2 > r
        })
        .max()
        .map(|i| Points(f32::from(i)))
}

/// The smallest `i` such that the pixel `i` points from a corner along its diagonal is wholly inside a corner of
/// `radius`: its corner farthest from the arc's center, `i` from each edge, is nearer to it than the radius.
fn inside_index(radius: Points) -> Option<Points> {
    let r = radius.0;
    (0_u16..64)
        .find(|i| (r - f32::from(*i)) * SQRT_2 < r)
        .map(|i| Points(f32::from(i)))
}

/// The pixels the checks know for the chip `cell` in a screenshot in `scheme`, with the mouse over it or not.
fn probes(cell: ChipCell, scheme: &ColorScheme, hovered: bool) -> Vec<Probe> {
    let (x, y) = cell.origin();
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let selected = cell.selection == Selection::Selected;
    let radius = radius(cell);
    let (container, container_tolerance) = container(cell, scheme, hovered);
    let probe = |what, (x, y): (Points, Points), (expected, tolerance)| Probe {
        cell,
        what,
        x,
        y,
        expected,
        tolerance,
    };
    let from = |dx: Points, dy: Points| (Points(x.0 + dx.0), Points(y.0 + dy.0));
    let mut probes = vec![probe(
        "container",
        from(INSET, Points(cell.kind.height().0 / 2.0)),
        (container, container_tolerance),
    )];
    if let Some(n) = outside_index(radius) {
        // A corner drawn square, or less round than its token, fills it.
        probes.push(probe("outside the corner", from(n, n), (surface, 0)));
    }
    // The first pixel row of the top edge, past the end of the top left corner's arc and the same distance again as the
    // container's color is read from the edge, where the edge is straight: a browser antialiases a 1-point line along an
    // arc. Every chip is wider than that (its padding and its label). The outline is 1 point wide, drawn inside the
    // edge, and the state layer lies inside it, so hovering leaves it as it is. A selected chip has a transparent
    // border (`Chip.kt` lines 2427 and 2848), which draws nothing: the container reaches the edge.
    let edge = from(Points(radius.0 + CONTAINER_INSET.0), Points(0.0));
    if selected {
        probes.push(probe("no outline", edge, (container, container_tolerance)));
        // Wholly inside the corner, where a container rounder than its token leaves the surface. Not for a chip with an
        // avatar: the pixel is the one read outside the avatar's circle, below.
        if !cell.content.has_avatar()
            && let Some(inside) = inside_index(radius)
        {
            probes.push(probe(
                "inside the corner",
                from(inside, inside),
                (container, container_tolerance),
            ));
        }
    } else {
        probes.push(probe("outline", edge, outline_color(cell, scheme)));
    }
    if let (Some((lx, ly)), Some(side)) = (cell.leading_origin(), cell.leading_side()) {
        let middle = (Points(lx.0 + side.0 / 2.0), Points(ly.0 + side.0 / 2.0));
        if cell.content.has_avatar() {
            probes.push(probe("avatar", middle, avatar_color(cell, scheme)));
            // One point in from the corner of the avatar's box, outside its circle (the box's corner pixel is nearer the
            // chip's outline than the arcs allow to read), where the container shows: an avatar not clipped to a circle
            // fills it.
            probes.push(probe(
                "outside the avatar",
                (Points(lx.0 + 1.0), Points(ly.0 + 1.0)),
                (container, container_tolerance),
            ));
        } else {
            // The middle of the icon, `catalog::ICON`, which is filled there.
            probes.push(probe("icon", middle, icon_color(cell, scheme)));
        }
    }
    probes
}

/// The pixels the checks know in `shot`, for every chip of the catalog.
fn shot_probes(shot: Shot) -> Vec<Probe> {
    let scheme = shot.appearance.scheme();
    ChipCell::all()
        .flat_map(|cell| probes(cell, &scheme, shot.hovers(cell)))
        .collect()
}

/// How many pixels a point is: the image is the chips' viewport at a whole scale (1 on a desktop, 2 on a high-density
/// screen).
fn scale(image: &Image) -> Result<u32, String> {
    let (width, height) = chips::viewport();
    let scale = image.width / points_to_u32(width).max(1);
    if scale == 0
        || image.width != scale * points_to_u32(width)
        || image.height != scale * points_to_u32(height)
    {
        return Err(format!(
            "the image is {}x{} pixels, not the chips' {}x{} points at a whole scale",
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

/// Checks the chips' screenshots in `directory`, as the test script saves them, and returns an error for the first one
/// it cannot read or the pixels that are not the tokens' colors.
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

/// The result `before` of the other checks in `directory`, joined with the chips': both run, so that a failure of one
/// does not hide the other's, and their mismatches add up. Another failure (a screenshot that cannot be read) is the
/// first one's.
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
    use catalog::chips::KINDS;

    use super::*;

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;

    fn cell(kind: ChipKind, selection: Selection, content: ChipContent) -> ChipCell {
        ChipCell::new(kind, selection, content, Availability::Enabled)
    }

    // `ChipCell` is non-exhaustive: another crate builds one with `new`.
    fn disabled(cell: ChipCell) -> ChipCell {
        ChipCell::new(
            cell.kind,
            cell.selection,
            cell.content,
            Availability::Disabled,
        )
    }

    fn with_selection(cell: ChipCell, selection: Selection) -> ChipCell {
        ChipCell::new(cell.kind, selection, cell.content, cell.availability)
    }

    fn rgb(role: ColorRole) -> Rgb {
        Rgb::of(&LIGHT, role)
    }

    #[test]
    fn the_hovered_chips_are_the_label_only_ones_of_each_kind_and_selection() {
        let names: Vec<String> = HOVERED.into_iter().map(ChipCell::test_id).collect();
        // The ids `chips.test.tsx` hovers.
        assert_eq!(
            names,
            [
                "chip-assist-unselected-label-enabled",
                "chip-filter-unselected-label-enabled",
                "chip-filter-selected-label-enabled",
                "chip-input-unselected-label-enabled",
                "chip-input-selected-label-enabled",
                "chip-suggestion-unselected-label-enabled",
            ]
        );
        // None is missing: the enabled chip that shows only its label, for each selection of each kind.
        let wanted: Vec<ChipCell> = KINDS
            .into_iter()
            .flat_map(|kind| {
                kind.selections().iter().copied().map(move |selection| {
                    ChipCell::new(kind, selection, ChipContent::Label, Availability::Enabled)
                })
            })
            .collect();
        assert_eq!(HOVERED.to_vec(), wanted);
        for cell in HOVERED {
            assert!(ChipCell::all().any(|c| c == cell), "{cell:?}");
        }
    }

    #[test]
    fn containers_are_the_tokens_colors() {
        let surface = rgb(ColorRole::Surface);
        let secondary = rgb(ColorRole::SecondaryContainer);
        // Selected, the filter and input chips have the secondary container's color.
        for kind in [ChipKind::Filter, ChipKind::Input] {
            let selected = cell(kind, Selection::Selected, ChipContent::Label);
            assert_eq!(
                container(selected, &LIGHT, false),
                (Rgb([0xe8, 0xde, 0xf8]), 0)
            );
            assert_eq!(rgb(ColorRole::SecondaryContainer), Rgb([0xe8, 0xde, 0xf8]));
            // Disabled: on surface at 0.12 over the surface.
            assert_eq!(
                container(disabled(selected), &LIGHT, false),
                (
                    rgb(ColorRole::OnSurface).over(0.12, surface),
                    crate::BLEND_TOLERANCE
                )
            );
            // Hovered: the label's color at 0.08 over the container; a disabled chip shows no hover.
            assert_eq!(
                container(selected, &LIGHT, true),
                (
                    rgb(ColorRole::OnSecondaryContainer).over(0.08, secondary),
                    crate::BLEND_TOLERANCE
                )
            );
            assert_eq!(
                container(disabled(selected), &LIGHT, true),
                container(disabled(selected), &LIGHT, false)
            );
            // Not selected: no container, the surface; hovered, the label's color over it.
            let unselected = cell(kind, Selection::Unselected, ChipContent::Label);
            assert_eq!(container(unselected, &LIGHT, false), (surface, 0));
            assert_eq!(container(disabled(unselected), &LIGHT, false), (surface, 0));
            assert_eq!(
                container(unselected, &LIGHT, true),
                (
                    rgb(ColorRole::OnSurfaceVariant).over(0.08, surface),
                    crate::BLEND_TOLERANCE
                )
            );
        }
        // The assist and suggestion chips have none, selected or not.
        let assist = cell(ChipKind::Assist, Selection::Unselected, ChipContent::Label);
        assert_eq!(container(assist, &LIGHT, false), (surface, 0));
        assert_eq!(
            container(assist, &LIGHT, true),
            (
                rgb(ColorRole::OnSurface).over(0.08, surface),
                crate::BLEND_TOLERANCE
            )
        );
        let suggestion = cell(
            ChipKind::Suggestion,
            Selection::Unselected,
            ChipContent::Label,
        );
        assert_eq!(
            container(suggestion, &LIGHT, true),
            (
                rgb(ColorRole::OnSurfaceVariant).over(0.08, surface),
                crate::BLEND_TOLERANCE
            )
        );
    }

    #[test]
    fn the_state_layer_takes_each_kinds_label_color() {
        let roles: Vec<ColorRole> = HOVERED.into_iter().map(state_role).collect();
        assert_eq!(
            roles,
            [
                ColorRole::OnSurface,
                ColorRole::OnSurfaceVariant,
                ColorRole::OnSecondaryContainer,
                ColorRole::OnSurfaceVariant,
                ColorRole::OnSecondaryContainer,
                ColorRole::OnSurfaceVariant,
            ]
        );
        // The chips' ripple takes the label's color (`Chip.kt` reads none of the `Hover` tokens): the hovered label tokens
        // name the same roles as the labels' do, and the checks read the former where the components read the latter.
        assert_eq!(
            assist_chip_tokens::HOVER_LABEL_TEXT_COLOR,
            assist_chip_tokens::LABEL_TEXT_COLOR
        );
        assert_eq!(
            suggestion_chip_tokens::HOVER_LABEL_TEXT_COLOR,
            suggestion_chip_tokens::LABEL_TEXT_COLOR
        );
        assert_eq!(
            filter_chip_tokens::UNSELECTED_HOVER_LABEL_TEXT_COLOR,
            filter_chip_tokens::UNSELECTED_LABEL_TEXT_COLOR
        );
        assert_eq!(
            filter_chip_tokens::SELECTED_HOVER_LABEL_TEXT_COLOR,
            filter_chip_tokens::SELECTED_LABEL_TEXT_COLOR
        );
        assert_eq!(
            input_chip_tokens::UNSELECTED_HOVER_LABEL_TEXT_COLOR,
            input_chip_tokens::UNSELECTED_LABEL_TEXT_COLOR
        );
        assert_eq!(
            input_chip_tokens::SELECTED_HOVER_LABEL_TEXT_COLOR,
            input_chip_tokens::SELECTED_LABEL_TEXT_COLOR
        );
    }

    #[test]
    fn the_icon_takes_each_kinds_color_and_fades_over_the_container_when_disabled() {
        let label = ChipContent::LeadingIcon;
        let expected = [
            (ChipKind::Assist, Selection::Unselected, ColorRole::Primary),
            (
                ChipKind::Suggestion,
                Selection::Unselected,
                ColorRole::Primary,
            ),
            (
                ChipKind::Filter,
                Selection::Unselected,
                ColorRole::OnSurfaceVariant,
            ),
            (
                ChipKind::Filter,
                Selection::Selected,
                ColorRole::OnSecondaryContainer,
            ),
            (
                ChipKind::Input,
                Selection::Unselected,
                ColorRole::OnSurfaceVariant,
            ),
            (ChipKind::Input, Selection::Selected, ColorRole::Primary),
        ];
        for (kind, selection, role) in expected {
            assert_eq!(
                icon_color(cell(kind, selection, label), &LIGHT),
                (rgb(role), 0),
                "{kind:?} {selection:?}"
            );
        }
        // The unselected filter chip's icon is `Chip.kt`'s, not its token's.
        assert_eq!(
            filter_chip_tokens::UNSELECTED_LEADING_ICON_COLOR,
            ColorRole::Primary
        );
        // Disabled: on surface at 0.38, over the surface or, selected, over the container at 0.12 of it.
        let surface = rgb(ColorRole::Surface);
        let on_surface = rgb(ColorRole::OnSurface);
        let blend = crate::BLEND_TOLERANCE;
        for kind in KINDS {
            assert_eq!(
                icon_color(disabled(cell(kind, Selection::Unselected, label)), &LIGHT),
                (on_surface.over(0.38, surface), blend),
                "{kind:?}"
            );
        }
        let container = on_surface.over(0.12, surface);
        for kind in [ChipKind::Filter, ChipKind::Input] {
            assert_eq!(
                icon_color(disabled(cell(kind, Selection::Selected, label)), &LIGHT),
                (on_surface.over(0.38, container), 2 * blend),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn the_avatar_is_the_catalogs_color_and_fades_over_the_container_when_disabled() {
        let avatar = cell(ChipKind::Input, Selection::Unselected, ChipContent::Avatar);
        let surface = rgb(ColorRole::Surface);
        // The tertiary color: the baseline's, which none of the chips' colors is.
        assert_eq!(AVATAR_ROLE, ColorRole::Tertiary);
        assert_eq!(avatar_color(avatar, &LIGHT), (Rgb([0x7d, 0x52, 0x60]), 0));
        assert_eq!(
            avatar_color(disabled(avatar), &LIGHT),
            (rgb(AVATAR_ROLE).over(0.38, surface), crate::BLEND_TOLERANCE)
        );
        // Over the container of a selected chip, itself faded.
        let selected = with_selection(avatar, Selection::Selected);
        assert_eq!(
            avatar_color(disabled(selected), &LIGHT),
            (
                rgb(AVATAR_ROLE).over(0.38, rgb(ColorRole::OnSurface).over(0.12, surface)),
                2 * crate::BLEND_TOLERANCE
            )
        );
        for role in [
            ColorRole::Surface,
            ColorRole::SecondaryContainer,
            ColorRole::Primary,
            ColorRole::OnSurface,
            ColorRole::OnSurfaceVariant,
            ColorRole::OnSecondaryContainer,
            ColorRole::OutlineVariant,
        ] {
            assert_ne!(rgb(role), rgb(AVATAR_ROLE), "{role:?}");
        }
    }

    #[test]
    fn outlines_are_the_kinds_outline_tokens() {
        let surface = rgb(ColorRole::Surface);
        for kind in KINDS {
            let chip = cell(kind, Selection::Unselected, ChipContent::Label);
            assert_eq!(
                outline_color(chip, &LIGHT),
                (rgb(ColorRole::OutlineVariant), 0),
                "{kind:?}"
            );
            // Disabled: on surface at 0.12 over the surface.
            assert_eq!(
                outline_color(disabled(chip), &LIGHT),
                (
                    rgb(ColorRole::OnSurface).over(0.12, surface),
                    crate::BLEND_TOLERANCE
                ),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn corners_are_the_shapes_radius_at_most_half_the_height() {
        let radii: Vec<f32> = [
            (ChipKind::Assist, Selection::Unselected),
            (ChipKind::Suggestion, Selection::Unselected),
            (ChipKind::Filter, Selection::Unselected),
            (ChipKind::Filter, Selection::Selected),
            (ChipKind::Input, Selection::Unselected),
            (ChipKind::Input, Selection::Selected),
        ]
        .into_iter()
        .map(|(kind, selection)| radius(cell(kind, selection, ChipContent::Label)).0)
        .collect();
        // Small corners (8) for the assist and suggestion chips; for the filter and input chips medium ones (12) and,
        // selected, fully round: half their 32 points.
        assert_eq!(radii, [8.0, 8.0, 12.0, 16.0, 12.0, 16.0]);
    }

    /// The points of the unit pixel `i` along a corner's diagonal, on a fine grid, that are inside the arc's circle of
    /// `radius`, centered `radius` from both edges.
    fn samples_inside(radius: f32, i: u16) -> (usize, usize) {
        let mut inside = 0;
        let mut total = 0;
        for a in 0..=16_u16 {
            for b in 0..=16_u16 {
                let x = f32::from(i) + f32::from(a) / 16.0;
                let y = f32::from(i) + f32::from(b) / 16.0;
                let distance = ((radius - x).powi(2) + (radius - y).powi(2)).sqrt();
                if distance <= radius {
                    inside += 1;
                }
                total += 1;
            }
        }
        (inside, total)
    }

    #[test]
    fn the_corner_pixels_are_wholly_outside_or_inside_the_arc() {
        for (radius, outside, inside) in [(8.0, 1, 3), (12.0, 2, 4), (16.0, 3, 5)] {
            assert_eq!(
                outside_index(Points(radius)),
                Some(Points(f32::from(outside)))
            );
            assert_eq!(
                inside_index(Points(radius)),
                Some(Points(f32::from(inside)))
            );
            // Every sample of the outside pixel is outside the circle, and the next pixel has some inside it: it is the
            // last wholly outside.
            assert_eq!(samples_inside(radius, outside).0, 0, "{radius}");
            assert!(samples_inside(radius, outside + 1).0 > 0, "{radius}");
            // Every sample of the inside pixel is inside, and the one before it has some outside: the first wholly inside.
            let (count, total) = samples_inside(radius, inside);
            assert_eq!(count, total, "{radius}");
            let (count, total) = samples_inside(radius, inside - 1);
            assert!(count < total, "{radius}");
        }
        // A rectangle's corner has no pixel outside it, and no corner cut for one to be inside of.
        assert_eq!(outside_index(Points(0.0)), None);
    }

    #[test]
    fn probes_read_the_container_the_corner_the_outline_and_the_icon() {
        // An unselected assist chip with an icon: small corners, an outline, an icon.
        let assist = cell(
            ChipKind::Assist,
            Selection::Unselected,
            ChipContent::LeadingIcon,
        );
        let (x, y) = assist.origin();
        let found = probes(assist, &LIGHT, false);
        let surface = rgb(ColorRole::Surface);
        assert_eq!(
            found.iter().map(|p| p.what).collect::<Vec<_>>(),
            ["container", "outside the corner", "outline", "icon"]
        );
        assert_eq!(
            (found[0].x, found[0].y),
            (Points(x.0 + 2.0), Points(y.0 + 16.0))
        );
        assert_eq!(found[0].expected, surface);
        assert_eq!(
            (found[1].x, found[1].y),
            (Points(x.0 + 1.0), Points(y.0 + 1.0))
        );
        assert_eq!(found[1].expected, surface);
        // The edge, 8 points of arc and 4 more in.
        assert_eq!((found[2].x, found[2].y), (Points(x.0 + 12.0), y));
        assert_eq!(found[2].expected, rgb(ColorRole::OutlineVariant));
        // The icon's middle: 8 points of padding and half of 18.
        assert_eq!(
            (found[3].x, found[3].y),
            (Points(x.0 + 17.0), Points(y.0 + 16.0))
        );
        assert_eq!(found[3].expected, rgb(ColorRole::Primary));
        assert!(found.iter().all(|p| p.cell == assist));
        // Hovering lays the state layer inside the outline, not over it, and under the icon.
        let hovered = probes(assist, &LIGHT, true);
        assert_ne!(hovered[0], found[0]);
        assert_eq!(hovered[1..], found[1..]);

        // A selected filter chip: fully round, no outline, the container to its edge and inside its corner.
        let selected = cell(ChipKind::Filter, Selection::Selected, ChipContent::Label);
        let (x, y) = selected.origin();
        let found = probes(selected, &LIGHT, false);
        assert_eq!(
            found.iter().map(|p| p.what).collect::<Vec<_>>(),
            [
                "container",
                "outside the corner",
                "no outline",
                "inside the corner"
            ]
        );
        assert_eq!(
            (found[1].x, found[1].y),
            (Points(x.0 + 3.0), Points(y.0 + 3.0))
        );
        assert_eq!(found[1].expected, surface);
        // Half the height of arc, and 4 more in.
        assert_eq!((found[2].x, found[2].y), (Points(x.0 + 20.0), y));
        assert_eq!(found[2].expected, rgb(ColorRole::SecondaryContainer));
        assert_eq!(
            (found[3].x, found[3].y),
            (Points(x.0 + 5.0), Points(y.0 + 5.0))
        );
        assert_eq!(found[3].expected, rgb(ColorRole::SecondaryContainer));
        // The unselected one is less round and has an outline in place of the container's edge.
        let unselected = cell(ChipKind::Filter, Selection::Unselected, ChipContent::Label);
        let found = probes(unselected, &LIGHT, false);
        let (x, y) = unselected.origin();
        assert_eq!(
            (found[1].x, found[1].y),
            (Points(x.0 + 2.0), Points(y.0 + 2.0))
        );
        assert_eq!((found[2].x, found[2].y), (Points(x.0 + 16.0), y));
        assert_eq!(found[2].what, "outline");
        assert_eq!(found.len(), 3);
    }

    #[test]
    fn an_input_chip_with_an_avatar_reads_the_avatar_and_the_container_beside_it() {
        let chip = cell(
            ChipKind::Input,
            Selection::Selected,
            ChipContent::AvatarAndTrailingIcon,
        );
        let (x, y) = chip.origin();
        let found = probes(chip, &LIGHT, false);
        assert_eq!(
            found.iter().map(|p| p.what).collect::<Vec<_>>(),
            [
                "container",
                "outside the corner",
                "no outline",
                "avatar",
                "outside the avatar"
            ]
        );
        // The avatar's middle: 4 points of padding and half of 24, centered on the height.
        assert_eq!(
            (found[3].x, found[3].y),
            (Points(x.0 + 16.0), Points(y.0 + 16.0))
        );
        assert_eq!(found[3].expected, rgb(ColorRole::Tertiary));
        // One point in from the avatar's box, which is 4 points from the chip's corner.
        assert_eq!(
            (found[4].x, found[4].y),
            (Points(x.0 + 5.0), Points(y.0 + 5.0))
        );
        assert_eq!(found[4].expected, rgb(ColorRole::SecondaryContainer));
        // An unselected input chip with an avatar reads the same pixel for the avatar's corner, and no inside pixel.
        let unselected = with_selection(chip, Selection::Unselected);
        let found = probes(unselected, &LIGHT, false);
        assert_eq!(found[3].what, "avatar");
        assert_eq!(found[4].what, "outside the avatar");
        assert_eq!(found[4].expected, rgb(ColorRole::Surface));
        // The trailing icon is not read: it is after the label, whose width the font gives.
        assert_eq!(found.len(), 5);
    }

    #[test]
    fn every_chip_is_read_where_its_cell_says() {
        for cell in ChipCell::all() {
            let (x, y) = cell.origin();
            let found = probes(cell, &LIGHT, false);
            assert!(found.len() >= 3, "{cell:?}");
            for probe in &found {
                // Inside the chip's column and its row.
                assert!(probe.x.0 >= x.0 && probe.x.0 < x.0 + chips::COLUMN_WIDTH.0);
                assert!(probe.y.0 >= y.0 && probe.y.0 < y.0 + cell.kind.height().0);
                // Inside the chips' viewport, below the buttons' one, which holds no chip.
                assert!(probe.y.0 >= catalog::grid::viewport().1.0, "{probe:?}");
                assert!(probe.y.0 < chips::viewport().1.0, "{probe:?}");
                assert_eq!(probe.cell, cell);
            }
            // One probe per pixel: two would hide each other's mismatch.
            for (i, a) in found.iter().enumerate() {
                for b in &found[i + 1..] {
                    assert_ne!(
                        (a.x, a.y),
                        (b.x, b.y),
                        "{cell:?}: {} and {}",
                        a.what,
                        b.what
                    );
                }
            }
        }
    }

    /// The catalog painted as the tokens say, at `scale`: every probe of `shot` holds its expected color, and the rest
    /// a color no probe expects.
    fn painted(shot: Shot, scale: u32) -> Image {
        let (width, height) = chips::viewport();
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

    /// `image` as an RGBA PNG with the given compression: none for a painted catalog, which the unoptimized test build
    /// writes quickly, a light one for a blank page, which makes a small file.
    fn encode(image: &Image, compression: png::Compression) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, image.width, image.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(compression);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&image.rgba).unwrap();
        writer.finish().unwrap();
        bytes
    }

    fn encode_uncompressed(image: &Image) -> Vec<u8> {
        encode(image, png::Compression::NoCompression)
    }

    fn probe(cell: ChipCell, scheme: &ColorScheme, hovered: bool, what: &str) -> Probe {
        probes(cell, scheme, hovered)
            .into_iter()
            .find(|probe| probe.what == what)
            .unwrap_or_else(|| panic!("no probe named {what}"))
    }

    #[test]
    fn the_shots_are_the_test_script_files() {
        let names: Vec<String> = Shot::all().into_iter().map(Shot::file_name).collect();
        assert_eq!(
            names,
            [
                "chips-light-rest.png",
                "chips-light-hover-assist.png",
                "chips-light-hover-filter.png",
                "chips-light-hover-filter-selected.png",
                "chips-light-hover-input.png",
                "chips-light-hover-input-selected.png",
                "chips-light-hover-suggestion.png",
                "chips-dark-rest.png",
                "chips-dark-hover-assist.png",
                "chips-dark-hover-filter.png",
                "chips-dark-hover-filter-selected.png",
                "chips-dark-hover-input.png",
                "chips-dark-hover-input-selected.png",
                "chips-dark-hover-suggestion.png",
            ]
        );
        assert_eq!(Shot::all()[0].appearance.scheme(), LIGHT);
        assert_eq!(
            Shot::all()[7].appearance.scheme(),
            ColorScheme::BASELINE_DARK
        );
        // The script saves these files, over these chips, in this viewport.
        let script = include_str!("../../../examples/catalog/chips.test.tsx");
        let (width, height) = chips::viewport();
        assert!(
            script.contains(&format!(
                "const VIEWPORT = {{ width: {}, height: {} }};",
                width.0, height.0
            )),
            "the script's viewport is not {}x{}",
            width.0,
            height.0
        );
        // Each hovered chip, by its test id, with the end of its file name.
        for (cell, shot) in HOVERED.into_iter().zip(&Shot::all()[1..7]) {
            let name = shot
                .file_name()
                .strip_prefix("chips-light-hover-")
                .and_then(|name| name.strip_suffix(".png"))
                .unwrap()
                .to_owned();
            assert!(
                script.contains(&format!(
                    "{{ id: \"{}\", name: \"{name}\" }}",
                    cell.test_id()
                )),
                "{cell:?}"
            );
        }
        assert!(script.contains("`target/visual/chips-${scheme}-rest.png`"));
        assert!(script.contains("`target/visual/chips-${scheme}-hover-${name}.png`"));
        assert!(script.contains("`id:${id}`"));
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
    fn hovering_changes_only_the_hovered_chips_container_and_what_it_fills() {
        let rest = Shot::all()[0];
        for hover in &Shot::all()[1..7] {
            let hovered = hover.hovered.unwrap();
            let changed: Vec<(ChipCell, &str)> = shot_probes(rest)
                .into_iter()
                .zip(shot_probes(*hover))
                .filter(|(a, b)| a != b)
                .map(|(a, b)| {
                    assert_eq!((a.cell, a.what, a.x, a.y), (b.cell, b.what, b.x, b.y));
                    (b.cell, b.what)
                })
                .collect();
            // Only the hovered chip changes: its state layer is the container's color, which a selected chip's edge and
            // corner pixels show too; an unselected chip's outline and corner lie outside the layer.
            let expected: &[&str] = match hovered.selection {
                Selection::Selected => &["container", "no outline", "inside the corner"],
                _ => &["container"],
            };
            assert_eq!(
                changed,
                expected
                    .iter()
                    .map(|what| (hovered, *what))
                    .collect::<Vec<_>>(),
                "{hovered:?}"
            );
        }
    }

    #[test]
    fn one_wrong_pixel_fails_and_is_named() {
        let shot = Shot::all()[1];
        let mut image = painted(shot, 1);
        let hovered = shot.hovered.unwrap();
        let wanted = probe(hovered, &LIGHT, true, "container");
        let (x, y) = pixel_of(&wanted, 1);
        let start = ((y * image.width + x) * 4) as usize;
        // The surface where the mouse is over the chip: its state layer is missing.
        image.rgba[start..start + 3].copy_from_slice(&Rgb::of(&LIGHT, ColorRole::Surface).0);
        let found = check(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe, wanted);
        assert_eq!(found[0].found, Some(Rgb::of(&LIGHT, ColorRole::Surface)));
        // The first chip starts at the grid's margin (24) and the block at 2688; its container is read 2 points in and
        // at mid height, 16 points down.
        assert_eq!(
            found[0].to_string(),
            format!(
                "chip-assist-unselected-label-enabled container at (26, 2704) points: expected {} (± 2), found {}",
                wanted.expected,
                Rgb::of(&LIGHT, ColorRole::Surface)
            )
        );
        // The same image is not the dark shot.
        assert_ne!(check(&image, Shot::all()[8]).unwrap(), Vec::new());
    }

    #[test]
    fn a_square_corner_a_missing_outline_and_an_avatar_that_is_not_round_fail() {
        let shot = Shot::all()[0];
        let wrong = |cell: ChipCell, what: &str, paint: Rgb| {
            let mut image = painted(shot, 1);
            let wanted = probe(cell, &LIGHT, false, what);
            let (x, y) = pixel_of(&wanted, 1);
            let start = ((y * image.width + x) * 4) as usize;
            image.rgba[start..start + 3].copy_from_slice(&paint.0);
            let found = check(&image, shot).unwrap();
            assert_eq!(found.len(), 1, "{cell:?} {what}");
            assert_eq!(found[0].probe, wanted, "{cell:?} {what}");
        };
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let container = Rgb::of(&LIGHT, ColorRole::SecondaryContainer);
        // A corner drawn square fills the pixel outside the arc: with the container's color, or with the outline's.
        let assist = cell(ChipKind::Assist, Selection::Unselected, ChipContent::Label);
        wrong(
            assist,
            "outside the corner",
            Rgb::of(&LIGHT, ColorRole::OutlineVariant),
        );
        // An outline missing from an unselected chip, and one drawn on a selected chip that has none.
        wrong(assist, "outline", surface);
        let selected = cell(ChipKind::Filter, Selection::Selected, ChipContent::Label);
        wrong(
            selected,
            "no outline",
            Rgb::of(&LIGHT, ColorRole::OutlineVariant),
        );
        // A container less round than the token leaves the surface inside the corner of a selected chip.
        wrong(selected, "inside the corner", surface);
        // An avatar that fills its box fills the pixel at its corner; a missing one leaves the container.
        let input = cell(ChipKind::Input, Selection::Selected, ChipContent::Avatar);
        wrong(input, "outside the avatar", Rgb::of(&LIGHT, AVATAR_ROLE));
        wrong(input, "avatar", container);
        // An icon of another color.
        let icon = cell(
            ChipKind::Assist,
            Selection::Unselected,
            ChipContent::LeadingIcon,
        );
        wrong(icon, "icon", surface);
    }

    #[test]
    fn an_image_of_another_size_is_refused() {
        // The buttons' viewport is shorter than the chips': a screenshot of it holds no chip.
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
            std::env::temp_dir().join(format!("visual-checks-chips-{name}-{}", std::process::id()));
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
                .contains("chips-light-rest.png: ok")
        );
        // The rest screenshot saved as the hovered one: the state layer is missing.
        fs::copy(
            directory.join("chips-light-rest.png"),
            directory.join("chips-light-hover-assist.png"),
        )
        .unwrap();
        let mut out = Vec::new();
        assert!(
            matches!(run_shots(&directory, shots, &mut out), Err(Error::Mismatches(n)) if n > 0)
        );
        let out = String::from_utf8(out).unwrap();
        assert!(out.contains("chips-light-hover-assist.png: FAILED"));
        assert!(out.contains("chip-assist-unselected-label-enabled container"));
        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn the_chips_checks_join_the_buttons_without_hiding_either() {
        // An empty directory: no screenshot to read, so the chips' checks fail on reading.
        let directory = directory("also");
        let mut out = Vec::new();
        assert!(matches!(
            also(Ok(()), &directory, &mut out),
            Err(Error::Read { .. })
        ));
        // The buttons' own failure comes first, and the chips' reading failure does not replace it.
        assert!(matches!(
            also(Err(Error::Mismatches(3)), &directory, &mut out),
            Err(Error::Mismatches(3))
        ));
        assert!(matches!(
            also(Err(Error::Usage), &directory, &mut out),
            Err(Error::Usage)
        ));
        // Both with wrong pixels: the counts add up. Every screenshot is a blank page of the right size, written once.
        let (width, height) = chips::viewport();
        let blank = Image {
            width: points_to_u32(width),
            height: points_to_u32(height),
            rgba: vec![255; (points_to_u32(width) * points_to_u32(height) * 4) as usize],
        };
        let bytes = encode(&blank, png::Compression::Fast);
        for shot in Shot::all() {
            fs::write(directory.join(shot.file_name()), &bytes).unwrap();
        }
        let mut out = Vec::new();
        let Err(Error::Mismatches(chips_only)) = also(Ok(()), &directory, &mut out) else {
            panic!("blank pages are not the tokens' colors");
        };
        assert!(chips_only > 0);
        let mut out = Vec::new();
        assert!(matches!(
            also(Err(Error::Mismatches(7)), &directory, &mut out),
            Err(Error::Mismatches(n)) if n == chips_only + 7
        ));
        fs::remove_dir_all(&directory).unwrap();
    }
}
