//! Design tokens: Material 3's values, as Rust constants.
//!
//! [`androidx`] is generated from the vendored androidx token files and never edited by hand (ADR 0003). The types
//! its constants use are defined here.

pub mod androidx;

use androidx::motion_tokens::EASING_LINEAR_CUBIC_BEZIER;
use androidx::palette_tokens::BLACK;
use androidx::shape_tokens::CORNER_VALUE_NONE;
use androidx::typography_tokens::BODY_LARGE;
use nsh_core::element::FontWeight;
use nsh_core::geometry::Points;

/// A cubic Bézier easing curve from `(0, 0)` to `(1, 1)` through the control points `(x1, y1)` and `(x2, y2)`, as in
/// CSS `cubic-bezier()` and NSH's `Easing::Bezier(x1, y1, x2, y2)`.
///
/// Only tokens build one, so every curve in this crate is Material 3's:
///
/// ```
/// use nsh_m3e::tokens::androidx::motion_tokens::EASING_EMPHASIZED_DECELERATE_CUBIC_BEZIER as CURVE;
///
/// assert_eq!((CURVE.x1, CURVE.y1, CURVE.x2, CURVE.y2), (0.05, 0.7, 0.1, 1.0));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    /// The first control point's time, from 0 to 1.
    pub x1: f32,
    /// The first control point's progress.
    pub y1: f32,
    /// The second control point's time, from 0 to 1.
    pub x2: f32,
    /// The second control point's progress.
    pub y2: f32,
}

impl CubicBezier {
    pub(crate) const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

impl Default for CubicBezier {
    /// Constant speed: androidx `MotionTokens.EasingLinearCubicBezier`.
    fn default() -> Self {
        EASING_LINEAR_CUBIC_BEZIER
    }
}

/// An opaque color in 8-bit sRGB, as the androidx palette gives it.
///
/// Only tokens build one, so every color in this crate is Material 3's. [`Srgb::to_hex`] gives the `0xRRGGBB` that
/// NSH's `color::rgb` and `AdaptiveColor::rgb` take:
///
/// ```
/// use nsh_m3e::tokens::androidx::palette_tokens::PRIMARY40;
///
/// assert_eq!((PRIMARY40.red, PRIMARY40.green, PRIMARY40.blue), (103, 80, 164));
/// assert_eq!(PRIMARY40.to_hex(), 0x6750a4);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Srgb {
    /// The red channel, from 0 to 255.
    pub red: u8,
    /// The green channel, from 0 to 255.
    pub green: u8,
    /// The blue channel, from 0 to 255.
    pub blue: u8,
}

impl Srgb {
    pub(crate) const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// The color as `0xRRGGBB`.
    #[must_use]
    pub const fn to_hex(self) -> u32 {
        // Bit positions of the `0xRRGGBB` layout, not design values. The channels never overlap, so `+` is `|`; it
        // is written `+` because `|` and `^` would give the same result, and no test could tell them apart.
        ((self.red as u32) << 16) + ((self.green as u32) << 8) + self.blue as u32
    }
}

impl Default for Srgb {
    /// Black: androidx `PaletteTokens.Black`.
    fn default() -> Self {
        BLACK
    }
}

/// A generic font family: a kind of typeface the platform chooses, as androidx `FontFamily.SansSerif` is.
///
/// Only tokens name one, so every family in this crate is Material 3's:
///
/// ```
/// use nsh_m3e::tokens::GenericFontFamily;
/// use nsh_m3e::tokens::androidx::typeface_tokens::PLAIN;
///
/// assert_eq!(PLAIN, GenericFontFamily::SansSerif);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenericFontFamily {
    /// The platform's sans-serif typeface: androidx `FontFamily.SansSerif` (CSS `sans-serif`). NSH draws text that
    /// sets no font family in the app's default font, its `font-sans` token, or else the system font.
    SansSerif,
}

/// A type style of the Material 3 type scale: the font, weight, size, line height and tracking a text is drawn with,
/// as androidx's `TextStyle` holds them.
///
/// Only tokens build one, so every type style in this crate is Material 3's:
///
/// ```
/// use nsh_m3e::tokens::GenericFontFamily;
/// use nsh_m3e::tokens::androidx::typography_tokens::LABEL_LARGE;
///
/// assert_eq!(LABEL_LARGE.font, GenericFontFamily::SansSerif);
/// assert_eq!(LABEL_LARGE.weight.0, 500);
/// assert_eq!((LABEL_LARGE.size.0, LABEL_LARGE.line_height.0, LABEL_LARGE.tracking.0), (14.0, 20.0, 0.1));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeStyle {
    /// The font family.
    pub font: GenericFontFamily,
    /// The weight, from 100 to 900.
    pub weight: FontWeight,
    /// The text's size: androidx gives it in sp, which NSH points are, scaled by the user's text size.
    pub size: Points,
    /// The height of each line, in the same unit as the size. androidx centers a line's text in it.
    pub line_height: Points,
    /// The extra space between letters (CSS `letter-spacing`). NSH does not scale it by the user's text size.
    pub tracking: Points,
}

impl TypeStyle {
    pub(crate) const fn new(
        font: GenericFontFamily,
        weight: FontWeight,
        size: Points,
        line_height: Points,
        tracking: Points,
    ) -> Self {
        Self {
            font,
            weight,
            size,
            line_height,
            tracking,
        }
    }
}

impl Default for TypeStyle {
    /// The style of text that sets none: androidx `MaterialTheme` provides `typography.bodyLarge`
    /// (`MaterialTheme.kt`), which is `TypographyTokens.BodyLarge`.
    fn default() -> Self {
        BODY_LARGE
    }
}

/// The shape of a component's container, as androidx builds it.
///
/// Only tokens build one, so every shape in this crate is Material 3's:
///
/// ```
/// use nsh_m3e::tokens::Shape;
/// use nsh_m3e::tokens::androidx::shape_tokens::{CORNER_EXTRA_SMALL_TOP, CORNER_FULL};
///
/// assert_eq!(CORNER_FULL, Shape::Circle);
/// let Shape::Rounded(corners) = CORNER_EXTRA_SMALL_TOP else {
///     unreachable!("androidx builds `CornerExtraSmallTop` with `RoundedCornerShape`");
/// };
/// assert_eq!((corners.top_start.0, corners.bottom_end.0), (4.0, 0.0));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// Square corners: androidx `RectangleShape`.
    Rectangle,
    /// Each corner rounded by half the box's shorter side, which makes a circle of a square and a pill of a longer
    /// box: androidx `CircleShape`. NSH's `rounded_full()` draws it.
    Circle,
    /// Each corner rounded by its own radius: androidx `RoundedCornerShape`.
    Rounded(RoundedCorners),
}

impl Shape {
    /// A [`Shape::Rounded`] with the radius of each corner, in androidx's order (`topStart`, `topEnd`, `bottomEnd`,
    /// `bottomStart`).
    pub(crate) const fn rounded(
        top_start: Points,
        top_end: Points,
        bottom_end: Points,
        bottom_start: Points,
    ) -> Self {
        Self::Rounded(RoundedCorners {
            top_start,
            top_end,
            bottom_end,
            bottom_start,
        })
    }

    /// A [`Shape::Rounded`] with the same radius at every corner.
    pub(crate) const fn rounded_all(radius: Points) -> Self {
        Self::rounded(radius, radius, radius, radius)
    }
}

/// The radius of each corner of a [`Shape::Rounded`], in NSH points (one dp each).
///
/// The corners are named by the reading direction, as androidx and NSH name them: in a left-to-right layout, the
/// start is the left. NSH's `rounded_ss`, `rounded_se`, `rounded_ee` and `rounded_es` take them in this order.
///
/// ```
/// use nsh_m3e::tokens::Shape;
/// use nsh_m3e::tokens::androidx::shape_tokens::CORNER_LARGE_END;
///
/// let Shape::Rounded(corners) = CORNER_LARGE_END else {
///     unreachable!("androidx builds `CornerLargeEnd` with `RoundedCornerShape`");
/// };
/// assert_eq!(
///     [corners.top_start, corners.top_end, corners.bottom_end, corners.bottom_start].map(|r| r.0),
///     [0.0, 16.0, 16.0, 0.0]
/// );
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoundedCorners {
    /// The top corner on the start side.
    pub top_start: Points,
    /// The top corner on the end side.
    pub top_end: Points,
    /// The bottom corner on the end side.
    pub bottom_end: Points,
    /// The bottom corner on the start side.
    pub bottom_start: Points,
}

impl Default for RoundedCorners {
    /// Square corners: androidx `ShapeTokens.CornerValueNone` at each.
    fn default() -> Self {
        Self {
            top_start: CORNER_VALUE_NONE,
            top_end: CORNER_VALUE_NONE,
            bottom_end: CORNER_VALUE_NONE,
            bottom_start: CORNER_VALUE_NONE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{ColorRole, ColorScheme};
    use androidx::motion_tokens::EASING_EMPHASIZED_ACCELERATE_CUBIC_BEZIER;
    use androidx::{
        button_large_tokens, button_medium_tokens, button_small_tokens, button_xlarge_tokens,
        button_xsmall_tokens, color_dark_tokens, color_light_tokens, elevated_button_tokens,
        elevation_tokens, filled_button_tokens, palette_tokens, shape_tokens, type_scale_tokens,
        typeface_tokens, typography_tokens,
    };
    use nsh_core::element::FontWeight;

    #[test]
    fn new_keeps_each_control_point_in_place() {
        // androidx: `MotionTokens.EasingEmphasizedAccelerateCubicBezier` = (0.3, 0.0, 0.8, 0.15). Every point
        // differs, so a swapped argument shows.
        let curve = EASING_EMPHASIZED_ACCELERATE_CUBIC_BEZIER;
        assert_eq!(
            [curve.x1, curve.y1, curve.x2, curve.y2].map(f32::to_bits),
            [0.3_f32, 0.0, 0.8, 0.15].map(f32::to_bits)
        );
    }

    #[test]
    fn the_default_is_the_linear_token_and_not_another_curve() {
        let default = CubicBezier::default();
        assert_eq!(default, EASING_LINEAR_CUBIC_BEZIER);
        assert_eq!(
            [default.x1, default.y1, default.x2, default.y2].map(f32::to_bits),
            [0.0_f32, 0.0, 1.0, 1.0].map(f32::to_bits)
        );
    }

    #[test]
    fn new_keeps_each_channel_in_place() {
        // androidx: `PaletteTokens.Primary40` = (103, 80, 164). Every channel differs, so a swapped argument shows.
        let color = palette_tokens::PRIMARY40;
        assert_eq!((color.red, color.green, color.blue), (103, 80, 164));
    }

    #[test]
    fn to_hex_puts_each_channel_in_its_own_byte() {
        assert_eq!(Srgb::new(0x12, 0x34, 0x56).to_hex(), 0x0012_3456);
        assert_eq!(Srgb::new(0xff, 0, 0).to_hex(), 0x00ff_0000);
        assert_eq!(Srgb::new(0, 0xff, 0).to_hex(), 0x0000_ff00);
        assert_eq!(Srgb::new(0, 0, 0xff).to_hex(), 0x0000_00ff);
        assert_eq!(Srgb::new(0xff, 0xff, 0xff).to_hex(), 0x00ff_ffff);
    }

    #[test]
    fn the_default_is_black_and_not_another_color() {
        assert_eq!(Srgb::default(), BLACK);
        assert_eq!(Srgb::default().to_hex(), 0);
        assert_ne!(Srgb::default(), palette_tokens::WHITE);
    }

    #[test]
    fn scheme_roles_resolve_to_their_palette_tones() {
        // androidx: `ColorLightTokens.Primary` = `PaletteTokens.Primary40`, `ColorDarkTokens.Primary` =
        // `PaletteTokens.Primary80`, `ColorLightTokens.Background` = `PaletteTokens.Neutral98`.
        assert_eq!(color_light_tokens::PRIMARY, palette_tokens::PRIMARY40);
        assert_eq!(color_dark_tokens::PRIMARY, palette_tokens::PRIMARY80);
        assert_eq!(color_light_tokens::BACKGROUND, palette_tokens::NEUTRAL98);
        assert_ne!(color_light_tokens::PRIMARY, color_dark_tokens::PRIMARY);
    }

    #[test]
    fn a_length_in_dp_is_the_same_number_of_nsh_points() {
        // androidx: `ElevationTokens.Level0` = 0.0.dp, `Level2` = 3.0.dp, `Level5` = 12.0.dp. One dp is one NSH
        // point (`nsh_core::geometry::Points`).
        assert_eq!(elevation_tokens::LEVEL0.0.to_bits(), 0.0_f32.to_bits());
        assert_eq!(elevation_tokens::LEVEL2.0.to_bits(), 3.0_f32.to_bits());
        assert_eq!(elevation_tokens::LEVEL5.0.to_bits(), 12.0_f32.to_bits());
    }

    fn corners(shape: Shape) -> [f32; 4] {
        let Shape::Rounded(corners) = shape else {
            panic!("{shape:?} is not a rounded shape");
        };
        [
            corners.top_start,
            corners.top_end,
            corners.bottom_end,
            corners.bottom_start,
        ]
        .map(|radius| radius.0)
    }

    #[test]
    fn rounded_keeps_each_corner_in_place() {
        // androidx: `ShapeTokens.CornerLargeEnd` = `RoundedCornerShape(topStart = 0.0.dp, topEnd = 16.0.dp,
        // bottomEnd = 16.0.dp, bottomStart = 0.0.dp)`, `CornerLargeStart` the mirror image. A start and an end
        // swapped would turn one into the other.
        assert_eq!(
            corners(shape_tokens::CORNER_LARGE_END),
            [0.0, 16.0, 16.0, 0.0]
        );
        assert_eq!(
            corners(shape_tokens::CORNER_LARGE_START),
            [16.0, 0.0, 0.0, 16.0]
        );
        // androidx: `ShapeTokens.CornerExtraSmallTop`: only the top is rounded, so a top and a bottom swapped shows.
        assert_eq!(
            corners(shape_tokens::CORNER_EXTRA_SMALL_TOP),
            [4.0, 4.0, 0.0, 0.0]
        );
        // Direct, so every corner differs.
        assert_eq!(
            corners(Shape::rounded(
                Points(1.0),
                Points(2.0),
                Points(3.0),
                Points(4.0)
            )),
            [1.0, 2.0, 3.0, 4.0]
        );
    }

    #[test]
    fn rounded_all_puts_the_radius_at_every_corner() {
        // androidx: `ShapeTokens.CornerExtraLarge` = `RoundedCornerShape(28.0.dp)`.
        assert_eq!(corners(shape_tokens::CORNER_EXTRA_LARGE), [28.0; 4]);
        assert_eq!(corners(Shape::rounded_all(Points(5.0))), [5.0; 4]);
    }

    #[test]
    fn full_and_none_are_not_rounded_shapes() {
        // androidx: `ShapeTokens.CornerFull` = `CircleShape`, `CornerNone` = `RectangleShape`.
        assert_eq!(shape_tokens::CORNER_FULL, Shape::Circle);
        assert_eq!(shape_tokens::CORNER_NONE, Shape::Rectangle);
        assert_ne!(shape_tokens::CORNER_FULL, shape_tokens::CORNER_NONE);
    }

    #[test]
    fn the_default_corners_are_square_and_not_another_size() {
        let default = corners(Shape::Rounded(RoundedCorners::default())).map(f32::to_bits);
        assert_eq!(default, [CORNER_VALUE_NONE.0.to_bits(); 4]);
        assert_eq!(default, [0.0_f32.to_bits(); 4]);
    }

    #[test]
    fn the_weights_are_androidx_numbers_and_nsh_names() {
        // androidx `FontWeight.kt`: `Normal = W400`, `Medium = W500`, `Bold = W700`; `TypefaceTokens` names them
        // regular, medium and bold.
        assert_eq!(typeface_tokens::WEIGHT_REGULAR, FontWeight(400));
        assert_eq!(typeface_tokens::WEIGHT_MEDIUM, FontWeight(500));
        assert_eq!(typeface_tokens::WEIGHT_BOLD, FontWeight(700));
        assert_eq!(typeface_tokens::WEIGHT_REGULAR, FontWeight::NORMAL);
        assert_eq!(typeface_tokens::WEIGHT_MEDIUM, FontWeight::MEDIUM);
        assert_eq!(typeface_tokens::WEIGHT_BOLD, FontWeight::BOLD);
    }

    #[test]
    fn a_type_style_keeps_each_value_in_its_place() {
        // androidx `TypeScaleTokens.BodyLarge*`: Plain, WeightRegular, 16.sp, 24.0.sp, 0.5.sp. Size, line height
        // and tracking all differ, so a swap shows.
        assert_eq!(type_scale_tokens::BODY_LARGE_FONT, typeface_tokens::PLAIN);
        assert_eq!(
            type_scale_tokens::BODY_LARGE_WEIGHT,
            typeface_tokens::WEIGHT_REGULAR
        );
        assert_eq!(
            [
                type_scale_tokens::BODY_LARGE_SIZE,
                type_scale_tokens::BODY_LARGE_LINE_HEIGHT,
                type_scale_tokens::BODY_LARGE_TRACKING,
            ]
            .map(|length| length.0.to_bits()),
            [16.0_f32, 24.0, 0.5].map(f32::to_bits)
        );
        // androidx: `TypeScaleTokens.DisplayLargeTracking` = -0.2.sp, the one negative value.
        assert_eq!(
            type_scale_tokens::DISPLAY_LARGE_TRACKING.0.to_bits(),
            (-0.2_f32).to_bits()
        );
    }

    #[test]
    fn an_emphasized_style_is_heavier_than_its_plain_one() {
        // androidx: `TypeScaleTokens.TitleSmallWeight` = `WeightMedium`, `TitleSmallEmphasizedWeight` =
        // `WeightBold`; `LabelLargeWeight` = `WeightMedium`, `BodyLargeEmphasizedWeight` = `WeightMedium`.
        assert_eq!(
            type_scale_tokens::TITLE_SMALL_WEIGHT,
            typeface_tokens::WEIGHT_MEDIUM
        );
        assert_eq!(
            type_scale_tokens::TITLE_SMALL_EMPHASIZED_WEIGHT,
            typeface_tokens::WEIGHT_BOLD
        );
        assert_ne!(
            type_scale_tokens::BODY_LARGE_WEIGHT,
            type_scale_tokens::BODY_LARGE_EMPHASIZED_WEIGHT
        );
        assert_eq!(
            type_scale_tokens::DISPLAY_LARGE_FONT,
            typeface_tokens::BRAND
        );
    }

    #[test]
    fn a_button_color_is_a_role_resolved_by_the_scheme() {
        // androidx: `FilledButtonTokens.ContainerColor` = `ColorSchemeKeyTokens.Primary`, and
        // `ColorLightTokens.Primary` = `PaletteTokens.Primary40`, `ColorDarkTokens.Primary` = `Primary80`.
        assert_eq!(filled_button_tokens::CONTAINER_COLOR, ColorRole::Primary);
        assert_eq!(
            ColorScheme::BASELINE_LIGHT.get(filled_button_tokens::CONTAINER_COLOR),
            palette_tokens::PRIMARY40
        );
        assert_eq!(
            ColorScheme::BASELINE_DARK.get(filled_button_tokens::CONTAINER_COLOR),
            palette_tokens::PRIMARY80
        );
        // androidx: `FilledButtonTokens.LabelTextColor` = `ColorSchemeKeyTokens.OnPrimary`: not the container's role.
        assert_eq!(filled_button_tokens::LABEL_TEXT_COLOR, ColorRole::OnPrimary);
    }

    #[test]
    fn a_button_shape_is_the_shape_its_key_names() {
        // androidx: `ButtonSmallTokens.ContainerShapeRound` = `ShapeKeyTokens.CornerFull`, which is
        // `ShapeTokens.CornerFull` = `CircleShape`; `PressedContainerShape` = `CornerSmall` = 8.0.dp at every corner.
        assert_eq!(button_small_tokens::CONTAINER_SHAPE_ROUND, Shape::Circle);
        assert_eq!(
            button_small_tokens::PRESSED_CONTAINER_SHAPE,
            shape_tokens::CORNER_SMALL
        );
        assert_eq!(
            corners(button_small_tokens::PRESSED_CONTAINER_SHAPE),
            [8.0; 4]
        );
    }

    #[test]
    fn button_sizes_grow_from_extra_small_to_extra_large() {
        // androidx: `ContainerHeight` of `ButtonXSmallTokens` to `ButtonXLargeTokens`: 32, 40, 56, 96 and 136 dp.
        let heights = [
            button_xsmall_tokens::CONTAINER_HEIGHT,
            button_small_tokens::CONTAINER_HEIGHT,
            button_medium_tokens::CONTAINER_HEIGHT,
            button_large_tokens::CONTAINER_HEIGHT,
            button_xlarge_tokens::CONTAINER_HEIGHT,
        ]
        .map(|height| height.0.to_bits());
        assert_eq!(
            heights,
            [32.0_f32, 40.0, 56.0, 96.0, 136.0].map(f32::to_bits)
        );
    }

    #[test]
    fn a_button_elevation_is_an_elevation_level() {
        // androidx: `ElevatedButtonTokens.ContainerElevation` = `ElevationTokens.Level1` (1.0.dp) and
        // `HoveredContainerElevation` = `Level2` (3.0.dp): hovering raises the button.
        assert_eq!(
            elevated_button_tokens::CONTAINER_ELEVATION.0.to_bits(),
            1.0_f32.to_bits()
        );
        assert_eq!(
            elevated_button_tokens::HOVERED_CONTAINER_ELEVATION
                .0
                .to_bits(),
            3.0_f32.to_bits()
        );
        assert_eq!(
            elevated_button_tokens::HOVERED_CONTAINER_ELEVATION,
            elevation_tokens::LEVEL2
        );
    }

    /// The font, weight, size, line height and tracking of `style`, as numbers.
    fn values(style: TypeStyle) -> (GenericFontFamily, u16, [u32; 3]) {
        (
            style.font,
            style.weight.0,
            [style.size.0, style.line_height.0, style.tracking.0].map(f32::to_bits),
        )
    }

    fn expected(
        weight: u16,
        size: f32,
        line_height: f32,
        tracking: f32,
    ) -> (GenericFontFamily, u16, [u32; 3]) {
        (
            GenericFontFamily::SansSerif,
            weight,
            [size, line_height, tracking].map(f32::to_bits),
        )
    }

    #[test]
    fn each_type_style_of_a_button_label_holds_its_own_tokens_in_place() {
        // androidx `TypeScaleTokens` (31.0.11), the styles `Button.kt`'s `textStyleFor` gives each size. Size, line
        // height and tracking all differ in each, so a swapped argument shows.
        assert_eq!(
            values(typography_tokens::LABEL_LARGE),
            expected(500, 14.0, 20.0, 0.1)
        );
        assert_eq!(
            values(typography_tokens::TITLE_MEDIUM),
            expected(500, 16.0, 24.0, 0.2)
        );
        assert_eq!(
            values(typography_tokens::HEADLINE_SMALL),
            expected(400, 24.0, 32.0, 0.0)
        );
        assert_eq!(
            values(typography_tokens::HEADLINE_LARGE),
            expected(400, 32.0, 40.0, 0.0)
        );
        // And each names the tokens of its own name.
        let label = typography_tokens::LABEL_LARGE;
        assert_eq!(label.weight, type_scale_tokens::LABEL_LARGE_WEIGHT);
        assert_eq!(label.size, type_scale_tokens::LABEL_LARGE_SIZE);
        assert_eq!(
            label.line_height,
            type_scale_tokens::LABEL_LARGE_LINE_HEIGHT
        );
        assert_eq!(label.tracking, type_scale_tokens::LABEL_LARGE_TRACKING);
    }

    #[test]
    fn the_default_type_style_is_body_large_and_not_another_style() {
        let default = TypeStyle::default();
        assert_eq!(default, BODY_LARGE);
        // androidx `TypeScaleTokens.BodyLarge*`: regular, 16 on 24, tracking 0.5.
        assert_eq!(values(default), expected(400, 16.0, 24.0, 0.5));
        assert_ne!(default, typography_tokens::LABEL_LARGE);
        assert_ne!(default, typography_tokens::BODY_LARGE_EMPHASIZED);
    }
}
