use std::borrow::Cow;
use std::sync::mpsc::{Receiver, SyncSender};

use eframe::egui::{CtxRef, FontDefinitions, FontFamily, Color32, Label, Layout, Hyperlink, Separator, TopBottomPanel, TextStyle, Button, Frame, Window};
use eframe::egui;
use serde::{Serialize, Deserialize};

pub const PADDING: f32 = 5.0;

const WHITE: Color32 = Color32::from_rgb(255,255,255);
const CYAN: Color32 = Color32::from_rgb(0,255,255);
const BLACK: Color32 = Color32::from_rgb(0,0,0);
const RED: Color32 = Color32::from_rgb(255,0,0);

pub enum Msg {
    ApiKeySet(String),
    RefreshHit(bool)
}

#[derive(Serialize, Deserialize)]
pub struct HeadlinesConfig {
    pub dark_mode: bool,
    pub api_key: String,

}

impl Default for HeadlinesConfig {
    fn default() -> Self {
        Self { 
            dark_mode: Default::default(), 
            api_key: String::new(),
        }
    }
}
pub struct Headlines {
    pub articles: Vec<NewsCardData>,
    pub config: HeadlinesConfig,
    pub api_key_initialized: bool,
    pub news_rx: Option<Receiver<NewsCardData>>,
    pub app_tx: Option<SyncSender<Msg>>
}

impl Headlines {
    pub fn new() -> Headlines {
        // let iter = (0..20).map(|a| NewsCardData {
        //     title: format!("title{}", a),
        //     desc: format!("desc{}", a),
        //     url: format!("https://example.com/{}", a),
        // });

        let config: HeadlinesConfig = confy::load("headlines", None).unwrap_or_default();

        Headlines { 
            api_key_initialized: !config.api_key.is_empty(),
            articles: vec![],
            config,
            news_rx: None,
            app_tx: None
        }
    }

    pub fn configure_fonts(&self, ctx: &CtxRef) {
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert("Montserrat".to_string(), Cow::Borrowed(include_bytes!("../../Montserrat-Regular.ttf")));
        font_def.family_and_size.insert(eframe::egui::TextStyle::Heading, (FontFamily::Proportional, 35.));
        font_def.family_and_size.insert(eframe::egui::TextStyle::Body, (FontFamily::Proportional, 20.));
        font_def.fonts_for_family.get_mut(&FontFamily::Proportional).unwrap().insert(0, "Montserrat".to_string());

        ctx.set_fonts(font_def);
    }

    pub fn render_news_cards(&self, ui: &mut eframe::egui::Ui) {
        for a in &self.articles {
            ui.add_space(PADDING);
            let title = format!("> {}", a.title);
            if self.config.dark_mode {
                ui.colored_label(WHITE, title);
            } else {
                ui.colored_label(BLACK, title);
            }

            ui.add_space(PADDING);
            let desc = Label::new(&a.desc).text_style(eframe::egui::TextStyle::Button);
            ui.add(desc);

            if self.config.dark_mode {
                ui.style_mut().visuals.hyperlink_color = CYAN;
            } else {
                ui.style_mut().visuals.hyperlink_color = RED;
            }
            ui.add_space(PADDING);

            ui.with_layout(Layout::right_to_left(), |ui| {
                ui.add(Hyperlink::new(&a.url).text("read more >"));
            });

            ui.add_space(PADDING);
            ui.add(Separator::default());
            
        }
    }

    pub fn render_top_panel(&mut self, ctx: &CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new("_").text_style(TextStyle::Heading));
                });

                ui.with_layout(Layout::right_to_left(), |ui| {
                    let close_btn = ui.add(Button::new("close").text_style(TextStyle::Body));
                    if close_btn.clicked() {
                        // dbg!(confy::get_configuration_file_path("headlines", None));
                        // confy::store("headlines", None, &self.config);
                        frame.quit();
                    }
                    let refresh_btn = ui.add(Button::new("refresh").text_style(TextStyle::Body));
                    if refresh_btn.clicked() {
                        // self.called_api = false;
                        // tracing::error!("Clicked refreshbutton");
                        if let Some(tx) = &self.app_tx {
                            tx.send(Msg::RefreshHit(true));
                        }
                    }
                    let theme_btn = ui.add(Button::new({
                        if self.config.dark_mode {
                            "🌞"
                        } else {
                            "🌙"
                        }
                    }).text_style(TextStyle::Body));

                    if theme_btn.clicked() {
                        self.config.dark_mode = !self.config.dark_mode;
                    }
                });
            });
            ui.add_space(10.);
        });
    }

    pub fn render_config(&mut self, ctx: &CtxRef) {
        Window::new("Configuration").show(ctx, |ui| {
            ui.label("Enter your API_KEY for newsapi.org");
            let text_input = ui.text_edit_singleline(&mut self.config.api_key);

            if text_input.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                if let Err(e) = confy::store("headlines", None, HeadlinesConfig {
                    dark_mode: self.config.dark_mode,
                    api_key: self.config.api_key.to_string()
                }) {
                    tracing::error!("Failed to save app state: {}", e);
                }

                tracing::trace!("API Key set: {}", self.config.api_key);
                self.api_key_initialized = true;

                // Send config to other thread to load news when api key is available
                if let Some(tx) = &self.app_tx {
                    tx.send(Msg::ApiKeySet(self.config.api_key.to_string()));
                }
            }

            // tracing::error!("{}", &self.config.api_key);
            ui.label("If you have not registered, head over to");
            ui.hyperlink("https://newsapi.org");
            // self.config.api_key = text_input
        });
    }

    pub fn preload_articles(&mut self) {
        if let Some(rx) = &self.news_rx {
            match rx.try_recv() {
                Ok(news_data) => {
                    self.articles.push(news_data);
                },
                Err(e) => {
                    // keep getting an infinite warning messages
                    tracing::warn!("Error receiving msg: {}", e);
                }
            }
        }
    }
}

pub struct NewsCardData {
    pub title: String,
    pub desc: String,
    pub url: String
}