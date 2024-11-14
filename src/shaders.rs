
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
      "cloud_shader" => cloud_shader(fragment, uniforms),
      "dalmata_shader" => dalmata_shader(fragment, uniforms),
      "lines_shader" => lines_shader(fragment, uniforms),
      "cellular_shader" => cellular_shader(fragment, uniforms),
      "lava_shader" => lava_shader(fragment, uniforms),
      "gradient_shader"=> gradient_shader(fragment, uniforms),
      "continents_shader" => continents_shader(fragment, uniforms),
      "rings_shader" => rings_shader(fragment, uniforms),
      "spiral_shader" => spiral_shader(fragment, uniforms),
      "spaceship_shader" => spaceship_shader(fragment, uniforms),
      _ => Color::new(0, 0, 0),
  }
}

fn lines_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let seed = uniforms.time as f32 * fragment.vertex_position.y * fragment.vertex_position.x;
  
    let mut rng = StdRng::seed_from_u64(seed.abs() as u64);
  
    let random_number = rng.gen_range(0..=100);
  
    let black_or_white = if random_number < 50 {
      Color::new(92, 137, 182)
    } else {
      Color::new(188, 67, 67)
    };
  
    black_or_white * fragment.intensity
}
  
fn dalmata_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let zoom = 220.0; 
  let ox = 0.0;
  let oy = 0.0;
  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;

  let noise_value = uniforms.noise.get_noise_2d(
      (x + ox) * zoom,
      (y + oy) * zoom,
  );

  let spot_threshold = 0.2; 
  let spot_color = Color::new(159, 182, 255);
  let base_color = Color::new(30, 58, 55); 

  let noise_color = if noise_value < spot_threshold {
      spot_color
  } else {
      base_color
  };

  noise_color * fragment.intensity
}

fn cloud_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 100.0;  // to move our values 
    let ox = 100.0; // offset x in the noise map
    let oy = 100.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let t = uniforms.time as f32 * 0.5;
  
    let noise_value = uniforms.noise.get_noise_2d(x * zoom + ox + t, y * zoom + oy);
  
    // Define cloud threshold and colors
    let cloud_threshold = 0.5; // Adjust this value to change cloud density
    let cloud_color = Color::new(255, 255, 255); // White for clouds
    let sky_color = Color::new(30, 97, 145); // Sky blue
  
    // Determine if the pixel is part of a cloud or sky
    let noise_color = if noise_value > cloud_threshold {
      cloud_color
    } else {
      sky_color
    };
  
    noise_color * fragment.intensity
}
  
fn cellular_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let zoom = 30.0;  // Zoom factor to adjust the scale of the cell pattern
  let ox = 50.0;    // Offset x in the noise map
  let oy = 50.0;    // Offset y in the noise map
  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;

  // Use a cellular noise function to create the plant cell pattern
  let cell_noise_value = uniforms.noise.get_noise_2d(x * zoom + ox, y * zoom + oy).abs();

  // Define different shades of green for the plant cells
  let cell_color_1 = Color::new(91, 206, 219);  
  let cell_color_2 = Color::new(108, 146, 212);   
  let cell_color_3 = Color::new(69, 121, 91);   
  let cell_color_4 = Color::new(43, 0, 186);  

  // Use the noise value to assign a different color to each cell
  let final_color = if cell_noise_value < 0.15 {
    cell_color_1
  } else if cell_noise_value < 0.7 {
    cell_color_2
  } else if cell_noise_value < 0.75 {
    cell_color_3
  } else {
    cell_color_4
  };

  // Adjust intensity to simulate lighting effects (optional)
  final_color * fragment.intensity
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

fn rings_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let _ = uniforms;
  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;

  // Distancia al centro
  let distance = (x * x + y * y).sqrt();

  let color_blue = Color::new(0, 0, 255);       
  let color_silver = Color::new(192, 192, 192);
  let color_emerald = Color::new(80, 200, 120);

  let ring_width = 0.3; // Ancho de los anillos
  let ring_spacing = 0.6; // Espaciado entre anillos

  let ring_pattern = (distance / ring_spacing).floor();

  let base_color = if ring_pattern % 1.5 == 0.0 {
      color_blue
  } else if ring_pattern % 3.0 == 1.0 {
      color_silver
  } else {
      color_emerald
  };

  //Intensidad de los anillos con gradiente
  let normalized_distance = (distance % ring_spacing) / ring_width;

  let opacity = if normalized_distance < 1.0 {
      1.0 - normalized_distance 
  } else {
      0.0
  };

  let final_color = base_color * (opacity * fragment.intensity);

  final_color
}

fn spiral_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;

  // Calcula la distancia desde el centro
  let radius = (x * x + y * y).sqrt();

  let angle = y.atan2(x) + uniforms.time as f32 * 0.1; // Rotación
  let spiral_pattern = (radius * 3.0 + angle).sin() * 0.7; //frecuencia y amplitud

  let base_color = Color::new(227, 228, 229); 
  let spiral_color = Color::new(62, 95, 138);
  let threshold = 0.0;

  let final_color = if spiral_pattern > threshold {
      spiral_color
  } else {
      base_color
  };

  final_color * fragment.intensity
}

fn spaceship_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let light_dir = Vec3::new(0.5, -1.0, -0.5).normalize();  // Dirección de la luz
  let view_dir = Vec3::new(0.0, 0.0, 1.0).normalize();     // Dirección de la cámara
  let normal = fragment.normal.normalize();
  let diffuse_intensity = normal.dot(&light_dir).max(0.0);

  //reflejo por metal
  let reflect_dir = (2.0 * normal * normal.dot(&light_dir) - light_dir).normalize();
  let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(32.0);
  let base_color = Color::new(202, 107, 7);

  let color = base_color * diffuse_intensity + Color::new(202, 107, 7)* specular_intensity;
  color.clamp()
}