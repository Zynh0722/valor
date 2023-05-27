use std::println;

use valor::Valor;

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Unable to start tokio runtime");

    // This lets tokio::spawn work
    let _enter = rt.enter();

    let valor = rt.block_on(Valor::new());

    eframe::run_native(
        "Valor",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| Box::new(valor)),
    )
    .expect("Failed to start graphics context");
}
