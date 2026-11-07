mod math;
mod renderer;
mod shader;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use renderer::{render_shader, render_shader_with_saturn_rings, FrameBuffer};
use shader::{CelestialShader, Planet};
use std::{env, fs};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = env::args().collect();
    let mut current_planet = planet_argument(&arguments).unwrap_or(Planet::Sun);
    let mut frame = FrameBuffer::new(WIDTH, HEIGHT);
    render_planet(&mut frame, current_planet);

    if arguments
        .iter()
        .any(|argument| argument == "--all-screenshots")
    {
        fs::create_dir_all("screenshots")?;
        for planet in Planet::ALL {
            render_planet(&mut frame, planet);
            save_planet_screenshot(&frame, planet)?;
        }
        return Ok(());
    }

    if arguments.iter().any(|argument| argument == "--screenshot") {
        save_planet_screenshot(&frame, current_planet)?;
        return Ok(());
    }

    let title = window_title(current_planet);
    let mut window = Window::new(&title, WIDTH, HEIGHT, WindowOptions::default())?;
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&frame.pixels, frame.width, frame.height)?;

        let selected_planet = planet_from_key(&window);
        let next_planet = if window.is_key_pressed(Key::Right, KeyRepeat::No) {
            Some(current_planet.next(1))
        } else if window.is_key_pressed(Key::Left, KeyRepeat::No) {
            Some(current_planet.next(-1))
        } else {
            selected_planet
        };

        if let Some(next_planet) = next_planet.filter(|planet| *planet != current_planet) {
            current_planet = next_planet;
            render_planet(&mut frame, current_planet);
            let title = window_title(current_planet);
            window.set_title(&title);
            println!("Mostrando {}", current_planet.name());
        }

        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            save_planet_screenshot(&frame, current_planet)?;
        }
    }

    Ok(())
}

fn render_planet(frame: &mut FrameBuffer, planet: Planet) {
    let shader = CelestialShader::new(planet);
    if planet.is_saturn() {
        render_shader_with_saturn_rings(frame, &shader);
    } else {
        render_shader(frame, &shader);
    }
}

fn save_planet_screenshot(
    frame: &FrameBuffer,
    planet: Planet,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("screenshots")?;
    let path = format!("screenshots/{}.png", planet.file_stem());
    frame.save_png(&path)?;
    println!("Screenshot guardado: {}", path);
    Ok(())
}

fn planet_argument(arguments: &[String]) -> Option<Planet> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == "--planet")
        .and_then(|pair| Planet::from_name(&pair[1]))
}

fn planet_from_key(window: &Window) -> Option<Planet> {
    let keys = [
        (Key::Key1, Planet::Sun),
        (Key::Key2, Planet::Venus),
        (Key::Key3, Planet::Neptune),
        (Key::Key4, Planet::Earth),
        (Key::Key5, Planet::Saturn),
        (Key::Key6, Planet::Uranus),
    ];

    keys.iter()
        .find(|(key, _)| window.is_key_pressed(*key, KeyRepeat::No))
        .map(|(_, planet)| *planet)
}

fn window_title(planet: Planet) -> String {
    format!("Solar System Shaders - {}", planet.name())
}
