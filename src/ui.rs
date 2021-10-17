use epi::egui::CtxRef;


#[derive(Default)]
pub struct Ui {}

impl Ui {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn tick(&mut self, _ctx: &CtxRef, _frame: &mut epi::Frame<'_>) {}
}
