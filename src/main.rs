#[macro_use]
extern crate glium;

pub const VERT_FULL_SCREEN_QUAD: [IVertex;3] = [
    // top left
    IVertex {
        coordinate: [-1.0, 1.0],
        tex_coords: [-1.0, 1.0],
    },
    // bottom left
    IVertex {
        coordinate: [-1.0, -1.0],
        tex_coords: [-1.0, -1.0],
    },
    // top right
    IVertex {
        coordinate: [1.0, 1.0],
        tex_coords: [1.0, 1.0],
    },
];

pub const INDICES_FULL_SCREEN_QUAD: glium::index::NoIndices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);


#[derive(Copy, Clone)]
pub struct IVertex {
    pub coordinate: [f32; 2],
    pub tex_coords: [f32; 2],
}

implement_vertex!(IVertex, coordinate, tex_coords);

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

implement_vertex!(Vertex, position, color);

fn main() {

    use glium::Surface;
    let initial_width = 1024;
    let initial_height = 768;

    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(initial_width, initial_height)
        .with_title("2055 - The Flood");
    let context = glium::glutin::ContextBuilder::new()
                    .with_gl_profile(glium::glutin::GlProfile::Core)
                    .with_multisampling(0)
                    .with_srgb(true);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    // ---- begin draw

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {

        glium::VertexBuffer::new(&display,
            &[
                Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
                Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
            ]
        ).unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
                                               &[0u16, 1, 2]).unwrap();

    let vertex_shader = "
        #version 140

        in vec2 position;
        in vec3 color;
        out vec3 vColor;
        uniform float t;

        void main() {
            gl_Position = vec4(position. x - t, position.y - t, 0.0, 1.0);
            vColor = color;
        }
    ";

    let vertex_shader_full_screen = "
        #version 140
        out vec2 v_tex_coords;
        void main() {
            float x = -1.0 + float((gl_VertexID & 1) << 2);
            float y = -1.0 + float((gl_VertexID & 2) << 1);
            v_tex_coords = vec2((x+1.0)*0.5, (y+1.0)*0.5);
            gl_Position = vec4(x, y, 0, 1);
        }
    ";

    let fragment_shader_full_screen = "
        #version 130

        in vec2 v_tex_coords;
        uniform sampler2D tex;
        out vec4 color;

        void main() {
            color = texture(tex, v_tex_coords);
        }
    ";

    let fragment_shader = "
        #version 140
        in vec3 vColor;

        // 8x8 Bayer ordered dithering
        // pattern. Each input pixel
        // is scaled to the 0..63 range
        // before looking in this table
        // to determine the action.
        const int dither[64] = int[64](
             0, 32, 8, 40, 2, 34, 10, 42,
             48, 16, 56, 24, 50, 18, 58, 26,
             12, 44, 4, 36, 14, 46, 6, 38,
             60, 28, 52, 20, 62, 30, 54, 22,
              3, 35, 11, 43, 1, 33, 9, 41,
             51, 19, 59, 27, 49, 17, 57, 25,
             15, 47, 7, 39, 13, 45, 5, 37,
             63, 31, 55, 23, 61, 29, 53, 21
        );

        float find_closest(int x, int y, float c0) {
            float limit = 0.0;
            if(x < 8) {
                limit = (dither[x + (y * 8)] + 1) / 64.0;
            }

            float diff = c0 - limit;
            if(diff < 0.0) {
                return 0.0;
            } else {
                return 1.0;
            }
        }

        void main() {
            bool grayscale = false;
            int x = int(mod(gl_FragCoord.x, 8));
            int y = int(mod(gl_FragCoord.y, 8));

            if (grayscale) {
                vec3 sepia = vec3(0.299, 0.587, 0.114);
                float grayscale = dot(vColor, sepia);
                gl_FragColor = vec4(vec3(find_closest(x, y, grayscale)), 1.0);
            } else {
                gl_FragColor = vec4(vec3(find_closest(x, y, vColor.r),
                                         find_closest(x, y, vColor.g),
                                         find_closest(x, y, vColor.b)), 1.0);
            }
        }
    ";

    let program = glium::Program::from_source(&display, vertex_shader, fragment_shader, None).unwrap();
    let program_full_screen = glium::Program::from_source(&display, vertex_shader_full_screen, fragment_shader_full_screen, None).unwrap();

    let mut t: f32 = -0.5;
    let mut window_is_closed = false;
    let full_screen_vertex_buffer = glium::VertexBuffer::new(&display, &VERT_FULL_SCREEN_QUAD).unwrap();
    let mut window_width = initial_width;
    let mut window_height = initial_height;

    while !window_is_closed {

        events_loop.poll_events(|event| {
            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    glium::glutin::WindowEvent::Closed => { window_is_closed = true; },
                    glium::glutin::WindowEvent::Resized(w, h) => { window_width = w; window_height = h; },
                    _  => { },
                },
                _ => {  },
            }
        });

        t += 0.002;
        if t > 0.5 {
            t = -0.5;
        }

        let uniforms = uniform! { t: t };
        let texture_half_size = glium::Texture2d::empty(&display, window_width / 4, window_height / 4).unwrap();
        texture_half_size.as_surface().clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        texture_half_size.as_surface().draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();

        let uniforms_full_screen = uniform! {
            tex: texture_half_size.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(
            &full_screen_vertex_buffer,
            &INDICES_FULL_SCREEN_QUAD,
            &program_full_screen,
            &uniforms_full_screen,
            &Default::default()
        ).unwrap();
        target.finish().unwrap();

        ::std::thread::sleep(::std::time::Duration::from_millis(16));
    }
}
