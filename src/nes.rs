use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::ppu::PPU;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, TextureAccess};
use sdl2::{pixels::PixelFormatEnum, video::Window};
use std::cell::RefCell;
use std::rc::Rc;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;

pub struct NES {
    cpu: CPU,
    ppu: Rc<RefCell<PPU>>,
}

impl NES {
    pub fn new(path: impl AsRef<str>) -> Self {
        let cartridge = Cartridge::from_path(path.as_ref()).unwrap();
        let cartridge = Rc::new(RefCell::new(cartridge));

        let ppu = PPU::new(cartridge.clone());
        let ppu = Rc::new(RefCell::new(ppu));

        let cpu = CPU::new(cartridge, ppu.clone());
        Self { cpu, ppu }
    }

    fn tick(&mut self) {
        let cycles = self.cpu.tick();
        let mut ppu = self.ppu.borrow_mut();
        ppu.tick(cycles);
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let sdl_context = sdl2::init()?;
        let video_subsystem: sdl2::VideoSubsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("NESMULATOR", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .opengl()
            .build()?;

        gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

        let mut event_pump = sdl_context.event_pump()?;
        let mut canvas: Canvas<Window> = window.into_canvas().accelerated().build()?;

        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator.create_texture(
            PixelFormatEnum::BGR24,
            TextureAccess::Streaming,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )?;

        'running: loop {
            self.tick();

            {
                let ppu = self.ppu.borrow();
                let screen = ppu.screen;
                if ppu.frame_complete {
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
                        self.ppu.borrow_mut().set_nmi();
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
        }

        Ok(())
    }
}