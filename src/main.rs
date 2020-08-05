use overlay::Color;

mod system;
mod overlay;

fn main() {
    let mut overlay = crate::overlay::looking_glass::LookingGlassOverlay::new("/tmp/hax0r-data").unwrap();
    let mut o = 0;
    loop {
        overlay.begin();
        overlay.draw_text((10 + o, 10 + o * 2), "test".parse().unwrap(), Color::from_rgb(255, 0, 0), 100);
        overlay.end();
        std::thread::sleep(std::time::Duration::from_millis(100));
        o += 1;
    }
}
