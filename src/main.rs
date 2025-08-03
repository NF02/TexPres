// Questo modulo contiene la logica principale dell'applicazione per la gestione delle diapositive.
mod viewer;

use eframe::egui;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Assicurati che un percorso del file sia fornito come argomento.
    if args.len() != 2 {
        eprintln!("Utilizzo: {} <path_to_text_slide_file>", args[0]);
        eprintln!("Il file di testo deve contenere diapositive separate da '{}'", viewer::SLIDE_DELIMITER);
        std::process::exit(1);
    }

    let file_path = PathBuf::from(&args[1]);

    // Definisci la dimensione del font qui. Puoi cambiare questo valore per regolare la dimensione del testo.
    let font_size = 40.0;

    // Inizializza l'applicazione con il file diapositive fornito e la dimensione del font.
    let app = match viewer::SentTextViewer::new(&file_path, font_size) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Errore di inizializzazione: {}", e);
            std::process::exit(1);
        }
    };

    // Configura le opzioni della finestra nativa per eframe.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(800.0, 600.0))
            .with_min_inner_size(egui::vec2(300.0, 200.0))
            .with_decorations(true)
            .with_title("TexPres"),
        ..Default::default()
    };

    // Esegui l'applicazione eframe.
    eframe::run_native(
        "",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    ).unwrap();
}
