use std::sync::mpsc;

use eframe::egui;

use super::config::{Color, SplashConfig};
use super::progress::SplashMessage;

pub struct SplashApp {
    config: SplashConfig,
    receiver: mpsc::Receiver<SplashMessage>,
    status: String,
    progress: f32,
    logo_texture: Option<egui::TextureHandle>,
    logo_loaded: bool,
    should_close: bool,
}

impl SplashApp {
    pub fn new(config: SplashConfig, receiver: mpsc::Receiver<SplashMessage>) -> SplashApp {
        SplashApp {
            config,
            receiver,
            status: "Preparing...".to_string(),
            progress: 0.0,
            logo_texture: None,
            logo_loaded: false,
            should_close: false,
        }
    }

    fn process_messages(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                SplashMessage::UpdateStatus(s) => self.status = s,
                SplashMessage::UpdateProgress(p) => self.progress = p,
                SplashMessage::Close => self.should_close = true,
            }
        }
    }

    fn load_logo(&mut self, ctx: &egui::Context) {
        if self.logo_loaded {
            return;
        }
        self.logo_loaded = true;

        let image_bytes: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/splash/embedded_logo.bin"
        ));

        if image_bytes.is_empty() {
            return;
        }

        if let Ok(img) = image::load_from_memory(image_bytes) {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let pixels = rgba.into_raw();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            self.logo_texture =
                Some(ctx.load_texture("splash_logo", color_image, egui::TextureOptions::LINEAR));
        }
    }

    fn color_to_egui32(color: &Color) -> egui::Color32 {
        egui::Color32::from_rgb(color.r, color.g, color.b)
    }
}

impl eframe::App for SplashApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_messages();

        if self.should_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.load_logo(ctx);

        let bg = Self::color_to_egui32(&self.config.bg_color);
        let text_color = Self::color_to_egui32(&self.config.text_color);
        let progress_color = Self::color_to_egui32(&self.config.progress_color);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(bg))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

                    // Logo image or fallback text
                    if let Some(ref texture) = self.logo_texture {
                        let mut size = texture.size_vec2();
                        let max_width = self.config.window_width as f32 * 0.6;
                        let max_height = self.config.window_height as f32 * 0.4;
                        let scale = (max_width / size.x).min(max_height / size.y).min(1.0);
                        size *= scale;
                        ui.image(egui::load::SizedTexture::new(texture.id(), size));
                    } else {
                        ui.add_space(40.0);
                        ui.label(
                            egui::RichText::new(&self.config.project_name)
                                .color(text_color)
                                .size(32.0)
                                .strong(),
                        );
                    }

                    ui.add_space(10.0);

                    // Version
                    if !self.config.project_version.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("v{}", self.config.project_version))
                                .color(text_color)
                                .size(14.0),
                        );
                    }

                    ui.add_space(20.0);

                    // Progress bar
                    let available_width = ui.available_width() * 0.8;
                    let bar_height = 20.0;
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(available_width, bar_height), egui::Sense::hover());

                    let painter = ui.painter();

                    // Background track
                    let track_color = egui::Color32::from_rgba_premultiplied(
                        text_color.r(),
                        text_color.g(),
                        text_color.b(),
                        30,
                    );
                    painter.rect_filled(rect, 4.0, track_color);

                    // Fill
                    let fill_width = rect.width() * self.progress;
                    let fill_rect = egui::Rect::from_min_size(
                        rect.min,
                        egui::vec2(fill_width, rect.height()),
                    );
                    painter.rect_filled(fill_rect, 4.0, progress_color);

                    // Percentage text
                    let pct_text = format!("{}%", (self.progress * 100.0) as u32);
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        pct_text,
                        egui::FontId::proportional(12.0),
                        text_color,
                    );

                    ui.add_space(15.0);

                    // Status text
                    ui.label(
                        egui::RichText::new(&self.status)
                            .color(text_color)
                            .size(14.0),
                    );
                });
            });

        // Request repaint to stay responsive to messages
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }
}
