//! The list item's known pixels (ADR 0013): what `list_item.test.tsx` screenshots show of each list item of the
//! catalog (`catalog::list_item`), at rest and with the mouse over the two-line clickable item and the two-line
//! selected one.
//!
//! For every list item: the container's color at its left edge, and the colors of its two icons in their middles.
//! Where the item is selected or hovered its corners are 12 or 16 points wide, and the pixel at its top left is the
//! surface outside the corner. For a selected item, the container is told from the surface by its last row and by the
//! row below it, which tells its height. Where the antialiasing of an edge or of the text makes a color unknown,
//! nothing is checked.

use catalog::list_item::{Kind, Lines, ListCell, WIDTH};
use nsh_core::geometry::Points;
use nsh_m3e::color::{ColorRole, ColorScheme};
use nsh_m3e::tokens::Shape;
use nsh_m3e::tokens::androidx::{list_tokens, state_tokens};

use super::{Appearance, CONTAINER_INSET, Image, Mismatch, Probe, Rgb, Target, faded, pixel_of};

/// One screenshot `examples/catalog/list_item.test.tsx` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Shot {
    pub(crate) appearance: Appearance,
    /// The list item the mouse is over, if any.
    pub(crate) hovered: Option<ListCell>,
}

impl Shot {
    /// Every screenshot the script takes, in each appearance: the catalog at rest, then the mouse over the two-line
    /// clickable list item and over the two-line selected one.
    pub(crate) fn all() -> Vec<Self> {
        Appearance::ALL
            .into_iter()
            .flat_map(|appearance| {
                let shot = move |hovered| Self {
                    appearance,
                    hovered,
                };
                std::iter::once(shot(None)).chain(
                    [Kind::Clickable, Kind::Selected]
                        .map(|kind| shot(Some(ListCell::new(Lines::Two, kind)))),
                )
            })
            .collect()
    }

    /// Its file name, as the test script saves it.
    pub(crate) fn file_name(self) -> String {
        let appearance = self.appearance.name();
        match self.hovered {
            None => format!("list-{appearance}-rest.png"),
            Some(cell) => format!(
                "list-{appearance}-hover-{}.png",
                cell.label().to_lowercase()
            ),
        }
    }
}

/// How far from the container's top left corner the pixel outside its corner is read, in each direction: the pixel
/// whose center is 2.5 points from both edges. It is outside the arc of a corner more than 8.5 points wide and inside
/// that of a 4-point corner (`the_corner_pixel_tells_the_wide_corners_from_the_narrow_one`). Not a design value: where
/// the check looks.
const CORNER_PIXEL: Points = Points(2.0);

/// How far inside the container's left edge its last row is read: past the widest corner, the large one of a selected
/// or pressed item (`ListTokens.ItemSelectedContainerExpressiveShape`), and [`CONTAINER_INSET`] more. Not a design
/// value: where the check looks.
fn past_the_corner() -> Points {
    Points(
        corner_radius(list_tokens::ITEM_SELECTED_CONTAINER_EXPRESSIVE_SHAPE).0 + CONTAINER_INSET.0,
    )
}

/// The radius of `shape`'s corners, which every list item shape has the same at each corner.
fn corner_radius(shape: Shape) -> Points {
    match shape {
        Shape::Rounded(corners) => corners.top_start,
        // Square corners: a rectangle. A circle has no radius of its own, and no list item shape is one.
        _ => Points(0.0),
    }
}

/// The color of the container of an item of `kind`: the selected one, or the one at rest. androidx
/// `ListItemDefaults.defaultListItemColors` (line 215) gives a disabled item the container at rest, not the disabled
/// container `ListTokens.kt` holds (ADR 0012), as the list item takes it.
fn container_role(kind: Kind) -> ColorRole {
    match kind {
        Kind::Selected => list_tokens::ITEM_SELECTED_CONTAINER_COLOR,
        _ => list_tokens::ITEM_CONTAINER_COLOR,
    }
}

/// The color of the headline of an enabled item of `kind`, which the state layer takes.
fn content_role(kind: Kind) -> ColorRole {
    match kind {
        Kind::Selected => list_tokens::ITEM_SELECTED_LABEL_TEXT_COLOR,
        _ => list_tokens::ITEM_LABEL_TEXT_COLOR,
    }
}

/// The container's color of `cell` and its tolerance, with the mouse over it or not: the state layer is the headline's
/// color at the hover opacity over the container, where the item takes presses.
fn container(cell: ListCell, scheme: &ColorScheme, hovered: bool) -> (Rgb, u8) {
    let rest = Rgb::of(scheme, container_role(cell.kind));
    if hovered && cell.takes_presses() {
        faded(
            scheme,
            content_role(cell.kind),
            state_tokens::HOVER_STATE_LAYER_OPACITY,
            rest,
        )
    } else {
        (rest, 0)
    }
}

/// The color of an icon of `cell`, leading or trailing, and its tolerance. androidx gives the icons the colors of an
/// enabled or disabled, selected or unselected item, never of a hover (`ListItemColors`, ADR 0012): the tokens' own
/// hover colors for a selected item are not read by androidx's component.
fn icon_color(cell: ListCell, scheme: &ColorScheme, leading: bool) -> (Rgb, u8) {
    let surface = Rgb::of(scheme, container_role(cell.kind));
    match (cell.kind, leading) {
        (Kind::Disabled, true) => faded(
            scheme,
            list_tokens::ITEM_DISABLED_LEADING_ICON_COLOR,
            list_tokens::ITEM_DISABLED_LEADING_ICON_OPACITY,
            surface,
        ),
        (Kind::Disabled, false) => faded(
            scheme,
            list_tokens::ITEM_DISABLED_TRAILING_ICON_COLOR,
            list_tokens::ITEM_DISABLED_TRAILING_ICON_OPACITY,
            surface,
        ),
        (Kind::Selected, true) => (
            Rgb::of(scheme, list_tokens::ITEM_SELECTED_LEADING_ICON_COLOR),
            0,
        ),
        (Kind::Selected, false) => (
            Rgb::of(scheme, list_tokens::ITEM_SELECTED_TRAILING_ICON_COLOR),
            0,
        ),
        (_, true) => (Rgb::of(scheme, list_tokens::ITEM_LEADING_ICON_COLOR), 0),
        (_, false) => (Rgb::of(scheme, list_tokens::ITEM_TRAILING_ICON_COLOR), 0),
    }
}

/// The vertical middle of `cell`'s icons: androidx aligns the content to the center when the space inside the padding
/// is under 60 points (the one-line item's 36 and the two-line item's 52) and to the top from there (the three-line
/// item's 68, `ListItemDefaults.kt` line 551), so an icon of the three-line item starts at the padding's top.
fn icon_middle(cell: ListCell) -> Points {
    let (_, y) = cell.origin();
    match cell.lines {
        Lines::Three => Points(
            y.0 + list_tokens::ITEM_TOP_SPACE.0 + list_tokens::ITEM_LEADING_ICON_SIZE.0 / 2.0,
        ),
        _ => Points(y.0 + cell.height().0 / 2.0),
    }
}

/// The pixels the checks know for the list item `cell` in a screenshot in `scheme`, with the mouse over it or not.
pub(crate) fn probes(cell: ListCell, scheme: &ColorScheme, hovered: bool) -> Vec<Probe> {
    let target = Target::ListItem(cell);
    let (x, y) = cell.origin();
    let height = cell.height();
    let surface = Rgb::of(scheme, ColorRole::Surface);
    let (expected, tolerance) = container(cell, scheme, hovered);
    let probe = |what, x, y, expected, tolerance| Probe {
        target,
        what,
        x,
        y,
        expected,
        tolerance,
    };
    let mut probes = vec![
        // Inside the left edge at mid height: past the antialiased edge and before the leading icon.
        probe(
            "container",
            Points(x.0 + CONTAINER_INSET.0),
            Points(y.0 + height.0 / 2.0),
            expected,
            tolerance,
        ),
    ];
    if cell.kind == Kind::Selected || (hovered && cell.takes_presses()) {
        // The item's corners are 12 points (hovered) or 16 (selected, which a hover leaves) wide, so the pixel at the
        // top left of its box is outside them: the surface. A corner left at rest (4 points) would fill it with the
        // state layer's color. An item that is neither has no color of its own to tell the corner by.
        probes.push(probe(
            "outside the corner",
            Points(x.0 + CORNER_PIXEL.0),
            Points(y.0 + CORNER_PIXEL.0),
            surface,
            0,
        ));
    }
    if cell.kind == Kind::Selected {
        // Its last row is the container's and the row below it is the surface, whose gap is `catalog::grid::ROW_GAP`:
        // the container ends at the height of its token.
        probes.push(probe(
            "last row",
            Points(x.0 + past_the_corner().0),
            Points(y.0 + height.0 - 1.0),
            expected,
            tolerance,
        ));
        probes.push(probe(
            "below",
            Points(x.0 + past_the_corner().0),
            Points(y.0 + height.0),
            surface,
            0,
        ));
    }
    let middle = icon_middle(cell);
    let (leading, leading_tolerance) = icon_color(cell, scheme, true);
    let (trailing, trailing_tolerance) = icon_color(cell, scheme, false);
    probes.push(probe(
        "leading icon",
        Points(
            x.0 + list_tokens::ITEM_LEADING_SPACE.0 + list_tokens::ITEM_LEADING_ICON_SIZE.0 / 2.0,
        ),
        middle,
        leading,
        leading_tolerance,
    ));
    probes.push(probe(
        "trailing icon",
        Points(
            x.0 + WIDTH.0
                - list_tokens::ITEM_TRAILING_SPACE.0
                - list_tokens::ITEM_TRAILING_ICON_SIZE.0 / 2.0,
        ),
        middle,
        trailing,
        trailing_tolerance,
    ));
    probes
}

/// The pixels the checks know in `shot`, for every list item of the catalog.
fn shot_probes(shot: Shot) -> Vec<Probe> {
    let scheme = shot.appearance.scheme();
    ListCell::all()
        .flat_map(|cell| probes(cell, &scheme, shot.hovered == Some(cell)))
        .collect()
}

/// Every probe of `shot` that `image` does not match; an error if the image is not the catalog's list item viewport at
/// a whole scale.
pub(crate) fn check(image: &Image, shot: Shot) -> Result<Vec<Mismatch>, String> {
    let scale = image.scale_in(catalog::list_item::viewport())?;
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
    let (width, height) = catalog::list_item::viewport();
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
    use catalog::list_item::{KINDS, LINES};

    use super::*;

    const LIGHT: ColorScheme = ColorScheme::BASELINE_LIGHT;
    const DARK: ColorScheme = ColorScheme::BASELINE_DARK;

    fn find<'a>(probes: &'a [Probe], what: &str) -> Option<&'a Probe> {
        probes.iter().find(|probe| probe.what == what)
    }

    #[test]
    fn the_corner_pixel_tells_the_wide_corners_from_the_narrow_one() {
        // The pixel at (2, 2) has its center at (2.5, 2.5): outside a corner of radius `r`, whose arc is centered at
        // (r, r), when that center is more than `r` away.
        let outside = |shape| {
            let r = corner_radius(shape).0;
            let center = CORNER_PIXEL.0 + 0.5;
            let distance = ((r - center) * (r - center) * 2.0).sqrt();
            distance > r
        };
        assert!(outside(
            list_tokens::ITEM_HOVERED_CONTAINER_EXPRESSIVE_SHAPE
        ));
        assert!(outside(
            list_tokens::ITEM_SELECTED_CONTAINER_EXPRESSIVE_SHAPE
        ));
        assert!(outside(
            list_tokens::ITEM_PRESSED_CONTAINER_EXPRESSIVE_SHAPE
        ));
        assert!(!outside(list_tokens::ITEM_CONTAINER_EXPRESSIVE_SHAPE));
        assert_eq!(
            corner_radius(list_tokens::ITEM_CONTAINER_EXPRESSIVE_SHAPE),
            Points(4.0)
        );
        assert_eq!(
            corner_radius(list_tokens::ITEM_HOVERED_CONTAINER_EXPRESSIVE_SHAPE),
            Points(12.0)
        );
        assert_eq!(
            corner_radius(list_tokens::ITEM_SELECTED_CONTAINER_EXPRESSIVE_SHAPE),
            Points(16.0)
        );
        // A rectangle has no corner to speak of.
        assert_eq!(corner_radius(Shape::Rectangle), Points(0.0));
        assert_eq!(corner_radius(Shape::Circle), Points(0.0));
    }

    #[test]
    fn the_last_row_is_read_past_the_widest_corner() {
        assert_eq!(past_the_corner(), Points(20.0));
    }

    #[test]
    fn containers_are_the_kinds_token_colors() {
        for (scheme, name) in [(LIGHT, "light"), (DARK, "dark")] {
            let surface = Rgb::of(&scheme, ColorRole::Surface);
            let secondary = Rgb::of(&scheme, ColorRole::SecondaryContainer);
            for kind in KINDS {
                let cell = ListCell::new(Lines::Two, kind);
                let (found, tolerance) = container(cell, &scheme, false);
                // Only the selected item has a container of its own; the disabled one has the surface, as the
                // enabled ones do.
                let expected = if kind == Kind::Selected {
                    secondary
                } else {
                    surface
                };
                assert_eq!((found, tolerance), (expected, 0), "{name} {kind:?}");
            }
        }
        // The two containers are different colors, or the checks could not tell them.
        assert_ne!(
            Rgb::of(&LIGHT, ColorRole::SecondaryContainer),
            Rgb::of(&LIGHT, ColorRole::Surface)
        );
    }

    #[test]
    fn the_state_layer_takes_the_headlines_color_where_the_item_takes_presses() {
        for scheme in [LIGHT, DARK] {
            let surface = Rgb::of(&scheme, ColorRole::Surface);
            let secondary = Rgb::of(&scheme, ColorRole::SecondaryContainer);
            let on_surface = Rgb::of(&scheme, ColorRole::OnSurface);
            let on_secondary = Rgb::of(&scheme, ColorRole::OnSecondaryContainer);
            let hover = state_tokens::HOVER_STATE_LAYER_OPACITY;
            for (kind, expected) in [
                (Kind::Clickable, (on_surface.over(hover, surface), 2)),
                (Kind::Selected, (on_secondary.over(hover, secondary), 2)),
                // A plain item and a disabled one show nothing of a mouse over them.
                (Kind::Plain, (surface, 0)),
                (Kind::Disabled, (surface, 0)),
            ] {
                let cell = ListCell::new(Lines::One, kind);
                assert_eq!(container(cell, &scheme, true), expected, "{kind:?}");
                // The mouse changes what an item that takes presses shows, and no other.
                assert_eq!(
                    container(cell, &scheme, true) != container(cell, &scheme, false),
                    cell.takes_presses(),
                    "{kind:?}"
                );
            }
        }
    }

    #[test]
    fn the_icons_take_each_kinds_color_and_fade_when_disabled() {
        let scheme = LIGHT;
        let surface = Rgb::of(&scheme, ColorRole::Surface);
        let on_surface = Rgb::of(&scheme, ColorRole::OnSurface);
        let variant = Rgb::of(&scheme, ColorRole::OnSurfaceVariant);
        let on_secondary = Rgb::of(&scheme, ColorRole::OnSecondaryContainer);
        for leading in [true, false] {
            let color = |kind| icon_color(ListCell::new(Lines::One, kind), &scheme, leading);
            assert_eq!(color(Kind::Plain), (variant, 0), "{leading}");
            assert_eq!(color(Kind::Clickable), (variant, 0), "{leading}");
            assert_eq!(color(Kind::Selected), (on_secondary, 0), "{leading}");
            assert_eq!(color(Kind::Disabled), (on_surface.over(0.38, surface), 2));
        }
        assert_ne!(variant, on_secondary);
        assert_ne!(variant, on_surface.over(0.38, surface));
    }

    #[test]
    fn the_icons_are_in_the_middle_of_the_row_or_at_the_top_of_the_three_line_one() {
        let (_, y) = ListCell::new(Lines::One, Kind::Plain).origin();
        assert_eq!(
            icon_middle(ListCell::new(Lines::One, Kind::Plain)),
            Points(y.0 + 28.0)
        );
        let (_, y) = ListCell::new(Lines::Two, Kind::Plain).origin();
        assert_eq!(
            icon_middle(ListCell::new(Lines::Two, Kind::Plain)),
            Points(y.0 + 36.0)
        );
        // The padding's top (10 points) and half the icon (12).
        let (_, y) = ListCell::new(Lines::Three, Kind::Plain).origin();
        assert_eq!(
            icon_middle(ListCell::new(Lines::Three, Kind::Plain)),
            Points(y.0 + 22.0)
        );
    }

    #[test]
    fn every_item_reads_its_container_and_two_icons() {
        for cell in ListCell::all() {
            let found = probes(cell, &LIGHT, false);
            assert_eq!(
                find(&found, "container").map(|p| p.x),
                Some(Points(cell.origin().0.0 + 4.0))
            );
            let leading = find(&found, "leading icon").unwrap();
            let trailing = find(&found, "trailing icon").unwrap();
            assert_eq!(leading.x.0, cell.origin().0.0 + 28.0, "{cell:?}");
            assert_eq!(trailing.x.0, cell.origin().0.0 + 360.0 - 28.0, "{cell:?}");
            assert_eq!(leading.y, trailing.y, "{cell:?}");
            assert!(
                found
                    .iter()
                    .all(|probe| probe.target == Target::ListItem(cell))
            );
        }
    }

    #[test]
    fn the_corner_and_the_edge_are_read_where_the_item_has_a_color_of_its_own() {
        let selected = ListCell::new(Lines::Two, Kind::Selected);
        let found = probes(selected, &LIGHT, false);
        let (x, y) = selected.origin();
        let surface = Rgb::of(&LIGHT, ColorRole::Surface);
        let corner = find(&found, "outside the corner").unwrap();
        assert_eq!(
            (corner.x, corner.y, corner.expected),
            (Points(x.0 + 2.0), Points(y.0 + 2.0), surface)
        );
        // The last row of the container, 72 points tall, and the row below it.
        let last = find(&found, "last row").unwrap();
        assert_eq!((last.x, last.y), (Points(x.0 + 20.0), Points(y.0 + 71.0)));
        assert_eq!(
            last.expected,
            Rgb::of(&LIGHT, ColorRole::SecondaryContainer)
        );
        let below = find(&found, "below").unwrap();
        assert_eq!(
            (below.x, below.y, below.expected),
            (Points(x.0 + 20.0), Points(y.0 + 72.0), surface)
        );
        // An unselected item at rest has no color to tell its corners or its end by; hovered, it has the corner's.
        let clickable = ListCell::new(Lines::Two, Kind::Clickable);
        let rest = probes(clickable, &LIGHT, false);
        assert!(find(&rest, "outside the corner").is_none());
        assert!(find(&rest, "last row").is_none());
        let hovered = probes(clickable, &LIGHT, true);
        assert!(find(&hovered, "outside the corner").is_some());
        assert!(find(&hovered, "last row").is_none());
        // A plain item is not hovered into another shape.
        let plain = probes(ListCell::new(Lines::Two, Kind::Plain), &LIGHT, true);
        assert!(find(&plain, "outside the corner").is_none());
        assert_eq!(plain.len(), 3);
    }

    #[test]
    fn the_probes_never_read_an_edge_of_a_selected_item_inside_its_corner() {
        for lines in LINES {
            let cell = ListCell::new(lines, Kind::Selected);
            let found = probes(cell, &LIGHT, false);
            let (x, y) = cell.origin();
            for probe in &found {
                if probe.what == "last row" || probe.what == "below" {
                    // Past the widest corner, whose arc ends 16 points from the left edge.
                    assert!(probe.x.0 - x.0 >= 16.0 + 4.0, "{lines:?} {}", probe.what);
                    assert!(probe.y.0 >= y.0, "{lines:?}");
                }
            }
        }
    }

    #[test]
    fn the_shots_are_the_test_script_files() {
        let names: Vec<String> = Shot::all().into_iter().map(Shot::file_name).collect();
        assert_eq!(
            names,
            [
                "list-light-rest.png",
                "list-light-hover-clickable.png",
                "list-light-hover-selected.png",
                "list-dark-rest.png",
                "list-dark-hover-clickable.png",
                "list-dark-hover-selected.png",
            ]
        );
        assert_eq!(Shot::all()[0].appearance.scheme(), LIGHT);
        assert_eq!(Shot::all()[3].appearance.scheme(), DARK);
        // The list items hovered are the two-line clickable one and the two-line selected one.
        assert_eq!(
            Shot::all()[1].hovered,
            Some(ListCell::new(Lines::Two, Kind::Clickable))
        );
        assert_eq!(
            Shot::all()[2].hovered,
            Some(ListCell::new(Lines::Two, Kind::Selected))
        );
        // The script's names are the shots' (`list_item.test.tsx`).
        let script = include_str!("../../../examples/catalog/list_item.test.tsx");
        assert!(script.contains("`target/visual/list-${scheme}-rest.png`"));
        assert!(script.contains("`target/visual/list-${scheme}-hover-${kind}.png`"));
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
    fn the_mouse_changes_the_pixels_of_the_item_under_it_and_of_no_other() {
        let rest = shot_probes(Shot::all()[0]);
        for shot in &Shot::all()[1..3] {
            let hovered = shot_probes(*shot);
            // What the mouse adds or changes: a probe the shot at rest does not have.
            let changed: Vec<&Probe> = hovered
                .iter()
                .filter(|probe| !rest.contains(probe))
                .collect();
            assert!(!changed.is_empty(), "{shot:?}");
            assert!(
                changed
                    .iter()
                    .all(|probe| Some(probe.target) == shot.hovered.map(Target::ListItem)),
                "{shot:?}: {changed:?}"
            );
        }
    }

    #[test]
    fn a_wrong_pixel_fails_and_is_named() {
        // The light image has one pixel wrong, and the dark shot is not the light one.
        let shot = Shot::all()[0];
        let mut image = painted(shot, 1);
        let cell = ListCell::new(Lines::Two, Kind::Selected);
        let probe = probes(cell, &LIGHT, false)
            .into_iter()
            .find(|probe| probe.what == "last row")
            .unwrap();
        let (x, y) = pixel_of(&probe, 1);
        let start = ((y * image.width + x) * 4) as usize;
        image.rgba[start..start + 3].copy_from_slice(&Rgb::of(&LIGHT, ColorRole::Surface).0);
        let found = check(&image, shot).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].probe, probe);
        assert_ne!(
            check(&painted(shot, 1), Shot::all()[3]).unwrap(),
            Vec::new()
        );
    }

    #[test]
    fn a_screenshot_of_another_viewport_is_refused() {
        // `button.test.tsx` opens the catalog at the viewport above the list items.
        let (width, height) = catalog::grid::viewport();
        let (width, height) = (crate::points_to_u32(width), crate::points_to_u32(height));
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
