//! Color: Material 3's color roles, and the schemes that give each role a color.
//!
//! A component never names a palette tone: it names a [`ColorRole`] (`Primary`, `OnPrimary`, `SurfaceContainer`,
//! ...), and the scheme in use gives the role its color. M3 pairs each role meant for text and icons with the roles it
//! is drawn on, and guarantees their contrast through the scheme's tones, never through the hue; this module checks
//! that guarantee with [`contrast_ratio`].
//!
//! The baseline schemes come from androidx `ColorLightTokens` and `ColorDarkTokens` (ADR 0003). Schemes made from a
//! seed color (dynamic color) are an algorithm, not tokens, and come with their own ADR.

use nsh_core::element::{AdaptiveColor, PropEnum, Tokens};

use crate::tokens::Srgb;
use crate::tokens::androidx::{color_dark_tokens, color_light_tokens};

/// Defines [`ColorRole`] and [`ColorScheme`] from one list, so a role, its field and its tokens cannot drift apart:
/// each line is the androidx name, the field, the generated constant and the M3 name, and a test checks that the four
/// spell the same name and that the list is androidx `ColorSchemeKeyTokens`, in its order.
macro_rules! roles {
    ($( $role:ident, $field:ident, $token:ident, $name:literal; )*) => {
        /// A color role of the Material 3 scheme: androidx `ColorSchemeKeyTokens`.
        ///
        /// A role is a prop value ([`PropEnum`]), so a component whose color androidx takes as a parameter takes a
        /// `Prop<ColorRole>`: the color follows the appearance and the app's scheme ([ADR 0008]).
        ///
        /// [ADR 0008]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0008-color-scheme-in-nsh-tokens.md
        ///
        /// ```
        /// use nsh_m3e::color::{ColorRole, ColorScheme};
        /// use nsh_m3e::tokens::androidx::palette_tokens::PRIMARY40;
        ///
        /// assert_eq!(ColorScheme::BASELINE_LIGHT.get(ColorRole::Primary), PRIMARY40);
        /// ```
        #[non_exhaustive]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PropEnum)]
        pub enum ColorRole {
            $(
                #[doc = concat!("androidx: `ColorSchemeKeyTokens.", stringify!($role), "`.")]
                $role,
            )*
        }

        impl ColorRole {
            /// Every role, in androidx `ColorSchemeKeyTokens` order.
            pub const ALL: &[ColorRole] = &[$(ColorRole::$role,)*];

            /// The name of the role's NSH design token: M3's own system token name, `md-sys-color-` and the role
            /// (ADR 0008).
            ///
            /// ```
            /// use nsh_m3e::color::ColorRole;
            ///
            /// assert_eq!(ColorRole::OnPrimaryContainer.token_name(), "md-sys-color-on-primary-container");
            /// ```
            #[must_use]
            pub const fn token_name(self) -> &'static str {
                match self {
                    $(ColorRole::$role => concat!("md-sys-color-", $name),)*
                }
            }
        }

        /// A color for each [`ColorRole`], for one appearance (light or dark).
        ///
        /// ```
        /// use nsh_m3e::color::ColorScheme;
        /// use nsh_m3e::tokens::androidx::palette_tokens::{PRIMARY40, PRIMARY80};
        ///
        /// assert_eq!(ColorScheme::BASELINE_LIGHT.primary, PRIMARY40);
        /// assert_eq!(ColorScheme::BASELINE_DARK.primary, PRIMARY80);
        /// ```
        #[non_exhaustive]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct ColorScheme {
            $(
                #[doc = concat!("The color of [`ColorRole::", stringify!($role), "`].")]
                pub $field: Srgb,
            )*
        }

        impl ColorScheme {
            /// The baseline light scheme: androidx `ColorLightTokens`.
            pub const BASELINE_LIGHT: Self = Self { $($field: color_light_tokens::$token,)* };

            /// The baseline dark scheme: androidx `ColorDarkTokens`.
            pub const BASELINE_DARK: Self = Self { $($field: color_dark_tokens::$token,)* };

            /// The color of `role` in this scheme.
            #[must_use]
            pub const fn get(&self, role: ColorRole) -> Srgb {
                match role {
                    $(ColorRole::$role => self.$field,)*
                }
            }
        }

        /// Each role with the names of its field, of its generated constant and its M3 name, as written in the list.
        #[cfg(test)]
        const NAMES: &[(ColorRole, &str, &str, &str)] = &[
            $((ColorRole::$role, stringify!($field), stringify!($token), $name),)*
        ];
    };
}

roles! {
    Background, background, BACKGROUND, "background";
    Error, error, ERROR, "error";
    ErrorContainer, error_container, ERROR_CONTAINER, "error-container";
    InverseOnSurface, inverse_on_surface, INVERSE_ON_SURFACE, "inverse-on-surface";
    InversePrimary, inverse_primary, INVERSE_PRIMARY, "inverse-primary";
    InverseSurface, inverse_surface, INVERSE_SURFACE, "inverse-surface";
    OnBackground, on_background, ON_BACKGROUND, "on-background";
    OnError, on_error, ON_ERROR, "on-error";
    OnErrorContainer, on_error_container, ON_ERROR_CONTAINER, "on-error-container";
    OnPrimary, on_primary, ON_PRIMARY, "on-primary";
    OnPrimaryContainer, on_primary_container, ON_PRIMARY_CONTAINER, "on-primary-container";
    OnPrimaryFixed, on_primary_fixed, ON_PRIMARY_FIXED, "on-primary-fixed";
    OnPrimaryFixedVariant, on_primary_fixed_variant, ON_PRIMARY_FIXED_VARIANT, "on-primary-fixed-variant";
    OnSecondary, on_secondary, ON_SECONDARY, "on-secondary";
    OnSecondaryContainer, on_secondary_container, ON_SECONDARY_CONTAINER, "on-secondary-container";
    OnSecondaryFixed, on_secondary_fixed, ON_SECONDARY_FIXED, "on-secondary-fixed";
    OnSecondaryFixedVariant, on_secondary_fixed_variant, ON_SECONDARY_FIXED_VARIANT, "on-secondary-fixed-variant";
    OnSurface, on_surface, ON_SURFACE, "on-surface";
    OnSurfaceVariant, on_surface_variant, ON_SURFACE_VARIANT, "on-surface-variant";
    OnTertiary, on_tertiary, ON_TERTIARY, "on-tertiary";
    OnTertiaryContainer, on_tertiary_container, ON_TERTIARY_CONTAINER, "on-tertiary-container";
    OnTertiaryFixed, on_tertiary_fixed, ON_TERTIARY_FIXED, "on-tertiary-fixed";
    OnTertiaryFixedVariant, on_tertiary_fixed_variant, ON_TERTIARY_FIXED_VARIANT, "on-tertiary-fixed-variant";
    Outline, outline, OUTLINE, "outline";
    OutlineVariant, outline_variant, OUTLINE_VARIANT, "outline-variant";
    Primary, primary, PRIMARY, "primary";
    PrimaryContainer, primary_container, PRIMARY_CONTAINER, "primary-container";
    PrimaryFixed, primary_fixed, PRIMARY_FIXED, "primary-fixed";
    PrimaryFixedDim, primary_fixed_dim, PRIMARY_FIXED_DIM, "primary-fixed-dim";
    Scrim, scrim, SCRIM, "scrim";
    Secondary, secondary, SECONDARY, "secondary";
    SecondaryContainer, secondary_container, SECONDARY_CONTAINER, "secondary-container";
    SecondaryFixed, secondary_fixed, SECONDARY_FIXED, "secondary-fixed";
    SecondaryFixedDim, secondary_fixed_dim, SECONDARY_FIXED_DIM, "secondary-fixed-dim";
    Surface, surface, SURFACE, "surface";
    SurfaceBright, surface_bright, SURFACE_BRIGHT, "surface-bright";
    SurfaceContainer, surface_container, SURFACE_CONTAINER, "surface-container";
    SurfaceContainerHigh, surface_container_high, SURFACE_CONTAINER_HIGH, "surface-container-high";
    SurfaceContainerHighest, surface_container_highest, SURFACE_CONTAINER_HIGHEST, "surface-container-highest";
    SurfaceContainerLow, surface_container_low, SURFACE_CONTAINER_LOW, "surface-container-low";
    SurfaceContainerLowest, surface_container_lowest, SURFACE_CONTAINER_LOWEST, "surface-container-lowest";
    SurfaceDim, surface_dim, SURFACE_DIM, "surface-dim";
    SurfaceTint, surface_tint, SURFACE_TINT, "surface-tint";
    SurfaceVariant, surface_variant, SURFACE_VARIANT, "surface-variant";
    Tertiary, tertiary, TERTIARY, "tertiary";
    TertiaryContainer, tertiary_container, TERTIARY_CONTAINER, "tertiary-container";
    TertiaryFixed, tertiary_fixed, TERTIARY_FIXED, "tertiary-fixed";
    TertiaryFixedDim, tertiary_fixed_dim, TERTIARY_FIXED_DIM, "tertiary-fixed-dim";
}

impl Default for ColorScheme {
    /// The baseline light scheme, [`ColorScheme::BASELINE_LIGHT`].
    fn default() -> Self {
        Self::BASELINE_LIGHT
    }
}

/// Sets the color of every role in NSH design tokens `tokens`, with `light`'s color in light appearance and `dark`'s
/// in dark, under the role's [`ColorRole::token_name`] (ADR 0008). Other tokens are left as they are, so NSH's own
/// (`primary`, `background`, ...) keep their meaning for `nsh-base` and the platform's controls.
///
/// The app hands the result to NSH's `set_tokens`; the platform then switches appearance with nothing running, and
/// Tailwind classes name the roles (`bg-md-sys-color-primary`).
///
/// ```
/// use nsh_core::element::{AdaptiveColor, Tokens};
/// use nsh_m3e::color::{ColorScheme, set_color_tokens};
///
/// let mut tokens = Tokens::default();
/// set_color_tokens(&mut tokens, &ColorScheme::BASELINE_LIGHT, &ColorScheme::BASELINE_DARK);
/// assert_eq!(
///     tokens.color("md-sys-color-primary"),
///     Some(AdaptiveColor::rgb(0x6750a4, 0xd0bcff)),
/// );
/// ```
pub fn set_color_tokens(tokens: &mut Tokens, light: &ColorScheme, dark: &ColorScheme) {
    for role in ColorRole::ALL {
        tokens.set_color(
            role.token_name(),
            AdaptiveColor::rgb(light.get(*role).to_hex(), dark.get(*role).to_hex()),
        );
    }
}

/// The contrast ratio of two colors, from 1 (the same luminance) to 21 (black and white), as WCAG 2.2 defines it:
/// `(L1 + 0.05) / (L2 + 0.05)`, where `L1` is the relative luminance of the lighter color and `L2` of the darker one.
/// The order of the arguments does not matter.
///
/// Text needs at least 4.5 ([WCAG 2.2 SC 1.4.3](https://www.w3.org/TR/WCAG22/#contrast-minimum)), and 7 in high
/// contrast ([SC 1.4.6](https://www.w3.org/TR/WCAG22/#contrast-enhanced)).
///
/// ```
/// use nsh_m3e::color::{ColorScheme, contrast_ratio};
///
/// let scheme = ColorScheme::BASELINE_LIGHT;
/// assert!(contrast_ratio(scheme.on_primary, scheme.primary) >= 4.5);
/// ```
#[must_use]
pub fn contrast_ratio(a: Srgb, b: Srgb) -> f32 {
    let (a, b) = (relative_luminance(a), relative_luminance(b));
    let (lighter, darker) = if a >= b { (a, b) } else { (b, a) };
    // The 0.05 is part of WCAG's formula (the flare of the viewing conditions), not a design value.
    (lighter + 0.05) / (darker + 0.05)
}

/// The relative luminance of a color, from 0 (black) to 1 (white), as WCAG 2.2 defines it
/// (<https://www.w3.org/TR/WCAG22/#dfn-relative-luminance>). Every number in it is the definition's, not a design
/// value.
#[must_use]
pub fn relative_luminance(color: Srgb) -> f32 {
    let linear = |channel: u8| {
        let value = f32::from(channel) / 255.0;
        if value <= 0.040_45 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.red) + 0.7152 * linear(color.green) + 0.0722 * linear(color.blue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::androidx::palette_tokens;
    use proptest::prelude::*;

    /// The minimum contrast for text: WCAG 2.2 SC 1.4.3, required by `docs/app-requirements.md`.
    const TEXT_CONTRAST: f32 = 4.5;

    /// Each role meant for text and icons, with the roles M3 draws it on. From M3's color roles
    /// (<https://m3.material.io/styles/color/roles>): an `On` role goes on the role it names; `OnSurface` and
    /// `OnSurfaceVariant` on every surface; the fixed `On` roles on their fixed and fixed dim roles; `InverseOnSurface`
    /// and `InversePrimary` on `InverseSurface`.
    const PAIRS: &[(ColorRole, &[ColorRole])] = {
        use ColorRole::*;
        const SURFACES: &[ColorRole] = &[
            Surface,
            SurfaceDim,
            SurfaceBright,
            SurfaceContainerLowest,
            SurfaceContainerLow,
            SurfaceContainer,
            SurfaceContainerHigh,
            SurfaceContainerHighest,
            SurfaceVariant,
        ];
        &[
            (OnPrimary, &[Primary]),
            (OnPrimaryContainer, &[PrimaryContainer]),
            (OnSecondary, &[Secondary]),
            (OnSecondaryContainer, &[SecondaryContainer]),
            (OnTertiary, &[Tertiary]),
            (OnTertiaryContainer, &[TertiaryContainer]),
            (OnError, &[Error]),
            (OnErrorContainer, &[ErrorContainer]),
            (OnBackground, &[Background]),
            (OnSurface, SURFACES),
            (OnSurfaceVariant, SURFACES),
            (OnPrimaryFixed, &[PrimaryFixed, PrimaryFixedDim]),
            (OnPrimaryFixedVariant, &[PrimaryFixed, PrimaryFixedDim]),
            (OnSecondaryFixed, &[SecondaryFixed, SecondaryFixedDim]),
            (
                OnSecondaryFixedVariant,
                &[SecondaryFixed, SecondaryFixedDim],
            ),
            (OnTertiaryFixed, &[TertiaryFixed, TertiaryFixedDim]),
            (OnTertiaryFixedVariant, &[TertiaryFixed, TertiaryFixedDim]),
            (InverseOnSurface, &[InverseSurface]),
            (InversePrimary, &[InverseSurface]),
        ]
    };

    fn schemes() -> [(&'static str, ColorScheme); 2] {
        [
            ("light", ColorScheme::BASELINE_LIGHT),
            ("dark", ColorScheme::BASELINE_DARK),
        ]
    }

    /// `ErrorContainer` → `error_container` (`upper`: `ERROR_CONTAINER`), the generator's rule.
    fn convert(name: &str, upper: bool) -> String {
        let mut out = String::new();
        let mut previous: Option<char> = None;
        for c in name.chars() {
            if c.is_ascii_uppercase()
                && previous.is_some_and(|p| p.is_ascii_lowercase() || p.is_ascii_digit())
            {
                out.push('_');
            }
            out.push(if upper {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            });
            previous = Some(c);
        }
        out
    }

    #[test]
    fn the_roles_are_androidx_s_color_scheme_keys_in_order() {
        let source =
            include_str!("../../../reference/androidx-material3-tokens/ColorSchemeKeyTokens.kt");
        let keys: Vec<&str> = source
            .lines()
            .filter_map(|line| line.trim().strip_prefix("val "))
            .filter_map(|rest| rest.split_once(" = ColorToken("))
            .map(|(name, _)| name)
            .collect();
        let roles: Vec<String> = ColorRole::ALL
            .iter()
            .map(|role| format!("{role:?}"))
            .collect();
        assert_eq!(roles, keys);
        assert_eq!(NAMES.len(), ColorRole::ALL.len());
    }

    #[test]
    fn each_role_reads_its_own_field_token_and_name() {
        for (role, field, token, kebab) in NAMES {
            let name = format!("{role:?}");
            assert_eq!(*field, convert(&name, false), "{name}");
            assert_eq!(*token, convert(&name, true), "{name}");
            assert_eq!(*kebab, field.replace('_', "-"), "{name}");
            assert_eq!(role.token_name(), format!("md-sys-color-{kebab}"), "{name}");
        }
    }

    #[test]
    fn every_role_gets_its_light_and_dark_color_under_its_own_name() {
        let (light, dark) = (ColorScheme::BASELINE_LIGHT, ColorScheme::BASELINE_DARK);
        let mut tokens = Tokens::default();
        set_color_tokens(&mut tokens, &light, &dark);
        for role in ColorRole::ALL {
            let expected = AdaptiveColor::rgb(light.get(*role).to_hex(), dark.get(*role).to_hex());
            assert_eq!(tokens.color(role.token_name()), Some(expected), "{role:?}");
        }
        assert_eq!(tokens.colors().count(), ColorRole::ALL.len());
        // androidx: light primary is `Primary40` (103, 80, 164), dark is `Primary80` (208, 188, 255): not swapped.
        assert_eq!(
            tokens.color("md-sys-color-primary"),
            Some(AdaptiveColor::rgb(0x0067_50a4, 0x00d0_bcff))
        );
    }

    #[test]
    fn setting_the_roles_leaves_nsh_s_own_tokens_alone() {
        // NSH's shadcn tokens share names with M3 roles (`primary`, `secondary`, `background`) but not their meaning:
        // they must keep their value, and no unprefixed role name may appear (ADR 0008).
        let shadcn = AdaptiveColor::rgb(0x0012_3456, 0x0065_4321);
        let mut tokens = Tokens::default();
        for name in ["primary", "secondary", "background"] {
            tokens.set_color(name, shadcn);
        }
        tokens.set_number("radius", 10.0);
        set_color_tokens(
            &mut tokens,
            &ColorScheme::BASELINE_LIGHT,
            &ColorScheme::BASELINE_DARK,
        );
        for name in ["primary", "secondary", "background"] {
            assert_eq!(tokens.color(name), Some(shadcn), "{name}");
        }
        assert_eq!(tokens.number("radius"), Some(10.0));
        assert_eq!(tokens.colors().count(), 3 + ColorRole::ALL.len());
        assert!(tokens.color("on-primary").is_none());
    }

    #[test]
    fn the_baseline_schemes_give_androidx_s_tones() {
        // androidx: `ColorLightTokens.Primary` = `PaletteTokens.Primary40`, `ColorDarkTokens.Primary` =
        // `PaletteTokens.Primary80`, `ColorLightTokens.SurfaceContainer` = `PaletteTokens.Neutral94`,
        // `ColorDarkTokens.OnSurfaceVariant` = `PaletteTokens.NeutralVariant80`.
        let (light, dark) = (ColorScheme::BASELINE_LIGHT, ColorScheme::BASELINE_DARK);
        assert_eq!(light.get(ColorRole::Primary), palette_tokens::PRIMARY40);
        assert_eq!(dark.get(ColorRole::Primary), palette_tokens::PRIMARY80);
        assert_eq!(
            light.get(ColorRole::SurfaceContainer),
            palette_tokens::NEUTRAL94
        );
        assert_eq!(
            dark.get(ColorRole::OnSurfaceVariant),
            palette_tokens::NEUTRAL_VARIANT80
        );
        assert_ne!(light, dark);
    }

    #[test]
    fn get_and_the_fields_agree_for_every_role() {
        let light = ColorScheme::BASELINE_LIGHT;
        assert_eq!(light.get(ColorRole::Background), light.background);
        assert_eq!(
            light.get(ColorRole::TertiaryFixedDim),
            light.tertiary_fixed_dim
        );
    }

    #[test]
    fn every_text_role_meets_wcag_aa_on_every_role_it_is_drawn_on() {
        for (name, scheme) in schemes() {
            for (foreground, backgrounds) in PAIRS {
                for background in *backgrounds {
                    let ratio = contrast_ratio(scheme.get(*foreground), scheme.get(*background));
                    assert!(
                        ratio >= TEXT_CONTRAST,
                        "{name}: {foreground:?} on {background:?} is {ratio:.2}:1"
                    );
                }
            }
        }
    }

    #[test]
    fn the_check_fails_where_m3_promises_no_contrast() {
        // `OutlineVariant` is for decorative dividers, not text: on the surface it is far below 4.5:1, so a check that
        // passes everything would be caught here.
        for (name, scheme) in schemes() {
            let ratio = contrast_ratio(scheme.outline_variant, scheme.surface);
            assert!(ratio < 3.0, "{name}: {ratio:.2}:1");
        }
    }

    #[test]
    fn black_on_white_is_21_and_a_color_on_itself_is_1() {
        let (black, white) = (palette_tokens::BLACK, palette_tokens::WHITE);
        assert!((contrast_ratio(black, white) - 21.0).abs() < 1e-4);
        assert!((contrast_ratio(white, black) - 21.0).abs() < 1e-4);
        assert!((contrast_ratio(white, white) - 1.0).abs() < 1e-6);
        assert!(relative_luminance(black).abs() < 1e-6);
        assert!((relative_luminance(white) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn luminance_weights_green_most_and_blue_least() {
        // WCAG's coefficients: 0.2126 red, 0.7152 green, 0.0722 blue, for a full channel.
        let red = relative_luminance(Srgb::new(255, 0, 0));
        let green = relative_luminance(Srgb::new(0, 255, 0));
        let blue = relative_luminance(Srgb::new(0, 0, 255));
        assert!((red - 0.2126).abs() < 1e-5, "{red}");
        assert!((green - 0.7152).abs() < 1e-5, "{green}");
        assert!((blue - 0.0722).abs() < 1e-5, "{blue}");
    }

    #[test]
    fn dark_channels_use_the_linear_segment_and_light_ones_the_curve() {
        // 10/255 = 0.0392 is below the 0.04045 threshold: 0.0392 / 12.92. 11/255 = 0.0431 is above:
        // ((0.0431 + 0.055) / 1.055)^2.4. Both are about 0.0030 and 0.0033, and the segments meet without a jump.
        let gray = |v: u8| relative_luminance(Srgb::new(v, v, v));
        assert!(
            (gray(10) - (10.0 / 255.0) / 12.92).abs() < 1e-7,
            "{}",
            gray(10)
        );
        let curve = ((11.0_f32 / 255.0 + 0.055) / 1.055).powf(2.4);
        assert!((gray(11) - curve).abs() < 1e-7, "{}", gray(11));
        assert!(gray(10) < gray(11));
        // Mid gray: 128/255 → about 0.2158, the well-known value.
        assert!((gray(128) - 0.2158).abs() < 1e-3, "{}", gray(128));
    }

    #[test]
    fn the_default_scheme_is_the_light_one_and_not_the_dark() {
        assert_eq!(ColorScheme::default(), ColorScheme::BASELINE_LIGHT);
        assert_ne!(ColorScheme::default(), ColorScheme::BASELINE_DARK);
    }

    fn srgb() -> impl Strategy<Value = Srgb> {
        (any::<u8>(), any::<u8>(), any::<u8>()).prop_map(|(r, g, b)| Srgb::new(r, g, b))
    }

    proptest! {
        #[test]
        fn the_ratio_is_symmetric_and_between_1_and_21(a in srgb(), b in srgb()) {
            let ratio = contrast_ratio(a, b);
            prop_assert!((ratio - contrast_ratio(b, a)).abs() < 1e-5);
            prop_assert!((1.0 - 1e-6..=21.0 + 1e-4).contains(&ratio), "{ratio}");
        }

        #[test]
        fn a_brighter_gray_never_has_less_luminance(v in 0_u8..255) {
            let gray = |v: u8| relative_luminance(Srgb::new(v, v, v));
            prop_assert!(gray(v) < gray(v + 1));
        }

        #[test]
        fn moving_away_in_luminance_never_lowers_the_contrast(v in 0_u8..254) {
            // From a fixed black, a lighter gray always contrasts more.
            let black = palette_tokens::BLACK;
            let ratio = |v: u8| contrast_ratio(black, Srgb::new(v, v, v));
            prop_assert!(ratio(v) <= ratio(v + 1));
        }
    }
}
