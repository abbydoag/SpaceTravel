use nalgebra::{Rotation3, Vector3, Unit};
use nalgebra_glm::Vec3;
pub struct Spaceship {
    pub position: Vec3,   
    pub forward: Vec3,     
    pub up: Vec3,         
}

impl Spaceship {
    pub fn new(start_position: Vec3) -> Self {
        Self {
            position: start_position,
            forward: Vec3::new(0.0, 0.0, 1.0),
            up: Vec3::new(0.0, 1.0, 0.0),       
        }
    }

    pub fn move_forward(&mut self, distance: f32) {
        self.position += self.forward * distance;
    }

    pub fn rotate(&mut self, angle: f32) {
        let up = Vector3::new(self.up.x, self.up.y, self.up.z);
        let forward = Vector3::new(self.forward.x, self.forward.y, self.forward.z);

        let unit_up = Unit::new_normalize(up);
        let rotation = Rotation3::from_axis_angle(&unit_up, angle);
        let rotated_forward = rotation * forward; 
        let rotated_up = rotation * up;

        self.forward = Vec3::new(rotated_forward.x, rotated_forward.y, rotated_forward.z);
        self.up = Vec3::new(rotated_up.x, rotated_up.y, rotated_up.z);
    }
}