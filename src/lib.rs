use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder, dpi::PhysicalSize,
};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

mod render;
mod systems;

use systems::GameState;

use crate::systems::SCREEN_SIZE;

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }    

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Solitaire")
        .with_inner_size(PhysicalSize { width: SCREEN_SIZE.x, height: SCREEN_SIZE.y})
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")] {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(SCREEN_SIZE.x, SCREEN_SIZE.y));
        
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut render_state = pollster::block_on(render::new(window));

    let mut game_state = GameState::new();

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == render_state.window().id() => {
            game_state.update();
            render_state.update(&game_state);
            match render_state.render() {
                Ok(_) => {},
                Err(wgpu::SurfaceError::Lost) => render_state.resize(render_state.size()),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e)
            }
        },
        Event::MainEventsCleared => {
            render_state.window().request_redraw();
        },
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == render_state.window().id() => if !game_state.input(event) {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    render_state.resize(*physical_size);
                },
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    render_state.resize(**new_inner_size);
                },
                _ => {}
            }
        },
        _ => {}
    });
}