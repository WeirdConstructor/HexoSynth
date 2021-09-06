use tuix::*;

fn color_paint(color: (f64, f64, f64)) -> femtovg::Paint {
    femtovg::Paint::color(
        femtovg::Color::rgbf(
            color.0 as f32,
            color.1 as f32,
            color.2 as f32))
}

#[derive(Default)]
struct HexGrid {
    id: usize,
}

impl HexGrid {
    pub fn new(id: usize) -> Self {
        HexGrid { id }
    }
}

impl Widget for HexGrid {
    type Ret  = Entity;
    type Data = ();

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity.set_position_type(state, PositionType::ParentDirected)
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
        let posx = state.data.get_posx(entity);
        let posy = state.data.get_posy(entity);
        let width = state.data.get_width(entity);
        let height = state.data.get_height(entity);
//            println!("WINEVENT[{}]: {:?} {:?}", self.id, window_event, width);
            match window_event {
                _ => {},
            }
        }
    }

    fn on_draw(&mut self, state: &mut State, entity: Entity, canvas: &mut Canvas) {
        let mut transform = state.data.get_transform(entity);


        let bounds = state.data.get_bounds(entity);

//        let mut clip_region = state.data.get_clip_region(entity);
//        canvas.scissor(
//            clip_region.x,
//            clip_region.y,
//            clip_region.w,
//            clip_region.h,
//        );
//
        canvas.save();
//        canvas.set_transform(transform[0], transform[1], transform[2], transform[3], transform[4], transform[5]);

        let segments = [
            (0.0, 0.0),
            (20.0 + 10.0 * self.id as f32, 0.0),
            (20.0, 20.0),
            (100.0, 400.0),
        ];

        let mut p = femtovg::Path::new();

        let mut paint = color_paint((1.0, 0.0, 0.0));

        paint.set_line_join(femtovg::LineJoin::Round);
        paint.set_line_width(2.0);

        let mut first = true;
        for s in segments {
            if first {
                p.move_to(bounds.x + s.0 as f32, bounds.y + s.1 as f32);
                first = false;
            } else {
                p.line_to(bounds.x + s.0 as f32, bounds.y + s.1 as f32);
            }
        }

        let closed = true;

        if closed { p.close(); }

        canvas.stroke_path(&mut p, paint);


        canvas.restore();
    }
}

fn main() {
    let mut app =
        Application::new(
            WindowDescription::new(),
            |state, window| {
                let row = Row::new().build(state, window, |builder| builder);

                let hex = HexGrid::new(1).build(state, row, |builder| builder);
                let hex2 = HexGrid::new(2).build(state, row, |builder| builder);
            });
    app.run();
}
