mod layout;
use std::collections::HashSet;

use alot::LotId;
use gooey_core::graphics::Point;
use gooey_core::math::{IntoSigned, Rect, Size};
use gooey_core::style::{Px, UPx};
use gooey_core::{WidgetTransmogrifier, WidgetValue};
use gooey_raster::{
    ConstraintLimit, RasterContext, Rasterizable, RasterizedApp, Renderer, SurfaceHandle,
    WidgetRasterizer,
};

use crate::flex::{FlexDimension, FlexTransmogrifier, Orientation};
use crate::Flex;

struct FlexRasterizer {
    children: RasterizedChildren,
    flex: layout::Flex,
    mouse_tracking: Option<LotId>,
    hovering: HashSet<LotId>,
}

type RasterizedChildren = Vec<(LotId, Rasterizable)>;

impl<Surface> WidgetTransmogrifier<RasterizedApp<Surface>> for FlexTransmogrifier
where
    Surface: gooey_raster::Surface,
{
    type Widget = Flex;

    fn transmogrify(
        &self,
        widget: &Self::Widget,
        style: gooey_core::reactor::Value<stylecs::Style>,
        context: &RasterContext<Surface>,
    ) -> Rasterizable {
        let mut raster_children = RasterizedChildren::default();
        let mut flex = layout::Flex::new(Orientation::Column);
        widget.children.map_ref(|children| {
            for (id, child) in children.entries() {
                flex.push(FlexDimension::FitContent);
                raster_children.push((
                    id,
                    context
                        .widgets()
                        .instantiate(child.widget.as_ref(), style, context),
                ));
            }
        });

        if let WidgetValue::Value(value) = &widget.children {
            value.for_each({
                let handle = context.handle().clone();
                move |_| {
                    handle.invalidate();
                }
            })
        }

        Rasterizable::new(FlexRasterizer {
            children: raster_children,
            flex,
            mouse_tracking: None,
            hovering: HashSet::new(),
        })
    }
}

impl WidgetRasterizer for FlexRasterizer {
    type Widget = Flex;

    fn measure(
        &mut self,
        _available_space: Size<ConstraintLimit>,
        _renderer: &mut dyn Renderer,
    ) -> Size<UPx> {
        todo!("implement flex measurement")
    }

    fn draw(&mut self, renderer: &mut dyn Renderer) {
        self.flex.update(
            Size::new(
                ConstraintLimit::Known(renderer.size().width),
                ConstraintLimit::Known(renderer.size().height),
            ),
            |child_index, constraints| self.children[child_index].1.measure(constraints, renderer),
        );

        for (layout, (_id, rasterizable)) in self.flex.iter().zip(self.children.iter_mut()) {
            renderer.clip_to(Rect::new(
                self.flex.orientation.make_point(layout.offset, UPx(0)),
                self.flex
                    .orientation
                    .make_size(layout.size, self.flex.other),
            ));
            rasterizable.draw(renderer);
            renderer.pop_clip();
        }
    }

    fn mouse_down(&mut self, location: Point<Px>, surface: &dyn SurfaceHandle) {
        for (layout, (id, rasterizable)) in self.flex.iter().zip(self.children.iter_mut()) {
            let rect = Rect::new(
                self.flex.orientation.make_point(layout.offset, UPx(0)),
                self.flex
                    .orientation
                    .make_size(layout.size, self.flex.other),
            )
            .into_signed();
            let relative = location - rect.origin;
            if relative.x >= 0
                && relative.y >= 0
                && relative.x < rect.size.width
                && relative.y < rect.size.height
            {
                self.mouse_tracking = Some(*id);
                rasterizable.mouse_down(relative, surface);
                break;
            }
        }
    }

    fn cursor_moved(&mut self, location: Option<Point<Px>>, surface: &dyn SurfaceHandle) {
        for (layout, (id, rasterizable)) in self.flex.iter().zip(self.children.iter_mut()) {
            let rect = Rect::new(
                self.flex.orientation.make_point(layout.offset, UPx(0)),
                self.flex
                    .orientation
                    .make_size(layout.size, self.flex.other),
            )
            .into_signed();
            let relative = location.map(|location| location - rect.origin);
            if relative.map_or(false, |relative| {
                relative.x >= 0
                    && relative.y >= 0
                    && relative.x < rect.size.width
                    && relative.y < rect.size.height
            }) {
                rasterizable.cursor_moved(relative, surface);
                self.hovering.insert(*id);
            } else if self.hovering.remove(id) {
                rasterizable.cursor_moved(None, surface);
            }
        }
    }

    fn mouse_up(&mut self, location: Option<Point<Px>>, surface: &dyn SurfaceHandle) {
        if let Some((layout, (_, rasterizable))) = self
            .flex
            .iter()
            .zip(self.children.iter_mut())
            .find(|(_, (id, _))| Some(*id) == self.mouse_tracking)
        {
            let rect = Rect::new(
                self.flex.orientation.make_point(layout.offset, UPx(0)),
                self.flex
                    .orientation
                    .make_size(layout.size, self.flex.other),
            )
            .into_signed();
            let relative = location.map(|location| location - rect.origin);
            if relative.map_or(false, |relative| {
                relative.x >= 0
                    && relative.y >= 0
                    && relative.x < rect.size.width
                    && relative.y < rect.size.height
            }) {
                rasterizable.mouse_up(relative, surface);
            } else {
                rasterizable.mouse_up(None, surface);
            }
        }
        self.mouse_tracking = None;
    }
}
