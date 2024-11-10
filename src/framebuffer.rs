pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    pub zbuffer: Vec<f32>,
    background_color: u32,
    current_color: u32,
    background_buffer: Vec<u32>
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let background_color = 0x151515;
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],
            zbuffer: vec![f32::INFINITY; width * height],
            background_color: 0x151515,
            current_color: 0xFFFFFF,
            background_buffer: vec![background_color; width * height]
        }
    }

    pub fn clear(&mut self) {
        // Copiar el contenido de background_buffer a buffer
        self.buffer.copy_from_slice(&self.background_buffer);
    
        for depth in self.zbuffer.iter_mut() {
            *depth = f32::INFINITY;
        }
    }

    pub fn point(&mut self, x: usize, y: usize, depth: f32) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;

            if self.zbuffer[index] > depth {
                self.buffer[index] = self.current_color;
                self.zbuffer[index] = depth;
            }
        }
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }
    //espacio
    pub fn set_background_star(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.background_buffer[index] = color;
        }
    }
}