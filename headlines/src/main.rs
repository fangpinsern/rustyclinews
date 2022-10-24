mod headlines;

use std::{thread, sync::mpsc::{channel, sync_channel}};

use eframe::{epi::App, run_native, egui::{CentralPanel, ScrollArea, Vec2, Ui, Separator, TopBottomPanel, CtxRef, Label, Hyperlink, Visuals}, NativeOptions};

use headlines::{Headlines, PADDING, NewsCardData, Msg};
use newsapi::NewsAPI;

fn fetch_news(api_key: &str, articles: &mut Vec<NewsCardData>) {
    if let Ok(response) = NewsAPI::new(&api_key).fetch() {
        let res_article = response.articles;
        for a in res_article.iter() {
            let news = NewsCardData{
                title: a.title().to_string(),
                url: a.url().to_string(),
                desc: a.desc().map(|s| s.to_string()).unwrap_or("...".to_string())
            };
            articles.push(news);
        }
    }
}

impl App for Headlines {
    fn setup(
            &mut self,
            ctx: &eframe::egui::CtxRef,
            _frame: &mut eframe::epi::Frame<'_>,
            _storage: Option<&dyn eframe::epi::Storage>,
        ) {
        //     let api_key = self.config.api_key.to_string();



        // let (app_tx, app_rx) = sync_channel(1);

        // self.app_tx = Some(app_tx);

        // if !api_key.is_empty() {
        //     fetch_news(&api_key, &mut self.articles);
        // } 
        // if all on the same thread then we have to move on only fetch news after we get the config
        // else {
        //     loop {
        //         match app_rx.recv() {
        //             Ok(Msg::ApiKeySet(api_key)) => {
        //                 fetch_news(&api_key,&mut self.articles);
        //             },
        //             Err(e) => {
        //                 tracing::error!("Failed receiving app tx message");
        //             }

        //         }
        //     }
        // }
        self.configure_fonts(&ctx);
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {

        ctx.request_repaint();
        if self.config.dark_mode {
            ctx.set_visuals(Visuals::dark());
        } else {
            ctx.set_visuals(Visuals::light());
        }

        if !self.api_key_initialized {
            self.render_config(ctx);
        } else {
            // self.preload_articles();
            if !self.called_api {
                self.articles = vec![];
                fetch_news(&self.config.api_key, &mut self.articles);
                self.called_api = true;
            }
            self.render_top_panel(ctx, frame);
            CentralPanel::default().show(ctx, |ui| {
                render_header(ui);
                ScrollArea::auto_sized().show(ui, |ui| {
                    self.render_news_cards(ui);
                });
    
                render_footer(ctx);
                
            });

        }
        
    }

    fn name(&self) -> &str {
        "Headlines"
    }
}

fn render_header(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("headlines");
    });

    ui.add_space(PADDING);
    let sep = Separator::default().spacing(20.);
    ui.add(sep);
}

fn render_footer(ctx: &CtxRef) {
    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            ui.add(Label::new("API source: newsapi.org").monospace());

            ui.add(Hyperlink::new("https://github.com/emilk/egui").text("Made with egui").text_style(eframe::egui::TextStyle::Monospace));
            ui.add(Hyperlink::new("https://github.com/fangpinsern/rustyclinews").text("fangpinsern/rustyclinews").text_style(eframe::egui::TextStyle::Monospace));
            ui.add_space(10.);
        });
    });
}

fn main() {
    tracing_subscriber::fmt::init();
    
    let app = Headlines::new();
    let mut win_options = NativeOptions::default();
    win_options.initial_window_size = Some(Vec2::new(540., 960.));
    run_native(Box::new(app), win_options)
}


