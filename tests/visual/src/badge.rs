//! The badges' known pixels in the catalog's screenshots.
//!
//! Each badge of the catalog sits on an anchor ([`catalog::badge`]), at a place androidx's `BadgedBox` gives it and the
//! catalog's layout test checks: this reads, in every screenshot, the color inside the badge (the error color, where
//! neither the label nor a corner reaches) and the surface just outside it on each side, which together say where
//! the badge is and that it has the size its tokens give. A label's width is its font's, so no probe depends on it:
//! the probes lie at the left of a label, where its padding is, and at its middle top and bottom, past the corners'
//! arcs and short of the text.

use std::fmt;

use catalog::badge::BadgeCell;
use nsh_core::geometry::Points;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::tokens::androidx::badge_tokens;

use super::{Appearance, Image, Rgb, points_to_u32};

/// How far inside a badge, and how far outside it, a probe lies, from the edge it is read near: past the
/// antialiased edge. Not a design value: where the check looks.
const INSET: Points = Points(2.0);

/// How far outside a badge a probe of the surface lies: past the antialiased edge, and short of the anchor's icon,
/// which starts 8 points inside the anchor. Not a design value: where the check looks.
const OUTSIDE: Points = Points(3.0);

/// A pixel the checks know, in the catalog's points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Probe {
    pub(crate) cell: BadgeCell,
    pub(crate) what: &'static str,
    pub(crate) x: Points,
    pub(crate) y: Points,
    pub(crate) expected: Rgb,
}

impl Probe {
    /// The pixel it reads: the top left one of the point it names, at `scale` pixels to a point.
    pub(crate) fn pixel(&self, scale: u32) -> (u32, u32) {
        (points_to_u32(self.x) * scale, points_to_u32(self.y) * scale)
    }
}

/// What a probe found when it was not what was expected.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Mismatch {
    pub(crate) probe: Probe,
    pub(crate) found: Option<Rgb>,
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let probe = &self.probe;
        write!(
            f,
            "{} {} at ({}, {}) points: expected {}, found ",
            probe.cell.test_id(),
            probe.what,
            probe.x.0,
            probe.y.0,
            probe.expected
        )?;
        match self.found {
            Some(found) => write!(f, "{found}"),
            None => write!(f, "no pixel"),
        }
    }
}

/// The pixels the checks know of the badge `cell` in `appearance`.
pub(crate) fn cell_probes(cell: BadgeCell, appearance: Appearance) -> Vec<Probe> {
    let scheme: ColorScheme = appearance.scheme();
    let badge = Rgb::of(&scheme, badge_tokens::COLOR);
    let surface = Rgb::of(&scheme, ColorRole::Surface);
    let (left, top) = cell.badge_origin();
    let height = cell.badge_height().0;
    let probe = |what, x: f32, y: f32, expected| Probe {
        cell,
        what,
        x: Points(x),
        y: Points(y),
        expected,
    };
    let middle = height / 2.0;
    if cell.label().is_none() {
        // A dot: its middle, and the surface a few points from each of its four sides, in line with its middle.
        vec![
            probe("dot", left.0 + middle, top.0 + middle, badge),
            probe(
                "left of the dot",
                left.0 - OUTSIDE.0,
                top.0 + middle,
                surface,
            ),
            probe(
                "right of the dot",
                left.0 + height + INSET.0,
                top.0 + middle,
                surface,
            ),
            probe("above the dot", left.0 + middle, top.0 - OUTSIDE.0, surface),
            probe(
                "below the dot",
                left.0 + middle,
                top.0 + height + INSET.0,
                surface,
            ),
        ]
    } else {
        // A label: its padding at the left, at mid height; its top and bottom at the middle of the height, which is
        // as far from the left end as the corners' radius; and the surface outside the left end, above and below.
        vec![
            probe("left of the label", left.0 + INSET.0, top.0 + middle, badge),
            probe("top of the label", left.0 + middle, top.0 + 1.0, badge),
            probe(
                "bottom of the label",
                left.0 + middle,
                top.0 + height - 2.0,
                badge,
            ),
            probe(
                "outside the label's left",
                left.0 - OUTSIDE.0,
                top.0 + middle,
                surface,
            ),
            probe(
                "above the label",
                left.0 + middle,
                top.0 - OUTSIDE.0,
                surface,
            ),
            probe(
                "below the label",
                left.0 + middle,
                top.0 + height + INSET.0,
                surface,
            ),
        ]
    }
}

/// The pixels the checks know of every badge of the catalog in `appearance`.
pub(crate) fn probes(appearance: Appearance) -> Vec<Probe> {
    BadgeCell::all()
        .flat_map(|cell| cell_probes(cell, appearance))
        .collect()
}

/// Every probe of `appearance` that `image` does not match.
pub(crate) fn check(image: &Image, appearance: Appearance) -> Result<Vec<Mismatch>, String> {
    let scale = image.scale()?;
    Ok(probes(appearance)
        .into_iter()
        .filter_map(|probe| {
            let (x, y) = probe.pixel(scale);
            let found = image.pixel(x, y);
            match found {
                Some(found) if found == probe.expected => None,
                _ => Some(Mismatch { probe, found }),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The probes of `cell` in light, as (what, x, y, expected) with the colors as roles' names.
    fn light(cell: BadgeCell) -> Vec<(&'static str, f32, f32, Rgb)> {
        cell_probes(cell, Appearance::Light)
            .into_iter()
            .map(|p| (p.what, p.x.0, p.y.0, p.expected))
            .collect()
    }

    /// Error and surface in the baseline light scheme: `ColorSchemeKeyTokens.Error` (`md_theme_light_error`) and
    /// `Surface`.
    const ERROR: Rgb = Rgb([0xb3, 0x26, 0x1e]);
    const SURFACE: Rgb = Rgb([0xfe, 0xf7, 0xff]);

    #[test]
    fn a_dot_is_read_in_its_middle_and_on_each_side() {
        // The dot's anchor is at (24, 2664), 40 points wide: the dot is [58, 64] x [2664, 2670], its middle (61, 2667).
        assert_eq!(
            light(BadgeCell::Dot),
            [
                ("dot", 61.0, 2667.0, ERROR),
                ("left of the dot", 55.0, 2667.0, SURFACE),
                ("right of the dot", 66.0, 2667.0, SURFACE),
                ("above the dot", 61.0, 2661.0, SURFACE),
                ("below the dot", 61.0, 2672.0, SURFACE),
            ]
        );
    }

    #[test]
    fn a_label_is_read_at_its_left_top_bottom_and_outside() {
        // The label "1": its anchor is at (224, 2664), the badge starts 12 points before the anchor's end, at 252, and
        // 2 above the anchor's top, at 2662; it is 16 tall.
        assert_eq!(
            light(BadgeCell::One),
            [
                ("left of the label", 254.0, 2670.0, ERROR),
                ("top of the label", 260.0, 2663.0, ERROR),
                ("bottom of the label", 260.0, 2676.0, ERROR),
                ("outside the label's left", 249.0, 2670.0, SURFACE),
                ("above the label", 260.0, 2659.0, SURFACE),
                ("below the label", 260.0, 2680.0, SURFACE),
            ]
        );
        // The other labels' probes are the same relative to their own badges: they do not depend on the width.
        for (cell, x) in [
            (BadgeCell::Nine, 424.0),
            (BadgeCell::NinetyNinePlus, 624.0),
            (BadgeCell::LongText, 824.0),
        ] {
            let found = light(cell);
            assert_eq!(found.len(), 6, "{cell:?}");
            assert_eq!(found[0].1, x + 30.0, "{cell:?}");
            assert_eq!(found[1].1, x + 36.0, "{cell:?}");
            assert_eq!(found[0].2, 2670.0, "{cell:?}");
        }
    }

    #[test]
    fn the_colors_are_the_tokens_in_each_appearance() {
        assert_eq!(
            Rgb::of(&ColorScheme::BASELINE_LIGHT, badge_tokens::COLOR),
            ERROR
        );
        assert_eq!(
            Rgb::of(&ColorScheme::BASELINE_LIGHT, ColorRole::Surface),
            SURFACE
        );
        let dark = cell_probes(BadgeCell::Dot, Appearance::Dark);
        // The dark error color and surface: `md_theme_dark_error` and `md_theme_dark_surface`.
        assert_eq!(dark[0].expected, Rgb([0xf2, 0xb8, 0xb5]));
        assert_eq!(dark[1].expected, Rgb([0x14, 0x12, 0x18]));
        assert_ne!(dark[0].expected, light(BadgeCell::Dot)[0].3);
    }

    #[test]
    fn every_badge_has_its_probes() {
        let all = probes(Appearance::Light);
        // A dot has 5, each of the four labels 6.
        assert_eq!(all.len(), 5 + 4 * 6);
        for cell in BadgeCell::all() {
            let own = all.iter().filter(|p| p.cell == cell).count();
            assert_eq!(own, if cell.label().is_none() { 5 } else { 6 }, "{cell:?}");
        }
    }

    #[test]
    fn a_probe_reads_the_top_left_pixel_of_its_point_at_the_scale() {
        let probe = cell_probes(BadgeCell::Dot, Appearance::Light)[0];
        assert_eq!(probe.pixel(1), (61, 2667));
        assert_eq!(probe.pixel(2), (122, 5334));
    }

    /// The catalog painted as the tokens say, at `scale`: every probe holds its expected color, and the rest a color
    /// no probe expects.
    fn painted(appearance: Appearance, scale: u32) -> Image {
        let (width, height) = catalog::grid::viewport();
        let (width, height) = (points_to_u32(width) * scale, points_to_u32(height) * scale);
        let mut rgba = [1, 2, 3, 255].repeat((width * height) as usize);
        for probe in probes(appearance) {
            let (x, y) = probe.pixel(scale);
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
    fn badges_drawn_as_the_tokens_say_pass_in_each_appearance_at_any_whole_scale() {
        for appearance in Appearance::ALL {
            for scale in [1, 2] {
                assert_eq!(
                    check(&painted(appearance, scale), appearance),
                    Ok(Vec::new())
                );
            }
        }
    }

    #[test]
    fn a_wrong_pixel_is_named_and_a_badge_in_the_other_appearance_fails() {
        let mut image = painted(Appearance::Light, 1);
        // The dot's middle painted as the surface: the dot is not there.
        let probe = cell_probes(BadgeCell::Dot, Appearance::Light)[0];
        let (x, y) = probe.pixel(1);
        let start = ((y * image.width + x) * 4) as usize;
        image.rgba[start..start + 3].copy_from_slice(&SURFACE.0);
        let found = check(&image, Appearance::Light).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe, probe);
        assert_eq!(found[0].found, Some(SURFACE));
        assert_eq!(
            found[0].to_string(),
            "badge-dot dot at (61, 2667) points: expected #b3261e, found #fef7ff"
        );
        // The light screenshot is not the dark one: every probe of it differs.
        let other = check(&painted(Appearance::Light, 1), Appearance::Dark).unwrap();
        assert_eq!(other.len(), probes(Appearance::Dark).len());
    }

    #[test]
    fn a_pixel_that_is_off_by_one_does_not_pass() {
        // A solid color must be exact: no tolerance, as the button checks' solid colors.
        let mut image = painted(Appearance::Light, 1);
        let probe = cell_probes(BadgeCell::One, Appearance::Light)[0];
        let (x, y) = probe.pixel(1);
        let start = ((y * image.width + x) * 4) as usize;
        image.rgba[start] = image.rgba[start].saturating_add(1);
        assert_eq!(check(&image, Appearance::Light).unwrap().len(), 1);
    }

    #[test]
    fn an_image_of_another_size_is_refused() {
        let mut image = painted(Appearance::Light, 1);
        image.height -= 1;
        assert!(check(&image, Appearance::Light).is_err());
        let missing = Mismatch {
            probe: cell_probes(BadgeCell::Dot, Appearance::Light)[0],
            found: None,
        };
        assert_eq!(
            missing.to_string(),
            "badge-dot dot at (61, 2667) points: expected #b3261e, found no pixel"
        );
    }
}
