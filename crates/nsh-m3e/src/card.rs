//! The Material 3 card: a container for content about one subject, filled, elevated or outlined, that can also be
//! pressed.
//!
//! androidx has one `Card` with two forms, one that is only a container and one that takes an `onClick`, and the
//! same pair for `ElevatedCard` and `OutlinedCard` (`Card.kt`, vendored in `reference/androidx-material3-components/`).
//! This module has the same two forms as two components, since a component's structure is made once: [`Card`], a
//! container with no behavior, and [`ClickableCard`], whose container is `nsh-base`'s button (presses, focus, the
//! disabled state, the button role). Both take a [`CardStyle`] and the content as children, and lay the content out
//! in a column, as androidx's `Column(content = content)` does.
//!
//! # What a card is made of
//!
//! The container takes its color, shape and elevation from `FilledCardTokens`, `ElevatedCardTokens` and
//! `OutlinedCardTokens`: surface container highest, surface container low and surface, all of them with the
//! `CornerMedium` shape. The elevated card rests at level 1, the other two at level 0; the shadow is the two layers of
//! [ADR 0018](https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0018-draw-elevation-as-two-box-shadows-per-level.md)
//! ([`Elevation::shadows`]), cast by the container, with no shadow value written here.
//!
//! The outlined card has an outline 1 point wide in the outline variant color. It is drawn over the container and its
//! content, inside its edge, by a view of its own that takes no place in the layout: androidx's `Surface` draws its
//! border with `Modifier.border`, which adds no size either, so an outlined card is as large as a filled one of the
//! same content and the content does not move when the style does.
//!
//! The clickable card has a state layer too: the on-surface color laid over the container, in its shape, at the state
//! layer opacities of `StateTokens`, shown by the button's own states with nothing running. It is what androidx's
//! ripple draws (`Surface(onClick)` takes `LocalIndication`, in the content color), and the content color of all three
//! containers is on-surface (`ColorScheme.contentColorFor`, `ColorScheme.kt` lines 1108, 1113 and 1114, not vendored).
//! It is drawn over the content, as the ripple is, by a view that takes no touch.
//!
//! # Where androidx overrides the tokens (ADR 0012)
//!
//! Each of these is pinned by a test that reads the vendored `Card.kt`.
//!
//! - **The outlined card does not rise.** `OutlinedCardTokens` gives level 1 hovered, but `outlinedCardElevation` takes
//!   the pressed, focused and hovered elevations from the default one (lines 459 to 461): level 0 in all three.
//! - **The disabled elevated card is composited over its own disabled color.** The token says surface at 0.38, and
//!   `defaultElevatedCardColors` composites that over `ElevatedCardTokens.DisabledContainerColor` (lines 558 to 563),
//!   which is surface itself: the disabled elevated card is plain surface. (The filled card's is composited over its
//!   container color, lines 509 to 512, and the outlined card's is its container color, line 609.)
//! - **The disabled outline is composited over the elevated container.** `outlinedCardBorder` takes the disabled
//!   outline color at 0.12 and composites it over `ElevatedCardTokens.ContainerColor` (lines 628 to 630), which is
//!   surface container low, whatever the card it is on.
//!
//! # Not ported
//!
//! - **The clip.** androidx's `Surface` clips the content to the shape. A card here does not: NSH draws a view's shadow
//!   and clips its content on one view, and no host's result with both was measured, so a content that reaches a
//!   corner (an image across the top) is rounded by the app, by [`CardStyle::shape`].
//! - **The dragged elevation** (`DragInteraction`): no NSH state stands for a drag.
//! - **A change of level animates.** It snaps, as the FAB's does: androidx animates it (`animateElevation`, lines 673
//!   to 756), and whether this library follows that animation or the rule that every motion is a spring of the motion
//!   scheme is not decided. androidx's own newer, style-based `Card` has the same open question (line 901:
//!   "TODO(b/554027431): Animate shadowElevation transitions across interaction states when enabled").
//! - **The content colors** (`contentColor`, `disabledContentColor`): the content is the app's own views, which take
//!   their own colors; no color is passed down to them.
//! - **Several states at once.** When several apply, NSH's order of the variants decides: pressed over hovered over
//!   focused. androidx takes the interaction that came last (`interactions.lastOrNull()`, line 715), which the
//!   variants do not record.
//!
//! [ADR 0012]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0012-follow-androidx-components-where-tokens-are-wrong.md

use nsh_core::element::AdaptiveColor;
use nsh_core::geometry::Points;
use nsh_core::prelude::*;

use crate::color::ColorRole;
use crate::elevation::Elevation;
use crate::morph::{Corners, corners};
use crate::tint::role_color;
use crate::tokens::Shape;
use crate::tokens::androidx::{
    elevated_card_tokens, filled_card_tokens, outlined_card_tokens, state_tokens,
};

/// The style of a card: androidx's `Card`, `ElevatedCard` and `OutlinedCard`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PropEnum)]
pub enum CardStyle {
    /// A container in surface container highest, flat at rest: androidx's `Card`, its default (`FilledCardTokens`).
    #[default]
    Filled,
    /// A container in surface container low, raised to level 1 at rest: androidx's `ElevatedCard`
    /// (`ElevatedCardTokens`).
    Elevated,
    /// A container in surface with a 1-point outline in the outline variant color, flat: androidx's `OutlinedCard`
    /// (`OutlinedCardTokens`).
    Outlined,
}

/// What a card is going through: the interactions that change its elevation (androidx's `CardElevation`).
///
/// Only a [`ClickableCard`] has them; a [`Card`] is always at [`CardState::Rest`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum CardState {
    /// Nothing: no pointer over it, no focus, no press.
    #[default]
    Rest,
    /// A pointer is over it (androidx `HoverInteraction.Enter`).
    Hovered,
    /// It has the focus (androidx `FocusInteraction.Focus`).
    Focused,
    /// A press is down on it (androidx `PressInteraction.Press`).
    Pressed,
    /// It takes no press (androidx `enabled = false`).
    Disabled,
}

impl CardStyle {
    /// The shape of the container, which no state changes: `CornerMedium` for every style. An app rounds the content
    /// that reaches a corner of the card with it, as the card does not clip.
    ///
    /// ```
    /// use nsh_m3e::card::CardStyle;
    /// use nsh_m3e::tokens::androidx::shape_tokens;
    ///
    /// assert_eq!(CardStyle::Outlined.shape(), shape_tokens::CORNER_MEDIUM);
    /// ```
    #[must_use]
    pub const fn shape(self) -> Shape {
        match self {
            CardStyle::Filled => filled_card_tokens::CONTAINER_SHAPE,
            CardStyle::Elevated => elevated_card_tokens::CONTAINER_SHAPE,
            CardStyle::Outlined => outlined_card_tokens::CONTAINER_SHAPE,
        }
    }

    /// The role of the scheme the container is in when the card takes presses: surface container highest, surface
    /// container low or surface.
    ///
    /// ```
    /// use nsh_m3e::card::CardStyle;
    /// use nsh_m3e::color::ColorRole;
    ///
    /// assert_eq!(CardStyle::Elevated.container(), ColorRole::SurfaceContainerLow);
    /// ```
    #[must_use]
    pub const fn container(self) -> ColorRole {
        match self {
            CardStyle::Filled => filled_card_tokens::CONTAINER_COLOR,
            CardStyle::Elevated => elevated_card_tokens::CONTAINER_COLOR,
            CardStyle::Outlined => outlined_card_tokens::CONTAINER_COLOR,
        }
    }

    /// The width of the outline: 1 point for the outlined card, none for the others.
    ///
    /// ```
    /// use nsh_m3e::card::CardStyle;
    ///
    /// assert_eq!(CardStyle::Outlined.outline_width().map(|width| width.0), Some(1.0));
    /// assert_eq!(CardStyle::Filled.outline_width(), None);
    /// ```
    #[must_use]
    pub const fn outline_width(self) -> Option<Points> {
        match self {
            CardStyle::Outlined => Some(outlined_card_tokens::OUTLINE_WIDTH),
            CardStyle::Filled | CardStyle::Elevated => None,
        }
    }

    /// How far above the surface the container sits in `state`: the style's tokens, except for the outlined card,
    /// which androidx keeps at its resting elevation hovered, focused and pressed (this module's documentation says
    /// why).
    ///
    /// This is the token's length; [`CardStyle::level`] is the level of the elevation scale it is, which the
    /// container casts the shadow of.
    ///
    /// ```
    /// use nsh_m3e::card::{CardState, CardStyle};
    ///
    /// assert_eq!(CardStyle::Elevated.elevation(CardState::Rest).0, 1.0);
    /// assert_eq!(CardStyle::Elevated.elevation(CardState::Hovered).0, 3.0);
    /// // The outlined card does not rise under the pointer, though its tokens say so.
    /// assert_eq!(CardStyle::Outlined.elevation(CardState::Hovered).0, 0.0);
    /// ```
    #[must_use]
    pub const fn elevation(self, state: CardState) -> Points {
        match (self, state) {
            (CardStyle::Filled, CardState::Rest) => filled_card_tokens::CONTAINER_ELEVATION,
            (CardStyle::Filled, CardState::Hovered) => {
                filled_card_tokens::HOVER_CONTAINER_ELEVATION
            }
            (CardStyle::Filled, CardState::Focused) => {
                filled_card_tokens::FOCUS_CONTAINER_ELEVATION
            }
            (CardStyle::Filled, CardState::Pressed) => {
                filled_card_tokens::PRESSED_CONTAINER_ELEVATION
            }
            (CardStyle::Filled, CardState::Disabled) => {
                filled_card_tokens::DISABLED_CONTAINER_ELEVATION
            }
            (CardStyle::Elevated, CardState::Rest) => elevated_card_tokens::CONTAINER_ELEVATION,
            (CardStyle::Elevated, CardState::Hovered) => {
                elevated_card_tokens::HOVER_CONTAINER_ELEVATION
            }
            (CardStyle::Elevated, CardState::Focused) => {
                elevated_card_tokens::FOCUS_CONTAINER_ELEVATION
            }
            (CardStyle::Elevated, CardState::Pressed) => {
                elevated_card_tokens::PRESSED_CONTAINER_ELEVATION
            }
            (CardStyle::Elevated, CardState::Disabled) => {
                elevated_card_tokens::DISABLED_CONTAINER_ELEVATION
            }
            // `outlinedCardElevation`: `pressedElevation`, `focusedElevation` and `hoveredElevation` default to
            // `defaultElevation` (`Card.kt` lines 459 to 461), not to the tokens' own.
            (
                CardStyle::Outlined,
                CardState::Rest | CardState::Hovered | CardState::Focused | CardState::Pressed,
            ) => outlined_card_tokens::CONTAINER_ELEVATION,
            (CardStyle::Outlined, CardState::Disabled) => {
                outlined_card_tokens::DISABLED_CONTAINER_ELEVATION
            }
        }
    }

    /// The level of the elevation scale the container sits at in `state`: [`CardStyle::elevation`]'s length as a
    /// level, whose [two shadow layers](Elevation::shadows) the container casts.
    ///
    /// The levels are worked out when the crate is compiled, from the same tokens: a token that is none of the six
    /// levels' lengths fails the build, so a card never has a level that is not its token's.
    ///
    /// ```
    /// use nsh_m3e::card::{CardState, CardStyle};
    /// use nsh_m3e::elevation::Elevation;
    ///
    /// assert_eq!(CardStyle::Elevated.level(CardState::Rest), Elevation::Level1);
    /// assert_eq!(CardStyle::Elevated.level(CardState::Hovered), Elevation::Level2);
    /// ```
    #[must_use]
    pub const fn level(self, state: CardState) -> Elevation {
        let levels = match self {
            CardStyle::Filled => FILLED_LEVELS,
            CardStyle::Elevated => ELEVATED_LEVELS,
            CardStyle::Outlined => OUTLINED_LEVELS,
        };
        match state {
            CardState::Rest => levels.rest,
            CardState::Hovered => levels.hovered,
            CardState::Focused => levels.focused,
            CardState::Pressed => levels.pressed,
            CardState::Disabled => levels.disabled,
        }
    }

    /// The container's paint when the card takes presses (`disabled` false) or does not.
    fn container_paint(self, disabled: bool) -> Paint {
        if !disabled {
            return Paint::Solid(self.container());
        }
        match self {
            // `defaultCardColors`: the disabled color at its opacity over the container color (lines 509 to 512).
            CardStyle::Filled => Paint::Faded {
                color: filled_card_tokens::DISABLED_CONTAINER_COLOR,
                opacity: filled_card_tokens::DISABLED_CONTAINER_OPACITY,
                over: filled_card_tokens::CONTAINER_COLOR,
            },
            // `defaultElevatedCardColors`: over `ElevatedCardTokens.DisabledContainerColor`, which is the color
            // itself (lines 558 to 563).
            CardStyle::Elevated => Paint::Faded {
                color: elevated_card_tokens::DISABLED_CONTAINER_COLOR,
                opacity: elevated_card_tokens::DISABLED_CONTAINER_OPACITY,
                over: elevated_card_tokens::DISABLED_CONTAINER_COLOR,
            },
            // `defaultOutlinedCardColors`: the container color, with no opacity (line 609).
            CardStyle::Outlined => Paint::Solid(outlined_card_tokens::CONTAINER_COLOR),
        }
    }

    /// The outline's paint when the card takes presses (`disabled` false) or does not; none for a style with no
    /// outline.
    fn outline_paint(self, disabled: bool) -> Option<Paint> {
        match (self, disabled) {
            (CardStyle::Filled | CardStyle::Elevated, _) => None,
            (CardStyle::Outlined, false) => Some(Paint::Solid(outlined_card_tokens::OUTLINE_COLOR)),
            // `outlinedCardBorder(enabled = false)`: the disabled outline color at its opacity, composited over
            // `ElevatedCardTokens.ContainerColor` (lines 628 to 630).
            (CardStyle::Outlined, true) => Some(Paint::Faded {
                color: outlined_card_tokens::DISABLED_OUTLINE_COLOR,
                opacity: outlined_card_tokens::DISABLED_OUTLINE_OPACITY,
                over: elevated_card_tokens::CONTAINER_COLOR,
            }),
        }
    }
}

/// The level of each state of one style.
#[derive(Debug, Clone, Copy)]
struct Levels {
    rest: Elevation,
    hovered: Elevation,
    focused: Elevation,
    pressed: Elevation,
    disabled: Elevation,
}

/// The levels of `style`, worked out from its tokens.
const fn levels(style: CardStyle) -> Levels {
    Levels {
        rest: level_of(style.elevation(CardState::Rest)),
        hovered: level_of(style.elevation(CardState::Hovered)),
        focused: level_of(style.elevation(CardState::Focused)),
        pressed: level_of(style.elevation(CardState::Pressed)),
        disabled: level_of(style.elevation(CardState::Disabled)),
    }
}

/// The levels of the filled card, worked out when the crate is compiled.
const FILLED_LEVELS: Levels = levels(CardStyle::Filled);

/// The levels of the elevated card, worked out when the crate is compiled.
const ELEVATED_LEVELS: Levels = levels(CardStyle::Elevated);

/// The levels of the outlined card, worked out when the crate is compiled.
const OUTLINED_LEVELS: Levels = levels(CardStyle::Outlined);

/// The level whose length is `points`, exactly: the levels' own lengths are compared bit for bit, as
/// `Elevation::from_points` compares them, so no length is taken for its neighbor.
///
/// Only the constants above call it, so that it runs while the crate is compiled and a length that is none of the
/// six levels' fails the build (the assertion). Called with such a length at run time it would panic: it is private
/// for that reason. The FAB has the same function, private to it.
const fn level_of(points: Points) -> Elevation {
    let bits = points.0.to_bits();
    if bits == Elevation::Level0.points().0.to_bits() {
        Elevation::Level0
    } else if bits == Elevation::Level1.points().0.to_bits() {
        Elevation::Level1
    } else if bits == Elevation::Level2.points().0.to_bits() {
        Elevation::Level2
    } else if bits == Elevation::Level3.points().0.to_bits() {
        Elevation::Level3
    } else if bits == Elevation::Level4.points().0.to_bits() {
        Elevation::Level4
    } else {
        assert!(
            bits == Elevation::Level5.points().0.to_bits(),
            "an elevation token of a card is not a level of the elevation scale"
        );
        Elevation::Level5
    }
}

/// How a color of the card is made: a role as it is, or one composited over another.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Paint {
    /// The role's color, opaque.
    Solid(ColorRole),
    /// `color` at `opacity` composited over `over`: androidx's `.copy(alpha = …).compositeOver(…)`, so the result is
    /// opaque.
    Faded {
        color: ColorRole,
        opacity: f32,
        over: ColorRole,
    },
}

impl Paint {
    /// The color in the app's NSH design tokens (ADR 0008), each appearance composited in its own colors.
    fn color(self) -> AdaptiveColor {
        match self {
            Paint::Solid(role) => role_color(role),
            Paint::Faded {
                color,
                opacity,
                over,
            } => {
                let over = role_color(over);
                let color = role_color(color).opacity(opacity);
                AdaptiveColor::new(over.light.blend(color.light), over.dark.blend(color.dark))
            }
        }
    }
}

/// `style` with the corners that `shape` gives a container: rounded by the shape's radius, or by half the shorter
/// side.
fn rounded<S: Styled>(style: S, shape: Shape) -> S {
    match corners(shape) {
        Corners::Full => style.rounded_full(),
        Corners::Radius(radius) => style.rounded(radius),
    }
}

/// The view that draws a card's outline: over the container and the content, inside the card's edge, taking no place
/// and no touch. A style without an outline gives it no width, so that a card can change its style without changing
/// its structure.
fn outline(style: Prop<CardStyle>, disabled: Prop<bool>) -> impl IntoElement {
    view()
        .absolute()
        .inset(px(0.0))
        .pointer_events(PointerEvents::None)
        .style_with(move |s| rounded(s, style.get().shape()))
        .border(move || style.get().outline_width().unwrap_or(Points::ZERO))
        .border_color(move || {
            style
                .get()
                .outline_paint(disabled.get())
                .map_or(AdaptiveColor::TRANSPARENT, Paint::color)
        })
}

/// The props of a [`Card`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct CardProps {
    /// Its style.
    #[prop(default)]
    pub style: Prop<CardStyle>,
    /// What it holds, laid out in a column.
    pub children: Prop<Children>,
}

/// A Material 3 card with no behavior: a container for its content, as androidx's `Card`, `ElevatedCard` or
/// `OutlinedCard` without an `onClick`.
///
/// It takes no press, no hover and no focus, and animates nothing. Its content is laid out in a column, and it is as
/// large as its content, or as large as the space it is stretched across.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::card::{CardStyle, card};
///
/// fn album() -> impl IntoElement {
///     card()
///         .style(CardStyle::Elevated)
///         .child(text("A Love Supreme"))
///         .child(text("John Coltrane"))
/// }
/// ```
#[component]
pub fn Card(props: CardProps) -> impl IntoElement {
    let style = props.style;
    view()
        .flex_col()
        .bg(move || style.get().container_paint(false).color())
        .style_with(move |s| {
            let style = style.get();
            rounded(s, style.shape()).shadow(style.level(CardState::Rest).shadows())
        })
        .child(props.children)
        .child(outline(style, Prop::new(false)))
}

/// The props of a [`ClickableCard`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Props)]
pub struct ClickableCardProps {
    /// Its style.
    #[prop(default)]
    pub style: Prop<CardStyle>,
    /// No presses: the disabled colors and elevation, and screen readers say it is dimmed.
    #[prop(default)]
    pub disabled: Prop<bool>,
    /// What it holds, laid out in a column.
    pub children: Prop<Children>,
}

/// The events of a [`ClickableCard`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Event)]
pub enum CardEvent {
    /// The card was pressed.
    Press,
}

/// A Material 3 card that takes presses: androidx's `Card`, `ElevatedCard` or `OutlinedCard` with an `onClick`.
///
/// It has the behavior of `nsh-base`'s button (presses, focus, the disabled state, the button role) and the look of
/// the card. Hover, focus and press lay the on-surface color over it at the state layer opacities of `StateTokens`,
/// as androidx's ripple does, and change the level of its shadow (the outlined card's does not change); nothing is
/// laid out again.
///
/// ```no_run
/// use nsh_core::prelude::*;
/// use nsh_m3e::card::{CardStyle, clickable_card};
///
/// fn album() -> impl IntoElement {
///     clickable_card()
///         .style(CardStyle::Outlined)
///         .child(text("A Love Supreme"))
///         .on_press(|| println!("opened"))
/// }
/// ```
#[component]
pub fn ClickableCard(props: ClickableCardProps, cx: Scope<CardEvent>) -> impl IntoElement {
    let style = props.style;
    let disabled = props.disabled;
    // The two shadow layers of the container's level in `state` (ADR 0018). A disabled card is at the disabled
    // level whatever the pointer does.
    let shadows = move |state| {
        let state = if disabled.get() {
            CardState::Disabled
        } else {
            state
        };
        style.get().level(state).shadows()
    };
    nsh_base::button()
        .disabled(move || disabled.get())
        .on_press(move || cx.emit(CardEvent::Press))
        .group()
        .flex_col()
        .bg(move || style.get().container_paint(disabled.get()).color())
        .style_with(move |s| rounded(s, style.get().shape()).shadow(shadows(CardState::Rest)))
        // The shadow of each state the container is in: NSH resolves them natively, with nothing running, and a
        // change of level snaps (this module's documentation says why).
        .hover(move |s| s.style_with(move |s| s.shadow(shadows(CardState::Hovered))))
        .focus_within(move |s| s.style_with(move |s| s.shadow(shadows(CardState::Focused))))
        .active(move |s| s.style_with(move |s| s.shadow(shadows(CardState::Pressed))))
        .child(props.children)
        .child(
            // The state layer: the on-surface color over the container, in its shape, shown by the button's own
            // states, over the content as the ripple is. A disabled card takes no press and shows no state.
            view()
                .absolute()
                .inset(px(0.0))
                .pointer_events(PointerEvents::None)
                .bg(role_color(ColorRole::OnSurface))
                .opacity(0.0)
                .group_hover(|s| s.opacity(state_tokens::HOVER_STATE_LAYER_OPACITY))
                .group_focus_within(|s| s.opacity(state_tokens::FOCUS_STATE_LAYER_OPACITY))
                .group_active(|s| s.opacity(state_tokens::PRESSED_STATE_LAYER_OPACITY))
                .style_with(move |s| {
                    let s = rounded(s, style.get().shape());
                    if disabled.get() { s.hidden() } else { s }
                }),
        )
        .child(outline(style, disabled))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::SystemTime;

    use nsh_core::geometry::{Bounds, Point, Size};
    use nsh_core::platform::{ColorScheme as Appearance, Environment, Os};
    use nsh_core::{AccessibilityRole, BoxShadow, Color, MountItem, NodeId, ViewKind, ViewStyle};
    use nsh_engine::{Clock, EngineConfig, Root};
    use nsh_testing::Headless;

    use super::*;
    use crate::animation_probe::Recording;
    use crate::color::ColorScheme;

    const STYLES: [CardStyle; 3] = [CardStyle::Filled, CardStyle::Elevated, CardStyle::Outlined];

    const STATES: [CardState; 5] = [
        CardState::Rest,
        CardState::Hovered,
        CardState::Focused,
        CardState::Pressed,
        CardState::Disabled,
    ];

    /// What the probe places: one card, a clickable one or not, with two lines of text in it, and a line counting its
    /// presses.
    #[derive(Debug, Clone, PartialEq, Props)]
    struct ProbeProps {
        #[prop(default)]
        style: Prop<CardStyle>,
        #[prop(default)]
        clickable: Prop<bool>,
        #[prop(default)]
        disabled: Prop<bool>,
    }

    #[component]
    fn Probe(props: ProbeProps) -> impl IntoElement {
        let presses = signal(0_u32);
        // A row, so that the card is as wide as its content, not stretched across a column.
        // The test id goes on the component's own view: `.0`.
        let card = if props.clickable.get_untracked() {
            clickable_card()
                .style(props.style)
                .disabled(props.disabled)
                .on_press(move || presses.update(|n| *n += 1))
                .child(text("Album"))
                .child(text("Artist"))
                .0
                .test_id("card")
                .into_element()
        } else {
            card()
                .style(props.style)
                .child(text("Album"))
                .child(text("Artist"))
                .0
                .test_id("card")
                .into_element()
        };
        view()
            .child(view().flex_row().child(card))
            .child(text(move || format!("presses: {}", presses.get())))
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct Config {
        style: CardStyle,
        clickable: bool,
        disabled: bool,
        dark: bool,
    }

    fn start(config: Config) -> (Headless, Recording) {
        let root = Root::new::<Probe>(move || ProbeProps {
            style: Prop::new(config.style),
            clickable: Prop::new(config.clickable),
            disabled: Prop::new(config.disabled),
        });
        // The web has a mouse to hover with, whatever its idiom.
        let environment = Environment {
            os: Os::Web,
            color_scheme: if config.dark {
                Appearance::Dark
            } else {
                Appearance::Light
            },
            ..Environment::default()
        };
        let mut app = Headless::start(EngineConfig {
            clock: Clock::manual(SystemTime::UNIX_EPOCH),
            environment,
            ..EngineConfig::new(root)
        })
        .unwrap();
        let recording = Recording::attach(&mut app);
        app.wait_for("text:presses: 0").unwrap();
        (app, recording)
    }

    /// The card's views, and the last style, frame and props each was sent.
    struct Mounted {
        /// The container, which has the test id `card` (and the button role when the card is clickable).
        container: NodeId,
        /// What is in it, in order: the two lines of text, then the state layer if the card is clickable, then the
        /// outline.
        inside: Vec<NodeId>,
        kinds: HashMap<NodeId, ViewKind>,
        styles: HashMap<NodeId, ViewStyle>,
        frames: HashMap<NodeId, Bounds<Points>>,
        props: HashMap<NodeId, nsh_core::ViewProps>,
    }

    impl Mounted {
        fn of(items: &[MountItem]) -> Self {
            let mut kinds = HashMap::new();
            let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let mut container = None;
            let mut styles = HashMap::new();
            let mut frames = HashMap::new();
            let mut props = HashMap::new();
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
                        if sent.test_id.as_deref() == Some("card") {
                            container = Some(*id);
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
            let container = container.expect("a view with the test id `card`");
            let inside = children.get(&container).cloned().unwrap_or_default();
            Self {
                container,
                inside,
                kinds,
                styles,
                frames,
                props,
            }
        }

        /// The last style `id` was sent; the default one if it was sent none.
        fn style(&self, id: NodeId) -> ViewStyle {
            self.styles.get(&id).cloned().unwrap_or_default()
        }

        fn role(&self) -> AccessibilityRole {
            self.props[&self.container].accessibility.role
        }

        fn disabled(&self) -> bool {
            self.props[&self.container].accessibility.state.disabled
        }

        /// The outline's view, the last of the container's.
        fn outline(&self) -> NodeId {
            *self.inside.last().expect("an outline")
        }

        /// The state layer's view, between the content and the outline of a clickable card.
        fn layer(&self) -> NodeId {
            let [_, _, layer, _] = self.inside[..] else {
                panic!(
                    "a text, a text, a state layer and an outline: {:?}",
                    self.inside
                );
            };
            layer
        }

        fn frame(&self, id: NodeId) -> Bounds<Points> {
            self.frames[&id]
        }
    }

    fn mount(config: Config) -> Mounted {
        let (_app, recording) = start(config);
        Mounted::of(&recording.items())
    }

    /// NSH's color of `hex` (`0xRRGGBB`) at alpha `alpha` (0 to 255).
    fn color(hex: u32, alpha: u32) -> Color {
        Color((hex << 8) | alpha)
    }

    fn scheme(dark: bool) -> ColorScheme {
        if dark {
            ColorScheme::BASELINE_DARK
        } else {
            ColorScheme::BASELINE_LIGHT
        }
    }

    /// The opaque color of `role` in the baseline scheme.
    fn solid(role: ColorRole, dark: bool) -> Color {
        color(scheme(dark).get(role).to_hex(), 255)
    }

    /// Whether `a` and `b` are the same color but for a unit in a channel: the compositing rounds its own way.
    fn near(a: Color, b: Color) -> bool {
        let (a, b) = (a.0.to_be_bytes(), b.0.to_be_bytes());
        a.iter().zip(b).all(|(a, b)| a.abs_diff(b) <= 1)
    }

    /// `top` at `percent` percent over `base`, as androidx's `copy(alpha).compositeOver(base)` computes it, channel by
    /// channel in sRGB and rounded to the nearest.
    fn composited(top: ColorRole, percent: u32, base: ColorRole, dark: bool) -> Color {
        let (top, base) = (
            scheme(dark).get(top).to_hex(),
            scheme(dark).get(base).to_hex(),
        );
        let channel = |shift: u32| {
            let (top, base) = ((top >> shift) & 0xff, (base >> shift) & 0xff);
            (top * percent + base * (100 - percent) + 50) / 100
        };
        color((channel(16) << 16) | (channel(8) << 8) | channel(0), 255)
    }

    /// The box shadows `style` has, top layer first.
    fn shadows_of(style: &ViewStyle) -> Vec<BoxShadow> {
        style
            .effects
            .as_deref()
            .map(|effects| effects.box_shadows.clone())
            .unwrap_or_default()
    }

    /// The two layers of `level`, as a list.
    fn layers(level: Elevation) -> Vec<BoxShadow> {
        level.shadows().to_vec()
    }

    /// Every style and state: androidx's elevation in points. `ElevationTokens`: `Level0` is 0, `Level1` 1, `Level2`
    /// 3. `Card.kt`'s `outlinedCardElevation` keeps the outlined card at its resting one hovered, focused and pressed.
    const ELEVATIONS: [(CardStyle, [f32; 5]); 3] = [
        (CardStyle::Filled, [0.0, 1.0, 0.0, 0.0, 0.0]),
        (CardStyle::Elevated, [1.0, 3.0, 1.0, 1.0, 1.0]),
        (CardStyle::Outlined, [0.0, 0.0, 0.0, 0.0, 0.0]),
    ];

    /// The levels those are.
    const LEVELS: [(CardStyle, [Elevation; 5]); 3] = [
        (
            CardStyle::Filled,
            [
                Elevation::Level0,
                Elevation::Level1,
                Elevation::Level0,
                Elevation::Level0,
                Elevation::Level0,
            ],
        ),
        (
            CardStyle::Elevated,
            [
                Elevation::Level1,
                Elevation::Level2,
                Elevation::Level1,
                Elevation::Level1,
                Elevation::Level1,
            ],
        ),
        (
            CardStyle::Outlined,
            [
                Elevation::Level0,
                Elevation::Level0,
                Elevation::Level0,
                Elevation::Level0,
                Elevation::Level0,
            ],
        ),
    ];

    #[test]
    fn the_defaults_are_androidxs_card() {
        // androidx's `Card` with no `Elevated` or `Outlined` in its name is the filled one.
        assert_eq!(CardStyle::default(), CardStyle::Filled);
        assert_eq!(CardState::default(), CardState::Rest);
    }

    #[test]
    fn each_style_has_its_container_color_shape_and_outline_from_its_tokens() {
        for (style, container, outline) in [
            (CardStyle::Filled, ColorRole::SurfaceContainerHighest, None),
            (CardStyle::Elevated, ColorRole::SurfaceContainerLow, None),
            (CardStyle::Outlined, ColorRole::Surface, Some(1.0)),
        ] {
            assert_eq!(style.container(), container, "{style:?}");
            assert_eq!(
                style.shape(),
                crate::tokens::androidx::shape_tokens::CORNER_MEDIUM
            );
            assert_eq!(
                style.outline_width().map(|width| width.0),
                outline,
                "{style:?}"
            );
            assert_eq!(
                Corners::of(style.shape()),
                Some(Corners::Radius(Points(12.0))),
                "{style:?}"
            );
        }
    }

    #[test]
    fn each_style_and_state_has_its_elevation_and_level() {
        for ((style, points), (_, levels)) in ELEVATIONS.into_iter().zip(LEVELS) {
            for ((state, points), level) in STATES.into_iter().zip(points).zip(levels) {
                let what = format!("{style:?} {state:?}");
                assert_eq!(style.elevation(state).0, points, "{what}");
                assert_eq!(style.level(state), level, "{what}");
                // The level is the one of the length, and the length is the level's.
                assert_eq!(
                    Elevation::from_points(style.elevation(state)),
                    Some(level),
                    "{what}"
                );
            }
        }
    }

    #[test]
    fn only_the_outlined_card_ignores_its_tokens_elevation_and_androidx_says_so() {
        // The outlined card's tokens raise it one level when hovered, and the other two's do the same.
        assert_eq!(
            outlined_card_tokens::HOVER_CONTAINER_ELEVATION,
            Elevation::Level1.points()
        );
        assert_ne!(
            CardStyle::Outlined.elevation(CardState::Hovered),
            outlined_card_tokens::HOVER_CONTAINER_ELEVATION
        );
        for (style, hovered) in [
            (
                CardStyle::Filled,
                filled_card_tokens::HOVER_CONTAINER_ELEVATION,
            ),
            (
                CardStyle::Elevated,
                elevated_card_tokens::HOVER_CONTAINER_ELEVATION,
            ),
        ] {
            assert_eq!(style.elevation(CardState::Hovered), hovered, "{style:?}");
        }
        // Focused and pressed are the same in the tokens and in androidx, for all three.
        for (style, focused, pressed) in [
            (
                CardStyle::Filled,
                filled_card_tokens::FOCUS_CONTAINER_ELEVATION,
                filled_card_tokens::PRESSED_CONTAINER_ELEVATION,
            ),
            (
                CardStyle::Elevated,
                elevated_card_tokens::FOCUS_CONTAINER_ELEVATION,
                elevated_card_tokens::PRESSED_CONTAINER_ELEVATION,
            ),
            (
                CardStyle::Outlined,
                outlined_card_tokens::FOCUS_CONTAINER_ELEVATION,
                outlined_card_tokens::PRESSED_CONTAINER_ELEVATION,
            ),
        ] {
            assert_eq!(style.elevation(CardState::Focused), focused, "{style:?}");
            assert_eq!(style.elevation(CardState::Pressed), pressed, "{style:?}");
        }
    }

    /// The vendored `Card.kt`, at the pinned commit (ADR 0012).
    const CARD_KT: &str = include_str!("../../../reference/androidx-material3-components/Card.kt");

    /// Whether `file` holds `lines`, each trimmed, one after the other.
    fn cites(file: &str, lines: &[&str]) -> bool {
        let file: Vec<&str> = file.lines().map(str::trim).collect();
        file.windows(lines.len()).any(|window| window == lines)
    }

    #[test]
    fn the_overrides_are_what_card_kt_says() {
        // `outlinedCardElevation`: the three interactions default to the default elevation, where the other two
        // cards' default to their tokens.
        assert!(cites(
            CARD_KT,
            &[
                "public fun outlinedCardElevation(",
                "defaultElevation: Dp = OutlinedCardTokens.ContainerElevation,",
                "pressedElevation: Dp = defaultElevation,",
                "focusedElevation: Dp = defaultElevation,",
                "hoveredElevation: Dp = defaultElevation,",
                "draggedElevation: Dp = OutlinedCardTokens.DraggedContainerElevation,",
                "disabledElevation: Dp = OutlinedCardTokens.DisabledContainerElevation,",
            ]
        ));
        assert!(cites(
            CARD_KT,
            &[
                "public fun cardElevation(",
                "defaultElevation: Dp = FilledCardTokens.ContainerElevation,",
                "pressedElevation: Dp = FilledCardTokens.PressedContainerElevation,",
                "focusedElevation: Dp = FilledCardTokens.FocusContainerElevation,",
                "hoveredElevation: Dp = FilledCardTokens.HoverContainerElevation,",
                "draggedElevation: Dp = FilledCardTokens.DraggedContainerElevation,",
                "disabledElevation: Dp = FilledCardTokens.DisabledContainerElevation,",
            ]
        ));
        assert!(cites(
            CARD_KT,
            &[
                "public fun elevatedCardElevation(",
                "defaultElevation: Dp = ElevatedCardTokens.ContainerElevation,",
                "pressedElevation: Dp = ElevatedCardTokens.PressedContainerElevation,",
                "focusedElevation: Dp = ElevatedCardTokens.FocusContainerElevation,",
                "hoveredElevation: Dp = ElevatedCardTokens.HoverContainerElevation,",
                "draggedElevation: Dp = ElevatedCardTokens.DraggedContainerElevation,",
                "disabledElevation: Dp = ElevatedCardTokens.DisabledContainerElevation,",
            ]
        ));
        // The disabled containers: the filled one over its own color, the elevated one over its disabled color (which
        // is the color it fades), the outlined one as it is.
        assert!(cites(
            CARD_KT,
            &[
                "disabledContainerColor =",
                "fromToken(FilledCardTokens.DisabledContainerColor)",
                ".copy(alpha = FilledCardTokens.DisabledContainerOpacity)",
                ".compositeOver(fromToken(FilledCardTokens.ContainerColor)),",
            ]
        ));
        assert!(cites(
            CARD_KT,
            &[
                "disabledContainerColor =",
                "fromToken(ElevatedCardTokens.DisabledContainerColor)",
                ".copy(alpha = ElevatedCardTokens.DisabledContainerOpacity)",
                ".compositeOver(",
                "fromToken(ElevatedCardTokens.DisabledContainerColor)",
                "),",
            ]
        ));
        assert!(cites(
            CARD_KT,
            &["disabledContainerColor = fromToken(OutlinedCardTokens.ContainerColor),"]
        ));
        // The disabled outline, over the elevated card's container whatever the card.
        assert!(cites(
            CARD_KT,
            &[
                "OutlinedCardTokens.DisabledOutlineColor.value",
                ".copy(alpha = OutlinedCardTokens.DisabledOutlineOpacity)",
                ".compositeOver(ElevatedCardTokens.ContainerColor.value)",
            ]
        ));
        // A line that is not there is not found.
        assert!(!cites(
            CARD_KT,
            &["hoveredElevation: Dp = OutlinedCardTokens.HoverContainerElevation,"]
        ));
    }

    #[test]
    fn the_roles_and_opacities_the_disabled_paints_name_are_the_tokens() {
        assert_eq!(
            CardStyle::Filled.container_paint(true),
            Paint::Faded {
                color: ColorRole::SurfaceVariant,
                opacity: 0.38,
                over: ColorRole::SurfaceContainerHighest,
            }
        );
        // The elevated card fades surface over surface.
        assert_eq!(
            CardStyle::Elevated.container_paint(true),
            Paint::Faded {
                color: ColorRole::Surface,
                opacity: 0.38,
                over: ColorRole::Surface,
            }
        );
        assert_eq!(
            CardStyle::Outlined.container_paint(true),
            Paint::Solid(ColorRole::Surface)
        );
        assert_eq!(
            CardStyle::Outlined.outline_paint(true),
            Some(Paint::Faded {
                color: ColorRole::Outline,
                opacity: 0.12,
                over: ColorRole::SurfaceContainerLow,
            })
        );
        // Enabled, a paint is its role as it is, and the styles without an outline have none, enabled or not.
        for style in STYLES {
            assert_eq!(
                style.container_paint(false),
                Paint::Solid(style.container()),
                "{style:?}"
            );
        }
        assert_eq!(
            CardStyle::Outlined.outline_paint(false),
            Some(Paint::Solid(ColorRole::OutlineVariant))
        );
        for style in [CardStyle::Filled, CardStyle::Elevated] {
            assert_eq!(style.outline_paint(false), None, "{style:?}");
            assert_eq!(style.outline_paint(true), None, "{style:?}");
        }
    }

    #[test]
    fn every_container_has_its_color_shape_and_no_border_of_its_own() {
        for clickable in [false, true] {
            for style in STYLES {
                for dark in [false, true] {
                    let what = format!("{style:?} clickable: {clickable} dark: {dark}");
                    let mounted = mount(Config {
                        style,
                        clickable,
                        dark,
                        ..Config::default()
                    });
                    let container = mounted.style(mounted.container);
                    assert_eq!(
                        container.background_color,
                        solid(style.container(), dark),
                        "{what}"
                    );
                    assert_eq!(container.border_radius.0, 12.0, "{what}");
                    // No corner of its own: the four are the same.
                    assert_eq!(
                        container.corner_radius,
                        nsh_core::Corners::default(),
                        "{what}"
                    );
                    // Its outline is a view of its own, not its border, so that it takes no room.
                    assert!(!container.borders.is_visible(), "{what}");
                    assert_eq!(container.borders.widths.top, Points::ZERO, "{what}");
                }
            }
        }
    }

    #[test]
    fn the_outline_is_a_view_over_the_content_that_takes_no_room_and_no_touch() {
        for clickable in [false, true] {
            for style in STYLES {
                for dark in [false, true] {
                    let what = format!("{style:?} clickable: {clickable} dark: {dark}");
                    let mounted = mount(Config {
                        style,
                        clickable,
                        dark,
                        ..Config::default()
                    });
                    let outline = mounted.outline();
                    let sent = mounted.style(outline);
                    let (width, outline_color) = match style {
                        CardStyle::Outlined => (1.0, solid(ColorRole::OutlineVariant, dark)),
                        _ => (0.0, Color(0)),
                    };
                    for side in sent.borders.widths.to_array() {
                        assert_eq!(side.0, width, "{what}");
                    }
                    if width > 0.0 {
                        assert_eq!(sent.borders.colors, [outline_color; 4], "{what}");
                    }
                    // The outline is as round as the container.
                    assert_eq!(sent.border_radius.0, 12.0, "{what}");
                    // It covers the container exactly, from its corner, and takes no touch.
                    let container = mounted.frame(mounted.container);
                    let frame = mounted.frame(outline);
                    assert_eq!((frame.origin.x.0, frame.origin.y.0), (0.0, 0.0), "{what}");
                    assert_eq!(frame.size, container.size, "{what}");
                    assert_eq!(
                        mounted.props[&outline].pointer_events,
                        PointerEvents::None,
                        "{what}"
                    );
                    // Nothing in it: it is drawn over the content.
                    assert_eq!(mounted.kinds[&outline], ViewKind::View, "{what}");
                }
            }
        }
    }

    #[test]
    fn a_card_is_as_large_as_its_content_in_every_style_and_form() {
        let sizes: Vec<Size<Points>> = [false, true]
            .into_iter()
            .flat_map(|clickable| STYLES.into_iter().map(move |style| (clickable, style)))
            .map(|(clickable, style)| {
                let mounted = mount(Config {
                    style,
                    clickable,
                    ..Config::default()
                });
                mounted.frame(mounted.container).size
            })
            .collect();
        assert!(
            sizes[0].width.0 > 0.0 && sizes[0].height.0 > 0.0,
            "{sizes:?}"
        );
        // The outline adds no size, and neither does the state layer.
        assert!(sizes.iter().all(|size| *size == sizes[0]), "{sizes:?}");
    }

    #[test]
    fn the_content_is_laid_out_in_a_column() {
        for clickable in [false, true] {
            let mounted = mount(Config {
                clickable,
                ..Config::default()
            });
            let [first, second] = mounted.inside[..2] else {
                panic!("two texts: {:?}", mounted.inside);
            };
            let (first, second) = (mounted.frame(first), mounted.frame(second));
            // The second line is under the first, at the same left edge.
            assert_eq!(first.origin.x, second.origin.x, "clickable: {clickable}");
            assert!(
                second.origin.y.0 >= first.origin.y.0 + first.size.height.0,
                "clickable: {clickable}: {first:?} {second:?}"
            );
        }
    }

    #[test]
    fn a_card_has_no_role_and_a_clickable_one_is_a_button() {
        for style in STYLES {
            let card = mount(Config {
                style,
                ..Config::default()
            });
            assert_ne!(card.role(), AccessibilityRole::Button, "{style:?}");
            assert!(!card.disabled(), "{style:?}");
            let clickable = mount(Config {
                style,
                clickable: true,
                ..Config::default()
            });
            assert_eq!(clickable.role(), AccessibilityRole::Button, "{style:?}");
            assert!(!clickable.disabled(), "{style:?}");
            let disabled = mount(Config {
                style,
                clickable: true,
                disabled: true,
                ..Config::default()
            });
            assert_eq!(disabled.role(), AccessibilityRole::Button, "{style:?}");
            assert!(disabled.disabled(), "{style:?}");
        }
    }

    #[test]
    fn a_press_is_reported_once_and_a_disabled_card_takes_none() -> Result<(), String> {
        let (mut app, _recording) = start(Config {
            clickable: true,
            ..Config::default()
        });
        app.press("id:card")?;
        assert!(app.shows("presses: 1"));
        assert!(!app.shows("presses: 2"));
        let (mut app, _recording) = start(Config {
            clickable: true,
            disabled: true,
            ..Config::default()
        });
        let _ = app.press("id:card");
        assert!(app.shows("presses: 0"));
        Ok(())
    }

    #[test]
    fn a_disabled_card_has_the_disabled_colors() {
        for dark in [false, true] {
            for style in STYLES {
                let what = format!("{style:?} dark: {dark}");
                let mounted = mount(Config {
                    style,
                    clickable: true,
                    disabled: true,
                    dark,
                });
                let container = mounted.style(mounted.container).background_color;
                let expected = match style {
                    CardStyle::Filled => composited(
                        ColorRole::SurfaceVariant,
                        38,
                        ColorRole::SurfaceContainerHighest,
                        dark,
                    ),
                    // The elevated card fades surface over surface, and the outlined one is surface.
                    _ => solid(ColorRole::Surface, dark),
                };
                assert!(
                    near(container, expected),
                    "{what}: {container:?} {expected:?}"
                );
                // Opaque: a disabled card does not show what is under it.
                assert_eq!(container.0 & 0xff, 255, "{what}");
                let outline = mounted.style(mounted.outline()).borders;
                if style == CardStyle::Outlined {
                    let expected =
                        composited(ColorRole::Outline, 12, ColorRole::SurfaceContainerLow, dark);
                    assert!(
                        near(outline.colors[0], expected),
                        "{what}: {:?}",
                        outline.colors
                    );
                    assert_eq!(outline.colors[0].0 & 0xff, 255, "{what}");
                    assert_eq!(outline.widths.top.0, 1.0, "{what}");
                } else {
                    assert_eq!(outline.widths.top, Points::ZERO, "{what}");
                }
            }
        }
        // The same enabled card is not dimmed.
        let enabled = mount(Config {
            style: CardStyle::Filled,
            clickable: true,
            ..Config::default()
        });
        assert_eq!(
            enabled.style(enabled.container).background_color,
            solid(ColorRole::SurfaceContainerHighest, false)
        );
    }

    #[test]
    fn the_container_rests_at_the_styles_level_with_its_two_shadow_layers() {
        for (style, levels) in LEVELS {
            for clickable in [false, true] {
                let mounted = mount(Config {
                    style,
                    clickable,
                    ..Config::default()
                });
                assert_eq!(
                    shadows_of(&mounted.style(mounted.container)),
                    layers(levels[0]),
                    "{style:?} clickable: {clickable}"
                );
                // Only the container casts a shadow: the content, the state layer and the outline have none.
                for id in &mounted.inside {
                    assert_eq!(
                        shadows_of(&mounted.style(*id)),
                        Vec::new(),
                        "{style:?} {id:?}"
                    );
                }
            }
        }
        // The elevated card is raised, the others are flat: their shadows differ, and each has two layers.
        let raised = layers(Elevation::Level1);
        let flat = layers(Elevation::Level0);
        assert_ne!(raised, flat);
        assert_eq!(raised.len(), 2);
    }

    /// The shadow the container has once the platform reports `states` for it (bits of
    /// `nsh_core::tailwind::state`), as it does a hover, a focus or a press.
    fn shadows_in(app: &mut Headless, recording: &Recording, states: u16) -> Vec<BoxShadow> {
        let container = Mounted::of(&recording.items()).container;
        app.engine().control().set_states(container, states);
        assert!(app.settle());
        shadows_of(&Mounted::of(&recording.items()).style(container))
    }

    #[test]
    fn the_state_the_platform_reports_decides_the_level() {
        use nsh_core::tailwind::state::{FOCUSED, HOVERED, PRESSED};
        for (style, levels) in LEVELS {
            let (mut app, recording) = start(Config {
                style,
                clickable: true,
                ..Config::default()
            });
            let [rest, hovered, focused, pressed, _] = levels.map(layers);
            let what = |states: &str| format!("{style:?} {states}");
            // The headless runner does not apply a press to the styles, so the platform's report is sent to the
            // engine as a platform does it.
            assert_eq!(
                shadows_in(&mut app, &recording, HOVERED),
                hovered,
                "{}",
                what("hovered")
            );
            // A press that goes down on a hovered card is at the pressed level; the platform keeps reporting the
            // hover while the pointer is down.
            assert_eq!(
                shadows_in(&mut app, &recording, HOVERED | PRESSED),
                pressed,
                "{}",
                what("hovered and pressed")
            );
            assert_eq!(
                shadows_in(&mut app, &recording, PRESSED),
                pressed,
                "{}",
                what("pressed")
            );
            assert_eq!(
                shadows_in(&mut app, &recording, FOCUSED),
                focused,
                "{}",
                what("focused")
            );
            // NSH orders the hover's variant after the focus's, and the press's after both.
            assert_eq!(
                shadows_in(&mut app, &recording, HOVERED | FOCUSED),
                hovered,
                "{}",
                what("hovered and focused")
            );
            assert_eq!(
                shadows_in(&mut app, &recording, FOCUSED | PRESSED),
                pressed,
                "{}",
                what("focused and pressed")
            );
            assert_eq!(
                shadows_in(&mut app, &recording, 0),
                rest,
                "{}",
                what("at rest")
            );
        }
        // The elevated card is the one that tells a hover from a rest, and the filled one a press from a hover: the
        // levels are not all one.
        assert_ne!(layers(Elevation::Level2), layers(Elevation::Level1));
        assert_ne!(layers(Elevation::Level1), layers(Elevation::Level0));
    }

    #[test]
    fn a_disabled_card_stays_at_its_disabled_level_whatever_the_pointer_does() {
        use nsh_core::tailwind::state::{FOCUSED, HOVERED, PRESSED};
        let (mut app, recording) = start(Config {
            style: CardStyle::Elevated,
            clickable: true,
            disabled: true,
            ..Config::default()
        });
        let disabled = layers(CardStyle::Elevated.level(CardState::Disabled));
        // Hovered, an enabled elevated card is at level 2; this one stays at its disabled level 1.
        assert_ne!(disabled, layers(Elevation::Level2));
        for states in [0, HOVERED, FOCUSED, PRESSED, HOVERED | PRESSED] {
            assert_eq!(
                shadows_in(&mut app, &recording, states),
                disabled,
                "{states:#b}"
            );
        }
    }

    /// The opacities the state layer was sent in `items`, in order.
    fn layer_opacities(mounted: &Mounted, items: &[MountItem]) -> Vec<f32> {
        items
            .iter()
            .filter_map(|item| match item {
                MountItem::ViewStyle { id, style } if *id == mounted.layer() => Some(style.opacity),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_state_layer_is_on_surface_over_the_content_with_the_states_opacities()
    -> Result<(), String> {
        for dark in [false, true] {
            let (mut app, recording) = start(Config {
                clickable: true,
                dark,
                ..Config::default()
            });
            let mounted = Mounted::of(&recording.items());
            let layer = mounted.layer();
            let sent = mounted.style(layer);
            // On-surface, from its corner to its corner, as round as the container, and shown by nothing at rest.
            assert_eq!(sent.background_color, solid(ColorRole::OnSurface, dark));
            assert_eq!(sent.opacity, 0.0);
            assert_eq!(sent.border_radius.0, 12.0);
            assert_eq!(
                mounted.frame(layer),
                Bounds {
                    origin: Point::default(),
                    size: mounted.frame(mounted.container).size,
                }
            );
            // It takes no touch, so that what the card holds still gets its presses.
            assert_eq!(mounted.props[&layer].pointer_events, PointerEvents::None);
            // It is between the content and the outline, so over the content.
            assert_eq!(mounted.inside.len(), 4);
            // Hover and focus show it at `StateTokens`' opacities; a pointer that leaves takes it away.
            app.hover("id:card")?;
            assert_eq!(
                layer_opacities(&mounted, &recording.in_step("hover(id:card)")),
                [0.08]
            );
            app.step("leave");
            app.hover("text:presses: 0")?;
            assert_eq!(
                layer_opacities(&mounted, &recording.in_step("hover(text:presses: 0)")),
                [0.0]
            );
            app.press_key("Tab")?;
            assert_eq!(
                layer_opacities(&mounted, &recording.in_step("keyboard.press(Tab)")),
                [0.1]
            );
        }
        let bits = |opacity: f32| opacity.to_bits();
        assert_eq!(bits(state_tokens::HOVER_STATE_LAYER_OPACITY), bits(0.08));
        assert_eq!(bits(state_tokens::FOCUS_STATE_LAYER_OPACITY), bits(0.1));
        assert_eq!(bits(state_tokens::PRESSED_STATE_LAYER_OPACITY), bits(0.1));
        Ok(())
    }

    #[test]
    fn a_disabled_card_shows_no_state_layer() {
        let mounted = mount(Config {
            clickable: true,
            disabled: true,
            ..Config::default()
        });
        // `hidden` is CSS's `display: none`: the layer has no frame to show a state in.
        assert_eq!(
            mounted.frames.get(&mounted.layer()).map(|frame| frame.size),
            Some(Size::default())
        );
        let enabled = mount(Config {
            clickable: true,
            ..Config::default()
        });
        assert_ne!(
            enabled.frames.get(&enabled.layer()).map(|frame| frame.size),
            Some(Size::default())
        );
    }

    #[test]
    fn a_static_card_has_no_state_layer_and_takes_no_hover() -> Result<(), String> {
        let (mut app, recording) = start(Config::default());
        let mounted = Mounted::of(&recording.items());
        // The two lines of text, and the outline.
        assert_eq!(mounted.inside.len(), 3);
        let before = shadows_of(&mounted.style(mounted.container));
        app.hover("id:card")?;
        assert_eq!(
            shadows_of(&Mounted::of(&recording.items()).style(mounted.container)),
            before
        );
        Ok(())
    }

    #[test]
    fn an_elevation_token_that_is_not_a_level_cannot_be_taken_for_one() {
        // The levels' own lengths are levels; anything else would fail the build when a constant got it: called at
        // run time it panics. Between two levels, a hair off one, negative, and not a number: none is rounded to a
        // level.
        for level in [
            Elevation::Level0,
            Elevation::Level1,
            Elevation::Level2,
            Elevation::Level3,
            Elevation::Level4,
            Elevation::Level5,
        ] {
            assert_eq!(level_of(level.points()), level);
        }
        for points in [7.0, 6.001, 5.999, 13.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(
                std::panic::catch_unwind(|| level_of(Points(points))).is_err(),
                "{points}"
            );
        }
    }
}
