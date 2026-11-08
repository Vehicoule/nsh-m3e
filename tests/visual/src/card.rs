//! The card's known pixels (ADR 0013): what `card.test.tsx` screenshots show of each card of the catalog
//! (`catalog::card`), at rest and with the mouse over the card that takes presses of each style.
//!
//! A card of the elevated style, and a filled one raised by the mouse, casts a shadow (ADR 0018), so a pixel is never
//! read at the box's corner of a card that casts one, or next to its edge: the container's color is read inside it,
//! the corner from inside its arc, and the surface beyond the shadows' reach. The shadow itself is read just below the
//! container: darker than the surface where a card casts one, the surface itself where it does not (the filled card at
//! rest, the outlined card always, which androidx keeps at level 0 even under the mouse, `Card.kt` lines 459 to 461,
//! ADR 0012).

use std::fmt;

use catalog::card::{CardCell, CardKind, STYLES};
use nsh_core::BoxShadow;
use nsh_core::geometry::Points;
use nsh_m3e::card::CardStyle;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::elevation::Elevation;
use nsh_m3e::tokens::Shape;
use nsh_m3e::tokens::androidx::{
    elevated_card_tokens, filled_card_tokens, outlined_card_tokens, state_tokens,
};

use super::{Appearance, Image, Mismatch, Probe, Rgb, Target, faded, pixel_of, points_to_u32};

/// One screenshot `examples/catalog/card.test.tsx` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Shot {
    pub(crate) appearance: Appearance,
    /// The card the mouse is over, if any.
    pub(crate) hovered: Option<CardCell>,
}

impl Shot {
    /// Every screenshot the script takes, in each appearance: the catalog at rest, then the mouse over the card that
    /// takes presses of each style.
    pub(crate) fn all() -> Vec<Self> {
        Appearance::ALL
            .into_iter()
            .flat_map(|appearance| {
                let shot = move |hovered| Self {
                    appearance,
                    hovered,
                };
                std::iter::once(shot(None)).chain(
                    STYLES
                        .into_iter()
                        .map(move |style| shot(Some(CardCell::new(style, CardKind::Clickable)))),
                )
            })
            .collect()
    }

    /// Its file name, as the test script saves it.
    pub(crate) fn file_name(self) -> String {
        let appearance = self.appearance.name();
        match self.hovered {
            None => format!("card-{appearance}-rest.png"),
            Some(cell) => format!("card-{appearance}-hover-{}.png", cell.style_name()),
        }
    }
}

/// The role of the container of a card of `style`, when it takes presses or is only a container.
fn container_role(style: CardStyle) -> ColorRole {
    match style {
        CardStyle::Elevated => elevated_card_tokens::CONTAINER_COLOR,
        CardStyle::Outlined => outlined_card_tokens::CONTAINER_COLOR,
        // `CardStyle` is non-exhaustive; the default style is the filled one.
        _ => filled_card_tokens::CONTAINER_COLOR,
    }
}

/// The container's color of the card `cell` and its tolerance, with the mouse over it or not: the on-surface color
/// laid over it at the hover opacity of the state layer, for a card that takes presses.
///
/// A disabled card is composited as androidx composites it (`Card.kt`, ADR 0012): the filled card's disabled color at
/// its opacity over its container color (lines 509 to 512); the elevated card's over `ElevatedCardTokens`' own disabled
/// color (lines 558 to 563), which is surface itself, so the disabled elevated card is the surface and only its shadow
/// shows it; the outlined card's is its container color, with no opacity (line 609).
fn container(cell: CardCell, scheme: &ColorScheme, hovered: bool) -> (Rgb, u8) {
    let rest = Rgb::of(scheme, container_role(cell.style));
    match (cell.kind, cell.style) {
        (CardKind::Disabled, CardStyle::Elevated) => faded(
            scheme,
            elevated_card_tokens::DISABLED_CONTAINER_COLOR,
            elevated_card_tokens::DISABLED_CONTAINER_OPACITY,
            Rgb::of(scheme, elevated_card_tokens::DISABLED_CONTAINER_COLOR),
        ),
        (CardKind::Disabled, CardStyle::Outlined) => (rest, 0),
        (CardKind::Disabled, _) => faded(
            scheme,
            filled_card_tokens::DISABLED_CONTAINER_COLOR,
            filled_card_tokens::DISABLED_CONTAINER_OPACITY,
            rest,
        ),
        (CardKind::Clickable, _) if hovered => faded(
            scheme,
            ColorRole::OnSurface,
            state_tokens::HOVER_STATE_LAYER_OPACITY,
            rest,
        ),
        _ => (rest, 0),
    }
}

/// The color of the first row of pixels of the card `cell`, along its top edge and in the middle of it, where the edge
/// is straight, and its tolerance: the outline's for the outlined card, the container's for the others, which have no
/// outline. The outline is drawn over the state layer, so the mouse does not change it.
///
/// The disabled outline is the outline color at its opacity composited over the elevated container's color, whatever
/// the card it is on (`outlinedCardBorder`, `Card.kt` lines 628 to 630, ADR 0012).
fn edge(cell: CardCell, scheme: &ColorScheme, hovered: bool) -> (Rgb, u8) {
    match (cell.style, cell.kind) {
        (CardStyle::Outlined, CardKind::Disabled) => faded(
            scheme,
            outlined_card_tokens::DISABLED_OUTLINE_COLOR,
            outlined_card_tokens::DISABLED_OUTLINE_OPACITY,
            Rgb::of(scheme, elevated_card_tokens::CONTAINER_COLOR),
        ),
        (CardStyle::Outlined, _) => (Rgb::of(scheme, outlined_card_tokens::OUTLINE_COLOR), 0),
        _ => container(cell, scheme, hovered),
    }
}

/// The radius of the corners of the card `cell`: its style's `ContainerShape`, which is `CornerMedium` for the three.
fn corner_radius(cell: CardCell) -> Points {
    let shape = match cell.style {
        CardStyle::Elevated => elevated_card_tokens::CONTAINER_SHAPE,
        CardStyle::Outlined => outlined_card_tokens::CONTAINER_SHAPE,
        _ => filled_card_tokens::CONTAINER_SHAPE,
    };
    let (width, height) = cell.frame();
    radius(shape, Points(width.0.min(height.0)))
}

/// The radius of the corners of `shape` on a box whose shorter side is `side`: no more than half of it, which is the
/// radius of a circle shape. A square shape has no radius.
fn radius(shape: Shape, side: Points) -> Points {
    let half = side.0 / 2.0;
    match shape {
        Shape::Circle => Points(half),
        Shape::Rounded(corners) => Points(corners.top_start.0.min(half)),
        _ => Points(0.0),
    }
}

/// How far inside the arc of a corner a probe reads the container's color: past the antialiasing of the arc's edge,
/// which spreads over about a point, and past the outline of the outlined card, which is 1 point wide and follows the
/// arc. Not a design value: where the check looks.
const ARC_MARGIN: f32 = 2.0;

/// How far past the nominal reach of a card's shadows the surface is read. The shadow is a Gaussian whose standard
/// deviation is half its blur (`BoxShadow::blur`): 3 points for the highest level's ambient layer (level 2, blur 6).
/// The reach is the spread plus the blur, so it ends 2 standard deviations past the shadow's box (the spread's edge),
/// and a point 8 past the reach is 14 points, 4.7 standard deviations, past that edge, where the shadow is nothing.
/// Worked out from the shadow's definition, not measured on a host. Not a design value: where the check looks.
const SHADOW_TAIL: Points = Points(8.0);

/// How far below a card's container its shadow is read: inside the reach of level 1 (5 points below, ADR 0018), the
/// lowest level a card casts, and close enough to the container that the shadow is darkest there: at 1 point, level 1
/// is about a quarter of black over the surface, which is several steps of a channel of the dark scheme's surface.
/// Not a design value: where the check looks.
const SHADOW_BELOW: Points = Points(1.0);

/// The highest level a card sits at, in any state: the elevated card under the mouse, whose tokens give level 2.
const HIGHEST_LEVEL: Elevation = Elevation::Level2;

/// How far the shadows of `level` reach past a container, to the sides and below it, in points: the largest, over its
/// layers, of the offset, spread and blur that point that way. `BoxShadow::blur` is twice the Gaussian's standard
/// deviation, so the shadow is nothing past it.
fn shadow_reach(level: Elevation) -> (Points, Points) {
    let layers: [BoxShadow; 2] = level.shadows();
    let side = layers
        .iter()
        .map(|l| l.offset_x.0.abs() + l.spread.0 + l.blur.0)
        .fold(0.0, f32::max);
    let below = layers
        .iter()
        .map(|l| l.offset_y.0 + l.spread.0 + l.blur.0)
        .fold(0.0, f32::max);
    (Points(side), Points(below))
}

/// How far from the container's left edge, at mid height, the surface is read beside a card: past the reach to the
/// sides of the highest level a card has, by [`SHADOW_TAIL`]. The catalog's gaps leave room for it.
fn surface_distance() -> Points {
    Points(shadow_reach(HIGHEST_LEVEL).0.0 + SHADOW_TAIL.0)
}

/// The first whole number of points, counted from a container's corner along both edges, whose pixel is inside the
/// arc of a corner of `radius` by at least [`ARC_MARGIN`]. The pixel `n` points along each edge has its middle at
/// `n + 0.5`, which is `(radius - n - 0.5) * sqrt(2)` points from the arc's center `(radius, radius)`, so it is
/// `radius` less that distance inside the arc.
fn inside_the_arc(radius: Points) -> Points {
    let inside = |n: u16| {
        let along = f32::from(n) + 0.5;
        radius.0 - (radius.0 - along) * std::f32::consts::SQRT_2 >= ARC_MARGIN
    };
    Points(f32::from((0..=u16::MAX).find(|n| inside(*n)).unwrap_or(0)))
}

/// Whether the card `cell` casts a shadow, with the mouse over it or not (`hovered` is only ever true for a card that
/// takes presses).
///
/// - The elevated card casts one in every state: level 1 at rest and disabled (`ElevatedCardTokens`
///   `DisabledContainerElevation`), level 2 under the mouse.
/// - The filled card casts one only under the mouse, at level 1.
/// - The outlined card never does: androidx keeps its pressed, focused and hovered elevations at the default one,
///   level 0, where its tokens say level 1 hovered (`Card.kt` lines 459 to 461, ADR 0012).
fn casts_a_shadow(cell: CardCell, hovered: bool) -> bool {
    match (cell.style, cell.kind) {
        (CardStyle::Elevated, _) => true,
        (CardStyle::Filled, CardKind::Clickable) => hovered,
        _ => false,
    }
}

/// The pixels the checks know for the card `cell` in a screenshot in `scheme`, with the mouse over it or not.
///
/// None is read at the box's corner of a card that casts a shadow, or next to its edge, where the shadow lies: the
/// container is drawn over its shadow, so what is read inside it is the same with one. The surface is read
/// [`surface_distance`] away, past the reach of the shadows; [`shot_shadows`] reads the shadow.
fn card_probes(cell: CardCell, scheme: &ColorScheme, hovered: bool) -> Vec<Probe> {
    let target = Target::Card(cell);
    let (x, y) = cell.origin();
    let (width, height) = cell.frame();
    let middle = Points(y.0 + height.0 / 2.0);
    let (container, container_tolerance) = container(cell, scheme, hovered);
    let (edge, edge_tolerance) = edge(cell, scheme, hovered);
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let corner = inside_the_arc(corner_radius(cell));
    let mut probes = vec![
        // An eighth of the width in from the left edge, at mid height: past the edge, and clear of the card's name,
        // which is in the top left corner, above the middle.
        Probe {
            target,
            what: "container",
            x: Points(x.0 + width.0 / 8.0),
            y: middle,
            expected: container,
            tolerance: container_tolerance,
        },
        // Beside the container, past the reach of the shadows, in the gap to the next card or the margin: the
        // surface, and nothing drawn around the container.
        Probe {
            target,
            what: "surface beside it",
            x: Points(x.0 - surface_distance().0),
            y: middle,
            expected: surface,
            tolerance: 0,
        },
        // Just inside the arc of the top left corner, on its diagonal: the container's color if the corner is no
        // rounder than its token says.
        Probe {
            target,
            what: "inside the corner",
            x: Points(x.0 + corner.0),
            y: Points(y.0 + corner.0),
            expected: container,
            tolerance: container_tolerance,
        },
        // The first row of pixels, in the middle of the top edge: the outline's color for the outlined card, and for
        // the others the container's, which shows that no outline is drawn on them.
        Probe {
            target,
            what: "top edge",
            x: Points(x.0 + width.0 / 2.0),
            y,
            expected: edge,
            tolerance: edge_tolerance,
        },
    ];
    if !casts_a_shadow(cell, hovered) {
        // The top left pixel of the box: outside the arc of the corner, so the surface. A corner drawn square would
        // fill it. Only where no shadow lies on it.
        probes.push(Probe {
            target,
            what: "outside the corner",
            x,
            y,
            expected: surface,
            tolerance: 0,
        });
    }
    probes
}

/// A pixel in the shadow of a card, which must be darker than the surface in every channel (ADR 0018).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Shadow {
    target: Target,
    x: Points,
    y: Points,
}

/// Where the shadow below the card `cell` is read: the middle of its width, [`SHADOW_BELOW`] below the container.
fn below(cell: CardCell) -> (Points, Points) {
    let (x, y) = cell.origin();
    let (width, height) = cell.frame();
    (
        Points(x.0 + width.0 / 2.0),
        Points(y.0 + height.0 + SHADOW_BELOW.0),
    )
}

/// The cards of `shot` that cast a shadow, and the ones that do not, below which the surface must be as it is.
fn casts(shot: Shot, cell: CardCell) -> bool {
    casts_a_shadow(cell, shot.hovered == Some(cell))
}

/// The pixels in the shadows of the cards of `shot`: below each card that casts one, darker than the surface.
fn shot_shadows(shot: Shot) -> Vec<Shadow> {
    CardCell::all()
        .filter(|cell| casts(shot, *cell))
        .map(|cell| {
            let (x, y) = below(cell);
            Shadow {
                target: Target::Card(cell),
                x,
                y,
            }
        })
        .collect()
}

/// The pixels the checks know in `shot` that no shadow is on, for every card: below the card that casts none, the
/// surface, exactly, which is what shows that it does not.
fn shot_unshadowed(shot: Shot) -> Vec<Probe> {
    let surface = Rgb::of(&shot.appearance.scheme(), ColorRole::Surface);
    CardCell::all()
        .filter(|cell| !casts(shot, *cell))
        .map(|cell| {
            let (x, y) = below(cell);
            Probe {
                target: Target::Card(cell),
                what: "no shadow below it",
                x,
                y,
                expected: surface,
                tolerance: 0,
            }
        })
        .collect()
}

/// The pixels the checks know in `shot`, for every card of the catalog.
fn shot_probes(shot: Shot) -> Vec<Probe> {
    let scheme = shot.appearance.scheme();
    CardCell::all()
        .flat_map(|cell| card_probes(cell, &scheme, shot.hovered == Some(cell)))
        .chain(shot_unshadowed(shot))
        .collect()
}

/// What a shadow found when it was not darker than the surface.
#[derive(Debug, Clone, PartialEq)]
struct ShadowMismatch {
    probe: Shadow,
    surface: Rgb,
    found: Option<Rgb>,
}

impl fmt::Display for ShadowMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let probe = &self.probe;
        write!(
            f,
            "{} shadow below it at ({}, {}) points: expected darker than {}",
            probe.target.test_id(),
            probe.x.0,
            probe.y.0,
            self.surface,
        )?;
        match self.found {
            Some(found) => write!(f, ", found {found}"),
            None => write!(f, ", found no pixel"),
        }
    }
}

/// Whether every channel of `color` is lower than `surface`'s.
fn darker_in_every_channel(color: Rgb, surface: Rgb) -> bool {
    color.0.iter().zip(surface.0).all(|(a, b)| *a < b)
}

/// Every shadow of `shot` that `image` does not show.
fn check_shadows(image: &Image, shot: Shot) -> Result<Vec<ShadowMismatch>, String> {
    let scale = image.scale_in(catalog::card::viewport())?;
    let surface = Rgb::of(&shot.appearance.scheme(), ColorRole::Surface);
    Ok(shot_shadows(shot)
        .into_iter()
        .filter_map(|probe| {
            let found = image.pixel(
                points_to_u32(probe.x) * scale,
                points_to_u32(probe.y) * scale,
            );
            (!found.is_some_and(|found| darker_in_every_channel(found, surface))).then_some(
                ShadowMismatch {
                    probe,
                    surface,
                    found,
                },
            )
        })
        .collect())
}

/// Every probe of `shot` that `image` does not match; an error if the image is not the catalog's card viewport at a
/// whole scale.
fn check_colors(image: &Image, shot: Shot) -> Result<Vec<Mismatch>, String> {
    let scale = image.scale_in(catalog::card::viewport())?;
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

/// What `image` does not show of `shot`: the pixels the tokens give and the shadows of the cards, one line each.
pub(crate) fn check(image: &Image, shot: Shot) -> Result<Vec<String>, String> {
    let colors = check_colors(image, shot)?;
    let shadows = check_shadows(image, shot)?;
    Ok(colors
        .iter()
        .map(ToString::to_string)
        .chain(shadows.iter().map(ToString::to_string))
        .collect())
}

/// The catalog painted as the tokens say, at `scale`: every probe of `shot` holds its expected color, a pixel in a
/// shadow is the surface a quarter darker, and the rest a color no probe expects.
#[cfg(test)]
pub(crate) fn painted(shot: Shot, scale: u32) -> Image {
    let (width, height) = catalog::card::viewport();
    let (width, height) = (points_to_u32(width) * scale, points_to_u32(height) * scale);
    let mut rgba = [1, 2, 3, 255].repeat((width * height) as usize);
    let mut paint = |x: Points, y: Points, color: Rgb| {
        let (x, y) = (points_to_u32(x) * scale, points_to_u32(y) * scale);
        let start = ((y * width + x) * 4) as usize;
        rgba[start..start + 3].copy_from_slice(&color.0);
    };
    for probe in shot_probes(shot) {
        paint(probe.x, probe.y, probe.expected);
    }
    let surface = Rgb::of(&shot.appearance.scheme(), ColorRole::Surface);
    for shadow in shot_shadows(shot) {
        paint(shadow.x, shadow.y, shaded(surface, 3, 4));
    }
    Image {
        width,
        height,
        rgba,
    }
}

/// `color` with each channel multiplied by `numerator / denominator`.
#[cfg(test)]
fn shaded(color: Rgb, numerator: u16, denominator: u16) -> Rgb {
    Rgb(color
        .0
        .map(|c| u8::try_from(u16::from(c) * numerator / denominator).unwrap_or(c)))
}

#[cfg(test)]
mod tests {
    use catalog::card::{GAP, HEIGHT, KINDS, WIDTH};
    use nsh_m3e::card::CardState;
    use nsh_m3e::tokens::androidx::shape_tokens;

    use super::*;
    use crate::BLEND_TOLERANCE;

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;

    fn cell(style: CardStyle, kind: CardKind) -> CardCell {
        CardCell::new(style, kind)
    }

    #[test]
    fn containers_are_the_style_tokens_colors() {
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let highest = Rgb::of(&LIGHT, ColorRole::SurfaceContainerHighest);
        let low = Rgb::of(&LIGHT, ColorRole::SurfaceContainerLow);
        for kind in [CardKind::Static, CardKind::Clickable] {
            assert_eq!(
                container(cell(CardStyle::Filled, kind), &LIGHT, false),
                (highest, 0)
            );
            assert_eq!(
                container(cell(CardStyle::Elevated, kind), &LIGHT, false),
                (low, 0)
            );
            assert_eq!(
                container(cell(CardStyle::Outlined, kind), &LIGHT, false),
                (surface, 0)
            );
        }
        // No two of the three are the same, and the dark scheme has its own colors.
        assert_ne!(highest, low);
        assert_ne!(highest, surface);
        assert_ne!(low, surface);
        let dark = ColorScheme::BASELINE_DARK;
        assert_eq!(
            container(cell(CardStyle::Filled, CardKind::Static), &dark, false),
            (Rgb::of(&dark, ColorRole::SurfaceContainerHighest), 0)
        );
        assert_ne!(Rgb::of(&dark, ColorRole::SurfaceContainerHighest), highest);
    }

    #[test]
    fn a_disabled_card_is_composited_as_androidx_composites_it() {
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let highest = Rgb::of(&LIGHT, ColorRole::SurfaceContainerHighest);
        // The filled card: the surface variant color at 0.38 over its container color.
        let (filled, tolerance) =
            container(cell(CardStyle::Filled, CardKind::Disabled), &LIGHT, false);
        assert_eq!(
            filled,
            Rgb::of(&LIGHT, ColorRole::SurfaceVariant).over(0.38, highest)
        );
        assert_eq!(tolerance, BLEND_TOLERANCE);
        assert_ne!(filled, highest);
        // The elevated card: surface at 0.38 over surface, which is the surface. Only its shadow shows it.
        let (elevated, _) = container(cell(CardStyle::Elevated, CardKind::Disabled), &LIGHT, false);
        assert_eq!(elevated, surface);
        // The outlined card: its container color, exactly.
        assert_eq!(
            container(cell(CardStyle::Outlined, CardKind::Disabled), &LIGHT, false),
            (surface, 0)
        );
        // A disabled card takes no hover: the mouse changes nothing.
        for style in STYLES {
            let disabled = cell(style, CardKind::Disabled);
            assert_eq!(
                container(disabled, &LIGHT, true),
                container(disabled, &LIGHT, false),
                "{style:?}"
            );
        }
    }

    #[test]
    fn the_outline_is_the_edge_of_the_outlined_card_only() {
        let variant = Rgb::of(&LIGHT, ColorRole::OutlineVariant);
        for kind in [CardKind::Static, CardKind::Clickable] {
            assert_eq!(
                edge(cell(CardStyle::Outlined, kind), &LIGHT, false),
                (variant, 0)
            );
        }
        // The outline is drawn over the state layer: hovering leaves it as it is.
        assert_eq!(
            edge(cell(CardStyle::Outlined, CardKind::Clickable), &LIGHT, true),
            (variant, 0)
        );
        // Disabled: the outline color at 0.12 over the elevated container, whatever the card.
        let (disabled, tolerance) =
            edge(cell(CardStyle::Outlined, CardKind::Disabled), &LIGHT, false);
        assert_eq!(
            disabled,
            Rgb::of(&LIGHT, ColorRole::Outline)
                .over(0.12, Rgb::of(&LIGHT, ColorRole::SurfaceContainerLow))
        );
        assert_eq!(tolerance, BLEND_TOLERANCE);
        assert_ne!(disabled, variant);
        // The other styles have no outline: their edge is their container.
        for style in [CardStyle::Filled, CardStyle::Elevated] {
            for kind in KINDS {
                let card = cell(style, kind);
                assert_eq!(
                    edge(card, &LIGHT, false),
                    container(card, &LIGHT, false),
                    "{card:?}"
                );
            }
        }
    }

    #[test]
    fn hovering_lays_the_on_surface_color_over_the_card_that_takes_presses() {
        for style in STYLES {
            let card = cell(style, CardKind::Clickable);
            let rest = Rgb::of(&LIGHT, container_role(style));
            assert_eq!(
                container(card, &LIGHT, true),
                (
                    Rgb::of(&LIGHT, ColorRole::OnSurface).over(0.08, rest),
                    BLEND_TOLERANCE
                ),
                "{style:?}"
            );
            let before = card_probes(card, &LIGHT, false);
            let after = card_probes(card, &LIGHT, true);
            // The container, the corner and (but for the outlined card, whose edge is its outline) the edge change;
            // the surface does not.
            assert_ne!(after[0], before[0], "{style:?}");
            assert_eq!(after[1], before[1], "{style:?}");
            assert_ne!(after[2], before[2], "{style:?}");
            assert_eq!(
                after[3] == before[3],
                style == CardStyle::Outlined,
                "{style:?}"
            );
            assert_eq!(after[2].expected, after[0].expected, "{style:?}");
        }
        // A card that is only a container takes no hover.
        for style in STYLES {
            let card = cell(style, CardKind::Static);
            assert_eq!(
                container(card, &LIGHT, true),
                container(card, &LIGHT, false)
            );
        }
    }

    #[test]
    fn the_corner_is_read_inside_its_arc_past_the_outline() {
        // The depth of the pixel `n` points along both edges, read at its middle, inside the arc of `radius`.
        let depth = |n: f32, radius: f32| radius - (radius - n - 0.5).hypot(radius - n - 0.5);
        let offset = inside_the_arc(Points(12.0)).0;
        assert_eq!(offset, 5.0);
        // Deep enough for the arc's antialiasing and the 1-point outline, and the first such pixel.
        assert!(depth(offset, 12.0) >= ARC_MARGIN);
        assert!(depth(offset - 1.0, 12.0) < ARC_MARGIN);
        // Every card's corner is CornerMedium, 12 points, which fits in a box of 96 by 160.
        for card in CardCell::all() {
            assert_eq!(corner_radius(card), Points(12.0), "{card:?}");
        }
        assert_eq!(
            radius(shape_tokens::CORNER_FULL, Points(40.0)),
            Points(20.0)
        );
        assert_eq!(
            radius(shape_tokens::CORNER_EXTRA_LARGE, Points(40.0)),
            Points(20.0)
        );
        assert_eq!(radius(shape_tokens::CORNER_NONE, Points(40.0)), Points(0.0));
    }

    #[test]
    fn probes_read_the_container_the_surface_beside_it_the_corner_and_the_top_edge() {
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let card = cell(CardStyle::Outlined, CardKind::Clickable);
        let (x, y) = card.origin();
        let found = card_probes(card, &LIGHT, false);
        let whats: Vec<&str> = found.iter().map(|p| p.what).collect();
        assert_eq!(
            whats,
            [
                "container",
                "surface beside it",
                "inside the corner",
                "top edge",
                "outside the corner"
            ]
        );
        assert!(found.iter().all(|p| p.target == Target::Card(card)));
        let at: Vec<(f32, f32)> = found.iter().map(|p| (p.x.0, p.y.0)).collect();
        assert_eq!(
            at,
            [
                (x.0 + 20.0, y.0 + 48.0),
                (x.0 - 16.0, y.0 + 48.0),
                (x.0 + 5.0, y.0 + 5.0),
                (x.0 + 80.0, y.0),
                (x.0, y.0),
            ]
        );
        assert_eq!(found[0].expected, surface);
        assert_eq!(found[1].expected, surface);
        assert_eq!(found[2].expected, surface);
        assert_eq!(
            found[3].expected,
            Rgb::of(&LIGHT, ColorRole::OutlineVariant)
        );
        assert_eq!(found[4].expected, surface);
    }

    #[test]
    fn only_a_card_with_no_shadow_is_read_outside_its_corner() {
        for card in CardCell::all() {
            for hovered in [false, true] {
                let found = card_probes(card, &LIGHT, hovered);
                let outside = found.iter().any(|p| p.what == "outside the corner");
                assert_eq!(
                    outside,
                    !casts_a_shadow(card, hovered),
                    "{card:?} {hovered}"
                );
                assert_eq!(found.len(), 4 + usize::from(outside), "{card:?} {hovered}");
            }
        }
    }

    #[test]
    fn the_shadow_table_is_the_levels_the_cards_sit_at() {
        // The table above is written from androidx; the library's levels must be the same ones.
        for card in CardCell::all() {
            for hovered in [false, true] {
                // Only a card that takes presses is ever under the mouse.
                if hovered && card.kind != CardKind::Clickable {
                    continue;
                }
                let state = match (card.kind, hovered) {
                    (CardKind::Disabled, _) => CardState::Disabled,
                    (_, true) => CardState::Hovered,
                    _ => CardState::Rest,
                };
                assert_eq!(
                    casts_a_shadow(card, hovered),
                    card.style.level(state) != Elevation::Level0,
                    "{card:?} {hovered}"
                );
            }
        }
        // The highest level of any card in any state is the one the surface is read past.
        let highest = STYLES
            .into_iter()
            .flat_map(|style| {
                [
                    CardState::Rest,
                    CardState::Hovered,
                    CardState::Focused,
                    CardState::Pressed,
                    CardState::Disabled,
                ]
                .into_iter()
                .map(move |state| style.level(state))
            })
            .max();
        assert_eq!(highest, Some(HIGHEST_LEVEL));
    }

    #[test]
    fn the_shadows_reach_what_adr_0018_says_and_the_surface_is_read_past_it() {
        let reach = |level| {
            let (side, below) = shadow_reach(level);
            (side.0, below.0)
        };
        // Level 1: the ambient layer `0 1px 3px 1px` reaches 4 to the sides and 5 below; level 2: `0 2px 6px 2px`, 8
        // and 10 (the formula of `_elevation.scss`, ADR 0018).
        assert_eq!(reach(Elevation::Level1), (4.0, 5.0));
        assert_eq!(reach(Elevation::Level2), (8.0, 10.0));
        assert_eq!(reach(Elevation::Level0), (0.0, 0.0));
        assert_eq!(surface_distance(), Points(16.0));
        // Inside the reach below at the lowest level a card casts, and not at the container's edge.
        const { assert!(SHADOW_BELOW.0 > 0.0) };
        assert!(SHADOW_BELOW.0 < reach(Elevation::Level1).1);
    }

    #[test]
    fn probes_stay_clear_of_the_shadows_and_of_each_other() {
        let boxes: Vec<(CardCell, f32, f32, f32, f32)> = CardCell::all()
            .map(|card| {
                let ((x, y), (width, height)) = (card.origin(), card.frame());
                (card, x.0, y.0, x.0 + width.0, y.0 + height.0)
            })
            .collect();
        let (side, below_reach) = shadow_reach(HIGHEST_LEVEL);
        for &(card, left, top, right, bottom) in &boxes {
            // The surface beside a card is past the reach of every card's shadows, its own included, and the shadow
            // read below a card is in the gap, past no other card's.
            let surface = card_probes(card, &LIGHT, false)[1];
            let shadow = below(card);
            for &(other, o_left, o_top, o_right, o_bottom) in &boxes {
                assert!(
                    surface.x.0 < o_left - side.0
                        || surface.x.0 > o_right + side.0
                        || surface.y.0 < o_top - side.0
                        || surface.y.0 > o_bottom + below_reach.0,
                    "{card:?}'s surface is in the shadow of {other:?}"
                );
                if other != card {
                    assert!(
                        !(shadow.0.0 >= o_left
                            && shadow.0.0 <= o_right
                            && shadow.1.0 >= o_top
                            && shadow.1.0 <= o_bottom),
                        "{card:?}'s shadow is read in {other:?}"
                    );
                }
            }
            // Every other probe is inside the box, and none is at the box's corner of a card that casts a shadow.
            for probe in card_probes(card, &LIGHT, false)
                .iter()
                .filter(|p| p.what != "surface beside it" && p.what != "outside the corner")
            {
                assert!(
                    probe.x.0 >= left
                        && probe.x.0 < right
                        && probe.y.0 >= top
                        && probe.y.0 < bottom,
                    "{card:?} {}",
                    probe.what
                );
            }
            // The container is read above the card's name, which starts 16 points down from the top and 16 in.
            assert!(card_probes(card, &LIGHT, false)[0].y.0 - top >= 40.0);
        }
        // The surface is read less far than half the gap between two cards and than the margin.
        assert!(surface_distance().0 * 2.0 <= GAP.0);
        assert!(surface_distance().0 <= catalog::grid::MARGIN.0);
        assert_eq!((WIDTH, HEIGHT), (Points(160.0), Points(96.0)));
    }

    #[test]
    fn the_shadow_is_read_below_the_middle_of_each_container() {
        let card = cell(CardStyle::Elevated, CardKind::Static);
        let (x, y) = card.origin();
        assert_eq!(below(card), (Points(x.0 + 80.0), Points(y.0 + 97.0)));
        for card in CardCell::all() {
            let (px, py) = below(card);
            let (x, y) = card.origin();
            let (width, height) = card.frame();
            assert!(py.0 > y.0 + height.0, "{card:?}");
            assert!(px.0 > x.0 && px.0 < x.0 + width.0, "{card:?}");
        }
    }

    #[test]
    fn a_shot_reads_a_shadow_below_the_cards_that_cast_one_and_the_surface_below_the_others() {
        let rest = Shot::all()[0];
        // At rest: the three elevated cards cast one (level 1, disabled too), the six others none.
        let shadows = shot_shadows(rest);
        let cards: Vec<CardCell> = shadows
            .iter()
            .filter_map(|s| match s.target {
                Target::Card(card) => Some(card),
                _ => None,
            })
            .collect();
        assert_eq!(
            cards,
            [
                cell(CardStyle::Elevated, CardKind::Static),
                cell(CardStyle::Elevated, CardKind::Clickable),
                cell(CardStyle::Elevated, CardKind::Disabled),
            ]
        );
        assert_eq!(shot_unshadowed(rest).len(), 6);
        // Every card is read one way or the other, never both.
        for shot in Shot::all() {
            assert_eq!(
                shot_shadows(shot).len() + shot_unshadowed(shot).len(),
                CardCell::all().count(),
                "{shot:?}"
            );
        }
        // The filled card under the mouse casts one too: level 1 where it was 0.
        let filled = Shot::all()[1];
        assert_eq!(
            filled.hovered,
            Some(cell(CardStyle::Filled, CardKind::Clickable))
        );
        assert_eq!(shot_shadows(filled).len(), 4);
        assert!(
            shot_shadows(filled)
                .iter()
                .any(|s| s.target == Target::Card(cell(CardStyle::Filled, CardKind::Clickable)))
        );
        // The outlined card under the mouse does not rise: no shadow, as at rest.
        let outlined = Shot::all()[3];
        assert_eq!(
            outlined.hovered,
            Some(cell(CardStyle::Outlined, CardKind::Clickable))
        );
        assert_eq!(shot_shadows(outlined).len(), 3);
        let unshadowed = shot_unshadowed(outlined);
        assert!(
            unshadowed
                .iter()
                .any(|p| p.target == Target::Card(cell(CardStyle::Outlined, CardKind::Clickable)))
        );
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        assert!(
            unshadowed
                .iter()
                .all(|p| p.expected == surface && p.tolerance == 0)
        );
    }

    #[test]
    fn darker_is_every_channel() {
        let color = Rgb([10, 20, 30]);
        assert!(darker_in_every_channel(Rgb([9, 19, 29]), color));
        assert!(!darker_in_every_channel(Rgb([9, 20, 29]), color));
        assert!(!darker_in_every_channel(Rgb([9, 19, 31]), color));
        assert!(!darker_in_every_channel(color, color));
    }

    #[test]
    fn a_card_with_a_flat_corner_a_stray_outline_or_a_wider_container_fails_where_it_is_wrong() {
        let shot = Shot::all()[0];
        let card = cell(CardStyle::Filled, CardKind::Static);
        let found = card_probes(card, &LIGHT, false);
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let container = Rgb::of(&LIGHT, ColorRole::SurfaceContainerHighest);
        let paint = |probe: Probe, color: Rgb| {
            let mut image = painted(shot, 1);
            let (x, y) = pixel_of(&probe, 1);
            let start = ((y * image.width + x) * 4) as usize;
            image.rgba[start..start + 3].copy_from_slice(&color.0);
            check_colors(&image, shot).unwrap()
        };
        // A corner square instead of rounded: the container's color where the surface's is expected, outside it.
        let outside = found[4];
        assert_eq!(outside.what, "outside the corner");
        let wrong = paint(outside, container);
        assert_eq!(wrong.len(), 1);
        assert_eq!(wrong[0].probe, outside);
        // The same corner read inside the arc: the surface where the container's is expected.
        let wrong = paint(found[2], surface);
        assert_eq!(wrong.len(), 1);
        assert_eq!(wrong[0].probe.what, "inside the corner");
        // An outline on a card that has none: the outline's color along the edge.
        let wrong = paint(found[3], Rgb::of(&LIGHT, ColorRole::OutlineVariant));
        assert_eq!(wrong.len(), 1);
        assert_eq!(wrong[0].probe.what, "top edge");
        // A container drawn past its box: the container's color where the surface's is expected.
        let wrong = paint(found[1], container);
        assert_eq!(wrong.len(), 1);
        assert_eq!(wrong[0].probe.what, "surface beside it");
        assert_eq!(wrong[0].found, Some(container));
        // The surface is read exactly: a shadow's few levels of difference beside it are caught.
        let wrong = paint(
            found[1],
            Rgb([surface.0[0] - 1, surface.0[1], surface.0[2]]),
        );
        assert_eq!(wrong.len(), 1);
    }

    #[test]
    fn an_outlined_card_that_rises_or_an_elevated_one_with_no_shadow_fails_where_it_is_wrong() {
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let put = |image: &mut Image, x: Points, y: Points, color: Rgb| {
            let start = ((points_to_u32(y) * image.width + points_to_u32(x)) * 4) as usize;
            image.rgba[start..start + 3].copy_from_slice(&color.0);
        };
        // An elevated card with no shadow shows the surface below it, which is not darker than the surface.
        let rest = Shot::all()[0];
        let elevated = cell(CardStyle::Elevated, CardKind::Static);
        let (x, y) = below(elevated);
        let mut image = painted(rest, 1);
        assert_eq!(check_shadows(&image, rest), Ok(Vec::new()));
        assert_eq!(check_colors(&image, rest), Ok(Vec::new()));
        put(&mut image, x, y, surface);
        let found = check_shadows(&image, rest).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe.target, Target::Card(elevated));
        assert_eq!(found[0].found, Some(surface));
        assert_eq!(
            found[0].to_string(),
            format!(
                "card-elevated-static shadow below it at ({}, {}) points: expected darker than {surface}, found {surface}",
                x.0, y.0
            )
        );
        // One channel not darker is no shadow either.
        let mut image = painted(rest, 1);
        put(&mut image, x, y, Rgb([surface.0[0], 1, 1]));
        assert_eq!(check_shadows(&image, rest).unwrap().len(), 1);
        // An outlined card that rises under the mouse, as its tokens say but androidx does not: a shadow where the
        // surface is expected.
        let shot = Shot::all()[3];
        let outlined = cell(CardStyle::Outlined, CardKind::Clickable);
        assert_eq!(shot.hovered, Some(outlined));
        let (x, y) = below(outlined);
        let mut image = painted(shot, 1);
        assert_eq!(check(&image, shot), Ok(Vec::new()));
        put(&mut image, x, y, shaded(surface, 3, 4));
        let found = check_colors(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe.what, "no shadow below it");
        assert_eq!(found[0].probe.target, Target::Card(outlined));
        assert_eq!(check(&image, shot).unwrap().len(), 1);
        // A filled card that does not rise under the mouse: no shadow where one is expected.
        let shot = Shot::all()[1];
        let filled = cell(CardStyle::Filled, CardKind::Clickable);
        let (x, y) = below(filled);
        let mut image = painted(shot, 1);
        assert_eq!(check(&image, shot), Ok(Vec::new()));
        put(&mut image, x, y, surface);
        let found = check_shadows(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe.target, Target::Card(filled));
        // An image of another size is refused, as for the colors.
        let small = Image {
            width: 1,
            height: 1,
            rgba: vec![0; 4],
        };
        assert!(check_shadows(&small, rest).is_err());
        assert!(check_colors(&small, rest).is_err());
    }

    #[test]
    fn a_target_is_named_by_its_test_id() {
        let card = cell(CardStyle::Outlined, CardKind::Disabled);
        assert_eq!(Target::Card(card).test_id(), "card-outlined-disabled");
    }

    #[test]
    fn the_shots_are_the_test_script_files() {
        let names: Vec<String> = Shot::all().into_iter().map(Shot::file_name).collect();
        assert_eq!(
            names,
            [
                "card-light-rest.png",
                "card-light-hover-filled.png",
                "card-light-hover-elevated.png",
                "card-light-hover-outlined.png",
                "card-dark-rest.png",
                "card-dark-hover-filled.png",
                "card-dark-hover-elevated.png",
                "card-dark-hover-outlined.png",
            ]
        );
        assert_eq!(Shot::all()[0].appearance.scheme(), LIGHT);
        assert_eq!(
            Shot::all()[4].appearance.scheme(),
            ColorScheme::BASELINE_DARK
        );
        // The cards hovered are the ones that take presses, one of each style.
        assert_eq!(Shot::all()[0].hovered, None);
        for (index, style) in STYLES.into_iter().enumerate() {
            assert_eq!(
                Shot::all()[index + 1].hovered,
                Some(cell(style, CardKind::Clickable))
            );
        }
    }

    #[test]
    fn every_shot_reads_every_card_and_only_the_hovered_one_changes() {
        let rest = shot_probes(Shot::all()[0]);
        assert_eq!(
            rest.len(),
            CardCell::all()
                .map(|c| card_probes(c, &LIGHT, false).len())
                .sum::<usize>()
                + shot_unshadowed(Shot::all()[0]).len()
        );
        // The hovered card's container and corner, both in the container's color, and the rest of what changes with
        // its shadow: nothing of another card.
        for (index, style) in STYLES.into_iter().enumerate() {
            let shot = Shot::all()[index + 1];
            let hovered = Target::Card(cell(style, CardKind::Clickable));
            let changed: Vec<Probe> = shot_probes(shot)
                .into_iter()
                .filter(|probe| !rest.contains(probe))
                .collect();
            assert!(!changed.is_empty(), "{style:?}");
            assert!(changed.iter().all(|p| p.target == hovered), "{style:?}");
            assert!(changed.iter().any(|p| p.what == "container"), "{style:?}");
        }
    }

    #[test]
    fn a_catalog_drawn_as_the_tokens_say_passes_at_any_whole_scale() {
        for shot in Shot::all() {
            for scale in [1, 2] {
                assert_eq!(
                    check_colors(&painted(shot, scale), shot),
                    Ok(Vec::new()),
                    "{shot:?} at {scale}"
                );
                assert_eq!(
                    check_shadows(&painted(shot, scale), shot),
                    Ok(Vec::new()),
                    "{shot:?} at {scale}"
                );
                assert_eq!(
                    check(&painted(shot, scale), shot),
                    Ok(Vec::new()),
                    "{shot:?} at {scale}"
                );
            }
        }
    }

    #[test]
    fn a_shadow_is_read_at_the_pixel_of_its_point_at_any_whole_scale() {
        let shot = Shot::all()[0];
        let card = cell(CardStyle::Elevated, CardKind::Static);
        let (x, y) = below(card);
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        for scale in [1, 2] {
            // No shadow at the one pixel the point names, whatever the scale. A read at another pixel would find the
            // image's background, which is darker than the surface, and pass.
            let mut image = painted(shot, scale);
            let start =
                ((points_to_u32(y) * scale * image.width + points_to_u32(x) * scale) * 4) as usize;
            image.rgba[start..start + 3].copy_from_slice(&surface.0);
            let found = check_shadows(&image, shot).unwrap();
            assert_eq!(found.len(), 1, "scale {scale}");
            assert_eq!(found[0].probe.target, Target::Card(card), "scale {scale}");
            assert_eq!(found[0].found, Some(surface), "scale {scale}");
        }
    }

    #[test]
    fn a_screenshot_of_another_viewport_is_refused() {
        // `fab.test.tsx` opens the catalog at the viewport above the cards' grid.
        let (width, height) = catalog::fab::viewport();
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
