//! The Material 3 Expressive badge: a [`Badge`], a dot or a short label in the error color, and a [`BadgedBox`] that
//! places one at the top end corner of the content it badges.
//!
//! The badge follows androidx's `Badge` and `BadgedBox` in `Badge.kt`, at the pinned commit, as [ADR 0012] says: that
//! file reads `BadgeTokens` for the sizes, the shape, the colors and the type style, and holds the placement values
//! of its own, which are the constants below, each with its line.
//!
//! What the badge keeps from androidx:
//!
//! - A badge with no label is a dot of `BadgeTokens.Size` (6 points); one with a label is a pill at least
//!   `BadgeTokens.LargeSize` (16 points) tall and wide, 4 points of space on each side of its text, which is
//!   `BadgeTokens.LargeLabelTextFont` (label small) in `contentColorFor(Error)`, on-error. Both are fully rounded
//!   (`CornerFull`) and filled with `BadgeTokens.Color`, error (androidx's `Badge` reads `Color` for both, never
//!   `LargeColor`).
//! - Nothing in a badge is sized by a number of points but its minimum: the label grows the pill, and so does the
//!   user's text size (a label at 200 % is 32 points tall, never clipped).
//! - A badge takes no touch: androidx draws it without `Surface` "because it blocks touch propagation behind it", so
//!   a press goes to what is under the badge. It is not interactive, has no hover, focus or press state, no touch
//!   target and no animation, and it adds no semantics: its label is read as the text it is, and a dot is not read,
//!   so the app names what a dot means in the name of the content it badges, as with androidx.
//! - [`BadgedBox`] places the badge so that its leading edge is 6 points (a dot) or 12 points (a label) before the
//!   trailing edge of the content, and so that it overlaps the content by 6 points (a dot) or 14 points (a label)
//!   from the content's top edge. It is as large as its content, whatever the badge. Leading and trailing follow
//!   the layout direction, as androidx's `placeRelative` does.
//!
//! What it does not, and why:
//!
//! - androidx's `Badge` takes `containerColor` and `contentColor`. This one has neither: every component of this
//!   library takes its colors from the roles of the app's NSH design tokens (ADR 0008), so an app recolors its
//!   badges with `md-sys-color-error` and `md-sys-color-on-error`, and `contentColorFor`, which reads the content
//!   color of the component around, has no counterpart in NSH.
//! - androidx's `BadgedBox` takes any composable as its badge, and tells a dot from a label by the badge's measured
//!   width (`badgePlaceable.width > BadgeTokens.Size`). NSH lays out once, so a [`BadgedBox`] builds its badge from
//!   its own `label`, the same two states as [`Badge`], which gives the same offsets without a second layout.
//! - androidx's `BadgeTopRuler` and `BadgeEndRuler`, which a container such as a navigation bar item sets with the
//!   internal `Modifier.badgeBounds()` to keep the badge inside it, are not ported: NSH has no layout rulers (not
//!   found in its source at the pinned revision), and no component of this library sets them yet. A [`BadgedBox`]
//!   always places its badge as androidx does with no ruler (an infinite end ruler, which clamps nothing).
//!
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::LineHeight;
use nsh_core::geometry::Points;
use nsh_core::prelude::*;

use crate::tint::{Tint, tint_color};
use crate::tokens::androidx::badge_tokens;

/// androidx `Badge.kt`, `BadgeWithContentHorizontalPadding` (line 202): the space before and after a label, "when a
/// badge is displaying text that is too long to fit in a circular badge". No token holds it.
const LABEL_HORIZONTAL_PADDING: Points = Points(4.0);

/// androidx `Badge.kt`, `BadgeWithContentHorizontalOffset` (line 209): how far before the trailing edge of its
/// content a badge with a label starts. No token holds it.
const LABEL_HORIZONTAL_OFFSET: Points = Points(12.0);

/// androidx `Badge.kt`, `BadgeWithContentVerticalOffset` (line 211): how far a badge with a label overlaps its content
/// from the content's top edge. No token holds it.
const LABEL_VERTICAL_OFFSET: Points = Points(14.0);

/// androidx `Badge.kt`, `BadgeOffset` (line 218): both offsets of a dot, which sits against the top end corner of its
/// content. It equals `BadgeTokens.Size` without being it. No token holds it.
const DOT_OFFSET: Points = Points(6.0);

/// The props of a [`Badge`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct BadgeProps {
    /// Its label: `None` is a dot, `Some` a pill that shows the text. The text is short (a count, "99+"): the badge
    /// grows with it and never cuts it, and the app writes it (a count's format belongs to the app's translation
    /// files).
    #[prop(default)]
    pub label: Prop<Option<SharedString>>,
}

/// A Material 3 Expressive badge: a dot, or a pill that shows a short label, in the error color.
///
/// It is as large as its content: put it in a row, as with the other components, and use [`BadgedBox`] to place it at
/// the corner of what it badges. It takes no touch and says nothing to screen readers by itself, as androidx's.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::badge::badge;
///
/// fn unread() -> impl IntoElement {
///     view()
///         .flex_row()
///         .gap_2()
///         .child(badge())
///         .child(badge().label(Some(SharedString::from("99+"))))
/// }
/// ```
#[component]
pub fn Badge(props: BadgeProps) -> impl IntoElement {
    let has_label = move || props.label.get().is_some();
    // androidx `Badge`: `if (content != null) BadgeTokens.LargeSize else BadgeTokens.Size`, as `defaultMinSize`.
    let size = move || {
        if has_label() {
            badge_tokens::LARGE_SIZE
        } else {
            badge_tokens::SIZE
        }
    };
    view()
        .flex_row()
        .items_center()
        .justify_center()
        .min_w(size)
        .min_h(size)
        .px(move || {
            if has_label() {
                LABEL_HORIZONTAL_PADDING
            } else {
                Points::ZERO
            }
        })
        // `BadgeTokens.Shape` and `LargeShape` are both `CornerFull` (a test pins it): half the shorter side.
        .rounded_full()
        .bg(tint_color(Some(Tint::solid(badge_tokens::COLOR))))
        // androidx does not use `Surface`, "because it blocks touch propagation behind it".
        .pointer_events(PointerEvents::None)
        .child(when(has_label, move || {
            let font = badge_tokens::LARGE_LABEL_TEXT_FONT;
            text(move || props.label.get().unwrap_or_default())
                .text_color(tint_color(Some(Tint::solid(
                    badge_tokens::LARGE_LABEL_TEXT_COLOR,
                ))))
                .text_size(font.size)
                .line_height(LineHeight::Points(font.line_height))
                .letter_spacing(font.tracking)
                .style_with(move |s| s.font_weight(font.weight))
        }))
}

/// The props of a [`BadgedBox`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct BadgedBoxProps {
    /// The label of its badge, as [`BadgeProps::label`] says: `None` is a dot, `Some` a pill that shows the text.
    #[prop(default)]
    pub label: Prop<Option<SharedString>>,
    /// The content the badge is placed on (an icon): it sets the box's size.
    pub children: Prop<Children>,
}

/// A Material 3 Expressive badged box: its content, an icon for instance, with a [`Badge`] at its top end corner.
///
/// The box is as large as its content, whatever the badge, and the badge overflows it: nothing is clipped, and a
/// press goes through the badge to what is under it. Place the box in a row, as with the other components.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::badge::badged_box;
///
/// fn inbox() -> impl IntoElement {
///     view().flex_row().child(
///         badged_box()
///             .label(Some(SharedString::from("3")))
///             // The content the badge is placed on: here a 24-point square, where an app puts its icon.
///             .child(view().w(px(24.0)).h(px(24.0))),
///     )
/// }
/// ```
#[component]
pub fn BadgedBox(props: BadgedBoxProps) -> impl IntoElement {
    let has_label = move || props.label.get().is_some();
    // androidx `BadgedBox`: `if (hasContent) BadgeWithContentHorizontalOffset else BadgeOffset`, and the same for the
    // vertical one.
    let horizontal = move || {
        if has_label() {
            LABEL_HORIZONTAL_OFFSET
        } else {
            DOT_OFFSET
        }
    };
    let vertical = move || {
        if has_label() {
            LABEL_VERTICAL_OFFSET
        } else {
            DOT_OFFSET
        }
    };
    view().flex_row().child(props.children).child(
        // A corner with no size, right after the content at its top, where the content ends whichever the layout
        // direction (a row runs from the right in a right-to-left one): the badge is placed from it by margins on
        // its logical sides, which NSH mirrors, so it needs no side that is physical. Zero is not a design value.
        view()
            .flex_row()
            .items_end()
            .self_start()
            .w(Points::ZERO)
            .h(Points::ZERO)
            .child(
                badge()
                        .label(props.label)
                        // Its bottom edge is `vertical` below the corner, so that it overlaps the content by
                        // `vertical` whatever its height; its leading edge is `horizontal` before the corner.
                        .ms(move || Points(-horizontal().0))
                        .mb(move || Points(-vertical().0))
                        .flex_shrink_0(),
            ),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::SystemTime;

    use nsh_core::geometry::Bounds;
    use nsh_core::platform::{ColorScheme as Appearance, Environment};
    use nsh_core::{
        AccessibilityRole, AttributedText, Color, MountItem, NodeId, ResolvedTextStyle, ViewStyle,
    };
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::Recording;
    use crate::color::{ColorRole, ColorScheme};
    use crate::tokens::Shape;
    use crate::tokens::androidx::typography_tokens;

    /// The anchor's side in the tests: the 24-point icon a navigation bar item badges. A test value, not a token.
    const ANCHOR: f32 = 24.0;

    /// What the probes place.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        #[prop(default)]
        label: Prop<Option<SharedString>>,
    }

    /// One badge in a row, and two lines that give it a label and take it away.
    #[component]
    fn BadgeProbe(props: ProbeProps) -> impl IntoElement {
        let label = signal(props.label.get_untracked());
        // A row, so that the badge keeps its own size, not stretched across a column.
        view()
            .child(
                view()
                    .flex_row()
                    .test_id("row")
                    .child(badge().label(move || label.get())),
            )
            .child(text("Ready"))
            .child(text("Label").on_press(move |(): &()| label.set(Some(SharedString::from("7")))))
            .child(text("Dot").on_press(move |(): &()| label.set(None)))
    }

    /// A badged box on a square anchor, in a row, and two lines that give its badge a label and take it away.
    #[component]
    fn BoxProbe(props: ProbeProps) -> impl IntoElement {
        let label = signal(props.label.get_untracked());
        view()
            .child(
                view().flex_row().test_id("row").child(
                    badged_box()
                        .label(move || label.get())
                        .child(view().w(px(ANCHOR)).h(px(ANCHOR)).test_id("anchor")),
                ),
            )
            .child(text("Ready"))
            .child(text("Label").on_press(move |(): &()| label.set(Some(SharedString::from("7")))))
            .child(text("Dot").on_press(move |(): &()| label.set(None)))
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct Config {
        dark: bool,
        rtl: bool,
        /// The user's text size; the default one when `None`.
        font_scale: Option<f32>,
    }

    fn start(config: Config, boxed: bool, label: Option<&'static str>) -> (Headless, Recording) {
        let props = move || ProbeProps {
            label: Prop::new(label.map(SharedString::from)),
        };
        let root = if boxed {
            Root::new::<BoxProbe>(props)
        } else {
            Root::new::<BadgeProbe>(props)
        };
        let environment = Environment {
            color_scheme: if config.dark {
                Appearance::Dark
            } else {
                Appearance::Light
            },
            rtl: config.rtl,
            font_scale: config.font_scale.unwrap_or(1.0),
            ..Environment::default()
        };
        let mut app = Headless::start(EngineConfig {
            clock: Clock::manual(SystemTime::UNIX_EPOCH),
            environment,
            ..EngineConfig::new(root)
        })
        .unwrap();
        let recording = Recording::attach(&mut app);
        app.wait_for("text:Ready").unwrap();
        (app, recording)
    }

    /// The views as the mount items built them, with what a badge is checked by.
    #[derive(Default)]
    struct Mounted {
        children: HashMap<NodeId, Vec<NodeId>>,
        parent: HashMap<NodeId, NodeId>,
        frames: HashMap<NodeId, Bounds<Points>>,
        styles: HashMap<NodeId, ViewStyle>,
        texts: HashMap<NodeId, Arc<AttributedText>>,
        test_ids: HashMap<String, NodeId>,
        pointer_events: HashMap<NodeId, PointerEvents>,
        roles: HashMap<NodeId, AccessibilityRole>,
        labels: HashMap<NodeId, Option<Arc<str>>>,
        events: Vec<NodeId>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut mounted = Self::default();
            for item in items {
                match item {
                    MountItem::Insert {
                        parent,
                        child,
                        before,
                    } => {
                        let siblings = mounted.children.entry(*parent).or_default();
                        let at = before
                            .and_then(|before| siblings.iter().position(|id| *id == before))
                            .unwrap_or(siblings.len());
                        siblings.insert(at, *child);
                        mounted.parent.insert(*child, *parent);
                    }
                    MountItem::Remove { id } | MountItem::Delete { id } => {
                        if let Some(parent) = mounted.parent.remove(id)
                            && let Some(siblings) = mounted.children.get_mut(&parent)
                        {
                            siblings.retain(|sibling| sibling != id);
                        }
                    }
                    MountItem::ViewProps { id, props } => {
                        if let Some(test_id) = &props.test_id {
                            mounted.test_ids.insert(test_id.to_string(), *id);
                        }
                        mounted.pointer_events.insert(*id, props.pointer_events);
                        mounted.roles.insert(*id, props.accessibility.role);
                        mounted
                            .labels
                            .insert(*id, props.accessibility.label.clone());
                    }
                    MountItem::Frame { id, frame } => {
                        mounted.frames.insert(*id, *frame);
                    }
                    MountItem::ViewStyle { id, style } => {
                        mounted.styles.insert(*id, (**style).clone());
                    }
                    MountItem::Text { id, text, .. } => {
                        mounted.texts.insert(*id, Arc::clone(text));
                    }
                    MountItem::Events { id, .. } => mounted.events.push(*id),
                    _ => {}
                }
            }
            mounted
        }

        fn frame(&self, id: NodeId) -> Bounds<Points> {
            self.frames[&id]
        }

        fn style(&self, id: NodeId) -> ViewStyle {
            self.styles.get(&id).cloned().unwrap_or_default()
        }

        fn children_of(&self, id: NodeId) -> Vec<NodeId> {
            self.children.get(&id).cloned().unwrap_or_default()
        }

        /// `id` and every view below it.
        fn subtree(&self, id: NodeId) -> Vec<NodeId> {
            let mut all = vec![id];
            for child in self.children_of(id) {
                all.extend(self.subtree(child));
            }
            all
        }

        /// The texts below `id`, each with its node.
        fn texts_under(&self, id: NodeId) -> Vec<(NodeId, Arc<AttributedText>)> {
            self.subtree(id)
                .into_iter()
                .filter_map(|node| self.texts.get(&node).map(|text| (node, Arc::clone(text))))
                .collect()
        }
    }

    /// A plain badge.
    struct Plain {
        mounted: Mounted,
        badge: NodeId,
    }

    impl Plain {
        fn of(items: &[MountItem]) -> Self {
            let mounted = Mounted::of(items);
            let badge = mounted.children_of(mounted.test_ids["row"])[0];
            Self { mounted, badge }
        }

        fn frame(&self) -> Bounds<Points> {
            self.mounted.frame(self.badge)
        }

        fn style(&self) -> ViewStyle {
            self.mounted.style(self.badge)
        }

        /// The label's node, its text and its frame in the badge.
        fn label(&self) -> Option<(NodeId, Arc<AttributedText>, Bounds<Points>)> {
            let texts = self.mounted.texts_under(self.badge);
            assert!(texts.len() <= 1, "at most one text in a badge: {texts:?}");
            texts
                .into_iter()
                .next()
                .map(|(node, text)| (node, text, self.mounted.frame(node)))
        }
    }

    /// A badged box.
    struct Boxed {
        mounted: Mounted,
        container: NodeId,
        anchor: NodeId,
        corner: NodeId,
        badge: NodeId,
    }

    impl Boxed {
        fn of(items: &[MountItem]) -> Self {
            let mounted = Mounted::of(items);
            let container = mounted.children_of(mounted.test_ids["row"])[0];
            // The anchor first, then the corner the badge is placed from, and nothing else in the box.
            let [anchor, corner] = mounted.children_of(container)[..] else {
                panic!(
                    "the anchor and the corner in the box: {:?}",
                    mounted.children
                );
            };
            assert_eq!(mounted.test_ids["anchor"], anchor);
            let [badge] = mounted.children_of(corner)[..] else {
                panic!("the badge alone in the corner: {:?}", mounted.children);
            };
            Self {
                mounted,
                container,
                anchor,
                corner,
                badge,
            }
        }

        /// The badge's frame in the box's coordinates, where the anchor is at the origin.
        fn badge_in_box(&self) -> Bounds<Points> {
            let corner = self.mounted.frame(self.corner).origin;
            let mut frame = self.mounted.frame(self.badge);
            frame.origin.x = Points(frame.origin.x.0 + corner.x.0);
            frame.origin.y = Points(frame.origin.y.0 + corner.y.0);
            frame
        }
    }

    fn plain(config: Config, label: Option<&'static str>) -> Plain {
        let (_app, recording) = start(config, false, label);
        Plain::of(&recording.items())
    }

    fn boxed(config: Config, label: Option<&'static str>) -> Boxed {
        let (_app, recording) = start(config, true, label);
        Boxed::of(&recording.items())
    }

    /// NSH's color of `hex` (`0xRRGGBB`), opaque.
    fn opaque(hex: u32) -> Color {
        Color((hex << 8) | 255)
    }

    fn scheme_color(role: ColorRole, dark: bool) -> Color {
        let scheme = if dark {
            ColorScheme::BASELINE_DARK
        } else {
            ColorScheme::BASELINE_LIGHT
        };
        opaque(scheme.get(role).to_hex())
    }

    fn run_style(text: &AttributedText) -> ResolvedTextStyle {
        let [span] = text.spans[..] else {
            panic!("one run in a label: {text:?}");
        };
        span.style
    }

    /// `x` of a frame's right edge.
    fn right(frame: Bounds<Points>) -> f32 {
        frame.origin.x.0 + frame.size.width.0
    }

    /// `y` of a frame's bottom edge.
    fn bottom(frame: Bounds<Points>) -> f32 {
        frame.origin.y.0 + frame.size.height.0
    }

    /// Labels the tests give: a digit, two digits, the longest a count is written, and text that is not a count.
    const LABELS: [&str; 4] = ["1", "99", "999+", "Long label text"];

    #[test]
    fn the_tokens_are_the_ones_androidx_gives_the_badge() {
        // androidx `BadgeTokens` (v0_103): the dot is 6, the label's container at least 16, both fully rounded.
        assert_eq!(badge_tokens::SIZE, Points(6.0));
        assert_eq!(badge_tokens::LARGE_SIZE, Points(16.0));
        assert_eq!(badge_tokens::SHAPE, Shape::Circle);
        assert_eq!(badge_tokens::LARGE_SHAPE, Shape::Circle);
        // Error, on error; androidx's `Badge` reads `Color` for the dot and for the label.
        assert_eq!(badge_tokens::COLOR, ColorRole::Error);
        assert_eq!(badge_tokens::LARGE_COLOR, badge_tokens::COLOR);
        assert_eq!(badge_tokens::LARGE_LABEL_TEXT_COLOR, ColorRole::OnError);
        // Label small: 11 size, 16 line height, 0.5 tracking, medium weight.
        let font = badge_tokens::LARGE_LABEL_TEXT_FONT;
        assert_eq!(font, typography_tokens::LABEL_SMALL);
        assert_eq!(
            (font.size, font.line_height, font.tracking, font.weight.0),
            (Points(11.0), Points(16.0), Points(0.5), 500)
        );
    }

    /// The vendored `Badge.kt`, at the pinned commit (ADR 0012).
    const BADGE_KT: &str =
        include_str!("../../../reference/androidx-material3-components/Badge.kt");

    /// Whether `Badge.kt` holds `lines`, each trimmed, one after the other.
    fn cites(lines: &[&str]) -> bool {
        let file: Vec<&str> = BADGE_KT.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn every_value_taken_from_badge_kt_is_still_there() {
        // When androidx moves a value into a token, `Badge.kt` drops its literal, and this fails in the bump's pull
        // request. Each constant's value is the literal's.
        assert!(cites(&[
            "internal val BadgeWithContentHorizontalPadding",
            "get() = 4.dp"
        ]));
        assert_eq!(LABEL_HORIZONTAL_PADDING, Points(4.0));
        assert!(cites(&[
            "internal val BadgeWithContentHorizontalOffset",
            "get() = 12.dp"
        ]));
        assert_eq!(LABEL_HORIZONTAL_OFFSET, Points(12.0));
        assert!(cites(&[
            "internal val BadgeWithContentVerticalOffset",
            "get() = 14.dp"
        ]));
        assert_eq!(LABEL_VERTICAL_OFFSET, Points(14.0));
        assert!(cites(&["internal val BadgeOffset", "get() = 6.dp"]));
        assert_eq!(DOT_OFFSET, Points(6.0));
        // A dot's offsets are its size, so it sits against the corner of its content.
        assert_eq!(DOT_OFFSET, badge_tokens::SIZE);
        // The others are tokens, and these are the lines that read them.
        assert!(cites(&[
            "val size = if (content != null) BadgeTokens.LargeSize else BadgeTokens.Size"
        ]));
        assert!(cites(&[
            ".defaultMinSize(minWidth = size, minHeight = size)"
        ]));
        assert!(cites(&[
            "containerColor: Color = BadgeDefaults.containerColor,",
            "contentColor: Color = contentColorFor(containerColor),"
        ]));
        assert!(cites(&[
            "public val containerColor: Color",
            "@Composable get() = BadgeTokens.Color.value"
        ]));
        assert!(cites(&["val style = BadgeTokens.LargeLabelTextFont.value"]));
        // The offsets of a badge with a label and of a dot, and the placement from the content's end and top.
        assert!(cites(&[
            "if (hasContent) BadgeWithContentHorizontalOffset else BadgeOffset"
        ]));
        assert!(cites(&[
            "if (hasContent) BadgeWithContentVerticalOffset else BadgeOffset"
        ]));
        assert!(cites(&[
            "anchorPlaceable.width - badgeHorizontalOffset.roundToPx(),"
        ]));
        assert!(cites(&[
            "-badgePlaceable.height + badgeVerticalOffset.roundToPx(),"
        ]));
        // A line that is not there is not found.
        assert!(!cites(&["internal val BadgeOffset", "get() = 5.dp"]));
    }

    #[test]
    fn a_dot_is_a_six_point_circle_in_the_error_color_light_and_dark() {
        for dark in [false, true] {
            let dot = plain(
                Config {
                    dark,
                    ..Config::default()
                },
                None,
            );
            let what = format!("dark: {dark}");
            let frame = dot.frame();
            assert_eq!(
                (frame.size.width, frame.size.height),
                (Points(6.0), Points(6.0)),
                "{what}"
            );
            let style = dot.style();
            assert_eq!(
                style.background_color,
                scheme_color(ColorRole::Error, dark),
                "{what}"
            );
            // Fully rounded: a radius of at least half the side, which the platforms cut to half.
            assert!(
                style.border_radius.0 >= 3.0,
                "{what}: {:?}",
                style.border_radius
            );
            // Nothing in it, and no border.
            assert!(dot.mounted.children_of(dot.badge).is_empty(), "{what}");
            assert_eq!(style.borders.widths.top, px(0.0), "{what}");
        }
        // The two colors differ: the dark one is not the light one.
        assert_ne!(
            scheme_color(ColorRole::Error, false),
            scheme_color(ColorRole::Error, true)
        );
    }

    #[test]
    fn a_label_is_a_pill_sixteen_points_tall_in_error_with_on_error_text() {
        for dark in [false, true] {
            for label in LABELS {
                let what = format!("{label:?} dark: {dark}");
                let badge = plain(
                    Config {
                        dark,
                        ..Config::default()
                    },
                    Some(label),
                );
                let frame = badge.frame();
                let (_, text, content) = badge.label().expect("the label's text");
                assert_eq!(text.text, label, "{what}");
                // Sixteen points tall, whatever the label: one line of 11-point text fits in it.
                assert_eq!(frame.size.height, Points(16.0), "{what}");
                // At least as wide as tall, and the label and 4 points on each side when that is more.
                let wanted = 16.0_f32.max(content.size.width.0 + 2.0 * 4.0);
                assert_eq!(frame.size.width.0, wanted, "{what}");
                // Fully rounded, a pill: the radius is at least half the height.
                let style = badge.style();
                assert!(
                    style.border_radius.0 >= 8.0,
                    "{what}: {:?}",
                    style.border_radius
                );
                assert_eq!(
                    style.background_color,
                    scheme_color(ColorRole::Error, dark),
                    "{what}"
                );
                // The text: label small, in on error.
                let run = run_style(&text);
                assert_eq!(run.color, scheme_color(ColorRole::OnError, dark), "{what}");
                assert_eq!(
                    (
                        run.font_size,
                        run.line_height,
                        run.letter_spacing,
                        run.font_weight
                    ),
                    (Points(11.0), Points(16.0), Points(0.5), 500),
                    "{what}"
                );
                // It sits inside the pill, centered across: the space left and right of it differs by a rounding.
                let text_frame = badge.mounted.frame(badge.label().expect("a label").0);
                let (before, after) = (
                    text_frame.origin.x.0,
                    frame.size.width.0 - right(text_frame),
                );
                assert!(
                    before >= 0.0 && after >= 0.0,
                    "{what}: {text_frame:?} in {frame:?}"
                );
                assert!(
                    (before - after).abs() <= 1.0,
                    "{what}: {before} and {after}"
                );
                // And when the label makes the pill wider than a circle, the space is the 4 points of the padding.
                if frame.size.width.0 > 16.0 {
                    assert_eq!((before, after), (4.0, 4.0), "{what}");
                }
            }
        }
    }

    #[test]
    fn a_label_of_one_character_makes_a_circle_and_a_longer_one_a_wider_pill() {
        let one = plain(Config::default(), Some("1")).frame();
        let two = plain(Config::default(), Some("99")).frame();
        let more = plain(Config::default(), Some("999+")).frame();
        let long = plain(Config::default(), Some("Long label text")).frame();
        assert_eq!(
            (one.size.width, one.size.height),
            (Points(16.0), Points(16.0))
        );
        assert!(one.size.width < two.size.width);
        assert!(two.size.width < more.size.width);
        assert!(more.size.width < long.size.width);
        // A long text is one line: it never wraps, so the pill stays 16 points tall.
        assert_eq!(long.size.height, Points(16.0));
    }

    #[test]
    fn a_badge_follows_its_label_from_a_dot_to_a_pill_and_back() {
        let (mut app, recording) = start(Config::default(), false, None);
        assert_eq!(
            Plain::of(&recording.items()).frame().size.width,
            Points(6.0)
        );
        app.press("text:Label").unwrap();
        let pill = Plain::of(&recording.items());
        assert_eq!(pill.frame().size, size(Points(16.0), Points(16.0)));
        let (_, text, _) = pill.label().expect("the label's text");
        assert_eq!(text.text, "7");
        assert!(app.shows("7"));
        app.press("text:Dot").unwrap();
        let dot = Plain::of(&recording.items());
        assert_eq!(dot.frame().size, size(Points(6.0), Points(6.0)));
        // The label is gone with its text, and the dot keeps the same color.
        assert!(dot.label().is_none());
        assert!(!app.shows("7"));
        assert_eq!(
            dot.style().background_color,
            scheme_color(ColorRole::Error, false)
        );
    }

    #[test]
    fn at_twice_the_text_size_the_label_grows_and_stays_a_pill_with_nothing_clipped() {
        let big = Config {
            font_scale: Some(2.0),
            ..Config::default()
        };
        for label in LABELS {
            let what = format!("{label:?}");
            let normal = plain(Config::default(), Some(label));
            let scaled = plain(big, Some(label));
            let (_, text, content) = scaled.label().expect("the label's text");
            // The type style is scaled whole, size and line height, by the user's text size: 22 and 32 points.
            let run = run_style(&text);
            assert_eq!(
                (run.font_size, run.line_height),
                (Points(22.0), Points(32.0)),
                "{what}"
            );
            // The badge has no height of its own but its minimum: it grows with its text, and is wider with it.
            let frame = scaled.frame();
            assert!(frame.size.height.0 > 16.0, "{what}: {frame:?}");
            assert!(
                frame.size.height.0 >= content.size.height.0,
                "{what}: {frame:?}"
            );
            assert!(
                frame.size.width.0 > normal.frame().size.width.0,
                "{what}: {frame:?}"
            );
            // The text is inside the badge on every side, so nothing is cut.
            let text_frame = scaled.mounted.frame(scaled.label().expect("a label").0);
            assert!(
                text_frame.origin.x.0 >= 4.0 && text_frame.origin.y.0 >= 0.0,
                "{what}: {text_frame:?}"
            );
            assert!(
                right(text_frame) <= frame.size.width.0 - 4.0,
                "{what}: {text_frame:?} in {frame:?}"
            );
            assert!(
                bottom(text_frame) <= frame.size.height.0,
                "{what}: {text_frame:?} in {frame:?}"
            );
            // Still a pill: the radius reaches half the height.
            assert!(
                scaled.style().border_radius.0 >= frame.size.height.0 / 2.0,
                "{what}"
            );
        }
        // A dot is not text: the user's text size does not change it.
        let dot = plain(big, None).frame();
        assert_eq!(
            (dot.size.width, dot.size.height),
            (Points(6.0), Points(6.0))
        );
    }

    #[test]
    fn a_badge_takes_no_touch_has_no_semantics_and_no_events() {
        for label in [None, Some("1"), Some("999+")] {
            let what = format!("{label:?}");
            for boxed_in in [false, true] {
                let (badge, mounted) = if boxed_in {
                    let b = boxed(Config::default(), label);
                    (b.badge, b.mounted)
                } else {
                    let p = plain(Config::default(), label);
                    (p.badge, p.mounted)
                };
                let what = format!("{what} in a box: {boxed_in}");
                // androidx draws it without a `Surface`, "because it blocks touch propagation behind it".
                assert_eq!(
                    mounted.pointer_events.get(&badge),
                    Some(&PointerEvents::None),
                    "{what}"
                );
                // No role, no name, no state: it adds nothing to what a screen reader reads.
                for node in mounted.subtree(badge) {
                    assert_ne!(
                        mounted.roles.get(&node),
                        Some(&AccessibilityRole::Button),
                        "{what}"
                    );
                    assert_eq!(
                        mounted.roles.get(&node).copied().unwrap_or_default(),
                        AccessibilityRole::None,
                        "{what}"
                    );
                    assert_eq!(mounted.labels.get(&node).cloned().flatten(), None, "{what}");
                    // And listens to nothing: no press, no hover.
                    assert!(!mounted.events.contains(&node), "{what}: {node}");
                }
            }
        }
    }

    #[test]
    fn a_badged_box_is_as_large_as_its_content_and_puts_the_badge_where_androidx_does() {
        // (label, the badge's left edge from the box's left, its top from the box's top): a dot lies against the
        // corner (24 - 6, 0); a label starts 12 before the end and overlaps by 14, so its top is 14 - 16 = -2.
        let cases = [
            (None, 18.0, 0.0),
            (Some("1"), 12.0, -2.0),
            (Some("99"), 12.0, -2.0),
            (Some("999+"), 12.0, -2.0),
            (Some("Long label text"), 12.0, -2.0),
        ];
        for (label, x, y) in cases {
            let what = format!("{label:?}");
            let b = boxed(Config::default(), label);
            // The box is the anchor's size whatever the badge, and the anchor is at its origin.
            assert_eq!(
                b.mounted.frame(b.container).size,
                size(Points(ANCHOR), Points(ANCHOR)),
                "{what}"
            );
            assert_eq!(
                b.mounted.frame(b.anchor),
                Bounds::new(
                    point(Points(0.0), Points(0.0)),
                    size(Points(ANCHOR), Points(ANCHOR))
                ),
                "{what}"
            );
            // The corner has no size: it takes none of the box's.
            assert_eq!(
                b.mounted.frame(b.corner).size,
                size(Points(0.0), Points(0.0)),
                "{what}"
            );
            let badge = b.badge_in_box();
            assert_eq!((badge.origin.x.0, badge.origin.y.0), (x, y), "{what}");
            // The badge is the one a plain badge is: a dot, or a pill 16 points tall that is as wide as its text and
            // 4 points on each side, at least 16 (the box, a corner with no width, does not squeeze it or wrap it).
            let wanted = match b.mounted.texts_under(b.badge)[..] {
                [] => (6.0, 6.0),
                [(node, _)] => (16.0_f32.max(b.mounted.frame(node).size.width.0 + 8.0), 16.0),
                _ => panic!("at most one text: {:?}", b.mounted.children),
            };
            assert_eq!((badge.size.width.0, badge.size.height.0), wanted, "{what}");
        }
    }

    #[test]
    fn a_badged_box_keeps_the_badge_on_the_same_edges_at_twice_the_text_size() {
        let big = Config {
            font_scale: Some(2.0),
            ..Config::default()
        };
        // A dot is not text. A label overlaps by 14 below the top whatever its height, and starts 12 before the end.
        let dot = boxed(big, None).badge_in_box();
        assert_eq!((dot.origin.x.0, dot.origin.y.0), (18.0, 0.0));
        for label in LABELS {
            let what = format!("{label:?}");
            let b = boxed(big, Some(label));
            let badge = b.badge_in_box();
            assert!(badge.size.height.0 > 16.0, "{what}: {badge:?}");
            assert_eq!(badge.origin.x.0, 12.0, "{what}");
            assert_eq!(bottom(badge), 14.0, "{what}: {badge:?}");
            // The box is still the anchor's.
            assert_eq!(
                b.mounted.frame(b.container).size,
                size(Points(ANCHOR), Points(ANCHOR)),
                "{what}"
            );
        }
    }

    #[test]
    fn in_a_right_to_left_layout_the_badge_is_at_the_top_left_of_its_content() {
        let rtl = Config {
            rtl: true,
            ..Config::default()
        };
        // The same placement from the other end: a dot against the top left corner, a label whose right edge is 12
        // from the box's left edge and that overlaps by 14.
        let dot = boxed(rtl, None).badge_in_box();
        assert_eq!((dot.origin.x.0, dot.origin.y.0), (0.0, 0.0));
        for label in LABELS {
            let what = format!("{label:?}");
            let b = boxed(rtl, Some(label));
            let badge = b.badge_in_box();
            assert_eq!(right(badge), 12.0, "{what}: {badge:?}");
            assert_eq!(badge.origin.y.0, -2.0, "{what}");
            assert_eq!(
                b.mounted.frame(b.container).size,
                size(Points(ANCHOR), Points(ANCHOR)),
                "{what}"
            );
            // And the left to right layout of the same label is the other way: it is the badge that moved.
            assert_ne!(
                badge.origin.x.0,
                boxed(Config::default(), Some(label))
                    .badge_in_box()
                    .origin
                    .x
                    .0,
                "{what}"
            );
        }
    }

    #[test]
    fn a_badged_box_moves_its_badge_when_the_label_comes_and_goes() {
        let (mut app, recording) = start(Config::default(), true, None);
        let at = |items: &[MountItem]| Boxed::of(items).badge_in_box();
        let dot = at(&recording.items());
        assert_eq!(
            (dot.origin.x.0, dot.origin.y.0, dot.size.width.0),
            (18.0, 0.0, 6.0)
        );
        app.press("text:Label").unwrap();
        let pill = at(&recording.items());
        assert_eq!(
            (pill.origin.x.0, pill.origin.y.0, pill.size.width.0),
            (12.0, -2.0, 16.0)
        );
        assert!(app.shows("7"));
        app.press("text:Dot").unwrap();
        let back = at(&recording.items());
        assert_eq!(
            (back.origin.x.0, back.origin.y.0, back.size.width.0),
            (18.0, 0.0, 6.0)
        );
        assert!(!app.shows("7"));
    }

    #[test]
    fn a_badged_box_badge_has_the_colors_of_a_badge() {
        for dark in [false, true] {
            let config = Config {
                dark,
                ..Config::default()
            };
            let dot = boxed(config, None);
            assert_eq!(
                dot.mounted.style(dot.badge).background_color,
                scheme_color(ColorRole::Error, dark)
            );
            let pill = boxed(config, Some("99"));
            assert_eq!(
                pill.mounted.style(pill.badge).background_color,
                scheme_color(ColorRole::Error, dark)
            );
            let (_, text) = pill.mounted.texts_under(pill.badge).remove(0);
            assert_eq!(
                run_style(&text).color,
                scheme_color(ColorRole::OnError, dark)
            );
        }
    }
}
