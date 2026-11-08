//! Elevation: Material 3's six levels, each drawn as the two box shadows that Material Web gives it.
//!
//! androidx's tokens give a level as a length (`ElevationTokens.Level0` to `Level5`: 0, 1, 3, 6, 8 and 12 points) and
//! no shadow: the platform draws one from the length. NSH draws the shadows an app gives a view, as CSS's layers
//! (`BoxShadow`), the same on every host that draws them. This module turns a level into those layers, from the formula
//! Material Web draws its own levels with ([ADR 0018]).
//!
//! # What is implemented
//!
//! The `styles()` mixin of Material Web's `elevation/internal/_elevation.scss`, at the commit pinned in
//! `reference/README.md` and copied in `reference/material-web/_elevation.scss`. Each level is two shadows, from the
//! level's index, 0 to 5:
//!
//! - the **key** layer, for the key light (`.shadow::before`, lines 62 to 106): `0 y blur 0` at opacity 0.3, with `y`
//!   from lines 77 to 87 and `blur` from lines 92 to 102;
//! - the **ambient** layer, for the ambient light (`.shadow::after`, lines 108 to 175): `0 y blur spread` at opacity
//!   0.15, with `y` from lines 123 to 135, `blur` from lines 144 to 154 and `spread` from lines 168 to 171.
//!
//! The file writes each term as `clamp(0, level - start, max)`, which `ramp` is here, and adds the terms; `key` and
//! `ambient` write the same sums, in whole points. Both layers are `PaletteTokens.Black`, the black of the palette: no
//! androidx token file holds a shadow color. Material Web builds its own from the palette's `neutral0`
//! (`tokens/versions/v0_192/_md-sys-color.scss`, lines 60 and 119, at the pinned commit; not vendored), and androidx's
//! `Neutral0` is the same black. The file sets each layer's `opacity`, and NSH takes an alpha in the color. For a
//! shadow alone on its element the two are the same picture, so the alpha is the opacity in 255 steps, rounded to the
//! nearest, a half up: 0.3 is 77 (`0x4d`) and 0.15 is 38 (`0x26`).
//!
//! Every level has two layers, the key one first, never an inset one, and none has a horizontal offset: NSH's
//! transitions move a view's shadows from one list to another layer by layer, in order, and refuse to move an inset
//! shadow to an outer one (`lerp_box_shadows`, `crates/nsh-core/src/transition.rs` at the pinned NSH revision), so a
//! component can animate between two levels. Level 0 has two layers too, as the file's own table has (`level0:
//! box-shadow: 0px 0px 0px 0px`), and neither has any size: they draw nothing.
//!
//! # How a component uses it
//!
//! It reads its elevation token, which is a length, finds the level, and gives the layers to the view whose corner
//! radius is its container's. NSH's `Styled::shadow` takes a list of layers and sets them all at once, so no helper
//! stands between the two; it exists on every element and on the `hover(|s| …)`, `active(|s| …)` and similar
//! refinements, so a state's level is one more call.
//!
//! ```
//! use nsh_core::prelude::*;
//! use nsh_m3e::elevation::Elevation;
//! use nsh_m3e::tokens::androidx::elevation_tokens::LEVEL3;
//!
//! // A floating action button's container rests at `ElevationTokens.Level3`.
//! let level = Elevation::from_points(LEVEL3);
//! assert_eq!(level, Some(Elevation::Level3));
//!
//! let container = view().shadow(Elevation::Level3.shadows());
//! # drop(container);
//! ```
//!
//! # What is not here
//!
//! How the shadow moves when a component changes level (a spring, in the FAB's pull request, from androidx's
//! `animateElevation` and ADR 0004); which view carries it; what the visual checks read around it. A container drawn
//! on a canvas (the buttons that morph) casts the shadow of its view's radius, not of the drawn shape, so those
//! components answer that themselves ([ADR 0018], Options).
//!
//! # How drift is caught
//!
//! The SCSS is not compiled. The sync script pins the file, so it changes only in a pull request that moves the pin,
//! and the tests of this module read the vendored file: they fail when its tables of the twelve layers or its two
//! opacities are not what this module draws, or when a line of the formula that `key` and `ambient` transcribe is not
//! in the file as it is written here. They also pin every layer of every level to the numbers worked out from those
//! lines.
//!
//! [ADR 0018]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0018-draw-elevation-as-two-box-shadows-per-level.md

use std::cmp::Ordering;

use nsh_core::geometry::Points;
use nsh_core::{BoxShadow, Color};

use crate::tokens::androidx::elevation_tokens::{LEVEL0, LEVEL1, LEVEL2, LEVEL3, LEVEL4, LEVEL5};
use crate::tokens::androidx::palette_tokens::BLACK;

/// The opacity of the key layer in percent: Material Web's `opacity: 0.3` (`_elevation.scss`, line 105).
const KEY_OPACITY_PERCENT: u16 = 30;

/// The opacity of the ambient layer in percent: Material Web's `opacity: 0.15` (`_elevation.scss`, line 174).
const AMBIENT_OPACITY_PERCENT: u16 = 15;

/// A level of Material 3's elevation scale: how far a surface sits above the one under it.
///
/// Each variant is an androidx `ElevationTokens` level, and its index is the number Material Web's formula takes.
/// A component never builds one from a length it computed: [`Elevation::from_points`] takes a token's value.
///
/// ```
/// use nsh_m3e::elevation::Elevation;
/// use nsh_m3e::tokens::androidx::elevation_tokens::LEVEL4;
///
/// let level = Elevation::from_points(LEVEL4).unwrap();
/// assert_eq!(level, Elevation::Level4);
/// assert_eq!(level.index(), 4);
/// assert!(level > Elevation::Level3);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Elevation {
    /// androidx: `ElevationTokens.Level0`. Nothing rises: both layers have no size.
    Level0 = 0,
    /// androidx: `ElevationTokens.Level1`.
    Level1 = 1,
    /// androidx: `ElevationTokens.Level2`.
    Level2 = 2,
    /// androidx: `ElevationTokens.Level3`.
    Level3 = 3,
    /// androidx: `ElevationTokens.Level4`.
    Level4 = 4,
    /// androidx: `ElevationTokens.Level5`.
    Level5 = 5,
}

impl Elevation {
    /// Every level, from the lowest to the highest.
    ///
    /// ```
    /// use nsh_m3e::elevation::Elevation;
    ///
    /// assert_eq!(Elevation::ALL.len(), 6);
    /// assert_eq!(Elevation::ALL[5], Elevation::Level5);
    /// ```
    pub const ALL: [Elevation; 6] = [
        Elevation::Level0,
        Elevation::Level1,
        Elevation::Level2,
        Elevation::Level3,
        Elevation::Level4,
        Elevation::Level5,
    ];

    /// The level's index on the scale, from 0 to 5: the `level` that Material Web's formula takes, which is not the
    /// token's length (`Level2` is 3 points).
    ///
    /// ```
    /// use nsh_m3e::elevation::Elevation;
    ///
    /// assert_eq!(Elevation::Level2.index(), 2);
    /// ```
    #[must_use]
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The level's length, as androidx's `ElevationTokens` gives it.
    ///
    /// ```
    /// use nsh_m3e::elevation::Elevation;
    ///
    /// assert_eq!(Elevation::Level2.points().0, 3.0);
    /// ```
    #[must_use]
    pub const fn points(self) -> Points {
        match self {
            Elevation::Level0 => LEVEL0,
            Elevation::Level1 => LEVEL1,
            Elevation::Level2 => LEVEL2,
            Elevation::Level3 => LEVEL3,
            Elevation::Level4 => LEVEL4,
            Elevation::Level5 => LEVEL5,
        }
    }

    /// The level whose token is exactly `points`, or `None` for any other length.
    ///
    /// The match is exact, with no tolerance and no rounding to the nearest level: a component reads its elevation from
    /// a token, so a length that is not a token's is a mistake of the caller, who gets `None` and not a neighbour. `-0.0`
    /// is `0.0`, and `NaN` is no level.
    ///
    /// ```
    /// use nsh_core::geometry::Points;
    /// use nsh_m3e::elevation::Elevation;
    ///
    /// assert_eq!(Elevation::from_points(Points(6.0)), Some(Elevation::Level3));
    /// // Between `Level3` (6) and `Level4` (8): no level.
    /// assert_eq!(Elevation::from_points(Points(7.0)), None);
    /// ```
    #[must_use]
    pub fn from_points(points: Points) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|level| level.points().0.partial_cmp(&points.0) == Some(Ordering::Equal))
    }

    /// The level's two shadow layers, the key one first: what a view's `shadow` takes.
    ///
    /// ```
    /// use nsh_m3e::elevation::Elevation;
    ///
    /// // Material Web's level 3: `0 1px 3px 0` at 0.3, and `0 4px 8px 3px` at 0.15.
    /// let [key, ambient] = Elevation::Level3.shadows();
    /// assert_eq!([key.offset_y.0, key.blur.0, key.spread.0], [1.0, 3.0, 0.0]);
    /// assert_eq!([ambient.offset_y.0, ambient.blur.0, ambient.spread.0], [4.0, 8.0, 3.0]);
    /// ```
    #[must_use]
    pub fn shadows(self) -> [BoxShadow; 2] {
        let level = self.index();
        [key(level), ambient(level)]
    }
}

/// `clamp(0, level - start, max)` of the SCSS: how far `level` is past `start`, and at most `max`.
///
/// It never exceeds `max` whatever `level` is, and never goes below 0, so no sum in [`key`] and [`ambient`] can
/// overflow: the largest is 1 + 1 + 2 · 3.
fn ramp(level: u8, start: u8, max: u8) -> u8 {
    level.saturating_sub(start).min(max)
}

/// The key layer of `level` (`.shadow::before`, `_elevation.scss` lines 62 to 106).
fn key(level: u8) -> BoxShadow {
    // `$y`, lines 77 to 87: levels 1, 4 and 5 each move the shadow down, the last by two.
    let y = ramp(level, 0, 1) + ramp(level, 3, 1) + 2 * ramp(level, 4, 1);
    // `$blur`, lines 92 to 102: level 1 blurs by two, levels 3 and 5 by one more.
    let blur = 2 * ramp(level, 0, 1) + ramp(level, 2, 1) + ramp(level, 4, 1);
    // Line 104: `box-shadow: 0px $y $blur 0px`, and line 105: `opacity: 0.3`.
    layer(y, blur, 0, KEY_OPACITY_PERCENT)
}

/// The ambient layer of `level` (`.shadow::after`, `_elevation.scss` lines 108 to 175).
fn ambient(level: u8) -> BoxShadow {
    // `$y`, lines 123 to 135: levels 1 and 2 move the shadow down by one, levels 3 to 5 by two.
    let y = ramp(level, 0, 1) + ramp(level, 1, 1) + 2 * ramp(level, 2, 3);
    // `$blur`, lines 144 to 154: levels 1 and 2 blur by three, levels 3 to 5 by two more.
    let blur = 3 * ramp(level, 0, 2) + 2 * ramp(level, 2, 3);
    // `$spread`, lines 168 to 171: levels 1 to 4 grow it by one, level 5 by two more.
    let spread = ramp(level, 0, 4) + 2 * ramp(level, 4, 1);
    // Line 173: `box-shadow: 0px $y $blur $spread`, and line 174: `opacity: 0.15`.
    layer(y, blur, spread, AMBIENT_OPACITY_PERCENT)
}

/// An outer shadow, with no horizontal offset, whole points of offset, blur and spread, and black at `opacity_percent`.
fn layer(offset_y: u8, blur: u8, spread: u8, opacity_percent: u16) -> BoxShadow {
    BoxShadow {
        offset_x: Points::ZERO,
        offset_y: Points(f32::from(offset_y)),
        blur: Points(f32::from(blur)),
        spread: Points(f32::from(spread)),
        color: shadow_color(opacity_percent),
        inset: false,
    }
}

/// The palette's black at `opacity_percent` (at most 100): its alpha is 255 times the opacity, rounded to the nearest
/// whole number, a half up.
fn shadow_color(opacity_percent: u16) -> Color {
    // `255 * 100 + 50` fits in a `u16`, and the quotient is at most 255.
    let alpha = (255 * opacity_percent.min(100) + 50) / 100;
    Color::rgba(
        BLACK.red,
        BLACK.green,
        BLACK.blue,
        u8::try_from(alpha).unwrap_or(u8::MAX),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::SystemTime;

    use nsh_core::prelude::*;
    use nsh_core::{Effects, MountItem};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;
    use proptest::prelude::*;

    use super::*;
    use crate::animation_probe::Recording;
    use crate::tokens::androidx::palette_tokens::NEUTRAL0;

    /// Material Web's `elevation/internal/_elevation.scss` at the pinned commit, as vendored.
    const SCSS: &str = include_str!("../../../reference/material-web/_elevation.scss");

    /// A level's two layers as `(offset y, blur, spread)` of the key one, and of the ambient one.
    type Layers = ((u8, u8, u8), (u8, u8, u8));

    /// A layer's offset, blur and spread, which are whole numbers of points.
    fn numbers(layer: &BoxShadow) -> (u8, u8, u8) {
        let whole = |points: Points| {
            (0..=u8::MAX)
                .find(|value| f32::from(*value).to_bits() == points.0.to_bits())
                .unwrap_or_else(|| panic!("{points:?} is not a whole number of points"))
        };
        (
            whole(layer.offset_y),
            whole(layer.blur),
            whole(layer.spread),
        )
    }

    fn layers(level: Elevation) -> Layers {
        let [key, ambient] = level.shadows();
        (numbers(&key), numbers(&ambient))
    }

    // The arithmetic below is `_elevation.scss`'s, written `clamp(0, level - start, max)` as the file does, with
    // `level` filled in. Lines 77 to 87 and 92 to 102 give the key layer's `y` and `blur`, lines 123 to 135, 144 to
    // 154 and 168 to 171 the ambient layer's `y`, `blur` and `spread`. Each result is also in the file's tables of
    // comments (lines 64 to 69 and 110 to 115), which `the_layers_are_the_files_own_tables` reads.

    #[test]
    fn level_0_is_nothing() {
        // key: y = clamp(0, 0, 1) + clamp(0, 0 - 3, 1) + 2 * clamp(0, 0 - 4, 1) = 0 + 0 + 0 = 0
        //      blur = 2 * clamp(0, 0, 1) + clamp(0, 0 - 2, 1) + clamp(0, 0 - 4, 1) = 0 + 0 + 0 = 0
        // ambient: y = clamp(0, 0, 1) + clamp(0, 0 - 1, 1) + 2 * clamp(0, 0 - 2, 3) = 0
        //      blur = 3 * clamp(0, 0, 2) + 2 * clamp(0, 0 - 2, 3) = 0
        //      spread = clamp(0, 0, 4) + 2 * clamp(0, 0 - 4, 1) = 0
        assert_eq!(layers(Elevation::Level0), ((0, 0, 0), (0, 0, 0)));
    }

    #[test]
    fn level_1_is_a_small_shadow_and_a_tight_ambient_one() {
        // key: y = clamp(0, 1, 1) + clamp(0, 1 - 3, 1) + 2 * clamp(0, 1 - 4, 1) = 1 + 0 + 0 = 1
        //      blur = 2 * clamp(0, 1, 1) + clamp(0, 1 - 2, 1) + clamp(0, 1 - 4, 1) = 2 + 0 + 0 = 2
        // ambient: y = clamp(0, 1, 1) + clamp(0, 1 - 1, 1) + 2 * clamp(0, 1 - 2, 3) = 1 + 0 + 0 = 1
        //      blur = 3 * clamp(0, 1, 2) + 2 * clamp(0, 1 - 2, 3) = 3 + 0 = 3
        //      spread = clamp(0, 1, 4) + 2 * clamp(0, 1 - 4, 1) = 1 + 0 = 1
        assert_eq!(layers(Elevation::Level1), ((1, 2, 0), (1, 3, 1)));
    }

    #[test]
    fn level_2_has_the_key_layer_of_level_1_and_a_larger_ambient_one() {
        // key: y = clamp(0, 2, 1) + clamp(0, 2 - 3, 1) + 2 * clamp(0, 2 - 4, 1) = 1 + 0 + 0 = 1
        //      blur = 2 * clamp(0, 2, 1) + clamp(0, 2 - 2, 1) + clamp(0, 2 - 4, 1) = 2 + 0 + 0 = 2
        // ambient: y = clamp(0, 2, 1) + clamp(0, 2 - 1, 1) + 2 * clamp(0, 2 - 2, 3) = 1 + 1 + 0 = 2
        //      blur = 3 * clamp(0, 2, 2) + 2 * clamp(0, 2 - 2, 3) = 6 + 0 = 6
        //      spread = clamp(0, 2, 4) + 2 * clamp(0, 2 - 4, 1) = 2 + 0 = 2
        assert_eq!(layers(Elevation::Level2), ((1, 2, 0), (2, 6, 2)));
        // The file says so itself (lines 65 and 66: level 1 and level 2 share the key shadow `0px 1px 2px 0px`).
        assert_eq!(
            Elevation::Level1.shadows()[0],
            Elevation::Level2.shadows()[0]
        );
    }

    #[test]
    fn level_3_is_the_floating_action_button_at_rest() {
        // key: y = clamp(0, 3, 1) + clamp(0, 3 - 3, 1) + 2 * clamp(0, 3 - 4, 1) = 1 + 0 + 0 = 1
        //      blur = 2 * clamp(0, 3, 1) + clamp(0, 3 - 2, 1) + clamp(0, 3 - 4, 1) = 2 + 1 + 0 = 3
        // ambient: y = clamp(0, 3, 1) + clamp(0, 3 - 1, 1) + 2 * clamp(0, 3 - 2, 3) = 1 + 1 + 2 = 4
        //      blur = 3 * clamp(0, 3, 2) + 2 * clamp(0, 3 - 2, 3) = 6 + 2 = 8
        //      spread = clamp(0, 3, 4) + 2 * clamp(0, 3 - 4, 1) = 3 + 0 = 3
        // ADR 0018: `0 1px 3px 0` and `0 4px 8px 3px`.
        assert_eq!(layers(Elevation::Level3), ((1, 3, 0), (4, 8, 3)));
    }

    #[test]
    fn level_4_is_the_floating_action_button_under_the_mouse() {
        // key: y = clamp(0, 4, 1) + clamp(0, 4 - 3, 1) + 2 * clamp(0, 4 - 4, 1) = 1 + 1 + 0 = 2
        //      blur = 2 * clamp(0, 4, 1) + clamp(0, 4 - 2, 1) + clamp(0, 4 - 4, 1) = 2 + 1 + 0 = 3
        // ambient: y = clamp(0, 4, 1) + clamp(0, 4 - 1, 1) + 2 * clamp(0, 4 - 2, 3) = 1 + 1 + 4 = 6
        //      blur = 3 * clamp(0, 4, 2) + 2 * clamp(0, 4 - 2, 3) = 6 + 4 = 10
        //      spread = clamp(0, 4, 4) + 2 * clamp(0, 4 - 4, 1) = 4 + 0 = 4
        // ADR 0018: `0 2px 3px 0` and `0 6px 10px 4px`.
        assert_eq!(layers(Elevation::Level4), ((2, 3, 0), (6, 10, 4)));
    }

    #[test]
    fn level_5_is_the_highest_and_the_only_one_that_spreads_by_two_more() {
        // key: y = clamp(0, 5, 1) + clamp(0, 5 - 3, 1) + 2 * clamp(0, 5 - 4, 1) = 1 + 1 + 2 = 4
        //      blur = 2 * clamp(0, 5, 1) + clamp(0, 5 - 2, 1) + clamp(0, 5 - 4, 1) = 2 + 1 + 1 = 4
        // ambient: y = clamp(0, 5, 1) + clamp(0, 5 - 1, 1) + 2 * clamp(0, 5 - 2, 3) = 1 + 1 + 6 = 8
        //      blur = 3 * clamp(0, 5, 2) + 2 * clamp(0, 5 - 2, 3) = 6 + 6 = 12
        //      spread = clamp(0, 5, 4) + 2 * clamp(0, 5 - 4, 1) = 4 + 2 = 6
        assert_eq!(layers(Elevation::Level5), ((4, 4, 0), (8, 12, 6)));
    }

    #[test]
    fn the_ramp_is_a_clamp_and_never_goes_below_0_or_past_its_maximum() {
        // `clamp(0, level - start, max)`: before the start it is 0 (and `level - start` would be negative, which an
        // unsigned subtraction must not turn into a huge number), at the start 0, then one per level up to the maximum.
        assert_eq!(
            (0..=6).map(|level| ramp(level, 2, 3)).collect::<Vec<_>>(),
            [0, 0, 0, 1, 2, 3, 3]
        );
        assert_eq!(ramp(0, 4, 1), 0);
        assert_eq!(ramp(u8::MAX, 0, 4), 4);
        assert_eq!(ramp(u8::MAX, u8::MAX, 4), 0);
        // No level makes the layers overflow, even one the enum has no variant for.
        for level in 0..=u8::MAX {
            let _ = (key(level), ambient(level));
        }
    }

    #[test]
    fn the_layers_are_the_files_own_tables() {
        // The file lists each level's shadow in comments, `// level3: box-shadow: 0px 1px 3px 0px;`, six for the key
        // layer under `.shadow::before` and six for the ambient layer under `.shadow::after`, and ends each layer with
        // its `opacity`.
        let (mut key, mut ambient) = (Vec::new(), Vec::new());
        let (mut key_opacity, mut ambient_opacity) = (Vec::new(), Vec::new());
        let mut in_key = false;
        for line in SCSS.lines().map(str::trim) {
            if line == ".shadow::before {" {
                in_key = true;
            } else if line == ".shadow::after {" {
                in_key = false;
            } else if let Some(row) = line.strip_prefix("// level") {
                let Some((index, rest)) = row.split_once(": box-shadow: ") else {
                    continue;
                };
                let px: Vec<u8> = rest
                    .trim_end_matches(';')
                    .split(' ')
                    .map(|part| part.strip_suffix("px").unwrap().parse().unwrap())
                    .collect();
                // `0px y blur spread`: the horizontal offset is 0.
                let [x, y, blur, spread] = px[..] else {
                    panic!("{line}");
                };
                assert_eq!(x, 0, "{line}");
                let table = if in_key { &mut key } else { &mut ambient };
                table.push((index.parse::<usize>().unwrap(), (y, blur, spread)));
            } else if let Some(opacity) = line
                .strip_prefix("opacity: ")
                .and_then(|rest| rest.strip_suffix(';'))
            {
                let opacities = if in_key {
                    &mut key_opacity
                } else {
                    &mut ambient_opacity
                };
                opacities.push(opacity.to_owned());
            }
        }
        let indexes: Vec<usize> = (0..6).collect();
        assert_eq!(key.iter().map(|row| row.0).collect::<Vec<_>>(), indexes);
        assert_eq!(ambient.iter().map(|row| row.0).collect::<Vec<_>>(), indexes);
        for level in Elevation::ALL {
            let (key_layer, ambient_layer) = layers(level);
            let index = usize::from(level.index());
            assert_eq!(key[index].1, key_layer, "key, level {index}");
            assert_eq!(ambient[index].1, ambient_layer, "ambient, level {index}");
        }
        // The opacities, as the percent this module takes.
        assert_eq!(key_opacity, ["0.3"]);
        assert_eq!(ambient_opacity, ["0.15"]);
        assert_eq!(KEY_OPACITY_PERCENT, 30);
        assert_eq!(AMBIENT_OPACITY_PERCENT, 15);
    }

    #[test]
    fn the_file_still_holds_each_line_of_the_formula_that_key_and_ambient_transcribe() {
        // The tables above are comments. This is the arithmetic they describe: every line that `key` and `ambient` write
        // in Rust, in the layer it belongs to. A change to any of them in a newer file fails here, and the Rust is
        // checked against the new file by hand.
        let (key, ambient) = SCSS
            .split_once("// Key box shadow")
            .and_then(|(_, rest)| rest.split_once("// Ambient box shadow"))
            .unwrap();
        let key_formula = [
            // `y`: `ramp(level, 0, 1) + ramp(level, 3, 1) + 2 * ramp(level, 4, 1)`.
            "$level1-y: clamp(0, var(--_level), 1);",
            "$level4-y: clamp(0, var(--_level) - 3, 1);",
            "$level5-y: calc(2 * clamp(0, var(--_level) - 4, 1));",
            "$y: calc(1px * ($level1-y + $level4-y + $level5-y));",
            // `blur`: `2 * ramp(level, 0, 1) + ramp(level, 2, 1) + ramp(level, 4, 1)`.
            "$level1-blur: calc(2 * clamp(0, var(--_level), 1));",
            "$level3-blur: clamp(0, var(--_level) - 2, 1);",
            "$level5-blur: clamp(0, var(--_level) - 4, 1);",
            "$blur: calc(1px * ($level1-blur + $level3-blur + $level5-blur));",
            // No horizontal offset, no spread.
            "box-shadow: 0px $y $blur 0px var(--_shadow-color);",
        ];
        let ambient_formula = [
            // `y`: `ramp(level, 0, 1) + ramp(level, 1, 1) + 2 * ramp(level, 2, 3)`.
            "$level1-y: clamp(0, var(--_level), 1);",
            "$level2-y: clamp(0, var(--_level) - 1, 1);",
            "$level3to5-y: calc(2 * clamp(0, var(--_level) - 2, 3));",
            "$y: calc(1px * ($level1-y + $level2-y + $level3to5-y));",
            // `blur`: `3 * ramp(level, 0, 2) + 2 * ramp(level, 2, 3)`.
            "$level1to2-blur: calc(3 * clamp(0, var(--_level), 2));",
            "$level3to5-blur: calc(2 * clamp(0, var(--_level) - 2, 3));",
            "$blur: calc(1px * ($level1to2-blur + $level3to5-blur));",
            // `spread`: `ramp(level, 0, 4) + 2 * ramp(level, 4, 1)`.
            "$level1to4-spread: clamp(0, var(--_level), 4);",
            "$level5-spread: calc(2 * clamp(0, var(--_level) - 4, 1));",
            "$spread: calc(1px * ($level1to4-spread + $level5-spread));",
            "box-shadow: 0px $y $blur $spread var(--_shadow-color);",
        ];
        for (name, section, formula) in [
            ("key", key, &key_formula[..]),
            ("ambient", ambient, &ambient_formula[..]),
        ] {
            for line in formula {
                let found = section.lines().filter(|l| l.trim() == *line).count();
                assert_eq!(found, 1, "{name}: {line}");
            }
        }
        // Nothing else in a layer builds its shadow: a new term would be a line this list does not have.
        let terms = |section: &str| {
            section
                .lines()
                .map(str::trim)
                .filter(|line| line.starts_with('$') || line.starts_with("box-shadow:"))
                .count()
        };
        assert_eq!(terms(key), key_formula.len());
        assert_eq!(terms(ambient), ambient_formula.len());
    }

    #[test]
    fn the_layers_are_black_at_the_files_opacities_and_nothing_else() {
        // androidx has no shadow color: `PaletteTokens.Black` is the palette's black, and it is `Neutral0`, which is
        // what Material Web's own shadow color is made from. The alphas: 0.3 * 255 = 76.5 rounds up to 77 = 0x4d, and
        // 0.15 * 255 = 38.25 rounds to 38 = 0x26.
        assert_eq!(
            (BLACK.red, BLACK.green, BLACK.blue),
            (NEUTRAL0.red, NEUTRAL0.green, NEUTRAL0.blue)
        );
        assert_eq!(BLACK.to_hex(), 0);
        for level in Elevation::ALL {
            let [key, ambient] = level.shadows();
            assert_eq!(key.color, Color(0x0000_004d), "{level:?}");
            assert_eq!(ambient.color, Color(0x0000_0026), "{level:?}");
        }
    }

    #[test]
    fn the_alpha_is_the_opacity_in_255_steps_rounded_to_the_nearest() {
        let alpha = |percent| shadow_color(percent).0 & 0xff;
        assert_eq!(alpha(0), 0);
        assert_eq!(alpha(100), 255);
        // 50 % is 127.5, which rounds up, and 1 % is 2.55.
        assert_eq!(alpha(50), 128);
        assert_eq!(alpha(1), 3);
        // 0.3 is 76.5 (a half, up) and 0.15 is 38.25 (down).
        assert_eq!(alpha(30), 0x4d);
        assert_eq!(alpha(15), 0x26);
        // More than 100 % is 100 %, not a wrapped alpha.
        assert_eq!(alpha(101), 255);
        assert_eq!(alpha(u16::MAX), 255);
        // The color is black at every opacity.
        assert_eq!(shadow_color(30).0 >> 8, 0);
    }

    #[test]
    fn level_0_draws_nothing_and_every_other_level_does() {
        // No size at all: not an offset, a blur or a spread. Sizeless, the shadow is under its own box.
        for layer in Elevation::Level0.shadows() {
            assert_eq!(
                [
                    layer.offset_x.0,
                    layer.offset_y.0,
                    layer.blur.0,
                    layer.spread.0
                ]
                .map(f32::to_bits),
                [0.0_f32.to_bits(); 4]
            );
        }
        for level in &Elevation::ALL[1..] {
            let [key, ambient] = level.shadows();
            assert!(reach_below(&key) > 0.0, "{level:?}");
            assert!(reach_below(&ambient) > 0.0, "{level:?}");
        }
    }

    #[test]
    fn no_level_has_an_inset_layer_or_a_horizontal_offset_and_each_has_two_layers() {
        // An inset layer cannot move to an outer one in NSH's transitions, and the key light is above: a horizontal
        // offset would be a different light.
        for level in Elevation::ALL {
            let layers = level.shadows();
            assert_eq!(layers.len(), 2, "{level:?}");
            for layer in layers {
                assert!(!layer.inset, "{level:?}");
                assert_eq!(layer.offset_x, Points::ZERO, "{level:?}");
                assert!(layer.offset_y.0 >= 0.0, "{level:?}");
                assert!(layer.blur.0 >= 0.0, "{level:?}");
                assert!(layer.spread.0 >= 0.0, "{level:?}");
            }
        }
    }

    /// How far a layer reaches below its box: the offset down, the blur and the spread.
    fn reach_below(layer: &BoxShadow) -> f32 {
        layer.offset_y.0 + layer.blur.0 + layer.spread.0
    }

    /// How far a layer reaches to the sides of its box: the blur and the spread.
    fn reach_aside(layer: &BoxShadow) -> f32 {
        layer.blur.0 + layer.spread.0
    }

    #[test]
    fn a_higher_level_never_shrinks_a_layer_and_always_reaches_further() {
        for pair in Elevation::ALL.windows(2) {
            let [lower, higher] = pair else {
                panic!("windows of two: {pair:?}");
            };
            let (lower_layers, higher_layers) = (lower.shadows(), higher.shadows());
            for (name, (low, high)) in ["key", "ambient"]
                .into_iter()
                .zip(lower_layers.iter().zip(&higher_layers))
            {
                assert!(
                    high.offset_y.0 >= low.offset_y.0
                        && high.blur.0 >= low.blur.0
                        && high.spread.0 >= low.spread.0,
                    "{name}: {lower:?} to {higher:?}"
                );
                assert!(
                    reach_below(high) >= reach_below(low),
                    "{name}: {lower:?} to {higher:?}"
                );
            }
            // The ambient layer grows with every level, the key layer does not (levels 1 and 2 share it), and together
            // they always do.
            assert!(
                reach_below(&higher_layers[1]) > reach_below(&lower_layers[1]),
                "{lower:?} to {higher:?}"
            );
            let total = |layers: &[BoxShadow; 2]| layers.iter().map(reach_below).sum::<f32>();
            assert!(
                total(&higher_layers) > total(&lower_layers),
                "{lower:?} to {higher:?}"
            );
        }
    }

    #[test]
    fn the_ambient_layer_reaches_as_far_as_adr_0018_says() {
        // ADR 0018 (Consequences, the visual checks): at level 3, 11 points to the sides (spread 3, blur 8) and 15
        // below (plus its 4-point offset); at level 4, 14 and 20.
        let ambient = |level: Elevation| level.shadows()[1];
        assert_eq!(
            reach_aside(&ambient(Elevation::Level3)).to_bits(),
            11.0_f32.to_bits()
        );
        assert_eq!(
            reach_below(&ambient(Elevation::Level3)).to_bits(),
            15.0_f32.to_bits()
        );
        assert_eq!(
            reach_aside(&ambient(Elevation::Level4)).to_bits(),
            14.0_f32.to_bits()
        );
        assert_eq!(
            reach_below(&ambient(Elevation::Level4)).to_bits(),
            20.0_f32.to_bits()
        );
    }

    #[test]
    fn the_key_layer_comes_first_and_the_ambient_layer_second_at_every_level() {
        // NSH moves a shadow list to another one layer by layer, in order (`lerp_box_shadows`): a key layer must meet a
        // key layer. The key layer is the one at opacity 0.3, with no spread; the ambient layer is at 0.15.
        for level in Elevation::ALL {
            let [first, second] = level.shadows();
            assert_eq!(first.color, shadow_color(KEY_OPACITY_PERCENT), "{level:?}");
            assert_eq!(
                second.color,
                shadow_color(AMBIENT_OPACITY_PERCENT),
                "{level:?}"
            );
            assert_eq!(first.spread, Points::ZERO, "{level:?}");
        }
        // And they are not the same layer: from level 1 on, the ambient one is the larger.
        let [key, ambient] = Elevation::Level1.shadows();
        assert_ne!(key, ambient);
    }

    #[test]
    fn the_levels_are_in_order_and_their_index_is_their_place_on_the_scale() {
        let indexes: Vec<u8> = Elevation::ALL.iter().map(|level| level.index()).collect();
        assert_eq!(indexes, [0, 1, 2, 3, 4, 5]);
        assert!(Elevation::ALL.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(Elevation::Level0 as u8, 0);
        assert_eq!(Elevation::Level5 as u8, 5);
    }

    #[test]
    fn each_elevation_token_is_the_level_of_its_own_index() {
        // androidx `ElevationTokens`: `Level0` 0, `Level1` 1, `Level2` 3, `Level3` 6, `Level4` 8, `Level5` 12 points.
        // The index is the number in the token's name, not its length.
        let tokens = [
            (LEVEL0, Elevation::Level0, 0.0),
            (LEVEL1, Elevation::Level1, 1.0),
            (LEVEL2, Elevation::Level2, 3.0),
            (LEVEL3, Elevation::Level3, 6.0),
            (LEVEL4, Elevation::Level4, 8.0),
            (LEVEL5, Elevation::Level5, 12.0),
        ];
        for (index, (token, level, points)) in tokens.into_iter().enumerate() {
            assert_eq!(Elevation::from_points(token), Some(level), "Level{index}");
            assert_eq!(usize::from(level.index()), index);
            assert_eq!(level.points(), token, "{level:?}");
            assert_eq!(token.0.to_bits(), f32::to_bits(points), "Level{index}");
        }
    }

    #[test]
    fn a_length_between_two_tokens_is_no_level_and_is_not_rounded_to_one() {
        // 2.9 is nearer to `Level2` (3) than to any other token, and 2.0 is as near to `Level1` (1) as to `Level2`:
        // neither is a level.
        for points in [0.5, 2.0, 2.9, 3.1, 4.5, 7.0, 10.0, 11.99, 12.01, 13.0, -1.0] {
            assert_eq!(Elevation::from_points(Points(points)), None, "{points}");
        }
        // The next number either side of a token is not the token.
        for token in [LEVEL1, LEVEL2, LEVEL3, LEVEL4, LEVEL5] {
            let bits = token.0.to_bits();
            for neighbour in [bits - 1, bits + 1] {
                let points = Points(f32::from_bits(neighbour));
                assert_eq!(Elevation::from_points(points), None, "{points:?}");
            }
        }
    }

    #[test]
    fn what_is_not_a_length_is_no_level() {
        for points in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::MAX,
            f32::MIN,
        ] {
            assert_eq!(Elevation::from_points(Points(points)), None, "{points}");
        }
        // Zero is level 0 whichever way its sign is written.
        assert_eq!(Elevation::from_points(Points(0.0)), Some(Elevation::Level0));
        assert_eq!(
            Elevation::from_points(Points(-0.0)),
            Some(Elevation::Level0)
        );
    }

    proptest! {
        #[test]
        fn a_length_is_a_level_only_if_it_is_a_token(points in any::<f32>()) {
            let is_token = Elevation::ALL
                .iter()
                .any(|level| level.points().0.partial_cmp(&points) == Some(Ordering::Equal));
            match Elevation::from_points(Points(points)) {
                Some(level) => {
                    prop_assert!(is_token);
                    prop_assert_eq!(level.points().0.partial_cmp(&points), Some(Ordering::Equal));
                }
                None => prop_assert!(!is_token),
            }
        }

        #[test]
        fn a_level_is_found_from_its_own_length_and_from_no_other(
            first in 0_usize..6,
            second in 0_usize..6,
        ) {
            let (first, second) = (Elevation::ALL[first], Elevation::ALL[second]);
            prop_assert_eq!(Elevation::from_points(first.points()), Some(first));
            prop_assert_eq!(first == second, first.points() == second.points());
        }
    }

    /// What the probe mounts: one view with the shadows of the level whose index it is given.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        level: Prop<usize>,
    }

    #[component]
    fn Raised(props: ProbeProps) -> impl IntoElement {
        let level = Elevation::ALL
            .get(props.level.get_untracked())
            .copied()
            .unwrap_or(Elevation::Level0);
        view().shadow(level.shadows()).child(text("Raised"))
    }

    /// The effects NSH's engine sends for the view of [`Raised`] at `level`.
    fn mounted_effects(level: Elevation) -> Option<Arc<Effects>> {
        let root = Root::new::<Raised>(move || ProbeProps {
            level: Prop::new(usize::from(level.index())),
        });
        let mut app = Headless::start(EngineConfig {
            clock: Clock::manual(SystemTime::UNIX_EPOCH),
            ..EngineConfig::new(root)
        })
        .unwrap();
        let recording = Recording::attach(&mut app);
        app.wait_for("Raised").unwrap();
        let mut styles = HashMap::new();
        for item in recording.items() {
            if let MountItem::ViewStyle { id, style } = item {
                styles.insert(id, style.effects.clone());
            }
        }
        let with_effects: Vec<_> = styles.into_values().flatten().collect();
        assert!(with_effects.len() <= 1, "{level:?}: {with_effects:?}");
        with_effects.into_iter().next()
    }

    #[test]
    fn the_engine_receives_each_levels_layers_as_they_are_in_the_same_order() {
        // `Styled::shadow` takes the layers and sets them all at once: the view's style holds exactly them, key first,
        // and no other effect.
        for level in Elevation::ALL {
            let effects = mounted_effects(level).unwrap_or_else(|| panic!("{level:?}: no effects"));
            assert_eq!(
                *effects,
                Effects {
                    box_shadows: level.shadows().to_vec(),
                    ..Effects::default()
                },
                "{level:?}"
            );
        }
    }
}
