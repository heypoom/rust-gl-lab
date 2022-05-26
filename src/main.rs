extern crate image;

use std::io::Cursor;
use std::time::{Duration, Instant};

use glium::{Depth, DepthTest, Display, DrawParameters, implement_vertex, IndexBuffer, Program, Surface, texture, uniform, VertexBuffer};
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

fn perspective_matrix((width, height): (u32, u32)) -> [[f32; 4]; 4] {
    let aspect_ratio = height as f32 / width as f32;

    let fov: f32 = 3.141592 / 3.0;
    let z_far = 1024.0;
    let z_near = 0.1;

    let f = 1.0 / (fov / 2.0).tan();

    [
        [f * aspect_ratio, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (z_far + z_near) / (z_far - z_near), 1.0],
        [0.0, 0.0, -(2.0 * z_far * z_near) / (z_far - z_near), 0.0],
    ]
}

fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();

        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [up[1] * f[2] - up[2] * f[1],
        up[2] * f[0] - up[0] * f[2],
        up[0] * f[1] - up[1] * f[0]];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
        f[2] * s_norm[0] - f[0] * s_norm[2],
        f[0] * s_norm[1] - f[1] * s_norm[0]];

    let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
        -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
        -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new();
    let context_builder = ContextBuilder::new().with_depth_buffer(24);

    let display = Display::new(window_builder, context_builder, &event_loop).unwrap();
    let display_window_id = display.gl_window().window().id();

    let positions = VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
    let normals = VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
    let indices = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &teapot::INDICES).unwrap();

    let vertex_shader_src = include_str!("./shaders/vert.glsl");
    let fragment_shader_src = include_str!("./shaders/frag.glsl");

    let program = Program::from_source(
        &display,
        vertex_shader_src,
        fragment_shader_src,
        None,
    ).unwrap();

    event_loop.run(move |event, _, control_flow| {
        let mut target = display.draw();
        target.clear_color_and_depth((0.42, 0.69, 0.3, 1.0), 1.0);

        let model = [
            [0.01, 0.0, 0.0, 0.0],
            [0.0, 0.01, 0.0, 0.0],
            [0.0, 0.0, 0.01, 0.0],
            [0.0, 0.0, 2.0, 1.0f32]
        ];

        let view = view_matrix(&[2.0, -1.0, 1.0], &[-2.0, 1.0, 1.0], &[0.0, 1.0, 0.0]);

        let perspective = perspective_matrix(target.get_dimensions());

        let light = [-1.0, 0.4, 0.9f32];

        let uniforms = uniform! {
            model: model,
            view: view,
            perspective: perspective,
            u_light: light,
        };

        let params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        target.draw((&positions, &normals), &indices, &program, &uniforms, &params).unwrap();

        target.finish().unwrap();

        let next_frame_time = Instant::now() +
            Duration::from_millis(16);

        *control_flow = ControlFlow::WaitUntil(next_frame_time);


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
