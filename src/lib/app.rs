#[derive(Debug)]
pub struct Valor {}

impl Valor {
    pub fn new() -> Self {
        Self {}
    }
}

impl eframe::App for Valor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Valor or something idk");
        });
    }
}
