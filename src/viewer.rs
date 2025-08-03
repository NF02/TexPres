use eframe::egui;
use std::fs;
use std::path::PathBuf;
use egui::text::{LayoutJob, TextFormat};
use std::process;

pub const SLIDE_DELIMITER: &str = "---";

/// La struct principale per l'applicazione del visualizzatore di testo.
pub struct SentTextViewer {
    slides: Vec<String>,
    current_slide_index: usize,
    font_size: f32,
    should_quit: bool,
}

impl SentTextViewer {
    /// Costruttore per SentTextViewer.
    pub fn new(file_path: &PathBuf, font_size: f32) -> Result<Self, String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Errore nella lettura del file {}: {}", file_path.display(), e))?;

        let slides: Vec<String> = content
            .split(SLIDE_DELIMITER)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if slides.is_empty() {
            return Err(format!("Nessuna diapositiva trovata nel file '{}'. Assicurati che le diapositive siano separate da '{}'.", file_path.display(), SLIDE_DELIMITER));
        }

        Ok(SentTextViewer {
            slides,
            current_slide_index: 0,
            font_size,
            should_quit: false,
        })
    }

    /// Passa alla diapositiva successiva.
    fn next_slide(&mut self) {
        if self.current_slide_index < self.slides.len() - 1 {
            self.current_slide_index += 1;
        }
    }

    /// Torna alla diapositiva precedente.
    fn prev_slide(&mut self) {
        if self.current_slide_index > 0 {
            self.current_slide_index -= 1;
        }
    }

    /// Funzione di utilità per il layout e la centratura del testo.
    fn centered_text_layout(&self, ui: &mut egui::Ui, text: &str) {
        let wrap_width = ui.available_width();
        
        let mut job = LayoutJob::default();
        job.halign = egui::Align::Center;
        job.wrap.max_width = wrap_width;

        // Itera sulle righe del testo.
        let mut lines = text.lines();
        
        // Formatta la prima riga con un font più grande.
        if let Some(first_line) = lines.next() {
            let font_id_large = egui::FontId::proportional(self.font_size * 1.0);
            let start_idx = job.text.len();
            job.text.push_str(first_line);
            let end_idx = job.text.len();
            job.sections.push(egui::text::LayoutSection {
                leading_space: 0.0,
                byte_range: start_idx..end_idx,
                format: TextFormat {
                    font_id: font_id_large,
                    color: ui.style().visuals.text_color(),
                    ..Default::default()
                },
            });
            job.text.push('\n');
        }

        // Formatta le restanti righe con il font standard.
        let font_id_regular = egui::FontId::proportional(self.font_size);
        for line in lines {
            let start_idx = job.text.len();
            job.text.push_str(line);
            let end_idx = job.text.len();
            job.sections.push(egui::text::LayoutSection {
                leading_space: 0.0,
                byte_range: start_idx..end_idx,
                format: TextFormat {
                    font_id: font_id_regular.clone(), // Cloniamo il valore per ogni iterazione.
                    color: ui.style().visuals.text_color(),
                    ..Default::default()
                },
            });
            job.text.push('\n');
        }
        
        // Rimuove la newline finale se presente.
        if job.text.ends_with('\n') {
            job.text.pop();
        }

        let galley = ui.fonts(|f| f.layout_job(job));

        let text_height = galley.rect.height();
        let available_height = ui.available_height();
        let top_padding = (available_height - text_height) / 2.0;

        // Aggiunge un'imbottitura per l'allineamento verticale al centro
        if top_padding > 0.0 {
            ui.add_space(top_padding);
        }

        ui.add(egui::Label::new(galley));
    }
}

impl eframe::App for SentTextViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Gestione degli input da tastiera.
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed, .. } = event {
                    if *pressed {
                        match key {
                            egui::Key::ArrowRight | egui::Key::N => self.next_slide(),
                            egui::Key::ArrowLeft | egui::Key::P => self.prev_slide(),
                            egui::Key::Q | egui::Key::Escape => {
                                self.should_quit = true;
                            },
                            _ => {}
                        }
                    }
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(current_slide_content) = self.slides.get(self.current_slide_index) {
                self.centered_text_layout(ui, current_slide_content);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Errore: Nessuna diapositiva da mostrare.");
                });
            }
        });
        
        egui::TopBottomPanel::bottom("controls_panel").show(ctx, |ui| {
            ui.set_height(20.0);
            ui.add_space(5.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                 ui.label(format!("Diapositiva {} / {}", self.current_slide_index + 1, self.slides.len()));
            });
        });

        // Controlla il flag di uscita alla fine del frame.
        if self.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            process::exit(0);
        }
	
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        println!("SentTextViewer terminato.");
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.1, 0.1, 0.1, 1.0]
    }
}
