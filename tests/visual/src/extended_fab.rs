//! The extended FAB's known pixels (ADR 0013): what `extended_fab.test.tsx`'s screenshots show of each extended FAB of
//! the catalog (`catalog::extended_fab`), at rest and with the mouse over the medium expanded FAB of each style.
//!
//! An extended FAB casts the FAB's shadow (ADR 0018), so a pixel is never read at the box's corner or next to the
//! container's edge, and its width is its label's, which each host measures with its own font: no check reads where an
//! expanded FAB ends. What is read is what the tokens and the layout fix. The container is read inside it, left of the
//! icon, and the corner from inside its arc; the icon's middle; the surface beyond the shadows' reach to the left. A
//! collapsed FAB is read at its own right edge: its container just inside it, its shadow just outside, and the surface
//! beyond, because a collapsed FAB is as wide as it is high. An expanded one is read past that square, along the top,
//! above every label's line, where its container goes on. The shadows are read below each container, and beside the
//! collapsed ones, where they must be darker than the surface, and below the FAB the mouse is over, where the shadow
//! must be darker than the other style's FAB at rest, a level lower.

use catalog::extended_fab::{ExtendedFabCell, Kind, STYLES, icon_size, leading_space};
use nsh_core::geometry::Points;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::extended_fab::ExtendedFabSize;
use nsh_m3e::tokens::androidx::{
    extended_fab_large_tokens, extended_fab_small_tokens, shape_tokens,
};

use super::fab::{
    SHADOW_BELOW, Shadow, ShadowMismatch, Than, check_shadow_reads, corner_radius, fab_roles,
    inside_the_arc, style_container, style_surface_distance,
};
use super::{Appearance, Image, Mismatch, Probe, Rgb, Target, pixel_of};

/// One screenshot `examples/catalog/extended_fab.test.tsx` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Shot {
    pub(crate) appearance: Appearance,
    /// The extended FAB the mouse is over, if any.
    pub(crate) hovered: Option<ExtendedFabCell>,
}

impl Shot {
    /// Every screenshot the script takes, in each appearance: the catalog at rest, then the mouse over the medium
    /// expanded FAB of each style.
    pub(crate) fn all() -> Vec<Self> {
        Appearance::ALL
            .into_iter()
            .flat_map(|appearance| {
                let shot = move |hovered| Self {
                    appearance,
                    hovered,
                };
                std::iter::once(shot(None)).chain(STYLES.into_iter().map(move |style| {
                    shot(Some(ExtendedFabCell::new(
                        style,
                        ExtendedFabSize::Medium,
                        Kind::Expanded,
                    )))
                }))
            })
            .collect()
    }

    /// Its file name, as the test script saves it.
    pub(crate) fn file_name(self) -> String {
        let appearance = self.appearance.name();
        match self.hovered {
            None => format!("extended-fab-{appearance}-rest.png"),
            Some(cell) => format!("extended-fab-{appearance}-hover-{}.png", cell.style_name()),
        }
    }
}

/// How far inside a collapsed container's right edge its container is read: past the antialiasing of the edge, which
/// spreads over about a point, and clear of the icon, which ends 16 points or more from the edge in every size.
/// Not a design value: where the check looks.
const EDGE_MARGIN: Points = Points(3.0);

/// How far past the collapsed width, from the container's left edge, an expanded container is read. Not a design value:
/// where the check looks. Its read is inside the container if the label makes it wider than the square by this and by
/// the radius of its corner: with 11 characters at the size of its type style (14, 22 and 24 points) that is so in any
/// font that draws them, which the checks cannot tell without a host's font (not measured).
const BEYOND_THE_SQUARE: Points = Points(8.0);

/// How far below the container's top an expanded container is read past the square: above the first line of every
/// label, the narrowest space between the top and a label's line being 18 points (the small label's 20-point line in
/// 56 points). Not a design value: where the check looks.
const ABOVE_THE_LABEL: Points = Points(6.0);

/// The radius of the corners of an extended FAB of `size` whose shorter side, its height, is `shorter`: its size's
/// `ContainerShape`, or for the medium one, whose token file has none, `CornerLargeIncreased` (androidx
/// `FloatingActionButton.kt` `mediumExtendedFabShape`, ADR 0012), as the FAB draws it.
fn container_radius(size: ExtendedFabSize, shorter: Points) -> Points {
    // No size may fall back on another's: one would hide the other's arm. A size the catalog does not show has no shape
    // here.
    let shape = match size {
        ExtendedFabSize::Small => Some(extended_fab_small_tokens::CONTAINER_SHAPE),
        ExtendedFabSize::Medium => Some(shape_tokens::CORNER_LARGE_INCREASED),
        ExtendedFabSize::Large => Some(extended_fab_large_tokens::CONTAINER_SHAPE),
        _ => None,
    };
    corner_radius(shape, shorter)
}

/// The pixels the checks know for the extended FAB `cell` in a screenshot in `scheme`, with the mouse over it or not.
///
/// None is read at the box's corner or next to the container's edge, where a shadow (ADR 0018) lies; [`shot_shadows`]
/// reads the shadows.
fn cell_probes(cell: ExtendedFabCell, scheme: &ColorScheme, hovered: bool) -> Vec<Probe> {
    let target = Target::ExtendedFab(cell);
    let (x, y) = cell.origin();
    let side = cell.height();
    let middle = Points(y.0 + side.0 / 2.0);
    let (container, tolerance) = style_container(cell.style, scheme, hovered);
    let (_, icon) = fab_roles(cell.style);
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let corner = inside_the_arc(container_radius(cell.size, side));
    let probe = |what, x: Points, y: Points, expected, tolerance| Probe {
        target,
        what,
        x,
        y,
        expected,
        tolerance,
    };
    let mut probes = vec![
        // Half the leading space in from the left edge, at mid height: past the antialiased edge, where the edge is
        // straight whatever the corners' radius, and before the icon or the label, which start at the leading space.
        probe(
            "container",
            Points(x.0 + leading_space(cell.size).0 / 2.0),
            middle,
            container,
            tolerance,
        ),
        // Beside the container, past the reach of its shadows, in the gap to the next FAB or the margin: the surface,
        // and nothing drawn around the container.
        probe(
            "surface beside it",
            Points(x.0 - style_surface_distance(cell.style).0),
            middle,
            surface,
            0,
        ),
        // Just inside the arc of the top left corner, on its diagonal: the container's color if the corner is no
        // rounder than its size says.
        probe(
            "inside the corner",
            Points(x.0 + corner.0),
            Points(y.0 + corner.0),
            container,
            tolerance,
        ),
    ];
    if cell.kind.has_icon() {
        // The middle of the icon, `catalog::ICON`, which is filled there: after the leading space, collapsed or not
        // (a collapsed FAB keeps it).
        probes.push(probe(
            "icon",
            Points(x.0 + leading_space(cell.size).0 + icon_size(cell.size).0 / 2.0),
            middle,
            Rgb::of(scheme, icon),
            0,
        ));
    }
    if cell.kind.is_collapsed() {
        probes.extend([
            // The square's right edge: the container is there, and just inside the arc of the top right corner (the
            // pixel as far from the right edge as the left corner's is from the left), and nothing is beyond the
            // shadows' reach.
            probe(
                "container at its right edge",
                Points(x.0 + side.0 - EDGE_MARGIN.0),
                middle,
                container,
                tolerance,
            ),
            probe(
                "inside the top right corner",
                Points(x.0 + side.0 - corner.0 - 1.0),
                Points(y.0 + corner.0),
                container,
                tolerance,
            ),
            probe(
                "surface beyond it",
                Points(x.0 + side.0 + style_surface_distance(cell.style).0),
                middle,
                surface,
                0,
            ),
        ]);
    } else {
        // Past the square, above the label: the container goes on, as wide as its content.
        probes.push(probe(
            "container past the square",
            Points(x.0 + side.0 + BEYOND_THE_SQUARE.0),
            Points(y.0 + ABOVE_THE_LABEL.0),
            container,
            tolerance,
        ));
    }
    probes
}

/// Where the shadow below the extended FAB `cell` is read: the middle of its first `height` points, which every FAB is
/// at least as wide as, [`SHADOW_BELOW`] below the container.
fn below(cell: ExtendedFabCell) -> (Points, Points) {
    let (x, y) = cell.origin();
    let side = cell.height();
    (
        Points(x.0 + side.0 / 2.0),
        Points(y.0 + side.0 + SHADOW_BELOW.0),
    )
}

/// Where the shadow beside the collapsed extended FAB `cell` is read: [`SHADOW_BELOW`] right of its container, at mid
/// height.
fn beside(cell: ExtendedFabCell) -> (Points, Points) {
    let (x, y) = cell.origin();
    let side = cell.height();
    (
        Points(x.0 + side.0 + SHADOW_BELOW.0),
        Points(y.0 + side.0 / 2.0),
    )
}

/// The pixels in the shadows of the extended FABs of `shot`: below each container, and beside each collapsed one, the
/// pixel is darker than the surface; below the FAB the mouse is over, it is darker than below the same-sized FAB of the
/// other style, which is at rest in the same screenshot and so one level lower.
fn shot_shadows(shot: Shot) -> Vec<Shadow> {
    let scheme = shot.appearance.scheme();
    let surface = Rgb::of(&scheme, ColorRole::Surface);
    let mut shadows = Vec::new();
    for cell in ExtendedFabCell::all() {
        let target = Target::ExtendedFab(cell);
        let (x, y) = below(cell);
        shadows.push(Shadow {
            target,
            what: "shadow below it",
            x,
            y,
            than: Than::Color(surface),
        });
        if cell.kind.is_collapsed() {
            let (x, y) = beside(cell);
            shadows.push(Shadow {
                target,
                what: "shadow beside it",
                x,
                y,
                than: Than::Color(surface),
            });
        }
        if shot.hovered == Some(cell) {
            let (x, y) = below(cell);
            for other in ExtendedFabCell::all()
                .filter(|o| o.size == cell.size && o.kind == cell.kind && o.style != cell.style)
            {
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

/// The pixels the checks know in `shot`, for every extended FAB of the catalog.
fn shot_probes(shot: Shot) -> Vec<Probe> {
    let scheme = shot.appearance.scheme();
    ExtendedFabCell::all()
        .flat_map(|cell| cell_probes(cell, &scheme, shot.hovered == Some(cell)))
        .collect()
}

/// Every shadow of `shot` that `image` does not show.
fn check_shadows(image: &Image, shot: Shot) -> Result<Vec<ShadowMismatch>, String> {
    check_shadow_reads(image, catalog::extended_fab::viewport(), shot_shadows(shot))
}

/// Every probe of `shot` that `image` does not match; an error if the image is not the catalog's extended FAB viewport
/// at a whole scale.
fn check_colors(image: &Image, shot: Shot) -> Result<Vec<Mismatch>, String> {
    let scale = image.scale_in(catalog::extended_fab::viewport())?;
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

/// What `image` does not show of `shot`: the pixels the tokens give and the shadows of the extended FABs, one line each.
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
    use super::fab::shaded;
    use super::points_to_u32;

    let (width, height) = catalog::extended_fab::viewport();
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

#[cfg(test)]
mod tests {
    use catalog::extended_fab::{COLUMN_WIDTH, KINDS, SIZES, height};
    use nsh_m3e::fab::FabStyle;
    use nsh_m3e::tokens::androidx::typography_tokens;

    use super::*;
    use crate::fab::shaded;
    use crate::{BLEND_TOLERANCE, points_to_u32};

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;

    fn cell(style: FabStyle, size: ExtendedFabSize, kind: Kind) -> ExtendedFabCell {
        ExtendedFabCell::new(style, size, kind)
    }

    /// The probe of `cell` named `what`.
    fn named(cell: ExtendedFabCell, what: &str, hovered: bool) -> Probe {
        let found: Vec<Probe> = cell_probes(cell, &LIGHT, hovered)
            .into_iter()
            .filter(|p| p.what == what)
            .collect();
        let [probe] = found[..] else {
            panic!("one probe named {what} for {cell:?}: {found:?}");
        };
        probe
    }

    fn names(cell: ExtendedFabCell) -> Vec<&'static str> {
        cell_probes(cell, &LIGHT, false)
            .iter()
            .map(|p| p.what)
            .collect()
    }

    #[test]
    fn the_shots_are_the_test_script_files() {
        let names: Vec<String> = Shot::all().into_iter().map(Shot::file_name).collect();
        assert_eq!(
            names,
            [
                "extended-fab-light-rest.png",
                "extended-fab-light-hover-primary-container.png",
                "extended-fab-light-hover-secondary-container.png",
                "extended-fab-dark-rest.png",
                "extended-fab-dark-hover-primary-container.png",
                "extended-fab-dark-hover-secondary-container.png",
            ]
        );
        assert_eq!(Shot::all()[0].appearance.scheme(), LIGHT);
        assert_eq!(
            Shot::all()[3].appearance.scheme(),
            ColorScheme::BASELINE_DARK
        );
        // The FABs hovered are the medium expanded ones, one of each style.
        assert_eq!(Shot::all()[0].hovered, None);
        assert_eq!(
            Shot::all()[1].hovered,
            Some(cell(
                FabStyle::PrimaryContainer,
                ExtendedFabSize::Medium,
                Kind::Expanded
            ))
        );
        assert_eq!(
            Shot::all()[2].hovered,
            Some(cell(
                FabStyle::SecondaryContainer,
                ExtendedFabSize::Medium,
                Kind::Expanded
            ))
        );
    }

    #[test]
    fn corners_are_the_size_tokens_radius_and_no_more_than_half_the_side() {
        let radii: Vec<f32> = SIZES
            .into_iter()
            .map(|size| container_radius(size, height(size)).0)
            .collect();
        // Small `CornerLarge`, medium `CornerLargeIncreased` (androidx `mediumExtendedFabShape`), large
        // `CornerExtraLarge`: no size has another's.
        assert_eq!(radii, [16.0, 20.0, 28.0]);
        for size in SIZES {
            assert!(container_radius(size, height(size)).0 <= height(size).0 / 2.0);
        }
        // The first whole point inside each arc, which the probes read.
        let offsets: Vec<f32> = radii
            .into_iter()
            .map(|radius| inside_the_arc(Points(radius)).0)
            .collect();
        assert_eq!(offsets, [5.0, 7.0, 9.0]);
    }

    #[test]
    fn every_cell_reads_its_container_the_surface_beside_it_and_the_corner() {
        for cell in ExtendedFabCell::all() {
            let found = names(cell);
            assert_eq!(
                &found[..3],
                ["container", "surface beside it", "inside the corner"]
            );
            assert_eq!(found.contains(&"icon"), cell.kind.has_icon(), "{cell:?}");
            assert!(
                cell_probes(cell, &LIGHT, false)
                    .iter()
                    .all(|p| p.target == Target::ExtendedFab(cell)),
                "{cell:?}"
            );
        }
        assert_eq!(
            names(cell(
                FabStyle::PrimaryContainer,
                ExtendedFabSize::Small,
                Kind::Expanded
            )),
            [
                "container",
                "surface beside it",
                "inside the corner",
                "icon",
                "container past the square"
            ]
        );
        assert_eq!(
            names(cell(
                FabStyle::PrimaryContainer,
                ExtendedFabSize::Small,
                Kind::Collapsed
            )),
            [
                "container",
                "surface beside it",
                "inside the corner",
                "icon",
                "container at its right edge",
                "inside the top right corner",
                "surface beyond it"
            ]
        );
        assert_eq!(
            names(cell(
                FabStyle::PrimaryContainer,
                ExtendedFabSize::Small,
                Kind::NoIcon
            )),
            [
                "container",
                "surface beside it",
                "inside the corner",
                "container past the square"
            ]
        );
    }

    /// A probe's place and what it expects, to compare in one assertion.
    fn read(probe: Probe) -> (f32, f32, Rgb, u8) {
        (probe.x.0, probe.y.0, probe.expected, probe.tolerance)
    }

    #[test]
    fn probes_are_read_where_the_layout_and_the_tokens_say() {
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let container = Rgb::of(&LIGHT, ColorRole::PrimaryContainer);
        let on_container = Rgb::of(&LIGHT, ColorRole::OnPrimaryContainer);
        // The small expanded FAB: 56 points high, a 16-point leading space, a 24-point icon, 16-point corners.
        let small = cell(
            FabStyle::PrimaryContainer,
            ExtendedFabSize::Small,
            Kind::Expanded,
        );
        let (x, y) = small.origin();
        assert_eq!(
            read(named(small, "container", false)),
            (x.0 + 8.0, y.0 + 28.0, container, 0)
        );
        assert_eq!(
            read(named(small, "surface beside it", false)),
            (x.0 - 22.0, y.0 + 28.0, surface, 0)
        );
        assert_eq!(
            read(named(small, "inside the corner", false)),
            (x.0 + 5.0, y.0 + 5.0, container, 0)
        );
        assert_eq!(
            read(named(small, "icon", false)),
            (x.0 + 28.0, y.0 + 28.0, on_container, 0)
        );
        assert_eq!(
            read(named(small, "container past the square", false)),
            (x.0 + 64.0, y.0 + 6.0, container, 0)
        );
        // The large collapsed FAB, secondary: 96 points, a 28-point leading space, a 32-point icon, 28-point corners.
        let secondary_container = Rgb::of(&LIGHT, ColorRole::SecondaryContainer);
        let on_secondary_container = Rgb::of(&LIGHT, ColorRole::OnSecondaryContainer);
        let large = cell(
            FabStyle::SecondaryContainer,
            ExtendedFabSize::Large,
            Kind::Collapsed,
        );
        let (x, y) = large.origin();
        assert_eq!(
            read(named(large, "container", false)),
            (x.0 + 14.0, y.0 + 48.0, secondary_container, 0)
        );
        assert_eq!(
            read(named(large, "inside the corner", false)),
            (x.0 + 9.0, y.0 + 9.0, secondary_container, 0)
        );
        // The icon is where the container keeps its leading space: 28 points in, 4 left of the middle.
        assert_eq!(
            read(named(large, "icon", false)),
            (x.0 + 44.0, y.0 + 48.0, on_secondary_container, 0)
        );
        assert_eq!(
            read(named(large, "container at its right edge", false)),
            (x.0 + 93.0, y.0 + 48.0, secondary_container, 0)
        );
        assert_eq!(
            read(named(large, "inside the top right corner", false)),
            (x.0 + 86.0, y.0 + 9.0, secondary_container, 0)
        );
        assert_eq!(
            read(named(large, "surface beyond it", false)),
            (x.0 + 118.0, y.0 + 48.0, surface, 0)
        );
        // The medium one with no icon: 80 points, a 26-point leading space, 20-point corners.
        let medium = cell(
            FabStyle::PrimaryContainer,
            ExtendedFabSize::Medium,
            Kind::NoIcon,
        );
        let (x, y) = medium.origin();
        assert_eq!(
            read(named(medium, "container", false)),
            (x.0 + 13.0, y.0 + 40.0, container, 0)
        );
        assert_eq!(
            read(named(medium, "inside the corner", false)),
            (x.0 + 7.0, y.0 + 7.0, container, 0)
        );
        assert_eq!(
            read(named(medium, "container past the square", false)),
            (x.0 + 88.0, y.0 + 6.0, container, 0)
        );
    }

    #[test]
    fn the_icon_is_read_after_the_leading_space_by_half_its_side() {
        // Small: 16 + 12, medium: 26 + 14, large: 28 + 16 points from the left edge, in the middle of the height.
        for (size, from_left) in SIZES.into_iter().zip([28.0, 40.0, 44.0]) {
            let c = cell(FabStyle::PrimaryContainer, size, Kind::Expanded);
            let (x, y) = c.origin();
            let icon = named(c, "icon", false);
            assert_eq!(
                (icon.x.0, icon.y.0),
                (x.0 + from_left, y.0 + height(size).0 / 2.0)
            );
            // Collapsed, the icon stays where it is, in the same container: the FAB keeps its leading space.
            let collapsed = cell(FabStyle::PrimaryContainer, size, Kind::Collapsed);
            let (cx, cy) = collapsed.origin();
            let icon = named(collapsed, "icon", false);
            assert_eq!(
                (icon.x.0, icon.y.0),
                (cx.0 + from_left, cy.0 + height(size).0 / 2.0)
            );
        }
    }

    #[test]
    fn probes_stay_clear_of_the_shadow_the_corner_and_each_other() {
        // The widest an expanded FAB can be in its column, which the catalog's test checks.
        let widest = COLUMN_WIDTH.0 - 44.0;
        let boxes: Vec<(ExtendedFabCell, f32, f32, f32, f32)> = ExtendedFabCell::all()
            .map(|cell| {
                let (x, y) = cell.origin();
                let side = cell.height().0;
                let width = if cell.kind.is_collapsed() {
                    side
                } else {
                    widest
                };
                (cell, x.0, y.0, x.0 + width, y.0 + side)
            })
            .collect();
        for &(cell, left, top, right, bottom) in &boxes {
            for probe in cell_probes(cell, &LIGHT, false) {
                // Never at the box's corner, where a shadow lies.
                assert_ne!(
                    (probe.x.0, probe.y.0),
                    (left, top),
                    "{cell:?} {}",
                    probe.what
                );
                let outside = probe.what.starts_with("surface");
                let inside =
                    probe.x.0 > left && probe.x.0 < right && probe.y.0 > top && probe.y.0 < bottom;
                // The surface is outside the box and the others inside it.
                assert_eq!(inside, !outside, "{cell:?} {}", probe.what);
                if outside {
                    // Past the reach of every FAB's shadows, its own included: outside each box grown by the reach
                    // of the highest level, 14 points to the sides and 20 below (ADR 0018, the reach the tests of
                    // `fab.rs` pin).
                    for &(other, other_left, other_top, other_right, other_bottom) in &boxes {
                        let clear = probe.x.0 < other_left - 14.0
                            || probe.x.0 > other_right + 14.0
                            || probe.y.0 < other_top - 14.0
                            || probe.y.0 > other_bottom + 20.0;
                        assert!(
                            clear,
                            "{cell:?}'s {} is in the shadow of {other:?}",
                            probe.what
                        );
                    }
                }
            }
            // The container is read before the icon or the label, which start at the leading space.
            let container = named(cell, "container", false);
            assert!(
                container.x.0 - left < leading_space(cell.size).0,
                "{cell:?}"
            );
            // And the pixel past the square is above the first line of the label, in every size: its line is the
            // type style's, centered in the height.
            let line = match cell.size {
                ExtendedFabSize::Small => typography_tokens::TITLE_MEDIUM.line_height.0,
                ExtendedFabSize::Medium => typography_tokens::TITLE_LARGE.line_height.0,
                _ => typography_tokens::HEADLINE_SMALL.line_height.0,
            };
            assert!(
                ABOVE_THE_LABEL.0 < (cell.height().0 - line) / 2.0,
                "{cell:?}"
            );
        }
        // The narrowest gap a surface is read in is the column's, less the widest FAB: it holds the reach and the
        // distance the surface is read at, on the side of the next FAB.
        for cell in ExtendedFabCell::all() {
            let distance = style_surface_distance(cell.style).0;
            assert!(distance <= catalog::grid::MARGIN.0, "{cell:?}");
            assert!(widest + 14.0 + distance <= COLUMN_WIDTH.0, "{cell:?}");
        }
    }

    #[test]
    fn the_shadows_are_read_below_every_container_and_beside_the_collapsed_ones() {
        let small = cell(
            FabStyle::PrimaryContainer,
            ExtendedFabSize::Small,
            Kind::Expanded,
        );
        let (x, y) = small.origin();
        assert_eq!(below(small), (Points(x.0 + 28.0), Points(y.0 + 58.0)));
        let large = cell(
            FabStyle::SecondaryContainer,
            ExtendedFabSize::Large,
            Kind::Collapsed,
        );
        let (x, y) = large.origin();
        assert_eq!(below(large), (Points(x.0 + 48.0), Points(y.0 + 98.0)));
        assert_eq!(beside(large), (Points(x.0 + 98.0), Points(y.0 + 48.0)));
        let rest = shot_shadows(Shot::all()[0]);
        let collapsed = ExtendedFabCell::all()
            .filter(|c| c.kind.is_collapsed())
            .count();
        assert_eq!(collapsed, 6);
        assert_eq!(rest.len(), ExtendedFabCell::all().count() + collapsed);
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        assert!(rest.iter().all(|s| s.than == Than::Color(surface)));
        for cell in ExtendedFabCell::all() {
            let mine: Vec<&'static str> = rest
                .iter()
                .filter(|s| s.target == Target::ExtendedFab(cell))
                .map(|s| s.what)
                .collect();
            let expected: &[&str] = if cell.kind.is_collapsed() {
                &["shadow below it", "shadow beside it"]
            } else {
                &["shadow below it"]
            };
            assert_eq!(mine, expected, "{cell:?}");
            // Below the box, never over it or another FAB's, and in the first `height` points of its width.
            let (px, py) = below(cell);
            let (x, y) = cell.origin();
            assert!(py.0 > y.0 + cell.height().0, "{cell:?}");
            assert!(px.0 > x.0 && px.0 < x.0 + cell.height().0, "{cell:?}");
        }
    }

    #[test]
    fn a_shot_reads_a_higher_shadow_below_the_hovered_fab_than_below_the_other_styles() {
        let shot = Shot::all()[1];
        let hovered = cell(
            FabStyle::PrimaryContainer,
            ExtendedFabSize::Medium,
            Kind::Expanded,
        );
        let other = cell(
            FabStyle::SecondaryContainer,
            ExtendedFabSize::Medium,
            Kind::Expanded,
        );
        assert_eq!(shot.hovered, Some(hovered));
        let found = shot_shadows(shot);
        let rest = shot_shadows(Shot::all()[0]);
        assert_eq!(found.len(), rest.len() + 1);
        let extras: Vec<Shadow> = found
            .iter()
            .filter(|s| s.what == "shadow below it, hovered")
            .copied()
            .collect();
        let [extra] = extras[..] else {
            panic!("one more read for the hovered FAB: {extras:?}");
        };
        assert_eq!(extra.target, Target::ExtendedFab(hovered));
        assert_eq!((extra.x, extra.y), below(hovered));
        let (other_x, other_y) = below(other);
        assert_eq!(extra.than, Than::Pixel(other_x, other_y));
        // The same reads as at rest otherwise, and no shot but the hover ones has the extra one.
        let base: Vec<Shadow> = found
            .iter()
            .filter(|s| s.what != "shadow below it, hovered")
            .copied()
            .collect();
        assert_eq!(base, rest);
        for shot in Shot::all() {
            let count = shot_shadows(shot)
                .iter()
                .filter(|s| s.what == "shadow below it, hovered")
                .count();
            assert_eq!(count, usize::from(shot.hovered.is_some()), "{shot:?}");
        }
    }

    #[test]
    fn hovering_an_extended_fab_changes_its_container_and_nothing_else() {
        let rest = ColorScheme::BASELINE_LIGHT;
        for cell in ExtendedFabCell::all() {
            let at_rest = cell_probes(cell, &rest, false);
            let hovered = cell_probes(cell, &rest, true);
            assert_eq!(at_rest.len(), hovered.len(), "{cell:?}");
            let on_primary = Rgb::of(&rest, ColorRole::OnPrimaryContainer);
            let on_secondary = Rgb::of(&rest, ColorRole::OnSecondaryContainer);
            let (container_role, on_container) = match cell.style {
                FabStyle::SecondaryContainer => (ColorRole::SecondaryContainer, on_secondary),
                _ => (ColorRole::PrimaryContainer, on_primary),
            };
            let container = Rgb::of(&rest, container_role);
            for (a, b) in at_rest.iter().zip(&hovered) {
                if a.expected == container {
                    // Hovered: the content's color at 0.08 over the container, with a blend's tolerance.
                    assert_eq!(
                        b.expected,
                        on_container.over(0.08, container),
                        "{cell:?} {}",
                        a.what
                    );
                    assert_eq!(b.tolerance, BLEND_TOLERANCE, "{cell:?} {}", a.what);
                    assert_ne!(b.expected, a.expected, "{cell:?} {}", a.what);
                } else {
                    // The surface and the icon are what they were.
                    assert_eq!(a, b, "{cell:?}");
                }
            }
        }
        // Only the hovered FAB's probes differ in a shot.
        let rest = shot_probes(Shot::all()[0]);
        let shot = Shot::all()[1];
        let changed: Vec<Probe> = shot_probes(shot)
            .into_iter()
            .zip(&rest)
            .filter(|(a, b)| a != *b)
            .map(|(a, _)| a)
            .collect();
        assert_eq!(
            changed.iter().map(|p| p.what).collect::<Vec<_>>(),
            [
                "container",
                "inside the corner",
                "container past the square"
            ]
        );
        assert!(
            changed
                .iter()
                .all(|p| Some(p.target) == shot.hovered.map(Target::ExtendedFab))
        );
    }

    #[test]
    fn a_wrong_corner_width_or_surface_fails_where_it_is_wrong() {
        let shot = Shot::all()[0];
        let paint = |probe: Probe, color: Rgb| {
            let mut image = painted(shot, 1);
            let (x, y) = pixel_of(&probe, 1);
            let start = ((y * image.width + x) * 4) as usize;
            image.rgba[start..start + 3].copy_from_slice(&color.0);
            check_colors(&image, shot).unwrap()
        };
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let container = Rgb::of(&LIGHT, ColorRole::PrimaryContainer);
        let expanded = cell(
            FabStyle::PrimaryContainer,
            ExtendedFabSize::Small,
            Kind::Expanded,
        );
        let collapsed = cell(
            FabStyle::PrimaryContainer,
            ExtendedFabSize::Small,
            Kind::Collapsed,
        );
        // A corner square instead of rounded: the surface's color where the container's is expected.
        let wrong = paint(named(expanded, "inside the corner", false), surface);
        assert_eq!(wrong.len(), 1);
        assert_eq!(wrong[0].probe.what, "inside the corner");
        // An expanded FAB that stops at the square (collapsed, or too narrow): the surface where its container goes on.
        let wrong = paint(named(expanded, "container past the square", false), surface);
        assert_eq!(wrong.len(), 1);
        assert_eq!(wrong[0].probe.what, "container past the square");
        assert_eq!(wrong[0].found, Some(surface));
        // A collapsed FAB that is wider than its square (its label shown): the container where the surface is expected.
        let wrong = paint(named(collapsed, "surface beyond it", false), container);
        assert_eq!(wrong.len(), 1);
        assert_eq!(wrong[0].probe.what, "surface beyond it");
        assert_eq!(wrong[0].found, Some(container));
        // A collapsed FAB narrower than its square: the surface where its right edge's container is expected, and a
        // square top right corner.
        let wrong = paint(
            named(collapsed, "container at its right edge", false),
            surface,
        );
        assert_eq!(wrong.len(), 1);
        let wrong = paint(
            named(collapsed, "inside the top right corner", false),
            surface,
        );
        assert_eq!(wrong.len(), 1);
        assert_eq!(wrong[0].probe.what, "inside the top right corner");
        // A shadow's few levels of difference beside the container are caught: the surface is read exactly.
        let wrong = paint(
            named(expanded, "surface beside it", false),
            Rgb([surface.0[0] - 1, surface.0[1], surface.0[2]]),
        );
        assert_eq!(wrong.len(), 1);
        // The icon missing, or in another color.
        let wrong = paint(named(expanded, "icon", false), container);
        assert_eq!(wrong.len(), 1);
        assert_eq!(wrong[0].probe.what, "icon");
    }

    #[test]
    fn no_shadow_or_the_same_shadow_hovered_fails_where_it_is_wrong() {
        let hovered = cell(
            FabStyle::PrimaryContainer,
            ExtendedFabSize::Medium,
            Kind::Expanded,
        );
        let (x, y) = below(hovered);
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let put = |image: &mut Image, x: Points, y: Points, color: Rgb| {
            let start = ((points_to_u32(y) * image.width + points_to_u32(x)) * 4) as usize;
            image.rgba[start..start + 3].copy_from_slice(&color.0);
        };
        // At rest: an extended FAB with no shadow shows the surface below it, which is not darker than the surface.
        let rest = Shot::all()[0];
        let mut image = painted(rest, 1);
        assert_eq!(check_shadows(&image, rest), Ok(Vec::new()));
        put(&mut image, x, y, surface);
        let found = check_shadows(&image, rest).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe.target, Target::ExtendedFab(hovered));
        assert_eq!(found[0].found, Some(surface));
        assert_eq!(
            found[0].to_string(),
            format!(
                "extended-fab-primary-container-medium-expanded shadow below it at ({}, {}) points: expected darker than {surface}, found {surface}",
                x.0, y.0
            )
        );
        // No shadow beside a collapsed one either.
        let collapsed = cell(
            FabStyle::SecondaryContainer,
            ExtendedFabSize::Large,
            Kind::Collapsed,
        );
        let (bx, by) = beside(collapsed);
        let mut image = painted(rest, 1);
        put(&mut image, bx, by, surface);
        let found = check_shadows(&image, rest).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe.what, "shadow beside it");
        assert_eq!(found[0].probe.target, Target::ExtendedFab(collapsed));
        // Hovered: the FAB under the mouse casts the same shadow as its neighbor at rest, so the level did not rise.
        let shot = Shot::all()[1];
        let (other_x, other_y) = below(cell(
            FabStyle::SecondaryContainer,
            ExtendedFabSize::Medium,
            Kind::Expanded,
        ));
        let mut image = painted(shot, 1);
        assert_eq!(check_shadows(&image, shot), Ok(Vec::new()));
        put(&mut image, x, y, shaded(surface, 3, 4));
        let found = check_shadows(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe.what, "shadow below it, hovered");
        assert_eq!(found[0].probe.than, Than::Pixel(other_x, other_y));
        // A lighter shadow than the neighbor's, a level lower, fails too.
        put(&mut image, x, y, shaded(surface, 7, 8));
        assert_eq!(check_shadows(&image, shot).unwrap().len(), 1);
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
        let c = cell(
            FabStyle::SecondaryContainer,
            ExtendedFabSize::Medium,
            Kind::Collapsed,
        );
        assert_eq!(
            Target::ExtendedFab(c).test_id(),
            "extended-fab-secondary-container-medium-collapsed"
        );
    }

    #[test]
    fn a_catalog_drawn_as_the_tokens_say_passes_at_any_whole_scale() {
        for shot in Shot::all() {
            assert_eq!(
                check(&painted(shot, 1), shot),
                Ok(Vec::new()),
                "{shot:?} at 1"
            );
        }
        // At the scale of a high-density screen: the rest and a hovered shot, which read every kind of pixel.
        for shot in [Shot::all()[0], Shot::all()[1]] {
            assert_eq!(
                check_colors(&painted(shot, 2), shot),
                Ok(Vec::new()),
                "{shot:?} at 2"
            );
            assert_eq!(
                check_shadows(&painted(shot, 2), shot),
                Ok(Vec::new()),
                "{shot:?} at 2"
            );
        }
    }

    #[test]
    fn a_screenshot_of_another_viewport_is_refused() {
        // `fab.test.tsx` opens the catalog at the viewport above the extended FABs' grid.
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

    #[test]
    fn every_kind_and_size_is_a_row_or_column_of_the_probes() {
        // The grid the probes follow: 3 sizes by 3 kinds by 2 styles.
        assert_eq!(
            SIZES.len() * KINDS.len() * STYLES.len(),
            ExtendedFabCell::all().count()
        );
        let total: usize = ExtendedFabCell::all()
            .map(|c| cell_probes(c, &LIGHT, false).len())
            .sum();
        assert_eq!(shot_probes(Shot::all()[0]).len(), total);
        // 4 probes (container, surface, corner, past the square) or 5 with an icon; the collapsed ones 7 with the icon.
        assert_eq!(total, 2 * 3 * (5 + 7 + 4));
    }
}
