use nalgebra_glm::Vec3;

pub struct Camera {
  pub eye: Vec3,
  pub center: Vec3,
  pub up: Vec3
}

impl Camera {
  pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
    let forward = (center - eye).normalize();  // Dirección hacia donde mira la cámara
    let right = forward.cross(&up).normalize(); // Dirección a la derecha
    let up = right.cross(&forward).normalize();
    
    Camera {
      eye,
      center,
      up
    }
  }
}