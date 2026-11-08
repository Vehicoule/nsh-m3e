//! The Material 3 divider: a thin line that groups content in lists and layouts, across or down.
//!
//! The divider follows androidx's `HorizontalDivider` and `VerticalDivider` in `Divider.kt`, at the pinned commit
//! (lines 49 and 77), as [ADR 0012] says.
//!
//! - **Layout.** A horizontal divider fills the width of the space it is in and is `thickness` tall; a vertical one
//!   fills the height and is `thickness` wide (`Canvas(modifier.fillMaxWidth().height(thickness))` and
//!   `Canvas(modifier.fillMaxHeight().width(thickness))`, lines 54 and 82). androidx draws one line of `thickness`
//!   through the middle of that box, which covers the whole box, so the divider here is one view filled with the
//!   color: nothing is drawn on a canvas, and nothing more than that view is laid out. The thickness is exact:
//!   `height(thickness)` gives fixed constraints, and a view of NSH is not squeezed to fit unless it says so.
//! - **Color and thickness.** They default to `DividerDefaults.color` and `DividerDefaults.Thickness` (lines 113 and
//!   117), which read `DividerTokens`: `OutlineVariant` and 1 point. Both are parameters of androidx's dividers, and
//!   are props here. The color is a [`ColorRole`] of the scheme rather than any color, as everywhere in this library:
//!   it follows the appearance and the app's scheme with nothing running ([ADR 0008]).
//! - **Insets.** androidx's dividers have none, so this one has none: the app's layout indents it (a margin around
//!   it, or padding around what holds it).
//! - **Nothing to hear, nothing to touch.** `Divider.kt` sets no semantics, takes no press, hover or focus and
//!   animates nothing. The divider is decorative: it is hidden from assistive technologies, as NSH's own base
//!   components hide what they draw only for the eye (`accessibility_elements_hidden`).
//! - **Elevation.** None: a divider has no shadow.
//!
//! [ADR 0008]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0008-color-scheme-in-nsh-tokens.md
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::element::Length;
use nsh_core::geometry::Points;
use nsh_core::prelude::*;

use crate::color::ColorRole;
use crate::tint::role_color;
use crate::tokens::androidx::divider_tokens;

/// The direction a [`Divider`] runs in.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum DividerOrientation {
    /// Across: as wide as the space it is in, `thickness` tall. androidx `HorizontalDivider`.
    #[default]
    Horizontal,
    /// Down: as tall as the space it is in, `thickness` wide. androidx `VerticalDivider`.
    Vertical,
}

/// One device pixel, in points: what androidx's `Dp.Hairline` gives a divider (`Divider.kt` line 44: "a single pixel
/// divider regardless of screen density"), which its deprecated `Divider` makes `1f / LocalDensity.current.density`
/// (line 103). Density is NSH's pixel ratio, so a divider that takes it as its thickness follows a window moved to a
/// screen of another density.
///
/// Read it where it can run again (a binding), as the app reads any ambient value:
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::divider::{divider, hairline};
///
/// fn rule() -> impl IntoElement {
///     divider().thickness(hairline)
/// }
/// ```
#[must_use]
pub fn hairline() -> Points {
    // One device pixel is 1 / pixel ratio points: `1.0` is that definition, not a design value.
    Points(1.0 / pixel_ratio())
}

/// The thickness a divider lays out for `thickness`: a length that is not a positive number of points (zero, negative,
/// not a number, infinite) is no thickness at all, and the divider takes no room and draws nothing. Without this the
/// engine would be handed a negative or infinite size.
fn extent(thickness: Points) -> Points {
    if thickness.0.is_finite() {
        Points(thickness.0.max(0.0))
    } else {
        Points(0.0)
    }
}

/// The props of a [`Divider`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct DividerProps {
    /// The direction it runs in.
    #[prop(default)]
    pub orientation: Prop<DividerOrientation>,
    /// How thick the line is, in points: across the divider for a horizontal one, along its width for a vertical
    /// one. [`hairline`] is one device pixel. A thickness that is not a positive number of points makes the divider
    /// take no room and draw nothing.
    #[prop(default = divider_tokens::THICKNESS)]
    pub thickness: Prop<Points>,
    /// The role of the scheme the line is in.
    #[prop(default = divider_tokens::COLOR)]
    pub color: Prop<ColorRole>,
}

/// A Material 3 divider: a line, by default 1 point thick and in the outline variant color, that fills the width (or
/// the height) of the space it is in.
///
/// It is decorative: it takes no press, no hover and no focus, animates nothing, and screen readers do not meet it.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::divider::{DividerOrientation, divider};
///
/// fn row() -> impl IntoElement {
///     view()
///         .child(text("Before"))
///         .child(divider())
///         .child(text("After"))
/// }
///
/// fn column() -> impl IntoElement {
///     view()
///         .flex_row()
///         .h(px(48.0))
///         .child(text("Left"))
///         .child(divider().orientation(DividerOrientation::Vertical))
///         .child(text("Right"))
/// }
/// ```
#[component]
pub fn Divider(props: DividerProps) -> impl IntoElement {
    let thickness = move || extent(props.thickness.get());
    // The axis it fills is the whole of the space it is in (`fillMaxWidth()`, `fillMaxHeight()`); the other is its
    // thickness. `1.0` is all of it, not a design value.
    let fills = || Length::from(relative(1.0));
    view()
        .accessibility_elements_hidden(true)
        .w(move || match props.orientation.get() {
            DividerOrientation::Horizontal => fills(),
            DividerOrientation::Vertical => Length::from(thickness()),
        })
        .h(move || match props.orientation.get() {
            DividerOrientation::Horizontal => Length::from(thickness()),
            DividerOrientation::Vertical => fills(),
        })
        .bg(move || role_color(props.color.get()))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::SystemTime;

    use nsh_core::geometry::Bounds;
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{
        AccessibilityRole, Borders, Color, EventKind, MountItem, NodeId, ViewKind, ViewProps,
        ViewStyle,
    };
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;
    use proptest::prelude::*;

    use super::*;
    use crate::animation_probe::Recording;
    use crate::color::ColorScheme;

    const ORIENTATION: u32 = 1;
    const THICKNESS: u32 = 2;
    const COLOR: u32 = 4;
    const ALL: u32 = ORIENTATION | THICKNESS | COLOR;

    /// What the probe places: a slot of a known size, a view that overflows it, and the divider, with these props. The
    /// slot is a row for a vertical divider and a column for a horizontal one, so that the overflowing view is along the
    /// divider's thickness. Three lines change the divider: they turn it, thicken it and recolor it.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        /// Which props the divider is given, a bit each ([`ORIENTATION`], [`THICKNESS`], [`COLOR`]): the others are left
        /// to their defaults.
        #[prop(default)]
        given: Prop<u32>,
        #[prop(default)]
        orientation: Prop<DividerOrientation>,
        #[prop(default)]
        thickness: Prop<Points>,
        #[prop(default = divider_tokens::COLOR)]
        color: Prop<ColorRole>,
        /// Whether the thickness is one device pixel instead of `thickness`.
        #[prop(default)]
        hairline: Prop<bool>,
        #[prop(default)]
        slot_width: Prop<f32>,
        #[prop(default)]
        slot_height: Prop<f32>,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let orientation = signal(props.orientation.get_untracked());
        let thickness = signal(props.thickness.get_untracked());
        let color = signal(props.color.get_untracked());
        let row = props.orientation.get_untracked() == DividerOrientation::Vertical;
        let given = props.given.get_untracked();
        let mut line = divider();
        if given & ORIENTATION != 0 {
            line = line.orientation(move || orientation.get());
        }
        if given & THICKNESS != 0 {
            line = line.thickness(move || thickness.get());
        }
        if given & COLOR != 0 {
            line = line.color(move || color.get());
        }
        if props.hairline.get_untracked() {
            line = line.thickness(hairline);
        }
        let slot = view()
            .test_id("slot")
            .w(px(props.slot_width.get_untracked()))
            .h(px(props.slot_height.get_untracked()))
            .child(line.0.test_id("divider"))
            // Larger than the slot along both axes: whatever squeezes views to fit would squeeze the divider.
            .child(view().test_id("overflow").w(px(400.0)).h(px(400.0)));
        view()
            .child(if row { slot.flex_row() } else { slot })
            .child(text("ready"))
            .child(text("Turn").on_press(move |(): &()| {
                orientation.update(|orientation| {
                    *orientation = match *orientation {
                        DividerOrientation::Horizontal => DividerOrientation::Vertical,
                        _ => DividerOrientation::Horizontal,
                    };
                });
            }))
            .child(text("Thicken").on_press(move |(): &()| thickness.set(Points(4.0))))
            .child(text("Recolor").on_press(move |(): &()| color.set(ColorRole::Primary)))
    }

    #[derive(Debug, Clone, Copy)]
    struct Config {
        given: u32,
        orientation: DividerOrientation,
        thickness: Points,
        color: ColorRole,
        hairline: bool,
        slot: (f32, f32),
        dark: bool,
        pixel_ratio: f32,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                given: 0,
                orientation: DividerOrientation::Horizontal,
                thickness: divider_tokens::THICKNESS,
                color: divider_tokens::COLOR,
                hairline: false,
                slot: (300.0, 80.0),
                dark: false,
                pixel_ratio: 1.0,
            }
        }
    }

    impl Config {
        /// A divider given these props.
        fn with(orientation: DividerOrientation, thickness: f32) -> Self {
            Self {
                given: ALL,
                orientation,
                thickness: Points(thickness),
                ..Self::default()
            }
        }
    }

    const ORIENTATIONS: [DividerOrientation; 2] =
        [DividerOrientation::Horizontal, DividerOrientation::Vertical];

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            given: Prop::new(config.given),
            orientation: Prop::new(config.orientation),
            thickness: Prop::new(config.thickness),
            color: Prop::new(config.color),
            hairline: Prop::new(config.hairline),
            slot_width: Prop::new(config.slot.0),
            slot_height: Prop::new(config.slot.1),
        });
        // The web has a mouse to hover with, whatever its idiom.
        let environment = Environment {
            os: Os::Web,
            color_scheme: if config.dark {
                Appearance::Dark
            } else {
                Appearance::Light
            },
            pixel_ratio: config.pixel_ratio,
            ..Environment::default()
        };
        let mut app = Headless::start(EngineConfig {
            clock: Clock::manual(SystemTime::UNIX_EPOCH),
            environment,
            ..EngineConfig::new(root)
        })
        .unwrap();
        let recording = Recording::attach(&mut app);
        app.wait_for("text:ready").unwrap();
        (app, recording)
    }

    /// The divider's view, the slot and the view that overflows it, with what each was last sent.
    struct Mounted {
        divider: NodeId,
        slot: NodeId,
        overflow: NodeId,
        kinds: HashMap<NodeId, ViewKind>,
        children: HashMap<NodeId, Vec<NodeId>>,
        props: HashMap<NodeId, ViewProps>,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds = HashMap::new();
            let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let mut props = HashMap::new();
            let mut styles = HashMap::new();
            let mut frames = HashMap::new();
            let mut by_test_id: HashMap<String, NodeId> = HashMap::new();
            for item in items {
                match item {
                    MountItem::Create { id, kind } => {
                        kinds.insert(*id, *kind);
                    }
                    MountItem::Insert {
                        parent,
                        child,
                        before,
                    } => {
                        let siblings: &mut Vec<NodeId> = children.entry(*parent).or_default();
                        let at = before
                            .and_then(|before| siblings.iter().position(|id| *id == before))
                            .unwrap_or(siblings.len());
                        siblings.insert(at, *child);
                    }
                    MountItem::ViewProps { id, props: sent } => {
                        if let Some(test_id) = &sent.test_id {
                            by_test_id.insert(test_id.to_string(), *id);
                        }
                        props.insert(*id, (**sent).clone());
                    }
                    MountItem::ViewStyle { id, style } => {
                        styles.insert(*id, (**style).clone());
                    }
                    MountItem::Frame { id, frame } => {
                        frames.insert(*id, *frame);
                    }
                    _ => {}
                }
            }
            let node = |test_id: &str| {
                *by_test_id
                    .get(test_id)
                    .unwrap_or_else(|| panic!("a view with the test id {test_id}: {items:?}"))
            };
            Self {
                divider: node("divider"),
                slot: node("slot"),
                overflow: node("overflow"),
                kinds,
                children,
                props,
                styles,
                frames,
            }
        }

        /// The last style `id` was sent; the default one if it was sent none.
        fn style(&self, id: NodeId) -> ViewStyle {
            self.styles.get(&id).cloned().unwrap_or_default()
        }

        /// The last props `id` was sent; the default ones if it was sent none.
        fn props(&self, id: NodeId) -> ViewProps {
            self.props.get(&id).cloned().unwrap_or_default()
        }

        /// The divider's size, as its layout gave it.
        fn size(&self) -> Size<Points> {
            self.frames[&self.divider].size
        }

        /// The divider's color, as its style paints it.
        fn color(&self) -> Color {
            self.style(self.divider).background_color
        }
    }

    fn mount(config: Config) -> Mounted {
        let (_app, recording) = start(config);
        Mounted::of(&recording.items())
    }

    /// The divider's last size after `app` ran a step: what the whole recording says.
    fn size_now(recording: &Recording) -> Size<Points> {
        Mounted::of(&recording.items()).size()
    }

    /// NSH's color of `hex` (`0xRRGGBB`), opaque.
    fn opaque(hex: u32) -> Color {
        Color((hex << 8) | 0xFF)
    }

    fn scheme_color(role: ColorRole, dark: bool) -> Color {
        let scheme = if dark {
            ColorScheme::BASELINE_DARK
        } else {
            ColorScheme::BASELINE_LIGHT
        };
        opaque(scheme.get(role).to_hex())
    }

    fn size(width: f32, height: f32) -> Size<Points> {
        Size {
            width: Points(width),
            height: Points(height),
        }
    }

    /// The size a divider of `orientation` and `thickness` has in a slot of `slot`: the whole of the slot along its
    /// length, its thickness across.
    fn expected_size(
        orientation: DividerOrientation,
        thickness: f32,
        slot: (f32, f32),
    ) -> Size<Points> {
        match orientation {
            DividerOrientation::Horizontal => size(slot.0, thickness),
            _ => size(thickness, slot.1),
        }
    }

    #[test]
    fn the_defaults_are_androidx_defaults() {
        // `HorizontalDivider` and `VerticalDivider` are the two directions; the first is the one `Divider` has always
        // been (`Divider.kt` line 49).
        assert_eq!(
            DividerOrientation::default(),
            DividerOrientation::Horizontal
        );
        // `DividerDefaults.Thickness` and `DividerDefaults.color` read these tokens (lines 113 and 117).
        assert_eq!(divider_tokens::THICKNESS, Points(1.0));
        assert_eq!(divider_tokens::COLOR, ColorRole::OutlineVariant);
        // Given only its direction, or nothing, a divider is the token's thickness in the token's color.
        for (given, orientation) in [
            (0, DividerOrientation::Horizontal),
            (ORIENTATION, DividerOrientation::Horizontal),
            (ORIENTATION, DividerOrientation::Vertical),
        ] {
            let mounted = mount(Config {
                given,
                orientation,
                ..Config::default()
            });
            assert_eq!(
                mounted.size(),
                expected_size(orientation, 1.0, (300.0, 80.0)),
                "{given} {orientation:?}"
            );
            assert_eq!(
                mounted.color(),
                scheme_color(ColorRole::OutlineVariant, false),
                "{given} {orientation:?}"
            );
        }
    }

    #[test]
    fn the_color_is_the_outline_variant_in_both_appearances() {
        let light = scheme_color(ColorRole::OutlineVariant, false);
        let dark = scheme_color(ColorRole::OutlineVariant, true);
        // The two appearances do not share the color: a divider that ignored the appearance would pass one of these.
        assert_ne!(light, dark);
        for given in [0, COLOR] {
            for (is_dark, expected) in [(false, light), (true, dark)] {
                let mounted = mount(Config {
                    given,
                    dark: is_dark,
                    ..Config::default()
                });
                assert_eq!(mounted.color(), expected, "given {given}, dark {is_dark}");
            }
        }
    }

    #[test]
    fn any_role_of_the_scheme_can_be_the_color() {
        for role in ColorRole::ALL {
            for dark in [false, true] {
                let mounted = mount(Config {
                    given: COLOR,
                    color: *role,
                    dark,
                    ..Config::default()
                });
                assert_eq!(
                    mounted.color(),
                    scheme_color(*role, dark),
                    "{role:?}, dark {dark}"
                );
            }
        }
    }

    #[test]
    fn the_color_follows_the_appearance_with_nothing_running() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let light = scheme_color(ColorRole::OutlineVariant, false);
        let dark = scheme_color(ColorRole::OutlineVariant, true);
        assert_eq!(Mounted::of(&recording.items()).color(), light);
        app.set_environment("dark", |environment| {
            environment.color_scheme = Appearance::Dark;
        })?;
        assert_eq!(Mounted::of(&recording.items()).color(), dark);
        app.set_environment("light", |environment| {
            environment.color_scheme = Appearance::Light;
        })?;
        assert_eq!(Mounted::of(&recording.items()).color(), light);
        Ok(())
    }

    #[test]
    fn the_color_follows_its_prop() -> Result<(), String> {
        let (mut app, recording) = start(Config::with(DividerOrientation::Horizontal, 1.0));
        assert_eq!(
            Mounted::of(&recording.items()).color(),
            scheme_color(ColorRole::OutlineVariant, false)
        );
        app.press("Recolor")?;
        assert_eq!(
            Mounted::of(&recording.items()).color(),
            scheme_color(ColorRole::Primary, false)
        );
        Ok(())
    }

    #[test]
    fn the_line_is_exactly_as_thick_as_asked() {
        // The token's, whole ones, and ones between whole points on a screen whose device pixels they are whole
        // numbers of: androidx takes any `Dp`, and lays it out in whole pixels.
        for (pixel_ratio, thicknesses) in [
            (1.0, vec![1.0, 2.0, 4.0, 12.0]),
            (2.0, vec![0.5, 1.0, 1.5, 12.5]),
            (4.0, vec![0.25, 0.75]),
        ] {
            for thickness in thicknesses {
                for orientation in ORIENTATIONS {
                    let mounted = mount(Config {
                        pixel_ratio,
                        ..Config::with(orientation, thickness)
                    });
                    assert_eq!(
                        mounted.size(),
                        expected_size(orientation, thickness, (300.0, 80.0)),
                        "{orientation:?}, {thickness} at {pixel_ratio}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_line_fills_the_space_it_is_in_across_or_down() {
        for slot in [(300.0, 80.0), (120.0, 40.0), (7.0, 500.0)] {
            for orientation in ORIENTATIONS {
                let mounted = mount(Config {
                    slot,
                    ..Config::with(orientation, 2.0)
                });
                assert_eq!(
                    mounted.size(),
                    expected_size(orientation, 2.0, slot),
                    "{orientation:?} in {slot:?}"
                );
                // At the start of the space: no inset, as androidx's dividers have none.
                assert_eq!(
                    mounted.frames[&mounted.divider].origin,
                    Point::default(),
                    "{orientation:?} in {slot:?}"
                );
            }
        }
    }

    #[test]
    fn the_thickness_is_not_squeezed_when_the_space_is_too_small() {
        // The overflowing view is along the thickness: it takes more than the slot has, and views are squeezed to fit
        // only when they are told to be (`height(thickness)` is exact in androidx, `Divider.kt` line 54).
        for orientation in ORIENTATIONS {
            let mounted = mount(Config::with(orientation, 4.0));
            assert_eq!(
                mounted.size(),
                expected_size(orientation, 4.0, (300.0, 80.0)),
                "{orientation:?}"
            );
            assert_eq!(
                mounted.frames[&mounted.overflow].size,
                size(400.0, 400.0),
                "{orientation:?}"
            );
        }
    }

    #[test]
    fn the_divider_is_one_view_and_lays_out_nothing_more() {
        for orientation in ORIENTATIONS {
            let (_app, recording) = start(Config::with(orientation, 1.0));
            let items = recording.items();
            let mounted = Mounted::of(&items);
            // A plain view, directly in the slot, with nothing in it: no canvas, no text, no wrapper.
            assert_eq!(
                mounted.kinds.get(&mounted.divider),
                Some(&ViewKind::View),
                "{orientation:?}"
            );
            assert_eq!(
                mounted.children.get(&mounted.slot),
                Some(&vec![mounted.divider, mounted.overflow]),
                "{orientation:?}"
            );
            assert!(
                mounted
                    .children
                    .get(&mounted.divider)
                    .is_none_or(Vec::is_empty),
                "{orientation:?}"
            );
            // It is laid out once.
            let frames = items
                .iter()
                .filter(
                    |item| matches!(item, MountItem::Frame { id, .. } if *id == mounted.divider),
                )
                .count();
            assert_eq!(frames, 1, "{orientation:?}");
        }
    }

    #[test]
    fn a_line_that_is_not_thick_takes_no_room_and_nothing_breaks() {
        // Zero, negative, not a number and infinite are no thickness at all. Huge ones are thick, and the engine copes.
        for thickness in [0.0, -0.0, -3.0, f32::NEG_INFINITY, f32::NAN, f32::INFINITY] {
            for orientation in ORIENTATIONS {
                let mounted = mount(Config::with(orientation, thickness));
                assert_eq!(
                    mounted.size(),
                    expected_size(orientation, 0.0, (300.0, 80.0)),
                    "{orientation:?}, {thickness}"
                );
            }
        }
        for thickness in [1.0e9, f32::MAX] {
            for orientation in ORIENTATIONS {
                let mounted = mount(Config::with(orientation, thickness));
                assert_eq!(
                    mounted.size(),
                    expected_size(orientation, thickness, (300.0, 80.0)),
                    "{orientation:?}, {thickness}"
                );
            }
        }
    }

    #[test]
    fn the_line_follows_its_props() -> Result<(), String> {
        let (mut app, recording) = start(Config::with(DividerOrientation::Horizontal, 1.0));
        assert_eq!(size_now(&recording), size(300.0, 1.0));
        app.press("Thicken")?;
        assert_eq!(size_now(&recording), size(300.0, 4.0));
        // Turned, it is as tall as the slot and as wide as it is thick: the view is the same one.
        let before = Mounted::of(&recording.items()).divider;
        app.press("Turn")?;
        let mounted = Mounted::of(&recording.items());
        assert_eq!(mounted.size(), size(4.0, 80.0));
        assert_eq!(mounted.divider, before);
        app.press("Turn")?;
        assert_eq!(size_now(&recording), size(300.0, 4.0));
        Ok(())
    }

    #[test]
    fn a_hairline_is_one_device_pixel_on_every_screen() -> Result<(), String> {
        for pixel_ratio in [1.0_f32, 1.5, 2.0, 3.0, 4.0] {
            for orientation in ORIENTATIONS {
                let mounted = mount(Config {
                    given: ORIENTATION,
                    orientation,
                    hairline: true,
                    pixel_ratio,
                    ..Config::default()
                });
                let thickness = match orientation {
                    DividerOrientation::Horizontal => mounted.size().height,
                    _ => mounted.size().width,
                };
                // One device pixel, which the layout may place on the pixel grid to the last bit of a float.
                assert!(
                    (thickness.0 * pixel_ratio - 1.0).abs() < 1.0e-4,
                    "{orientation:?} at {pixel_ratio}: {thickness:?}"
                );
            }
        }
        // Where a device pixel is a whole fraction of a point it is that fraction, to the bit.
        for (pixel_ratio, thickness) in [(1.0, 1.0), (2.0, 0.5), (4.0, 0.25)] {
            let mounted = mount(Config {
                hairline: true,
                pixel_ratio,
                ..Config::default()
            });
            assert_eq!(mounted.size(), size(300.0, thickness), "at {pixel_ratio}");
        }
        // A window moved to a screen of another density changes it, with nothing else running.
        let (mut app, recording) = start(Config {
            hairline: true,
            ..Config::default()
        });
        assert_eq!(size_now(&recording), size(300.0, 1.0));
        app.set_environment("retina", |environment| environment.pixel_ratio = 2.0)?;
        assert_eq!(size_now(&recording), size(300.0, 0.5));
        app.set_environment("back", |environment| environment.pixel_ratio = 1.0)?;
        assert_eq!(size_now(&recording), size(300.0, 1.0));
        Ok(())
    }

    #[test]
    fn it_is_hidden_from_assistive_technologies_and_is_nothing_they_announce() {
        for orientation in ORIENTATIONS {
            let (_app, recording) = start(Config::with(orientation, 1.0));
            let items = recording.items();
            let mounted = Mounted::of(&items);
            let props = mounted.props(mounted.divider);
            assert!(props.accessibility.hidden, "{orientation:?}");
            // Hidden, and no button, link or text either: nothing that would be announced if it were not.
            assert_eq!(
                props.accessibility.role,
                AccessibilityRole::None,
                "{orientation:?}"
            );
            assert_eq!(props.accessibility.label, None, "{orientation:?}");
            assert!(
                !mounted.props(mounted.slot).accessibility.hidden,
                "{orientation:?}"
            );
            // The views around it are not hidden with it.
            assert!(
                !mounted.props(mounted.overflow).accessibility.hidden,
                "{orientation:?}"
            );
            assert!(
                items.iter().all(|item| !matches!(
                    item,
                    MountItem::ViewProps { id, props }
                        if *id == mounted.divider && props.accessibility.role != AccessibilityRole::None
                )),
                "{orientation:?}"
            );
        }
    }

    #[test]
    fn it_takes_no_press_no_hover_and_no_focus() -> Result<(), String> {
        for orientation in ORIENTATIONS {
            let (mut app, recording) = start(Config::with(orientation, 1.0));
            let items = recording.items();
            let mounted = Mounted::of(&items);
            let props = mounted.props(mounted.divider);
            // Not a stop of the keyboard focus, and it does not ask to be one.
            assert_eq!(props.focusable, None, "{orientation:?}");
            assert!(!props.focus_stop(false), "{orientation:?}");
            assert_eq!(props.hit_slop, EdgeInsets::default(), "{orientation:?}");
            assert_eq!(props.ripple, None, "{orientation:?}");
            // No event is asked for, and no interaction state (hover, press, focus) is tracked for it.
            assert!(
                items.iter().all(|item| match item {
                    MountItem::Events { id, events } =>
                        *id != mounted.divider || events.bits() == 0,
                    MountItem::TrackStates { id, states }
                    | MountItem::ProjectStates { id, states } => {
                        *id != mounted.divider || *states == 0
                    }
                    _ => true,
                }),
                "{orientation:?}"
            );
            assert!(
                !items.iter().any(|item| matches!(
                    item,
                    MountItem::Events { id, events }
                        if *id == mounted.divider && events.contains(EventKind::Press)
                )),
                "{orientation:?}"
            );
            // A press on it reaches nothing: there is no view that takes it.
            assert_eq!(app.screen().press_target("id:divider"), Ok(None));
            assert!(app.press("id:divider").is_err(), "{orientation:?}");
            // The mouse comes over it and nothing is sent: no hover style, no hover event, no layout.
            app.hover("id:divider")?;
            assert!(
                recording.in_step("hover(id:divider)").is_empty(),
                "{orientation:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn it_is_a_plain_filled_rectangle_that_animates_nothing() -> Result<(), String> {
        let (mut app, recording) = start(Config::with(DividerOrientation::Horizontal, 1.0));
        let first = Mounted::of(&recording.items());
        let style = first.style(first.divider);
        // No shadow, border, radius, opacity or effect: a line is its color and its box.
        assert_eq!(style.elevation, Points(0.0));
        assert!(style.effects.is_none());
        assert_eq!(style.borders, Borders::NONE);
        assert_eq!(style.border_radius, Points(0.0));
        assert_eq!(style.opacity, 1.0);
        assert!(!style.hidden);
        // Nothing animates it: no transition or keyframes in its style, and no animation of NSH's graph.
        assert!(style.transition.is_none());
        assert!(style.animation.is_none());
        // A change of thickness, direction or color is at once, in the step that makes it.
        for (step, label) in [
            ("press(Thicken)", "Thicken"),
            ("press(Turn)", "Turn"),
            ("press(Recolor)", "Recolor"),
        ] {
            app.press(label)?;
            let items = recording.in_step(step);
            assert!(!items.is_empty(), "{step}");
            assert!(
                items
                    .iter()
                    .all(|item| !matches!(item, MountItem::Animation(_))),
                "{step}: {items:?}"
            );
            assert!(
                items.iter().all(|item| !matches!(
                    item,
                    MountItem::ViewStyle { style, .. }
                        if style.transition.is_some() || style.animation.is_some()
                )),
                "{step}: {items:?}"
            );
        }
        assert!(
            recording
                .items()
                .iter()
                .all(|item| !matches!(item, MountItem::Animation(_)))
        );
        Ok(())
    }

    proptest! {
        #[test]
        fn a_thickness_is_laid_out_as_it_is_or_as_none(thickness in any::<f32>()) {
            let laid_out = extent(Points(thickness)).0;
            prop_assert!(laid_out.is_finite() && laid_out >= 0.0, "{laid_out}");
            if thickness.is_finite() && thickness > 0.0 {
                prop_assert_eq!(laid_out.to_bits(), thickness.to_bits());
            } else {
                prop_assert!(laid_out == 0.0, "{laid_out}");
            }
        }
    }

    /// The vendored `Divider.kt`, at the pinned commit (ADR 0012).
    const DIVIDER_KT: &str =
        include_str!("../../../reference/androidx-material3-components/Divider.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn every_choice_taken_from_androidx_is_still_there() {
        // The two directions fill the space along their length and are exactly as thick as asked across.
        assert!(cites(
            DIVIDER_KT,
            &["Canvas(modifier.fillMaxWidth().height(thickness)) {"]
        ));
        assert!(cites(
            DIVIDER_KT,
            &["Canvas(modifier.fillMaxHeight().width(thickness)) {"]
        ));
        // The line is drawn through the middle of that box, which it covers: one fill of the box is the same pixels.
        assert!(cites(
            DIVIDER_KT,
            &[
                "strokeWidth = thickness.toPx(),",
                "start = Offset(0f, thickness.toPx() / 2),",
                "end = Offset(size.width, thickness.toPx() / 2),",
            ]
        ));
        assert!(cites(
            DIVIDER_KT,
            &[
                "strokeWidth = thickness.toPx(),",
                "start = Offset(thickness.toPx() / 2, 0f),",
                "end = Offset(thickness.toPx() / 2, size.height),",
            ]
        ));
        // Thickness and color are parameters, and default to the tokens.
        for line in [
            "thickness: Dp = DividerDefaults.Thickness,",
            "color: Color = DividerDefaults.color,",
            "public val Thickness: Dp = DividerTokens.Thickness",
            "@Composable get() = DividerTokens.Color.value",
        ] {
            assert!(cites(DIVIDER_KT, &[line]), "{line}");
        }
        // A hairline is one device pixel: the deprecated `Divider` makes it 1 / density.
        assert!(cites(
            DIVIDER_KT,
            &[
                "if (thickness == Dp.Hairline) {",
                "(1f / LocalDensity.current.density).dp",
            ]
        ));
        // And what the dividers do not have: no semantics, no input, no animation, no inset, no shadow.
        for absent in [
            "semantics",
            "clickable",
            "hoverable",
            "focusable",
            "pointerInput",
            "animate",
            "padding(",
            "shadow",
            "elevation",
        ] {
            assert!(!DIVIDER_KT.contains(absent), "{absent}");
        }
        // A line that is not there is not found.
        assert!(!cites(
            DIVIDER_KT,
            &["Canvas(modifier.fillMaxWidth().width(thickness)) {"]
        ));
    }
}
