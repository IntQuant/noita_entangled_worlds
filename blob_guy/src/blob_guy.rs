use noita_api::game_print;
#[derive(Default)]
pub struct Pos {
    x: f64,
    y: f64,
}
impl Pos {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
#[derive(Default)]
pub struct Blob {
    pub pos: Pos,
}
impl Blob {
    pub fn update(&mut self) {
        game_print(self.pos.x.to_string());
        game_print(self.pos.y.to_string());
    }
}
