mod state;
use state::State;

fn main() {
    cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        wasm_bindgen_futures::spawn_local(run());
    } else {
        pollster::block_on(run());
    }
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
    dpi::LogicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        let dimensions = (
            web_sys::window()
                .unwrap()
                .inner_width()
                .unwrap()
                .as_f64()
                .unwrap(),
            web_sys::window()
                .unwrap()
                .inner_height()
                .unwrap()
                .as_f64()
                .unwrap(),
        );
        window.set_inner_size(LogicalSize::new(dimensions.0, dimensions.1));
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("renderer")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        // Give the state a chance to handle the event. If it returns true, then we don't need to
        if state.handle_event(&event) {
            return;
        }

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                handle_window_event(event, control_flow, &mut state)
            }
            Event::RedrawRequested(window_id) if window_id == state.window.id() => {
                redraw(&mut state, control_flow);
            }
            Event::MainEventsCleared => state.window.request_redraw(),
            _ => {}
        }
    });
}

fn redraw(state: &mut State, control_flow: &mut ControlFlow) {
    match state.render() {
        Ok(_) => {}
        // Reconfigure the surface if it's lost or outdated
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.size),
        // The system is out of memory, we should probably quit
        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
        // We're ignoring timeouts
        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
    }
}

fn handle_window_event(event: &WindowEvent, control_flow: &mut ControlFlow, state: &mut State) {
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
            state.resize(*physical_size);
        }
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
            state.resize(**new_inner_size);
        }
        _ => {}
    }
}
