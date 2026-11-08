//! The FAB's known pixels (ADR 0013): what `fab.test.tsx` screenshots show of each FAB of the catalog
//! (`catalog::fab`), at rest and with the mouse over the baseline FAB of each style.
//!
//! A FAB casts a shadow (ADR 0018), so a pixel is never read at the box's corner or next to the container's edge: the
//! container's color is read inside it, the corner from inside its arc, and the surface beyond the shadows' reach. The
//! shadow itself is read just below the container, where it must be darker than the surface, and darker still below
//! the FAB the mouse is over, which is a level higher.

use std::fmt;

use catalog::fab::{FabCell, STYLES};
use nsh_core::BoxShadow;
use nsh_core::geometry::Points;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::elevation::Elevation;
use nsh_m3e::fab::{FabSize, FabState, FabStyle};
use nsh_m3e::tokens::Shape;
use nsh_m3e::tokens::androidx::{
    fab_baseline_tokens, fab_large_tokens, fab_primary_container_tokens,
    fab_secondary_container_tokens, fab_small_tokens, shape_tokens, state_tokens,
};

use super::{Appearance, Image, Mismatch, Probe, Rgb, Target, faded, pixel_of, points_to_u32};

/// One screenshot `examples/catalog/fab.test.tsx` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Shot {
    pub(crate) appearance: Appearance,
    /// The FAB the mouse is over, if any.
    pub(crate) hovered: Option<FabCell>,
}

impl Shot {
    /// Every screenshot the script takes, in each appearance: the catalog at rest, then the mouse over the baseline
    /// FAB of each style.
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
                        .map(move |style| shot(Some(FabCell::new(style, FabSize::Baseline)))),
                )
            })
            .collect()
    }

    /// Its file name, as the test script saves it.
    pub(crate) fn file_name(self) -> String {
        let appearance = self.appearance.name();
        match self.hovered {
            None => format!("fab-{appearance}-rest.png"),
            Some(cell) => format!("fab-{appearance}-hover-{}.png", cell.style_name()),
        }
    }
}

impl Rgb {
    /// Whether every channel is lower than `other`'s.
    fn darker_in_every_channel(self, other: Self) -> bool {
        self.0.iter().zip(other.0).all(|(a, b)| *a < b)
    }

    /// Whether no channel is higher than `other`'s and the three together are lower.
    fn darker_overall(self, other: Self) -> bool {
        let sum = |color: Self| color.0.iter().map(|c| u16::from(*c)).sum::<u16>();
        self.0.iter().zip(other.0).all(|(a, b)| *a <= b) && sum(self) < sum(other)
    }
}

/// The container's color role of a FAB of `style`, and its icon's, which its state layer takes too.
fn fab_roles(style: FabStyle) -> (ColorRole, ColorRole) {
    match style {
        FabStyle::PrimaryContainer => (
            fab_primary_container_tokens::CONTAINER_COLOR,
            fab_primary_container_tokens::ICON_COLOR,
        ),
        FabStyle::SecondaryContainer => (
            fab_secondary_container_tokens::CONTAINER_COLOR,
            fab_secondary_container_tokens::ICON_COLOR,
        ),
        // `FabStyle` is non-exhaustive. A style the catalog does not show is expected in the surface color, which a
        // FAB's container and icon are never drawn in, so its checks fail until they know it.
        _ => (ColorRole::Surface, ColorRole::Surface),
    }
}

/// The container's color of the FAB `cell` and its tolerance, with the mouse over it or not: the icon's color laid
/// over it at the hover opacity of the state layer.
fn fab_container(cell: FabCell, scheme: &ColorScheme, hovered: bool) -> (Rgb, u8) {
    let (container, icon) = fab_roles(cell.style);
    let rest = Rgb::of(scheme, container);
    if hovered {
        faded(scheme, icon, state_tokens::HOVER_STATE_LAYER_OPACITY, rest)
    } else {
        (rest, 0)
    }
}

/// The radius of the FAB `cell`'s corners, at most half its shorter side: its size's `ContainerShape`, or for the
/// medium FAB, which has none in its tokens, `CornerLargeIncreased` (androidx `FloatingActionButton.kt`
/// `mediumShape`, ADR 0012), as the FAB draws it.
fn fab_corner_radius(cell: FabCell) -> Points {
    let (width, height) = cell.frame();
    // No size may fall back on another's: one would hide the other's arm. A size the catalog does not show has no
    // shape here.
    let shape = match cell.size {
        FabSize::Small => Some(fab_small_tokens::CONTAINER_SHAPE),
        FabSize::Baseline => Some(fab_baseline_tokens::CONTAINER_SHAPE),
        FabSize::Medium => Some(shape_tokens::CORNER_LARGE_INCREASED),
        FabSize::Large => Some(fab_large_tokens::CONTAINER_SHAPE),
        _ => None,
    };
    corner_radius(shape, Points(width.0.min(height.0)))
}

/// The radius of the corners of `shape` on a box whose shorter side is `side`: no more than half of it, which is the
/// radius of a circle shape. A square shape, or none, has no radius. Apart from [`fab_corner_radius`] so that a
/// test can give it the shapes no FAB token has (a circle) and a radius larger than the box.
fn corner_radius(shape: Option<Shape>, side: Points) -> Points {
    let half = side.0 / 2.0;
    match shape {
        Some(Shape::Circle) => Points(half),
        Some(Shape::Rounded(corners)) => Points(corners.top_start.0.min(half)),
        // Square corners: a rectangle, or no shape.
        _ => Points(0.0),
    }
}

/// How far inside the arc of a corner a probe reads the container's color: past the antialiasing of the arc's edge,
/// which spreads over about a point. Not a design value: where the check looks.
const ARC_MARGIN: f32 = 1.0;

/// How far past the nominal reach of a FAB's shadows the surface is read. The shadow is a Gaussian whose standard
/// deviation is half its blur (`BoxShadow::blur`): 5 points for the highest level's ambient layer (level 4, blur 10).
/// The reach is the spread plus the blur, so it ends 2 standard deviations past the shadow's box (the spread's edge),
/// and a point 8 past the reach is 18 points, 3.6 standard deviations, past that edge, where the shadow is under
/// 0.02 % of its opacity. Worked out from the shadow's definition, not measured on a host. Not a design value: where
/// the check looks.
const SHADOW_TAIL: Points = Points(8.0);

/// How far below a FAB's container its shadow is read: inside the reach of every level a FAB has (level 3 reaches 15
/// points below, level 4 reaches 20), close enough to the container that the levels differ most. Not a design value:
/// where the check looks.
const SHADOW_BELOW: Points = Points(2.0);

/// How far the shadows of `level` reach past a container, to the sides and below it, in points: the largest, over its
/// layers, of the offset, spread and blur that point that way (ADR 0018 gives 11 and 15 for level 3, 14 and 20 for
/// level 4). `BoxShadow::blur` is twice the Gaussian's standard deviation, so the shadow is nothing past it.
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

/// The highest level a FAB of `style` sits at, in any state: the one whose shadows reach furthest.
fn highest_level(style: FabStyle) -> Elevation {
    [
        FabState::Rest,
        FabState::Hovered,
        FabState::Focused,
        FabState::Pressed,
    ]
    .into_iter()
    .map(|state| style.level(state))
    .max()
    .unwrap_or(Elevation::Level0)
}

/// How far from the container's left edge, at mid height, the surface is read beside the FAB `cell`: past the reach
/// of its highest level's shadows to the sides, by [`SHADOW_TAIL`]. The catalog's FAB columns leave room for it.
fn surface_distance(cell: FabCell) -> Points {
    Points(shadow_reach(highest_level(cell.style)).0.0 + SHADOW_TAIL.0)
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

/// The pixels the checks know for the FAB `cell` in a screenshot in `scheme`, with the mouse over it or not.
///
/// None is read at the box's corner or next to the container's edge: a shadow (ADR 0018) is drawn around the container
/// and below it, and reaches those places. The container is drawn over its shadow, so what is read inside it is the
/// same with one. The surface is read [`surface_distance`] away, past the reach of the shadows; [`shot_shadows`] reads
/// the shadow.
fn fab_probes(cell: FabCell, scheme: &ColorScheme, hovered: bool) -> Vec<Probe> {
    let target = Target::Fab(cell);
    let (x, y) = cell.origin();
    let (width, height) = cell.frame();
    let middle = (Points(x.0 + width.0 / 2.0), Points(y.0 + height.0 / 2.0));
    let (container, container_tolerance) = fab_container(cell, scheme, hovered);
    let (_, icon) = fab_roles(cell.style);
    let corner = inside_the_arc(fab_corner_radius(cell));
    vec![
        // An eighth of the width in from the left edge, at mid height: past the antialiased edge, where the edge is
        // straight whatever the corners' radius, and before the icon, which starts 8 points or more from it (the
        // small FAB's 24-point icon in 40 points).
        Probe {
            target,
            what: "container",
            x: Points(x.0 + width.0 / 8.0),
            y: middle.1,
            expected: container,
            tolerance: container_tolerance,
        },
        // Beside the container, past the reach of its shadows, in the gap to the next FAB or the margin: the surface,
        // and nothing drawn around the container.
        Probe {
            target,
            what: "surface beside it",
            x: Points(x.0 - surface_distance(cell).0),
            y: middle.1,
            expected: Rgb::of(scheme, ColorRole::Surface),
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
        // The middle of the icon, `catalog::ICON`, which is filled there.
        Probe {
            target,
            what: "icon",
            x: middle.0,
            y: middle.1,
            expected: Rgb::of(scheme, icon),
            tolerance: 0,
        },
    ]
}

/// What a shadow's pixel is compared with.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Than {
    /// A color the pixel is darker than in every channel: the shadow over the surface.
    Color(Rgb),
    /// The pixel at this point of the same screenshot: a higher level's shadow over a lower level's, where the two
    /// differ by a few levels of a channel, so the check is that no channel is lighter and the three together are
    /// darker.
    Pixel(Points, Points),
}

/// A pixel in the shadow of a FAB, which must be darker than what it is compared with (ADR 0018).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Shadow {
    target: Target,
    what: &'static str,
    x: Points,
    y: Points,
    than: Than,
}

/// Where the shadow below the FAB `cell` is read: the middle of its width, [`SHADOW_BELOW`] below the container.
fn below(cell: FabCell) -> (Points, Points) {
    let (x, y) = cell.origin();
    let (width, height) = cell.frame();
    (
        Points(x.0 + width.0 / 2.0),
        Points(y.0 + height.0 + SHADOW_BELOW.0),
    )
}

/// The pixels in the shadows of the FABs of `shot`: below each container the pixel is darker than the surface, and
/// below the FAB the mouse is over, it is darker than below the same-sized FAB of the other style, which is at rest in
/// the same screenshot and so one level lower. The pixel below a container is not the container's: it lies outside
/// it, so it depends on the shadow and the surface alone, whatever the style.
fn shot_shadows(shot: Shot) -> Vec<Shadow> {
    let scheme = shot.appearance.scheme();
    let surface = Rgb::of(&scheme, ColorRole::Surface);
    let mut shadows = Vec::new();
    for cell in FabCell::all() {
        let (x, y) = below(cell);
        let target = Target::Fab(cell);
        shadows.push(Shadow {
            target,
            what: "shadow below it",
            x,
            y,
            than: Than::Color(surface),
        });
        if shot.hovered == Some(cell) {
            for other in FabCell::all().filter(|o| o.size == cell.size && o.style != cell.style) {
                let (other_x, other_y) = below(other);
                shadows.push(Shadow {
                    target,
                    what: "shadow below it, hovered",
                    x,
                    y,
                    than: Than::Pixel(other_x, other_y),
                });
            }
        }
    }
    shadows
}

/// The pixels the checks know in `shot`, for every FAB of the catalog.
fn shot_probes(shot: Shot) -> Vec<Probe> {
    let scheme = shot.appearance.scheme();
    FabCell::all()
        .flat_map(|cell| fab_probes(cell, &scheme, shot.hovered == Some(cell)))
        .collect()
}

/// What a shadow found when it was not darker than what it is compared with.
#[derive(Debug, Clone, PartialEq)]
struct ShadowMismatch {
    probe: Shadow,
    found: Option<Rgb>,
    /// The color it was compared with, if the image has one.
    reference: Option<Rgb>,
}

impl fmt::Display for ShadowMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let probe = &self.probe;
        write!(
            f,
            "{} {} at ({}, {}) points: expected darker than ",
            probe.target.test_id(),
            probe.what,
            probe.x.0,
            probe.y.0,
        )?;
        match (probe.than, self.reference) {
            (Than::Color(color), _) => write!(f, "{color}")?,
            (Than::Pixel(x, y), Some(color)) => {
                write!(f, "{color} (the pixel at ({}, {}))", x.0, y.0)?;
            }
            (Than::Pixel(x, y), None) => {
                write!(f, "the pixel at ({}, {}), which is not there", x.0, y.0)?;
            }
        }
        match self.found {
            Some(found) => write!(f, ", found {found}"),
            None => write!(f, ", found no pixel"),
        }
    }
}

/// Every shadow of `shot` that `image` does not show.
fn check_shadows(image: &Image, shot: Shot) -> Result<Vec<ShadowMismatch>, String> {
    let scale = image.scale_in(catalog::fab::viewport())?;
    let at = |x: Points, y: Points| image.pixel(points_to_u32(x) * scale, points_to_u32(y) * scale);
    Ok(shot_shadows(shot)
        .into_iter()
        .filter_map(|probe| {
            let found = at(probe.x, probe.y);
            let (reference, darker) = match (probe.than, found) {
                (Than::Color(color), found) => (
                    Some(color),
                    found.is_some_and(|found| found.darker_in_every_channel(color)),
                ),
                (Than::Pixel(x, y), found) => {
                    let reference = at(x, y);
                    let darker = match (found, reference) {
                        (Some(found), Some(reference)) => found.darker_overall(reference),
                        _ => false,
                    };
                    (reference, darker)
                }
            };
            (!darker).then_some(ShadowMismatch {
                probe,
                found,
                reference,
            })
        })
        .collect())
}

/// Every probe of `shot` that `image` does not match; an error if the image is not the catalog's FAB viewport at a
/// whole scale.
fn check_colors(image: &Image, shot: Shot) -> Result<Vec<Mismatch>, String> {
    let scale = image.scale_in(catalog::fab::viewport())?;
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

/// What `image` does not show of `shot`: the pixels the tokens give and the shadows of the FABs, one line each.
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
/// shadow is the surface a quarter darker (half as light below the FAB the mouse is over, which is a level higher),
/// and the rest a color no probe expects.
#[cfg(test)]
pub(crate) fn painted(shot: Shot, scale: u32) -> Image {
    let (width, height) = catalog::fab::viewport();
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
    let shadows = shot_shadows(shot);
    for shadow in shadows.iter().filter(|s| matches!(s.than, Than::Color(_))) {
        paint(shadow.x, shadow.y, shaded(surface, 3, 4));
    }
    for shadow in shadows.iter().filter(|s| matches!(s.than, Than::Pixel(..))) {
        paint(shadow.x, shadow.y, shaded(surface, 1, 2));
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
    use catalog::fab::SIZES;
    use nsh_m3e::tokens::androidx::fab_medium_tokens;

    use super::*;
    use crate::BLEND_TOLERANCE;

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;

    fn fab_cell(style: FabStyle, size: FabSize) -> FabCell {
        FabCell::new(style, size)
    }

    #[test]
    fn fab_containers_are_the_style_tokens_colors() {
        assert_eq!(
            fab_roles(FabStyle::PrimaryContainer),
            (ColorRole::PrimaryContainer, ColorRole::OnPrimaryContainer)
        );
        assert_eq!(
            fab_roles(FabStyle::SecondaryContainer),
            (
                ColorRole::SecondaryContainer,
                ColorRole::OnSecondaryContainer
            )
        );
        let primary = fab_cell(FabStyle::PrimaryContainer, FabSize::Baseline);
        let secondary = fab_cell(FabStyle::SecondaryContainer, FabSize::Baseline);
        let primary_container = Rgb::of(&LIGHT, ColorRole::PrimaryContainer);
        let secondary_container = Rgb::of(&LIGHT, ColorRole::SecondaryContainer);
        assert_eq!(
            fab_container(primary, &LIGHT, false),
            (primary_container, 0)
        );
        // The secondary container color, as the tonal button's container is (`containers_are_the_style_tokens_colors`).
        assert_eq!(
            fab_container(secondary, &LIGHT, false),
            (Rgb([0xe8, 0xde, 0xf8]), 0)
        );
        assert_ne!(primary_container, secondary_container);
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        assert_ne!(primary_container, surface);
        assert_ne!(secondary_container, surface);
        // Hovered: the icon's color at 0.08 over the container, whatever the size.
        let on_primary_container = Rgb::of(&LIGHT, ColorRole::OnPrimaryContainer);
        let on_secondary_container = Rgb::of(&LIGHT, ColorRole::OnSecondaryContainer);
        for size in SIZES {
            assert_eq!(
                fab_container(fab_cell(FabStyle::PrimaryContainer, size), &LIGHT, true),
                (
                    on_primary_container.over(0.08, primary_container),
                    BLEND_TOLERANCE
                ),
                "{size:?}"
            );
            assert_eq!(
                fab_container(fab_cell(FabStyle::SecondaryContainer, size), &LIGHT, true),
                (
                    on_secondary_container.over(0.08, secondary_container),
                    BLEND_TOLERANCE
                ),
                "{size:?}"
            );
        }
        // The dark scheme has its own colors.
        let dark = ColorScheme::BASELINE_DARK;
        assert_eq!(
            fab_container(primary, &dark, false),
            (Rgb::of(&dark, ColorRole::PrimaryContainer), 0)
        );
        assert_ne!(
            Rgb::of(&dark, ColorRole::PrimaryContainer),
            primary_container
        );
    }

    #[test]
    fn fab_corners_are_the_size_tokens_radius() {
        let radii: Vec<f32> = SIZES
            .into_iter()
            .map(|size| fab_corner_radius(fab_cell(FabStyle::PrimaryContainer, size)).0)
            .collect();
        // Small `CornerMedium`, baseline `CornerLarge`, medium `CornerLargeIncreased` (androidx `mediumShape`), large
        // `CornerExtraLarge`: no size has another's, and none is rounder than half its side.
        assert_eq!(radii, [12.0, 16.0, 20.0, 28.0]);
        for cell in FabCell::all() {
            let (width, height) = cell.frame();
            assert!(
                fab_corner_radius(cell).0 <= width.0.min(height.0) / 2.0,
                "{cell:?}"
            );
        }
    }

    #[test]
    fn a_corner_is_rounded_by_its_shape_and_by_no_more_than_half_the_side() {
        let side = Points(40.0);
        // A circle shape is half the side; a rounded one its own radius when that fits.
        assert_eq!(
            corner_radius(Some(shape_tokens::CORNER_FULL), side),
            Points(20.0)
        );
        assert_eq!(
            corner_radius(Some(shape_tokens::CORNER_MEDIUM), side),
            Points(12.0)
        );
        // A radius of 28 does not fit in a box of 40: the corners are cut at half the side, not at twice it.
        assert_eq!(
            corner_radius(Some(shape_tokens::CORNER_EXTRA_LARGE), side),
            Points(20.0)
        );
        assert_eq!(
            corner_radius(Some(shape_tokens::CORNER_EXTRA_LARGE), Points(100.0)),
            Points(28.0)
        );
        // Square corners and no shape have no radius.
        assert_eq!(
            corner_radius(Some(shape_tokens::CORNER_NONE), side),
            Points(0.0)
        );
        assert_eq!(corner_radius(None, side), Points(0.0));
    }

    #[test]
    fn the_corner_is_read_inside_its_arc_by_a_margin_and_no_deeper() {
        // The depth of the pixel `n` points along both edges, read at its middle, inside the arc of `radius`: its
        // distance to the arc's center subtracted from the radius.
        let depth = |n: f32, radius: f32| radius - (radius - n - 0.5).hypot(radius - n - 0.5);
        let offsets: Vec<f32> = [12.0, 16.0, 20.0, 28.0]
            .into_iter()
            .map(|radius| inside_the_arc(Points(radius)).0)
            .collect();
        assert_eq!(offsets, [4.0, 5.0, 7.0, 9.0]);
        for (radius, offset) in [12.0, 16.0, 20.0, 28.0].into_iter().zip(offsets) {
            // Deep enough for the arc's antialiasing, and the first such pixel: the one before is on the edge.
            assert!(depth(offset, radius) >= ARC_MARGIN, "{radius}");
            assert!(depth(offset - 1.0, radius) < ARC_MARGIN, "{radius}");
        }
    }

    #[test]
    fn fab_probes_read_the_container_the_surface_beside_it_the_corner_and_the_icon() {
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let on_container = Rgb::of(&LIGHT, ColorRole::OnPrimaryContainer);
        let container = Rgb::of(&LIGHT, ColorRole::PrimaryContainer);
        // The small FAB: 40 points, 12-point corners.
        let small = fab_cell(FabStyle::PrimaryContainer, FabSize::Small);
        let (x, y) = small.origin();
        let found = fab_probes(small, &LIGHT, false);
        assert_eq!(found.len(), 4);
        let whats: Vec<&str> = found.iter().map(|p| p.what).collect();
        assert_eq!(
            whats,
            [
                "container",
                "surface beside it",
                "inside the corner",
                "icon"
            ]
        );
        assert!(found.iter().all(|p| p.target == Target::Fab(small)));
        assert_eq!(
            (
                found[0].x,
                found[0].y,
                found[0].expected,
                found[0].tolerance
            ),
            (Points(x.0 + 5.0), Points(y.0 + 20.0), container, 0)
        );
        assert_eq!(
            (
                found[1].x,
                found[1].y,
                found[1].expected,
                found[1].tolerance
            ),
            (Points(x.0 - 22.0), Points(y.0 + 20.0), surface, 0)
        );
        assert_eq!(
            (
                found[2].x,
                found[2].y,
                found[2].expected,
                found[2].tolerance
            ),
            (Points(x.0 + 4.0), Points(y.0 + 4.0), container, 0)
        );
        assert_eq!(
            (
                found[3].x,
                found[3].y,
                found[3].expected,
                found[3].tolerance
            ),
            (Points(x.0 + 20.0), Points(y.0 + 20.0), on_container, 0)
        );
        // The large one: 96 points, 28-point corners; its middle follows its size.
        let large = fab_cell(FabStyle::SecondaryContainer, FabSize::Large);
        let (x, y) = large.origin();
        let found = fab_probes(large, &LIGHT, false);
        assert_eq!(
            (found[0].x, found[0].y),
            (Points(x.0 + 12.0), Points(y.0 + 48.0))
        );
        assert_eq!(
            (found[1].x, found[1].y),
            (Points(x.0 - 22.0), Points(y.0 + 48.0))
        );
        assert_eq!(
            (found[2].x, found[2].y),
            (Points(x.0 + 9.0), Points(y.0 + 9.0))
        );
        assert_eq!(
            (found[3].x, found[3].y),
            (Points(x.0 + 48.0), Points(y.0 + 48.0))
        );
        assert_eq!(
            found[3].expected,
            Rgb::of(&LIGHT, ColorRole::OnSecondaryContainer)
        );
    }

    #[test]
    fn fab_probes_stay_clear_of_the_shadow_the_corner_and_each_other() {
        let frames: Vec<(FabCell, f32, f32, f32, f32)> = FabCell::all()
            .map(|cell| {
                let ((x, y), (width, height)) = (cell.origin(), cell.frame());
                (cell, x.0, y.0, x.0 + width.0, y.0 + height.0)
            })
            .collect();
        for &(cell, left, top, right, bottom) in &frames {
            let found = fab_probes(cell, &LIGHT, false);
            let (side, _) = cell.frame();
            for probe in &found {
                // Never at the box's corner, where a shadow lies.
                assert_ne!(
                    (probe.x.0, probe.y.0),
                    (left, top),
                    "{cell:?} {}",
                    probe.what
                );
            }
            // The container, the corner and the icon are inside the box; the surface is in no FAB's.
            for probe in found.iter().filter(|p| p.what != "surface beside it") {
                assert!(
                    probe.x.0 > left && probe.x.0 < right && probe.y.0 > top && probe.y.0 < bottom,
                    "{cell:?} {}",
                    probe.what
                );
            }
            // The surface is past the reach of every FAB's shadows, its own included: outside each box grown by the
            // reach of the highest level, to the sides, above (as far as the sides, which is more than the shadow
            // reaches there) and below.
            let surface = found[1];
            for &(other, other_left, other_top, other_right, other_bottom) in &frames {
                let (side, below) = shadow_reach(highest_level(other.style));
                assert!(
                    surface.x.0 < other_left - side.0
                        || surface.x.0 > other_right + side.0
                        || surface.y.0 < other_top - side.0
                        || surface.y.0 > other_bottom + below.0,
                    "{cell:?}'s surface is in the shadow of {other:?}"
                );
            }
            // The container is read before the icon, whose box starts at ((side - icon) / 2).
            let icon = match cell.size {
                FabSize::Small => fab_small_tokens::ICON_SIZE.0,
                FabSize::Baseline => fab_baseline_tokens::ICON_SIZE.0,
                FabSize::Medium => fab_medium_tokens::ICON_SIZE.0,
                // The large FAB's icon is androidx's 36, where its token says 32 (`LargeIconSize`, ADR 0012).
                _ => 36.0,
            };
            assert!(found[0].x.0 - left < (side.0 - icon) / 2.0, "{cell:?}");
        }
        // The surface is read less far from a FAB than half the narrowest gap between two, and than the margin: the
        // gap and the margin are wide enough for the shadows' reach and the tail.
        let widest = SIZES
            .into_iter()
            .map(|size| catalog::fab::frame(size).0.0)
            .fold(0.0_f32, f32::max);
        assert_eq!(widest, 96.0);
        for cell in FabCell::all() {
            let distance = surface_distance(cell).0;
            assert!(
                distance * 2.0 <= catalog::fab::COLUMN_WIDTH.0 - widest,
                "{cell:?}"
            );
            assert!(distance <= catalog::grid::MARGIN.0, "{cell:?}");
        }
    }

    #[test]
    fn the_shadows_reach_what_adr_0018_says_and_the_surface_is_read_past_it() {
        let reach = |level| {
            let (side, below) = shadow_reach(level);
            (side.0, below.0)
        };
        // ADR 0018: level 3 reaches 11 points to the sides (spread 3, blur 8) and 15 below (plus the 4-point offset);
        // level 4, 14 and 20.
        assert_eq!(reach(Elevation::Level3), (11.0, 15.0));
        assert_eq!(reach(Elevation::Level4), (14.0, 20.0));
        // Nothing reaches at level 0; every level reaches further than the one before it.
        assert_eq!(reach(Elevation::Level0), (0.0, 0.0));
        for pair in Elevation::ALL.windows(2) {
            let (lower, higher) = (reach(pair[0]), reach(pair[1]));
            assert!(higher.0 >= lower.0 && higher.1 >= lower.1, "{pair:?}");
        }
        // The FAB's highest level is the hovered one, and the surface is read 8 points past its reach.
        for style in [FabStyle::PrimaryContainer, FabStyle::SecondaryContainer] {
            assert_eq!(highest_level(style), Elevation::Level4);
            for size in SIZES {
                assert_eq!(surface_distance(fab_cell(style, size)), Points(22.0));
            }
        }
        // Inside the reach below, at every level a FAB has, and not at the container's edge.
        const { assert!(SHADOW_BELOW.0 > 0.0) };
        assert!(SHADOW_BELOW.0 < reach(Elevation::Level3).1);
    }

    #[test]
    fn the_shadow_is_read_below_the_middle_of_each_container() {
        let small = fab_cell(FabStyle::PrimaryContainer, FabSize::Small);
        let (x, y) = small.origin();
        assert_eq!(below(small), (Points(x.0 + 20.0), Points(y.0 + 42.0)));
        let large = fab_cell(FabStyle::SecondaryContainer, FabSize::Large);
        let (x, y) = large.origin();
        assert_eq!(below(large), (Points(x.0 + 48.0), Points(y.0 + 98.0)));
        // Below the box, never over it or another FAB's: no container or other probe shares the pixel.
        let all: Vec<FabCell> = FabCell::all().collect();
        for cell in &all {
            let (px, py) = below(*cell);
            let (x, y) = cell.origin();
            let (width, height) = cell.frame();
            assert!(py.0 > y.0 + height.0, "{cell:?}");
            assert!(px.0 > x.0 && px.0 < x.0 + width.0, "{cell:?}");
            for other in &all {
                let ((ox, oy), (ow, oh)) = (other.origin(), other.frame());
                let (ox, oy, ow, oh) = (ox.0, oy.0, ow.0, oh.0);
                assert!(
                    !(px.0 >= ox && px.0 <= ox + ow && py.0 >= oy && py.0 <= oy + oh),
                    "{cell:?} {other:?}"
                );
            }
        }
    }

    #[test]
    fn a_shot_reads_a_shadow_below_every_fab_and_a_higher_one_below_the_hovered() {
        let rest = shot_shadows(Shot::all()[0]);
        assert_eq!(rest.len(), FabCell::all().count());
        assert!(rest.iter().all(|s| s.what == "shadow below it"));
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        assert!(rest.iter().all(|s| s.than == Than::Color(surface)));
        assert!(rest.iter().all(|s| matches!(s.target, Target::Fab(_))));
        // The first FAB hovered: the second style's baseline is the one at rest to compare with.
        let shot = Shot::all()[1];
        let hovered = fab_cell(FabStyle::PrimaryContainer, FabSize::Baseline);
        let other = fab_cell(FabStyle::SecondaryContainer, FabSize::Baseline);
        assert_eq!(shot.hovered, Some(hovered));
        let found = shot_shadows(shot);
        assert_eq!(found.len(), rest.len() + 1);
        // The same reads as at rest, and one more, for the hovered FAB.
        let base: Vec<Shadow> = found
            .iter()
            .filter(|s| s.what == "shadow below it")
            .copied()
            .collect();
        assert_eq!(base.len(), rest.len());
        for (a, b) in base.iter().zip(&rest) {
            assert_eq!((a.target, a.x, a.y), (b.target, b.x, b.y));
        }
        let extras: Vec<Shadow> = found
            .iter()
            .filter(|s| s.what == "shadow below it, hovered")
            .copied()
            .collect();
        let [extra] = extras[..] else {
            panic!("one more read for the hovered FAB: {extras:?}");
        };
        assert_eq!(extra.target, Target::Fab(hovered));
        assert_eq!((extra.x, extra.y), below(hovered));
        let (other_x, other_y) = below(other);
        assert_eq!(extra.than, Than::Pixel(other_x, other_y));
        // No shot but the hover ones has any.
        for shot in Shot::all() {
            let count = shot_shadows(shot)
                .iter()
                .filter(|s| s.what == "shadow below it, hovered")
                .count();
            assert_eq!(count, usize::from(shot.hovered.is_some()), "{shot:?}");
        }
    }

    #[test]
    fn darker_is_every_channel_or_overall() {
        let color = Rgb([10, 20, 30]);
        assert!(Rgb([9, 19, 29]).darker_in_every_channel(color));
        assert!(!Rgb([9, 20, 29]).darker_in_every_channel(color));
        assert!(!Rgb([9, 19, 31]).darker_in_every_channel(color));
        assert!(!color.darker_in_every_channel(color));
        // Overall: none higher and the sum lower; one channel may stay.
        assert!(Rgb([10, 19, 30]).darker_overall(color));
        assert!(Rgb([9, 19, 29]).darker_overall(color));
        assert!(!color.darker_overall(color));
        assert!(!Rgb([9, 19, 31]).darker_overall(color));
        assert!(!Rgb([11, 18, 30]).darker_overall(color));
    }

    #[test]
    fn hovering_a_fab_changes_its_container_and_its_corner_not_the_surface_or_the_icon() {
        for cell in FabCell::all() {
            let rest = fab_probes(cell, &LIGHT, false);
            let hovered = fab_probes(cell, &LIGHT, true);
            assert_ne!(hovered[0], rest[0], "{cell:?}");
            assert_eq!(hovered[1], rest[1], "{cell:?}");
            assert_ne!(hovered[2], rest[2], "{cell:?}");
            assert_eq!(hovered[3], rest[3], "{cell:?}");
            // The corner takes the container's color, hovered or not.
            assert_eq!(hovered[2].expected, hovered[0].expected);
            assert_eq!(hovered[2].tolerance, hovered[0].tolerance);
            assert_eq!(rest[2].expected, rest[0].expected);
        }
    }

    #[test]
    fn a_fab_with_a_flat_corner_or_a_wider_container_fails_where_it_is_wrong() {
        let shot = Shot::all()[0];
        let cell = fab_cell(FabStyle::PrimaryContainer, FabSize::Baseline);
        let found = fab_probes(cell, &LIGHT, false);
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let container = Rgb::of(&LIGHT, ColorRole::PrimaryContainer);
        let paint = |probe: Probe, color: Rgb| {
            let mut image = painted(shot, 1);
            let (x, y) = pixel_of(&probe, 1);
            let start = ((y * image.width + x) * 4) as usize;
            image.rgba[start..start + 3].copy_from_slice(&color.0);
            check_colors(&image, shot).unwrap()
        };
        // A corner square instead of rounded: the surface's color where the container's is expected.
        let wrong = paint(found[2], surface);
        assert_eq!(wrong.len(), 1);
        assert_eq!(wrong[0].probe, found[2]);
        assert_eq!(wrong[0].probe.what, "inside the corner");
        // A container drawn past its box, or a FAB too big: the container's color where the surface's is expected.
        let wrong = paint(found[1], container);
        assert_eq!(wrong.len(), 1);
        assert_eq!(wrong[0].probe.what, "surface beside it");
        assert_eq!(wrong[0].found, Some(container));
        // A shadow's few levels of difference beside it are caught too: the surface is read exactly.
        let wrong = paint(
            found[1],
            Rgb([surface.0[0] - 1, surface.0[1], surface.0[2]]),
        );
        assert_eq!(wrong.len(), 1);
    }

    #[test]
    fn a_fab_with_no_shadow_or_the_same_shadow_hovered_fails_where_it_is_wrong() {
        let hovered = fab_cell(FabStyle::PrimaryContainer, FabSize::Baseline);
        let (x, y) = below(hovered);
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let put = |image: &mut Image, x: Points, y: Points, color: Rgb| {
            let start = ((points_to_u32(y) * image.width + points_to_u32(x)) * 4) as usize;
            image.rgba[start..start + 3].copy_from_slice(&color.0);
        };
        // At rest: a FAB with no shadow shows the surface below it, which is not darker than the surface.
        let rest = Shot::all()[0];
        let mut image = painted(rest, 1);
        assert_eq!(check_shadows(&image, rest), Ok(Vec::new()));
        put(&mut image, x, y, surface);
        let found = check_shadows(&image, rest).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe.target, Target::Fab(hovered));
        assert_eq!(found[0].found, Some(surface));
        assert_eq!(
            found[0].to_string(),
            format!(
                "fab-primary-container-baseline shadow below it at ({}, {}) points: expected darker than {surface}, found {surface}",
                x.0, y.0
            )
        );
        // One channel not darker is no shadow either.
        let mut image = painted(rest, 1);
        put(&mut image, x, y, Rgb([surface.0[0], 1, 1]));
        assert_eq!(check_shadows(&image, rest).unwrap().len(), 1);
        // Hovered: the FAB under the mouse casts the same shadow as its neighbor at rest, so the level did not rise.
        let shot = Shot::all()[1];
        let (other_x, other_y) = below(fab_cell(FabStyle::SecondaryContainer, FabSize::Baseline));
        let mut image = painted(shot, 1);
        assert_eq!(check_shadows(&image, shot), Ok(Vec::new()));
        put(&mut image, x, y, shaded(surface, 3, 4));
        let found = check_shadows(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe.what, "shadow below it, hovered");
        assert_eq!(found[0].probe.than, Than::Pixel(other_x, other_y));
        assert_eq!(found[0].reference, Some(shaded(surface, 3, 4)));
        // A lighter shadow than the neighbor's, a level lower, fails too.
        put(&mut image, x, y, shaded(surface, 7, 8));
        assert_eq!(check_shadows(&image, shot).unwrap().len(), 1);
        // And the rest of the image is not what is read: the shadows of the other FABs are unchanged.
        assert!(
            check_shadows(&image, shot)
                .unwrap()
                .iter()
                .all(|m| m.probe.target == Target::Fab(hovered))
        );
        // An image of another size is refused, as for the colors.
        let small = Image {
            width: 1,
            height: 1,
            rgba: vec![0; 4],
        };
        assert!(check_shadows(&small, rest).is_err());
    }

    #[test]
    fn a_target_is_named_by_its_test_id() {
        let fab = fab_cell(FabStyle::SecondaryContainer, FabSize::Medium);
        assert_eq!(Target::Fab(fab).test_id(), "fab-secondary-container-medium");
    }

    #[test]
    fn the_shots_are_the_test_script_files() {
        let names: Vec<String> = Shot::all().into_iter().map(Shot::file_name).collect();
        assert_eq!(
            names,
            [
                "fab-light-rest.png",
                "fab-light-hover-primary-container.png",
                "fab-light-hover-secondary-container.png",
                "fab-dark-rest.png",
                "fab-dark-hover-primary-container.png",
                "fab-dark-hover-secondary-container.png",
            ]
        );
        assert_eq!(Shot::all()[0].appearance.scheme(), LIGHT);
        assert_eq!(
            Shot::all()[3].appearance.scheme(),
            ColorScheme::BASELINE_DARK
        );
        // The FABs hovered are the baseline ones, one of each style.
        assert_eq!(Shot::all()[0].hovered, None);
        assert_eq!(
            Shot::all()[1].hovered,
            Some(fab_cell(FabStyle::PrimaryContainer, FabSize::Baseline))
        );
        assert_eq!(
            Shot::all()[2].hovered,
            Some(fab_cell(FabStyle::SecondaryContainer, FabSize::Baseline))
        );
    }

    #[test]
    fn every_shot_reads_every_fab_and_the_hovered_one_hovered() {
        let rest = shot_probes(Shot::all()[0]);
        assert_eq!(
            rest.len(),
            FabCell::all()
                .map(|c| fab_probes(c, &LIGHT, false).len())
                .sum::<usize>()
        );
        assert_eq!(rest.len(), FabCell::all().count() * 4);
        // The hovered FAB's container and its corner, both in the container's color: nothing else changes.
        let shot = Shot::all()[1];
        let changed: Vec<Probe> = shot_probes(shot)
            .into_iter()
            .zip(&rest)
            .filter(|(a, b)| a != *b)
            .map(|(a, _)| a)
            .collect();
        assert_eq!(
            changed.iter().map(|p| p.what).collect::<Vec<_>>(),
            ["container", "inside the corner"]
        );
        assert!(
            changed
                .iter()
                .all(|p| Some(p.target) == shot.hovered.map(Target::Fab))
        );
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
        let cell = fab_cell(FabStyle::PrimaryContainer, FabSize::Baseline);
        let (x, y) = below(cell);
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        for scale in [1, 2] {
            // No shadow at the one pixel the point names, whatever the scale: the surface is not darker than itself.
            // A read at another pixel would find the image's background, which is darker than the surface, and pass.
            let mut image = painted(shot, scale);
            let start =
                ((points_to_u32(y) * scale * image.width + points_to_u32(x) * scale) * 4) as usize;
            image.rgba[start..start + 3].copy_from_slice(&surface.0);
            let found = check_shadows(&image, shot).unwrap();
            assert_eq!(found.len(), 1, "scale {scale}");
            assert_eq!(found[0].probe.target, Target::Fab(cell), "scale {scale}");
            assert_eq!(found[0].found, Some(surface), "scale {scale}");
        }
    }

    #[test]
    fn a_screenshot_of_another_viewport_is_refused() {
        // `checkbox.test.tsx` opens the catalog at the viewport above the FABs' row.
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
