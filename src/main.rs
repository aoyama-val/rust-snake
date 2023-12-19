use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime};
mod model;
use crate::model::*;

const FPS: u32 = 30;

struct Image<'a> {
    texture: Texture<'a>,
    w: u32,
    h: u32,
}

impl<'a> Image<'a> {
    fn new(texture: Texture<'a>) -> Self {
        let q = texture.query();
        let image = Image {
            texture,
            w: q.width,
            h: q.height,
        };
        image
    }
}

struct Resources<'a> {
    images: HashMap<String, Image<'a>>,
    chunks: HashMap<String, sdl2::mixer::Chunk>,
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rust-asteroids", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    sdl_context.mouse().show_cursor(false);

    init_mixer();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::Blend);

    let texture_creator = canvas.texture_creator();
    let mut resources = load_resources(&texture_creator, &mut canvas);

    let mut event_pump = sdl_context.event_pump()?;

    let mut game = Game::new();

    println!("Keys:");
    println!("    Up    : Move player up");
    println!("    Down  : Move player down");
    println!("    Left  : Move player left");
    println!("    Right : Move player right");
    println!("    Space : Restart when game over");

    'running: loop {
        let started = SystemTime::now();

        let mut command = Command::None;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    if game.is_over {
                        game = Game::new();
                    }
                }
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    command = match code {
                        Keycode::Left => Command::Left,
                        Keycode::Right => Command::Right,
                        Keycode::Up => Command::Up,
                        Keycode::Down => Command::Down,
                        _ => Command::None,
                    };
                }
                _ => {}
            }
        }
        game.update(command);
        render(&mut canvas, &game, &mut resources)?;

        play_sounds(&mut game, &resources);

        let finished = SystemTime::now();
        let elapsed = finished.duration_since(started).unwrap();
        let frame_duration = Duration::new(0, 1_000_000_000u32 / FPS);
        if elapsed < frame_duration {
            ::std::thread::sleep(frame_duration - elapsed)
        }
    }

    Ok(())
}

fn init_mixer() {
    let chunk_size = 1_024;
    mixer::open_audio(
        mixer::DEFAULT_FREQUENCY,
        mixer::DEFAULT_FORMAT,
        mixer::DEFAULT_CHANNELS,
        chunk_size,
    )
    .expect("cannot open audio");
    let _mixer_context = mixer::init(mixer::InitFlag::MP3).expect("cannot init mixer");
}

fn load_resources<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    canvas: &mut Canvas<Window>,
) -> Resources<'a> {
    let mut resources = Resources {
        images: HashMap::new(),
        chunks: HashMap::new(),
    };

    // create head texture
    let mut head_texture = texture_creator
        .create_texture(
            None,
            sdl2::render::TextureAccess::Target,
            CELL_SIZE as u32,
            CELL_SIZE as u32,
        )
        .unwrap();
    canvas
        .with_texture_canvas(&mut head_texture, |texture_canvas| {
            texture_canvas.set_draw_color(Color::RGBA(0, 255, 0, 255));
            texture_canvas
                .draw_line(Point::new(9, 0), Point::new(2, 19))
                .unwrap();
            texture_canvas
                .draw_line(Point::new(2, 19), Point::new(16, 19))
                .unwrap();
            texture_canvas
                .draw_line(Point::new(16, 19), Point::new(9, 0))
                .unwrap();
        })
        .unwrap();
    let head_image = Image::new(head_texture);
    resources.images.insert("head".to_string(), head_image);

    // create body texture
    let body_texture_size = 20;
    let mut body_texture = texture_creator
        .create_texture(
            None,
            sdl2::render::TextureAccess::Target,
            body_texture_size,
            body_texture_size,
        )
        .unwrap();
    canvas
        .with_texture_canvas(&mut body_texture, |texture_canvas| {
            texture_canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
            texture_canvas
                .draw_rect(Rect::new(0, 0, body_texture_size, body_texture_size))
                .unwrap();
        })
        .unwrap();
    let body_image = Image::new(body_texture);
    resources.images.insert("body".to_string(), body_image);

    let image_paths = ["numbers.bmp"];
    for path in image_paths {
        let full_path = "resources/image/".to_string() + path;
        let temp_surface = sdl2::surface::Surface::load_bmp(Path::new(&full_path)).unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&temp_surface)
            .expect(&format!("cannot load image: {}", path));

        let image = Image::new(texture);
        resources.images.insert(path.to_string(), image);
    }

    let sound_paths = ["crash.wav"];
    for path in sound_paths {
        let full_path = "resources/sound/".to_string() + path;
        let chunk =
            mixer::Chunk::from_file(full_path).expect(&format!("cannot load sound: {}", path));
        resources.chunks.insert(path.to_string(), chunk);
    }

    resources
}

fn render(
    canvas: &mut Canvas<Window>,
    game: &Game,
    resources: &mut Resources,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    // render player
    let head = resources.images.get_mut("head").unwrap();
    canvas
        .copy_ex(
            &head.texture,
            None,
            Rect::new(
                game.player.p.x * CELL_SIZE,
                game.player.p.y * CELL_SIZE,
                CELL_SIZE as u32,
                CELL_SIZE as u32,
            ),
            game.player.get_angle() as f64, /* SDLのangleは時計回りが正 */
            Point::new(CELL_SIZE / 2, CELL_SIZE / 2),
            false,
            false,
        )
        .unwrap();

    if game.is_over {
        canvas.set_draw_color(Color::RGBA(255, 0, 0, 128));
        canvas.fill_rect(Rect::new(0, 0, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32))?;
    }

    render_number(
        canvas,
        resources,
        SCREEN_WIDTH as i32 - 8 * 8,
        0,
        format!("{0: >8}", game.score),
    );

    canvas.present();

    Ok(())
}

fn render_number(
    canvas: &mut Canvas<Window>,
    resources: &Resources,
    x: i32,
    y: i32,
    numstr: String,
) {
    let mut x = x;
    let image = resources.images.get("numbers.bmp").unwrap();
    let digit_width_in_px = 8;
    for c in numstr.chars() {
        if 0x30 <= c as i32 && c as i32 <= 0x39 {
            canvas
                .copy(
                    &image.texture,
                    Rect::new(
                        digit_width_in_px * (c as i32 - 0x30),
                        0,
                        digit_width_in_px as u32,
                        image.h,
                    ),
                    Rect::new(x, y, digit_width_in_px as u32, image.h),
                )
                .unwrap();
        }
        x += digit_width_in_px;
    }
}

fn play_sounds(game: &mut Game, resources: &Resources) {
    for sound_key in &game.requested_sounds {
        let chunk = resources
            .chunks
            .get(&sound_key.to_string())
            .expect("cannot get sound");
        sdl2::mixer::Channel::all()
            .play(&chunk, 0)
            .expect("cannot play sound");
    }
    game.requested_sounds = Vec::new();
}
