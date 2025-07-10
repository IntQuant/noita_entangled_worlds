use blob_guy::blob_guy::Blob;
use blob_guy::chunk::{Chunks, Pos};
use eframe::egui;
use rupl::types::{Color, Graph, GraphType, Name, Show, Vec2};
fn main() {
    eframe::run_native(
        "blob",
        eframe::NativeOptions {
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(App::new()))),
    )
    .unwrap();
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.main(ctx);
    }
}

struct App {
    plot: Graph,
    data: Data,
}

struct Data {
    blob: Blob,
    world: Chunks,
}

impl Data {
    fn new() -> Self {
        Self {
            blob: Blob::new(0.0, 0.0),
            world: Default::default(),
        }
    }
    fn update(&mut self, plot: &mut Graph) {
        let s = &plot.names[0].vars[0];
        let s = &s[3..s.len() - 1];
        let (a, b) = s.split_once(',').unwrap();
        let GraphType::Point(p) = &mut plot.data[0] else {
            unreachable!()
        };
        p.x = a.parse().unwrap();
        p.y = b.parse().unwrap();
        self.blob.pos = Pos::new(p.x as f32, p.y as f32);
        self.blob
            .update(self.blob.pos.to_chunk(), &mut self.world)
            .unwrap();
        plot.data.drain(1..);
        plot.main_colors.drain(1..);
        for ((_x, _y), _p) in self.blob.pixels.iter() {
            let p1 = Vec2::new(*_x as f64 + 0.5, *_y as f64 + 0.5);
            //let p2 = Vec2::new(_p.pos.x as f64, _p.pos.y as f64);
            plot.data.push(GraphType::Point(p1));
            plot.main_colors.push(Color {
                r: 170,
                g: 170,
                b: 255,
            });
            /*plot.data.push(GraphType::Point(p2));
            plot.main_colors.push(Color {
                r: 170,
                g: 255,
                b: 170,
            })*/
        }
    }
}
impl App {
    fn new() -> Self {
        let mut plot = Graph::new(
            vec![GraphType::Point(Vec2::splat(0.0))],
            vec![Name {
                vars: vec!["a={0,0}".to_string()],
                name: "a".to_string(),
                show: Show::Real,
            }],
            false,
            -32.0,
            32.0,
        );
        plot.point_size = 8.0;
        App {
            plot,
            data: Data::new(),
        }
    }
    fn main(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(255, 255, 255)))
            .show(ctx, |ui| {
                self.plot.keybinds(ui);
                let rect = ctx.available_rect();
                self.plot
                    .set_screen(rect.width() as f64, rect.height() as f64, true, true);
                self.data.update(&mut self.plot);
                self.plot.update(ctx, ui);
                ctx.request_repaint();
            });
    }
}
