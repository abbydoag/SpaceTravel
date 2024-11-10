use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use triangle::triangle;
use shaders::{vertex_shader, fragment_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};
use rand::Rng;

//planetas
#[derive(PartialEq)]
enum Planet{
    None,
    PlainPlanet,
    RingPlanet,
    GasGigant,
    Sun,
    LinesPlanet,
    GradientPlanet,
    DotPlanet
}
pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise: FastNoiseLite
}

fn create_noise() -> FastNoiseLite {
    create_cloud_noise()
}

fn create_cloud_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}


fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], shader_type: &str) {
    // Vertex Shader
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Rasterization
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    // Fragment Processing
    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;

        if x < framebuffer.width && y < framebuffer.height {
            let shaded_color = fragment_shader(&fragment, &uniforms, shader_type);
            let color = shaded_color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn render_background(framebuffer: &mut Framebuffer, num_stars: u32) {
    let mut rng = rand::thread_rng();

    for _ in 0..num_stars {
        let x = rng.gen_range(0..framebuffer.width);
        let y = rng.gen_range(0..framebuffer.height);

        framebuffer.set_background_star(x, y, 0xFFFFFF);
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);


    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Space Travel",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(200, 40);
    window.update();

    framebuffer.set_background_color(0x151515);

    // Model parameters (translation, scale, rotation)
    let translation = Vec3::new(0.0, 0.0, 0.0);
    let rotation = Vec3::new(0.0, 0.0, 0.0);
    let scale = 1.0f32;

    // Camera parameters
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let num_stars = 40;
    render_background(&mut framebuffer, num_stars); 

    // Load 3D Objects
    let obj = Obj::load("assets/models/sphere.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array();
    let rings_obj = Obj::load("assets/models/rings.obj").expect("Failed to load rings obj");
    let rings_vertex_arrays = rings_obj.get_vertex_array();

    let mut current_planet = Planet::None;
    let mut time = 0;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        time += 1;

        handle_input(&window, &mut camera, &mut current_planet);

        framebuffer.clear(); 

        let noise = create_noise();
        let model_matrix = create_model_matrix(translation, scale, rotation);
        let view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
        let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);
        let uniforms = Uniforms {
            model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            noise
        };

        framebuffer.set_current_color(0xFFDDDD);
        match current_planet {
            Planet::PlainPlanet => {
                render(&mut framebuffer, &uniforms, &vertex_arrays, "continents_shader");
            }
            Planet::RingPlanet => {
                render(&mut framebuffer, &uniforms, &vertex_arrays, "spiral_shader");
                render(&mut framebuffer, &uniforms, &rings_vertex_arrays, "rings_shader");
            }
            Planet::GasGigant => {
                render(&mut framebuffer, &uniforms, &vertex_arrays, "cellular_shader");
            }
            Planet::Sun => {
                render(&mut framebuffer, &uniforms, &vertex_arrays, "lava_shader");
            }
            Planet::GradientPlanet => {
                render(&mut framebuffer, &uniforms, &vertex_arrays, "gradient_shader");
            }
            Planet::LinesPlanet => {
                render(&mut framebuffer, &uniforms, &vertex_arrays, "lines_shader");
            }
            Planet::DotPlanet => {
                render(&mut framebuffer, &uniforms, &vertex_arrays, "dalmata_shader");
            }
            Planet::None => {}
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}


fn handle_input(window: &Window, camera: &mut Camera, current_planet: &mut Planet) {
    let movement_speed = 1.0;
    let rotation_speed = PI/50.0;
    let zoom_speed = 0.1;
   
    //  camera orbit controls
    if window.is_key_down(Key::Left) {
      camera.orbit(rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
      camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::W) {
      camera.orbit(0.0, -rotation_speed);
    }
    if window.is_key_down(Key::S) {
      camera.orbit(0.0, rotation_speed);
    }

    // Camera movement controls
    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    if window.is_key_down(Key::A) {
      movement.x -= movement_speed;
    }
    if window.is_key_down(Key::D) {
      movement.x += movement_speed;
    }
    if window.is_key_down(Key::Q) {
      movement.y += movement_speed;
    }
    if window.is_key_down(Key::E) {
      movement.y -= movement_speed;
    }
    if movement.magnitude() > 0.0 {
      camera.move_center(movement);
    }

    // Camera zoom controls
    if window.is_key_down(Key::Up) {
      camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::Down) {
      camera.zoom(-zoom_speed);
    }

    //Planet render
    if window.is_key_pressed(Key::R, KeyRepeat::No) {
        *current_planet = if *current_planet == Planet::PlainPlanet { Planet::None } else { Planet::PlainPlanet };
    }
    if window.is_key_pressed(Key::T, KeyRepeat::No) {
        *current_planet = if *current_planet == Planet::RingPlanet { Planet::None } else { Planet::RingPlanet };
    }
    if window.is_key_pressed(Key::Y, KeyRepeat::No) {
        *current_planet = if *current_planet == Planet::GasGigant { Planet::None } else { Planet::GasGigant};
    }
    if window.is_key_pressed(Key::U, KeyRepeat::No) {
        *current_planet = if *current_planet == Planet::GradientPlanet { Planet::None } else { Planet::GradientPlanet };
    }
    if window.is_key_pressed(Key::I, KeyRepeat::No) {
        *current_planet = if *current_planet == Planet::LinesPlanet { Planet::None } else { Planet::LinesPlanet};
    }
    if window.is_key_pressed(Key::F, KeyRepeat::No) {
        *current_planet = if *current_planet == Planet::Sun { Planet::None } else { Planet::Sun };
    }
    if window.is_key_pressed(Key::G, KeyRepeat::No) {
        *current_planet = if *current_planet == Planet::DotPlanet { Planet::None } else { Planet::DotPlanet};
    }
}