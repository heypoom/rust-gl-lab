extern crate image;

use std::io::Cursor;
use std::time::{Duration, Instant};

use glium::{Display, implement_vertex, index, IndexBuffer, Program, Surface, texture, uniform, VertexBuffer};
use glium::glutin::ContextBuilder;
use glium::index::PrimitiveType;
use glium::texture::RawImage2d;
use image::ImageFormat;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod teapot;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

fn into_tex(bytes: &[u8], display: &Display) -> texture::SrgbTexture2d {
    let cur = Cursor::new(&bytes);
    let img = image::load(cur, ImageFormat::Png).unwrap().to_rgba8();
    let dim = img.dimensions();
    let img = RawImage2d::from_raw_rgba_reversed(&img.into_raw(), dim);

    texture::SrgbTexture2d::new(display, img).unwrap()
}

fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new();
    let context_builder = ContextBuilder::new();

    let display = Display::new(window_builder, context_builder, &event_loop).unwrap();
    let display_window_id = display.gl_window().window().id();

    let mut t: f32 = -0.5;

    let positions = VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
    let normals = VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
    let indices = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &teapot::INDICES).unwrap();

    // let shape = vec![
    //     Vertex { position: [-0.5, -0.5], tex_coords: [0., 1.] },
    //     Vertex { position: [0.0, 0.5], tex_coords: [0., 1.] },
    //     Vertex { position: [0.5, -0.25], tex_coords: [1., 0.] },
    // ];
    //
    // let vertex_buffer = VertexBuffer::new(&display, &shape).unwrap();
    // let indices = index::NoIndices(index::PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
        #version 140

        in vec3 position;
        in vec3 normal;

        uniform mat4 matrix;

        void main() {
            gl_Position = matrix * vec4(position, 1.0);
        }
    "#;


    let fragment_shader_src = r#"
        #version 140

        out vec4 color;

        void main() {
            color = vec4(0.73, 0.86, 0.35, 1.0);
        }
    "#;

    let program = Program::from_source(
        &display,
        vertex_shader_src,
        fragment_shader_src,
        None,
    ).unwrap();

    let tex = into_tex(include_bytes!("../tex.png"), &display);

    event_loop.run(move |event, _, control_flow| {
        let mut target = display.draw();

        target.clear_color(0.42, 0.69, 0.3, 1.0);

        let matrix = [
            [0.01, 0.0, 0.0, 0.0],
            [0.0, 0.01, 0.0, 0.0],
            [0.0, 0.0, 0.01, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ];

        target.draw((&positions, &normals), &indices, &program, &uniform! { matrix: matrix },
                    &Default::default()).unwrap();

        target.finish().unwrap();

        let next_frame_time = Instant::now() +
            Duration::from_millis(16);

        *control_flow = ControlFlow::WaitUntil(next_frame_time);

        t += 0.0002;

        // if t > 0.5 {
        //     t = -0.5;
        // }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == display_window_id => {
                println!("event loop - exit");

                *control_flow = ControlFlow::Exit
            }
            _ => (),
        }
    });
}
