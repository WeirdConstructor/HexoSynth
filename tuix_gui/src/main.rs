use tuix::*;
use femtovg::FontId;

mod painter;
use painter::FemtovgPainter;

#[derive(Default)]
struct HexGrid {
    id: usize,
    font: Option<FontId>,
    font_mono: Option<FontId>,
}

impl HexGrid {
    pub fn new(id: usize) -> Self {
        HexGrid {
            id,
            font: None,
            font_mono: None,
        }
    }
}

fn hex_size2wh(size: f64) -> (f64, f64) {
    (2.0_f64 * size, (3.0_f64).sqrt() * size)
}

impl Widget for HexGrid {
    type Ret  = Entity;
    type Data = ();

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity.set_position_type(state, PositionType::ParentDirected)
              .set_clip_widget(state, entity)
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
        if self.font.is_none() {
            self.font      = Some(canvas.add_font_mem(std::include_bytes!("font.ttf")).expect("can load font"));
            self.font_mono = Some(canvas.add_font_mem(std::include_bytes!("font_mono.ttf")).expect("can load font"));
        }

        let bounds = state.data.get_bounds(entity);

        let p = &mut FemtovgPainter {
            canvas:     canvas,
            font:       self.font.unwrap(),
            font_mono:  self.font_mono.unwrap(),
        };

        let x = bounds.x as f64 + 100.0;
        let y = bounds.y as f64 + 100.0;
        let (w, h) = hex_size2wh(60.0);

        p.path_stroke(
            3.0,
            (1.0, 0.0, 1.0),
            &mut ([
                (x - 0.50 * w, y          ),
                (x - 0.25 * w, y - 0.5 * h),
                (x + 0.25 * w, y - 0.5 * h),
                (x + 0.50 * w, y          ),
                (x + 0.25 * w, y + 0.5 * h),
                (x - 0.25 * w, y + 0.5 * h),
            ].iter().copied().map(|p| (p.0.floor(), p.1.floor()))), true);
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
