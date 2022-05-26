extern crate image;

use std::f32::consts::FRAC_PI_3;
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

    let fov: f32 = FRAC_PI_3;
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

fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new();
    let context_builder = ContextBuilder::new().with_depth_buffer(24);

    let display = Display::new(window_builder, context_builder, &event_loop).unwrap();
    let display_window_id = display.gl_window().window().id();

    let mut t: f32 = -0.5;

    let positions = VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
    let normals = VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
    let indices = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &teapot::INDICES).unwrap();

    let vertex_shader_src = r#"
        #version 150

        in vec3 position;
        in vec3 normal;

        out vec3 v_normal;

        uniform mat4 perspective;
        uniform mat4 matrix;

        void main() {
            v_normal = transpose(inverse(mat3(matrix))) * normal;
            gl_Position = perspective * matrix * vec4(position, 1.0);
        }
    "#;


    let fragment_shader_src = r#"
        #version 140

        in vec3 v_normal;
        out vec4 color;

        uniform vec3 u_light;

        void main() {
            float brightness = dot(normalize(v_normal), normalize(u_light));
            vec3 dark_color = vec3(0.6, 0.0, 0.0);
            vec3 regular_color = vec3(1.0, 0.0, 0.0);
            color = vec4(mix(dark_color, regular_color, brightness), 1.0);
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
        let perspective = perspective_matrix(target.get_dimensions());

        target.clear_color_and_depth((0.42, 0.69, 0.3, 1.0), 1.0);

        let matrix = [
            [0.01, 0.0, 0.0, 0.0],
            [0.0, 0.01, 0.0, 0.0],
            [0.0, 0.0, 0.01, 0.0],
            [0.0, 0.0, 2.0, 1.0f32]
        ];

        let uniforms = uniform! {
            matrix: matrix,
            u_light: [-1.0, 0.4, 0.9f32],
            perspective: perspective
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
