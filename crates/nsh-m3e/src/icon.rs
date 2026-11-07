//! Icons: one shape of SVG path data, drawn in one color ([ADR 0017]).
//!
//! An [`Icon`] is the path data and the view box it is drawn in. The components that show one draw it on a canvas at
//! their token size and fill it with their token color, the same on every host. The library ships no icon set: the
//! app gives its own, for example Material Symbols, whose SVG files each hold one path in a 960-point view box
//! ([`Icon::material_symbol`]).
//!
//! [ADR 0017]: https://github.com/Vehicoule/nsh-m3e/blob/main/docs/adr/0017-draw-icons-from-path-data.md

use nsh_core::SvgPaint;
use nsh_core::geometry::{Bounds, Point, Points, Size};
use nsh_core::prelude::*;

/// The view box of every Material Symbols SVG file, `viewBox="0 -960 960 960"`: its paths are drawn in the 960 points
/// above the baseline. It comes from those files (google/material-design-icons), not from a token.
const MATERIAL_SYMBOL_VIEW_BOX: Bounds<Points> = Bounds {
    origin: Point {
        x: Points(0.0),
        y: Points(-960.0),
    },
    size: Size {
        width: Points(960.0),
        height: Points(960.0),
    },
};

/// An icon: SVG path data (an SVG `<path>`'s `d`) and the view box it is drawn in.
///
/// It is filled with the color of the component that shows it, by the nonzero rule, as SVG fills a path. An icon
/// with no path draws nothing.
///
/// ```
/// use nsh_m3e::icon::Icon;
///
/// // Material Symbols' `add`, from its SVG file.
/// let add = Icon::material_symbol("M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z");
/// assert_eq!(add.view_box().size.width.0, 960.0);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct Icon {
    /// The SVG path data.
    path: SharedString,
    /// The view box the path is drawn in.
    view_box: Bounds<Points>,
}

impl Default for Icon {
    /// No path, in the view box of Material Symbols.
    fn default() -> Self {
        Self::material_symbol("")
    }
}

impl Icon {
    /// The icon `path` draws in `view_box`, which is scaled to the icon's size, its aspect ratio kept and centered, as
    /// SVG's default `preserveAspectRatio` fits a view box.
    pub fn new(path: impl Into<SharedString>, view_box: Bounds<Points>) -> Self {
        Self {
            path: path.into(),
            view_box,
        }
    }

    /// A Material Symbols icon: the `d` of the one `<path>` in its SVG file, in the view box all those files share.
    pub fn material_symbol(path: impl Into<SharedString>) -> Self {
        Self::new(path, MATERIAL_SYMBOL_VIEW_BOX)
    }

    /// Its SVG path data.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The view box its path is drawn in.
    #[must_use]
    pub fn view_box(&self) -> Bounds<Points> {
        self.view_box
    }

    /// The transform that fits its view box into a square of `side`: the scale, then the offset after it. A view box
    /// with no area fits nowhere: its scale is 0.
    fn fit(&self, side: Points) -> (f32, f32, f32) {
        let Bounds { origin, size } = self.view_box;
        let (width, height) = (size.width.0, size.height.0);
        if !(width > 0.0 && height > 0.0) {
            return (0.0, 0.0, 0.0);
        }
        let scale = (side.0 / width).min(side.0 / height);
        let dx = (side.0 - width * scale) / 2.0 - origin.x.0 * scale;
        let dy = (side.0 - height * scale) / 2.0 - origin.y.0 * scale;
        (scale, dx, dy)
    }

    /// What draws it in a square of `side` points, filled with the canvas view's fill color (`fill_color`).
    pub(crate) fn drawing(&self, side: Points) -> Painter {
        let (scale, dx, dy) = self.fit(side);
        Painter::new()
            .translate(dx, dy)
            .scale(scale, scale)
            .fill_style_from_view(SvgPaint::None)
            .fill_svg(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use nsh_core::canvas::{CanvasOp, FillRule, Num, ViewPaint};

    use nsh_core::geometry::{point, size};

    use super::*;

    fn points(x: f32, y: f32, width: f32, height: f32) -> Bounds<Points> {
        Bounds::new(
            point(Points(x), Points(y)),
            size(Points(width), Points(height)),
        )
    }

    #[test]
    fn a_material_symbol_fills_the_square_from_its_960_point_view_box() {
        let icon = Icon::material_symbol("M0 0");
        // 24 / 960 = 0.025; the view box's top, at -960, goes to 0.
        assert_eq!(icon.fit(Points(24.0)), (0.025, 0.0, 24.0));
        assert_eq!(icon.view_box(), points(0.0, -960.0, 960.0, 960.0));
        assert_eq!(icon.path(), "M0 0");
        assert_eq!(Icon::default(), Icon::material_symbol(""));
    }

    #[test]
    fn a_view_box_that_is_not_square_is_centered_with_its_aspect_ratio_kept() {
        // 48 wide and 24 tall into 24: half the scale, centered vertically.
        let wide = Icon::new("M0 0", points(0.0, 0.0, 48.0, 24.0));
        assert_eq!(wide.fit(Points(24.0)), (0.5, 0.0, 6.0));
        // 24 wide and 48 tall, with its origin away from 0: centered horizontally, and moved back to the origin.
        let tall = Icon::new("M0 0", points(10.0, 20.0, 24.0, 48.0));
        assert_eq!(tall.fit(Points(24.0)), (0.5, 6.0 - 5.0, -10.0));
    }

    #[test]
    fn a_view_box_with_no_area_draws_nothing() {
        for view_box in [
            points(0.0, 0.0, 0.0, 24.0),
            points(0.0, 0.0, 24.0, 0.0),
            points(0.0, 0.0, -1.0, 24.0),
        ] {
            assert_eq!(
                Icon::new("M0 0", view_box).fit(Points(24.0)),
                (0.0, 0.0, 0.0)
            );
        }
    }

    #[test]
    fn the_drawing_fits_the_view_box_and_fills_the_path_with_the_views_fill() {
        let drawing = Icon::new("M0 0L24 0L24 24Z", points(0.0, 0.0, 24.0, 24.0))
            .drawing(Points(48.0))
            .finish();
        let constant = |n: &Num| match n {
            Num::Const(v) => Some(*v),
            Num::Slot(_) => None,
        };
        let [translate, scale, fill_style, fill] = drawing.ops.as_slice() else {
            panic!("four operations expected: {:?}", drawing.ops);
        };
        assert!(
            matches!(translate, CanvasOp::Translate(x, y) if constant(x) == Some(0.0) && constant(y) == Some(0.0))
        );
        assert!(
            matches!(scale, CanvasOp::Scale(x, y) if constant(x) == Some(2.0) && constant(y) == Some(2.0))
        );
        assert!(matches!(fill_style, CanvasOp::FillStyle(_)));
        assert!(matches!(fill, CanvasOp::Fill(Some(_), FillRule::NonZero)));
        // The fill style is the view's fill, which the core puts in place of the third operation.
        assert_eq!(drawing.view_paints, [(2, ViewPaint::Fill(SvgPaint::None))]);
    }
}
