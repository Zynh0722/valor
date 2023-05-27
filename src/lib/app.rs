mod app_init;

use std::sync::Arc;

use crate::ConnectionState;

use std::sync::Mutex;

#[derive(Debug)]
pub struct Valor {
    connection: Arc<Mutex<ConnectionState>>,
}

impl eframe::App for Valor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let label = if self.connection.lock().unwrap().lockfile.is_some() {
                "Connected"
            } else {
                "Disconnected"
            };
            ui.label(label);
        });
    }
}
