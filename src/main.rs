mod cpu;
use cpu::*;

use sdl2::event::Event;
use sdl2::libc::wopen;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;


fn main() {
    let mut cpu = Chip8::new();
    let sdl_context = sdl2::init().unwrap();
    cpu.load_rom("IBM LOGO.ch8");
    
    let emulated_width:u16 = 64;
    let emulated_height:u16 = 32;
    let video_scale:u16 = 5;
     
    let video = sdl_context.video().expect("Unable to initialize video");
    let window = video.window("Chip8 Emulator", (emulated_width * video_scale).into(), (emulated_height * video_scale).into())
        .position_centered()
        .opengl()
        .build()
        .expect("Unable to build window");

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    
    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit{..} => {
                    println!("Quitttt");
                    break 'gameloop;
                },
                Event::KeyDown{..} => (),
                _ => ()
            }
        }
        cpu.cycle();
        //println!("{:?}", cpu.get_display());
        
        draw_screen(&cpu, &mut canvas, video_scale as u32)
    }
}

fn draw_screen(cpu: &Chip8, canvas: &mut Canvas<Window>,scale: u32) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = cpu.get_display();
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel != 0 {
            println!("here 1");
            let x = (i % 64) as u32;
            let y = (i / 64) as u32;

            let rect = Rect::new((x * scale) as i32, (y * scale) as i32, scale, scale);
            //let rect = Rect::new(100, 100, 100, 100);
            println!("{:?}", rect);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}
