use eframe::egui;
use std::{fs, path::PathBuf, sync::Arc, process};
use egui::text::{LayoutJob, TextFormat};

pub const SLIDE_DELIMITER: &str = "---";

#[derive(Debug)]
pub enum AppError {
    IoError(std::io::Error),
    EmptyFile,
    ParseError(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::IoError(e) => write!(f, "IO Error: {}", e),
            AppError::EmptyFile => write!(f, "Empty file or no slides found"),
            AppError::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
}

pub struct SentTextViewer {
    slides: Vec<Arc<str>>,
    current_slide_index: usize,
    font_size: f32,
    first_slide: bool,
    last_slide: bool,
    should_quit: bool,
    dark_mode: bool,
    auto_advance: bool,
    last_advance_time: Option<f64>,
    advance_interval: f64,
    fullscreen: bool,
}

impl SentTextViewer {
    pub fn new(file_path: &PathBuf, font_size: f32) -> Result<Self, AppError> {
        let content = fs::read_to_string(file_path)?;
        
        if content.trim().is_empty() {
            return Err(AppError::EmptyFile);
        }

        let slides: Vec<Arc<str>> = content
            .split(SLIDE_DELIMITER)
            .map(|s| Arc::from(s.trim()))
            .filter(|s: &Arc<str>| !s.is_empty())
            .collect();

        if slides.is_empty() {
            return Err(AppError::EmptyFile);
        }

        let mut viewer = Self {
            slides,
            current_slide_index: 0,
            font_size,
            first_slide: true,
            last_slide: false,
            should_quit: false,
            dark_mode: true,
            auto_advance: false,
	    fullscreen: false,
            last_advance_time: None,
            advance_interval: 5.0,
        };
        viewer.update_slide_status();
        Ok(viewer)
    }

    fn update_slide_status(&mut self) {
        self.first_slide = self.current_slide_index == 0;
        self.last_slide = self.current_slide_index == self.slides.len() - 1;
    }

    fn next_slide(&mut self) {
        if !self.last_slide {
            self.current_slide_index += 1;
            self.update_slide_status();
        }
    }

    fn prev_slide(&mut self) {
        if !self.first_slide {
            self.current_slide_index -= 1;
            self.update_slide_status();
        }
    }

    fn goto_slide(&mut self, index: usize) {
        if index < self.slides.len() {
            self.current_slide_index = index;
            self.update_slide_status();
        }
    }

    fn toggle_auto_advance(&mut self) {
        self.auto_advance = !self.auto_advance;
        self.last_advance_time = None;
    }
    fn centered_text_layout(&self, ui: &mut egui::Ui, text: &str) {
	// Calcola l'area disponibile
	let available_rect = ui.available_rect_before_wrap();
	let max_text_width = available_rect.width() * 0.8;
	
	// Dividi il testo in paragrafi
	let paragraphs: Vec<&str> = text.split("\n\n").collect();
	
	// Prepara il layout principale
	let mut main_job = LayoutJob::default();
	main_job.halign = egui::Align::Center;
	main_job.wrap.max_width = max_text_width;
	
	// Crea le font id necessarie (risolvendo l'errore di move)
	let base_font_id = egui::FontId {
            size: self.font_size,
            family: egui::FontFamily::Proportional,
	};
	let title_font_id = egui::FontId {
            size: self.font_size * 1.5,
            family: base_font_id.family.clone(),
	};
	let subtitle_font_id = egui::FontId {
            size: self.font_size * 1.2,
            family: base_font_id.family.clone(),
	};
	
	let text_color = ui.style().visuals.text_color();
	
	for (i, paragraph) in paragraphs.iter().enumerate() {
            if i > 0 {
		main_job.append("\n\n", 0.0, TextFormat::default());
            }
            
            if paragraph.starts_with("# ") {
		main_job.append(
                    &paragraph[2..],
                    0.0,
                    TextFormat {
			font_id: title_font_id.clone(),
			color: text_color,
			..Default::default()
                    },
		);
            } else if paragraph.starts_with("## ") {
		main_job.append(
                    &paragraph[3..],
                    0.0,
                    TextFormat {
			font_id: subtitle_font_id.clone(),
			color: text_color,
			..Default::default()
                    },
		);
            } else {
		main_job.append(
                    paragraph,
                    0.0,
                    TextFormat {
			font_id: base_font_id.clone(),
			color: text_color,
			..Default::default()
                    },
		);
            }
	}
	
	// Calcola e disegna il testo centrato
	let galley = ui.fonts(|f| f.layout_job(main_job));
	let pos = egui::pos2(
            available_rect.center().x - galley.size().x / 50.0,
            available_rect.center().y - galley.size().y / 2.0,
	);
	ui.painter().galley(pos, galley, text_color);
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Gestione tasti rapidi
            for event in &i.events {
                if let egui::Event::Key { key, pressed, .. } = event {
                    if *pressed {
                        match key {
                            egui::Key::ArrowRight | egui::Key::N => self.next_slide(),
                            egui::Key::ArrowLeft | egui::Key::P => self.prev_slide(),
                            egui::Key::Q | egui::Key::Escape => self.should_quit = true,
                            egui::Key::Space => self.toggle_auto_advance(),
                            egui::Key::D => self.dark_mode = !self.dark_mode,
                            _ => {}
                        }
                    }
                }
            }

            // Zoom con Ctrl+MouseWheel
            if i.modifiers.ctrl {
                let scroll_delta = i.raw_scroll_delta.y;
                self.font_size = (self.font_size + scroll_delta * 2.0)
                    .clamp(12.0, 72.0);
            }
        });

        // Avanzamento automatico
        if self.auto_advance {
            let now = ctx.input(|i| i.time);
            if let Some(last_time) = self.last_advance_time {
                if now - last_time > self.advance_interval {
                    self.next_slide();
                    self.last_advance_time = Some(now);
                }
            } else {
                self.last_advance_time = Some(now);
            }
        }
    }

    fn toggle_fullscreen(&mut self, ctx: &egui::Context) {
        self.fullscreen = !self.fullscreen;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
    }

    fn render_ui(&mut self, ctx: &egui::Context) {
        // Imposta il tema
        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        // Pannello principale con la slide
        egui::CentralPanel::default()
	    .frame(egui::Frame::NONE)
	    .show(ctx, |ui| {
            if let Some(current_slide) = self.slides.get(self.current_slide_index) {
		self.centered_text_layout(ui, current_slide);
	    } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Errore: Nessuna diapositiva da mostrare.");
                });
            }
        });

        // Barra di controllo in basso
        egui::TopBottomPanel::bottom("controls_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Pulsanti di navigazione
                ui.add_enabled_ui(!self.first_slide, |ui| {
                    if ui.button("← Prev").clicked() {
                        self.prev_slide();
                    }
                });

                ui.add_enabled_ui(!self.last_slide, |ui| {
                    if ui.button("Next →").clicked() {
                        self.next_slide();
                    }
                });

                // Indicatore slide
                ui.label(format!("Slide {}/{}", 
                    self.current_slide_index + 1, 
                    self.slides.len()));

                // Controllo avanzamento automatico
                if ui.button(if self.auto_advance { "⏸" } else { "▶" }).clicked() {
                    self.toggle_auto_advance();
                }

		// goto slide
		ui.horizontal(|ui| {
		    ui.label("Go to slide:");
		    if ui.add(egui::Slider::new(&mut self.current_slide_index, 0..=self.slides.len().saturating_sub(1)))
			.changed()
		    {
			self.goto_slide(self.current_slide_index);
		    }
		});

                // Controllo tema
                if ui.button(if self.dark_mode { "☀️" } else { "🌙" }).clicked() {
                    self.dark_mode = !self.dark_mode;
                }

		// Pulsante fullscreen
		if ui.button(if self.fullscreen { "🖵 Exit Fullscreen" } else { "🖵 Fullscreen" }).clicked() {
		    self.toggle_fullscreen(ctx);
		}

                // Controllo zoom
                ui.label(format!("Zoom: {:.0}%", (self.font_size / 24.0) * 100.0));

                // Pulsante uscita
                if ui.button("Quit").clicked() {
                    self.should_quit = true;
                }
            });
        });
    }
}

impl eframe::App for SentTextViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);
        self.render_ui(ctx);
        
        if self.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            process::exit(0);
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        if self.dark_mode {
            [0.1, 0.1, 0.1, 1.0]
        } else {
            [0.95, 0.95, 0.95, 1.0]
        }
    }
}
