use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::ppu::PPU;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, TextureAccess};
use sdl2::{
    pixels::PixelFormatEnum,
    video::{GLProfile, Window},
};
use std::sync::{Arc, RwLock};

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;

pub struct NES {
    cpu: CPU,
    ppu: Arc<RwLock<PPU>>,
}

impl NES {
    pub fn new(path: impl AsRef<str>) -> Self {
        let cartridge = Cartridge::from_path(path.as_ref()).unwrap();
        let cartridge = Arc::new(RwLock::new(cartridge));

        let ppu = PPU::new(cartridge.clone());
        let ppu = Arc::new(RwLock::new(ppu));

        let cpu = CPU::new(cartridge, ppu.clone());
        Self { cpu, ppu }
    }

    fn tick(&mut self) {
        let cycles = self.cpu.tick();
        let mut ppu = self.ppu.write().unwrap();
        ppu.tick(cycles);
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let sdl_context = sdl2::init()?;
        let video_subsystem: sdl2::VideoSubsystem = sdl_context.video()?;

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);

        let window = video_subsystem
            .window("NESMULATOR", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .opengl()
            .build()?;

        gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (3, 3));

        let mut event_pump = sdl_context.event_pump()?;
        let mut canvas: Canvas<Window> = window.into_canvas().build()?;

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        'running: loop {
            {
                let ppu = self.ppu.read().unwrap();
                if ppu.get_vblank() {
                    let texture_creator = canvas.texture_creator();
                    let mut texture = texture_creator.create_texture(
                        PixelFormatEnum::BGR24,
                        TextureAccess::Streaming,
                        SCREEN_WIDTH as u32,
                        SCREEN_HEIGHT as u32,
                    )?;
                    let screen = ppu.screen;
                    texture.update(None, &screen, SCREEN_WIDTH * 3)?;

                    canvas.clear();
                    canvas.copy(&texture, None, None)?;
                    canvas.present();
                }
            }

            while let Some(event) = event_pump.poll_event() {
                match event {
                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        ..
                    } => {
                        self.ppu.write().unwrap().set_nmi();
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::R),
                        ..
                    } => {
                        self.cpu.reset();
                    }
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            self.tick();
        }

        Ok(())
    }
}
