#![windows_subsystem = "windows"]

mod app;
mod ui;
mod providers;
mod resource;

fn main() -> Result<(), String> {
    let mut w = &mut app::windowing_bits();
    let mut app = app::App::new(&mut w);

    app.run()
}
