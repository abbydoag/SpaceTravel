use nalgebra::ComplexField;
use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
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
mod audio;
mod spaceship;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use triangle::triangle;
use shaders::{vertex_shader, fragment_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType};
use rand::Rng;
use audio::AudioPlayer;
use spaceship::Spaceship;

//planetas
#[derive(PartialEq)]
enum Planet{
    None
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

//limites
fn collision(position: Vec3, planet_position: Vec3, planet_radius: f32) -> bool{
    let distance = (position - planet_position).norm();
    distance < planet_radius
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

    // Música
    let audio_player = AudioPlayer::new("assets/music/September.mp3");
    audio_player.play();

    // Nave
    let mut spaceship = Spaceship::new(Vec3::new(0.0, 0.0, 4.0));

    // Cámara
    let mut camera = Camera::new(
        Vec3::new(0.0, 5.0, -10.0),
        spaceship.position,
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut zoom_level: f32 = 10.0;
    let zoom_speed = 1.0;

    // Planetas cordenandas
    let planet_positions = vec![
        Vec3::new(7.3, 0.0, 4.0),
        Vec3::new(-4.0, 5.0, -8.0),
        Vec3::new(8.0, -3.0, -3.0), 
        Vec3::new(-2.0, 2.0, 6.0),
        Vec3::new(2.0, 3.0, 1.0)
    ];

    let num_stars = 80;
    render_background(&mut framebuffer, num_stars);

    //modelos
    let obj = Obj::load("assets/models/sphere.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array();
    let ship = Obj::load("assets/models/nave.obj").expect("Failed to load obj");
    let ship_vertex_arrays = ship.get_vertex_array();
    let mut time = 0;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        time += 1;

        handle_input(&window, &mut spaceship, &mut camera,  &planet_positions);

        framebuffer.clear();

        //planetas
        for (i, &position) in planet_positions.iter().enumerate() {
            let model_matrix = create_model_matrix(position, 1.0, Vec3::new(0.0, 0.0, 0.0));
            let planet_shader = match i {
                0 => "continents_shader",
                1 => "another_shader",
                2 => "gradient_shader",
                3 => "lava_shader",
                4 => "lines_shader",
                _ => "default_shader",
            };

            let uniforms = Uniforms {
                model_matrix,
                view_matrix: create_view_matrix(camera.eye, camera.center, camera.up),
                projection_matrix: create_perspective_matrix(window_width as f32, window_height as f32),
                viewport_matrix: create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32),
                time,
                noise: create_noise(),
            };

            render(&mut framebuffer, &uniforms, &vertex_arrays, planet_shader);
        }

        //Render nave
        let model_matrix = create_model_matrix(spaceship.position, 1.0, spaceship.forward);
        let uniforms = Uniforms {
            model_matrix,
            view_matrix: create_view_matrix(camera.eye, camera.center, camera.up),
            projection_matrix: create_perspective_matrix(window_width as f32, window_height as f32),
            viewport_matrix: create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32),
            time,
            noise: create_noise(),
        };
        render(&mut framebuffer, &uniforms, &ship_vertex_arrays, "spaceship_shader");

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

fn handle_input(window: &Window, spaceship: &mut Spaceship, camera: &mut Camera, planet_positions: &[Vec3]) {
    let movement_speed = 0.1;
    let rotation_speed = 0.1;
    let planet_radius = 0.9;
    // Movimiento de la nave
    if window.is_key_down(Key::Up) {
        let new_position = spaceship.position - spaceship.forward * movement_speed;
        if !planet_positions.iter().any(|&planet_position| collision(new_position, planet_position, planet_radius)) {
            spaceship.move_forward(-movement_speed);
        }
    }
    if window.is_key_down(Key::Down) {
        let new_position = spaceship.position + spaceship.forward * movement_speed;
        if !planet_positions.iter().any(|&planet_position| collision(new_position, planet_position, planet_radius)) {
            spaceship.move_forward(movement_speed);
        }
    }
    //giro
    if window.is_key_down(Key::Right) {
        spaceship.rotate(-rotation_speed); 
    }
    if window.is_key_down(Key::Left) {
        spaceship.rotate(rotation_speed);
    }

    let mut movement = Vec3::new(0.0, 0.0, 0.0); // Movimiento 3D
    // Verificacion colisiones
    let new_camera_position = camera.eye + movement;
    if !planet_positions.iter().any(|&planet_position| collision(new_camera_position, planet_position, planet_radius)) {
        camera.eye = new_camera_position;
    }
    camera.center = spaceship.position;
    camera.eye = spaceship.position + spaceship.forward * 10.0;
}