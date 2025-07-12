use blob_guy::blob_guy::Blob;
use blob_guy::chunk::{Chunks, Pos};
use eframe::egui;
use rupl::types::{Color, Complex, Graph, GraphType, Name, Show, Vec2};
use std::f64::consts::PI;
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
    frame: u8,
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
            .update(
                self.blob.pos.to_chunk(),
                &mut self.world,
                self.blob.mean(),
                true,
            )
            .unwrap();
        let sx = p.x;
        let sy = p.y;
        plot.data.drain(1..);
        plot.main_colors.drain(1..);
        for (x, y) in self.blob.pixels.keys().copied() {
            let p1 = Vec2::new(x as f64 + 0.5, y as f64 + 0.5);
            plot.data.push(GraphType::Point(p1));
            plot.main_colors.push(Color {
                r: 170,
                g: 170,
                b: 255,
            });
        }
        let (x, y) = self.blob.mean();
        let (x, y) = (x as f64, y as f64);
        plot.data.push(GraphType::Point(Vec2 { x, y }));
        plot.main_colors.push(Color {
            r: 255,
            g: 170,
            b: 255,
        });
        let r = (self.blob.pixels.len() as f64 / PI).sqrt().ceil();
        let r2 = r * r;
        let c = 512;
        let mut values = Vec::with_capacity(2 * c + 1);
        let mut values2 = Vec::with_capacity(2 * c + 1);
        for i in -(c as isize)..=c as isize {
            let x = i as f64 / c as f64 * r;
            let y = (r2 - x * x).sqrt();
            values.push((x + sx, Complex::Real(y + sy)));
            values2.push((x + sx, Complex::Real(sy - y)));
        }
        plot.data.push(GraphType::List(vec![
            GraphType::Coord(values),
            GraphType::Coord(values2),
        ]));
        plot.main_colors.push(Color { r: 0, g: 0, b: 0 });
        self.blob.cull();
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
            frame: 8,
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
                if self.frame == 0 {
                    self.data.update(&mut self.plot);
                    self.frame = 8;
                } else {
                    self.frame -= 1;
                }
                self.plot.update(ctx, ui);
                ctx.request_repaint();
            });
    }
}
