
use nalgebra_glm::{Vec3, Vec4, Mat3, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

    let w = transformed.w;
    let transformed_position = Vec4::new(
        transformed.x / w,
        transformed.y / w,
        transformed.z / w,
        1.0
    );

    let screen_position = uniforms.viewport_matrix * transformed_position;

    let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

    let transformed_normal = normal_matrix * vertex.normal;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: transformed_normal
    }
}
//shader a usar
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, shader_type: &str) -> Color {
  match shader_type {
    "lines_shader" => lines_shader(fragment, uniforms),
    "lava_shader" => lava_shader(fragment, uniforms),
    "gradient_shader"=> gradient_shader(fragment, uniforms),
    "continents_shader" => continents_shader(fragment, uniforms),
    "spaceship_shader" => spaceship_shader(fragment, uniforms),
    "another_shader" => another_shader(fragment, uniforms),
    _ => Color::new(0, 0, 0),
  }
}

fn lines_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let seed = uniforms.time as f32 * fragment.vertex_position.y * fragment.vertex_position.x;
  
    let mut rng = StdRng::seed_from_u64(seed.abs() as u64);
  
    let random_number = rng.gen_range(0..=100);
  
    let color1_or_color2 = if random_number < 40 {
      Color::new(92, 137, 182)
    } else {
      Color::new(188, 67, 67)
    };
  
    color1_or_color2 * fragment.intensity
}

fn lava_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Base colors for the lava effect
    let bright_color = Color::new(255, 226, 107); // Bright orange (lava-like)
    let dark_color = Color::new(130, 20, 0);   // Darker red-orange
  
    // Get fragment position
    let position = Vec3::new(
      fragment.vertex_position.x,
      fragment.vertex_position.y,
      fragment.depth
    );
  
    // Base frequency and amplitude for the pulsating effect
    let base_frequency = 0.2;
    let pulsate_amplitude = 0.5;
    let t = uniforms.time as f32 * 0.01;
  
    // Pulsate on the z-axis to change spot size
    let pulsate = (t * base_frequency).sin() * pulsate_amplitude;
  
    // Apply noise to coordinates with subtle pulsating on z-axis
    let zoom = 1000.0; // Constant zoom factor
    let noise_value1 = uniforms.noise.get_noise_3d(
      position.x * zoom,
      position.y * zoom,
      (position.z + pulsate) * zoom
    );
    let noise_value2 = uniforms.noise.get_noise_3d(
      (position.x + 1000.0) * zoom,
      (position.y + 1000.0) * zoom,
      (position.z + 1000.0 + pulsate) * zoom
    );
    let noise_value = (noise_value1 + noise_value2) * 0.5;  // Averaging noise for smoother transitions
  
    // Use lerp for color blending based on noise value
    let color = dark_color.lerp(&bright_color, noise_value);
    //Brillo
    let glow_factor = 2.0; //Intensidas
    let glowing_color = color * glow_factor;
    let glow_edge = Color::new(198, 33, 0) * (1.0 - noise_value); // White edge for glow
    let final_color = glowing_color + glow_edge;
  
    final_color
}

fn gradient_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let _ = uniforms;
  let gradient_start = Color::new(0, 0, 255); // Color 1
  let gradient_end = Color::new(255, 0, 0);   //Color 2

  let t = (fragment.vertex_position.y + 1.0) * 0.5;
  let color = gradient_start.lerp(&gradient_end, t);

  color * fragment.intensity
}

fn continents_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let zoom = 95.0;  
  let ox = 45.0;    
  let oy = 45.0;
  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;
  let t = uniforms.time as f32 * 0.3; // Velocidad rotacion

  //Rotacion
  let noise_value = uniforms.noise.get_noise_2d(x * zoom + ox + t, y * zoom + oy);

  let land_color = Color::new(34, 139, 34);
  let ocean_color = Color::new(0, 0, 255);

  let land_threshold = 0.14;

  let terrain_color = if noise_value > land_threshold {
      land_color
  } else {
      ocean_color
  };

  terrain_color * fragment.intensity
}

fn another_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let base_color = Color::new(72, 37, 159);
  let rocky_color = Color::new(139, 69, 19);

  let zoom = 100.0;
  let noise_value = uniforms.noise.get_noise_2d(
      fragment.vertex_position.x * zoom,
      fragment.vertex_position.y * zoom,
  );
  let terrain_color = base_color.lerp(&rocky_color, noise_value);

  let hemisphere_factor = (fragment.vertex_position.y + 1.0) * 0.5;
  let blended_color = terrain_color.lerp(&Color::new(255, 222, 173), hemisphere_factor); // Arena clara

  blended_color * fragment.intensity
}

fn spaceship_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let base_color = Color::new(100, 100, 255); // Azul metálico base
  let highlight_color = Color::new(200, 200, 255); // Azul brillante para detalles

  let t = (fragment.vertex_position.y + 1.0) * 0.5;
  let blended_color = base_color.lerp(&highlight_color, t);

  let pattern = ((fragment.vertex_position.x * 10.0).sin()
      + (fragment.vertex_position.y * 10.0).cos()) * 0.5 + 0.5;
  let patterned_color = blended_color * pattern;

  patterned_color * fragment.intensity
}