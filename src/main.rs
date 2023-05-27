use std::println;

use valor::Valor;

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Unable to start tokio runtime");

    // This lets tokio::spawn work
    let _enter = rt.enter();

    println!("before init");
    let valor = rt.block_on(Valor::new());
    println!("after init");

    eframe::run_native(
        "Valor",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| Box::new(valor)),
    )
    .expect("Failed to start graphics context");
}
