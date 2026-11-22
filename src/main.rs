#![windows_subsystem = "windows"]

mod app;
mod config;
mod db;
mod dl;
mod i18n;
mod scan;
mod search;
mod theme;
mod ui;
mod windows;

use gpui::{App, AppContext as _};

fn main() {
    let app = gpui_kit::application().with_assets(gpui_kit::assets::AllAssets);
    app.run(|cx: &mut App| {
        gpui_component::init(cx);
        theme::apply(cx, gpui_component::ThemeMode::Light);
        app::init_globals(cx);

        // Open the main window.
        let handle = ui::open_main_window(cx);
        let _ = handle.update(cx, |_, window, _| {
            window.activate_window();
        });

        // Persist window geometry and config on quit.
        let (config, lang) = {
            let g = app::Globals::global(cx);
            let lang = g.lang.read(cx).clone();
            (g.config.clone(), lang)
        };
        cx.on_app_quit(move |cx| {
            let mut cfg = config.lock().unwrap().clone();
            let _ = handle.update(cx, |_, window, _| {
                let b = window.bounds();
                cfg.window_width = b.size.width.as_f32() as i32;
                cfg.window_height = b.size.height.as_f32() as i32;
                cfg.window_x = b.origin.x.as_f32() as i32;
                cfg.window_y = b.origin.y.as_f32() as i32;
            });
            let _ = config::save_config(&app::config_path(), &cfg, &lang);
            async {}
        });
    });
}
