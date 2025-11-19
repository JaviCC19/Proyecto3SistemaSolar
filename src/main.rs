use nalgebra_glm::{Vec3, Mat4, perspective, look_at};
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod line;
mod vertex;
mod fragment;
mod shaders;
mod obj;
mod matrix;
mod camera;
mod light;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use triangle::triangle;
use shaders::{vertex_shader, fragment_shader, PlanetShaderType};
use light::Light;
use raylib::prelude::Vector3;

pub struct Uniforms {
    pub model_matrix: Mat4,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub viewport_matrix: Mat4,
    pub time: f32,
}

fn simplify_mesh(vertices: &[Vertex], target_triangles: usize) -> Vec<Vertex> {
    if vertices.len() < 3 {
        return vertices.to_vec();
    }
    
    let current_triangles = vertices.len() / 3;
    if current_triangles <= target_triangles {
        return vertices.to_vec();
    }
    
    let mut simplified = Vec::new();
    let skip_factor = (current_triangles / target_triangles).max(1);
    
    for i in (0..current_triangles).step_by(skip_factor) {
        let idx = i * 3;
        if idx + 2 < vertices.len() {
            let v0 = &vertices[idx];
            let v1 = &vertices[idx + 1];
            let v2 = &vertices[idx + 2];
            
            let edge1 = Vec3::new(
                v1.position.x - v0.position.x,
                v1.position.y - v0.position.y,
                v1.position.z - v0.position.z,
            );
            let edge2 = Vec3::new(
                v2.position.x - v0.position.x,
                v2.position.y - v0.position.y,
                v2.position.z - v0.position.z,
            );
            
            let cross = edge1.cross(&edge2);
            let area = cross.norm();
            
            if area > 0.0001 {
                simplified.push(v0.clone());
                simplified.push(v1.clone());
                simplified.push(v2.clone());
            }
        }
    }
    
    simplified
}

struct CelestialBody {
    name: String,
    position: Vec3,
    scale: f32,
    rotation: Vec3,
    rotation_speed: Vec3,
    orbit_radius: f32,
    orbit_speed: f32,
    orbit_angle: f32,
    shader_type: PlanetShaderType,
    vertex_array: Vec<Vertex>,
}

impl CelestialBody {
    fn new(
        name: &str,
        orbit_radius: f32,
        orbit_speed: f32,
        scale: f32,
        rotation_speed: Vec3,
        shader_type: PlanetShaderType,
        vertex_array: Vec<Vertex>,
    ) -> Self {
        CelestialBody {
            name: name.to_string(),
            position: Vec3::new(orbit_radius, 0.0, 0.0),
            scale,
            rotation: Vec3::zeros(),
            rotation_speed,
            orbit_radius,
            orbit_speed,
            orbit_angle: 0.0,
            shader_type,
            vertex_array,
        }
    }

    fn update(&mut self, delta_time: f32) {
        self.orbit_angle += self.orbit_speed * delta_time;
        self.position.x = self.orbit_radius * self.orbit_angle.cos();
        self.position.z = self.orbit_radius * self.orbit_angle.sin();
        self.rotation.x += self.rotation_speed.x * delta_time;
        self.rotation.y += self.rotation_speed.y * delta_time;
        self.rotation.z += self.rotation_speed.z * delta_time;
    }
}

struct SpaceshipCamera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    velocity: Vec3,
    speed: f32,
    turn_speed: f32,
}

impl SpaceshipCamera {
    fn new(position: Vec3) -> Self {
        SpaceshipCamera {
            position,
            yaw: 0.0,
            pitch: 0.0,
            velocity: Vec3::zeros(),
            speed: 50.0,
            turn_speed: 1.5,
        }
    }

    fn get_forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
    }

    fn get_right(&self) -> Vec3 {
        Vec3::new(
            (self.yaw - PI / 2.0).cos(),
            0.0,
            (self.yaw - PI / 2.0).sin(),
        )
    }

    fn get_up(&self) -> Vec3 {
        self.get_right().cross(&self.get_forward())
    }

    fn update(&mut self, window: &Window, delta_time: f32, planets: &[CelestialBody]) {
        let mut movement = Vec3::zeros();

        if window.is_key_down(Key::W) {
            movement += self.get_forward();
        }
        if window.is_key_down(Key::S) {
            movement -= self.get_forward();
        }
        if window.is_key_down(Key::A) {
            movement -= self.get_right();
        }
        if window.is_key_down(Key::D) {
            movement += self.get_right();
        }
        if window.is_key_down(Key::Space) {
            movement += Vec3::new(0.0, 1.0, 0.0);
        }
        if window.is_key_down(Key::LeftShift) {
            movement -= Vec3::new(0.0, 1.0, 0.0);
        }

        if window.is_key_down(Key::Left) {
            self.yaw += self.turn_speed * delta_time;
        }
        if window.is_key_down(Key::Right) {
            self.yaw -= self.turn_speed * delta_time;
        }
        if window.is_key_down(Key::Up) {
            self.pitch += self.turn_speed * delta_time;
            self.pitch = self.pitch.clamp(-PI / 2.5, PI / 2.5);
        }
        if window.is_key_down(Key::Down) {
            self.pitch -= self.turn_speed * delta_time;
            self.pitch = self.pitch.clamp(-PI / 2.5, PI / 2.5);
        }

        let movement_length = movement.norm();
        if movement_length > 0.0 {
            movement = movement.normalize();
        }

        let new_position = self.position + movement * self.speed * delta_time;

        let mut collision = false;
        for planet in planets {
            let distance = (new_position - planet.position).norm();
            let min_distance = planet.scale + 15.0;
            
            if distance < min_distance {
                collision = true;
                break;
            }
        }

        if !collision {
            self.position = new_position;
        }
    }

    fn warp_to(&mut self, target: Vec3, offset: f32) {
        let direction = (target - self.position).normalize();
        self.position = target - direction * offset;
    }
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

fn create_projection_matrix(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    perspective(fov_y, aspect, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}

fn render_orbit(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    radius: f32,
    segments: usize,
) {
    let color = 0x444444;
    framebuffer.set_current_color(color);

    for i in 0..segments {
        let angle1 = (i as f32 / segments as f32) * 2.0 * PI;
        let angle2 = ((i + 1) as f32 / segments as f32) * 2.0 * PI;

        let p1 = nalgebra_glm::vec4(radius * angle1.cos(), 0.0, radius * angle1.sin(), 1.0);
        let p2 = nalgebra_glm::vec4(radius * angle2.cos(), 0.0, radius * angle2.sin(), 1.0);

        let vp_matrix = uniforms.viewport_matrix 
            * uniforms.projection_matrix 
            * uniforms.view_matrix;

        let screen1 = vp_matrix * p1;
        let screen2 = vp_matrix * p2;

        if screen1.w > 0.0 && screen2.w > 0.0 {
            let x1 = (screen1.x / screen1.w) as usize;
            let y1 = (screen1.y / screen1.w) as usize;
            let x2 = (screen2.x / screen2.w) as usize;
            let y2 = (screen2.y / screen2.w) as usize;

            if x1 < framebuffer.width && y1 < framebuffer.height {
                framebuffer.point(x1, y1, 0.0);
            }
            if x2 < framebuffer.width && y2 < framebuffer.height {
                framebuffer.point(x2, y2, 0.0);
            }
        }
    }
}

struct Skybox {
    stars: Vec<(usize, usize, u32, bool)>,
}

impl Skybox {
    fn new(width: usize, height: usize, star_count: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut stars = Vec::with_capacity(star_count);
        
        for _ in 0..star_count {
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);
            
            let star_type = rng.gen_range(0..100);
            let color = if star_type < 70 {
                let brightness = rng.gen_range(180..255) as u32;
                (brightness << 16) | (brightness << 8) | brightness
            } else if star_type < 85 {
                let b = rng.gen_range(200..255) as u32;
                let g = rng.gen_range(180..220) as u32;
                let r = rng.gen_range(150..200) as u32;
                (r << 16) | (g << 8) | b
            } else if star_type < 95 {
                let r = rng.gen_range(220..255) as u32;
                let g = rng.gen_range(200..240) as u32;
                let b = rng.gen_range(100..150) as u32;
                (r << 16) | (g << 8) | b
            } else {
                let r = rng.gen_range(230..255) as u32;
                let g = rng.gen_range(100..150) as u32;
                let b = rng.gen_range(80..120) as u32;
                (r << 16) | (g << 8) | b
            };
            
            let is_bright = rng.gen_range(0..100) < 10 && color > 0xCCCCCC;
            stars.push((x, y, color, is_bright));
        }
        
        Skybox { stars }
    }
    
    fn render(&self, framebuffer: &mut Framebuffer) {
        for &(x, y, color, is_bright) in &self.stars {
            if x < framebuffer.width && y < framebuffer.height {
                framebuffer.set_current_color(color);
                framebuffer.point(x, y, f32::INFINITY);
                
                if is_bright {
                    if x > 0 {
                        framebuffer.point(x - 1, y, f32::INFINITY);
                    }
                    if x < framebuffer.width - 1 {
                        framebuffer.point(x + 1, y, f32::INFINITY);
                    }
                    if y > 0 {
                        framebuffer.point(x, y - 1, f32::INFINITY);
                    }
                    if y < framebuffer.height - 1 {
                        framebuffer.point(x, y + 1, f32::INFINITY);
                    }
                }
            }
        }
    }
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    light: &Light,
    planet_type: PlanetShaderType,
) {
    let start_time = Instant::now();
    
    let max_vertices = 1500;
    let vertices_to_process = if vertex_array.len() > max_vertices {
        &vertex_array[..max_vertices]
    } else {
        vertex_array
    };

    let mut transformed_vertices = Vec::with_capacity(vertices_to_process.len());
    for vertex in vertices_to_process {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    let mut triangles_vec = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles_vec.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    let mut visible_triangles = Vec::new();
    for tri in triangles_vec {
        let avg_z = (tri[0].transformed_position.z + 
                     tri[1].transformed_position.z + 
                     tri[2].transformed_position.z) / 3.0;
        
        if avg_z > -2000.0 && avg_z < 2000.0 {
            visible_triangles.push(tri);
        }
    }

    let max_triangles = 500;
    let triangles_to_process = visible_triangles.len().min(max_triangles);

    let mut fragments = Vec::new();
    let max_fragments = 15000;
    
    for tri in &visible_triangles[..triangles_to_process] {
        if fragments.len() >= max_fragments {
            break;
        }
        
        let tri_fragments = triangle(&tri[0], &tri[1], &tri[2], light);
        
        let space_left = max_fragments - fragments.len();
        if tri_fragments.len() <= space_left {
            fragments.extend(tri_fragments);
        } else {
            fragments.extend(tri_fragments.into_iter().take(space_left));
            break;
        }
    }

    const BATCH_SIZE: usize = 1000;
    for batch_start in (0..fragments.len()).step_by(BATCH_SIZE) {
        let batch_end = (batch_start + BATCH_SIZE).min(fragments.len());
        
        for fragment in &mut fragments[batch_start..batch_end] {
            fragment.color = fragment_shader(fragment, uniforms, planet_type);
            
            let x = fragment.position.x as usize;
            let y = fragment.position.y as usize;
            
            if x < framebuffer.width && y < framebuffer.height {
                let r = (fragment.color.x.clamp(0.0, 1.0) * 255.0) as u32;
                let g = (fragment.color.y.clamp(0.0, 1.0) * 255.0) as u32;
                let b = (fragment.color.z.clamp(0.0, 1.0) * 255.0) as u32;
                let color = (r << 16) | (g << 8) | b;
                framebuffer.set_current_color(color);
                framebuffer.point(x, y, fragment.depth);
            }
        }
        
        if start_time.elapsed().as_millis() > 50 {
            break;
        }
    }
}

fn main() {
    println!("=== Sistema Solar Ultra-Optimizado v3 ===");
    
    let window_width = 1200;
    let window_height = 800;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    
    let mut window = Window::new(
        "Sistema Solar - WASD Space/Shift Flechas, F=warp, ESC=salir",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    window.set_position(100, 100);
    window.limit_update_rate(Some(Duration::from_micros(16600)));
    framebuffer.set_background_color(0x000011);

    let sphere_obj = Obj::load("assets/models/sphere.obj").unwrap();
    let sphere_vertices = simplify_mesh(&sphere_obj.get_vertex_array(), 100);

    let ywing_obj = Obj::load("assets/models/Y-wing.obj").unwrap();
    let ywing_vertices = simplify_mesh(&ywing_obj.get_vertex_array(), 80);

    let mut planets = vec![
        CelestialBody::new("Sol", 0.0, 0.0, 25.0, Vec3::new(0.0, 0.1, 0.0), 
            PlanetShaderType::Solarius, sphere_vertices.clone()),
        CelestialBody::new("Terra", 150.0, 0.3, 15.0, Vec3::new(0.0, 0.5, 0.0), 
            PlanetShaderType::Terra, sphere_vertices.clone()),
        CelestialBody::new("Vulcan", 250.0, 0.2, 14.0, Vec3::new(0.0, 0.4, 0.0), 
            PlanetShaderType::Vulcan, sphere_vertices.clone()),
        CelestialBody::new("Nepturion", 400.0, 0.15, 22.0, Vec3::new(0.1, 0.3, 0.0), 
            PlanetShaderType::Nepturion, sphere_vertices.clone()),
        CelestialBody::new("Mossar", 550.0, 0.1, 18.0, Vec3::new(0.0, 0.35, 0.1), 
            PlanetShaderType::Mossar, sphere_vertices.clone()),
    ];

    let mut camera = SpaceshipCamera::new(Vec3::new(0.0, 100.0, 300.0));
    let mut light = Light::new(Vector3::new(0.0, 0.0, 0.0));
    let skybox = Skybox::new(framebuffer_width, framebuffer_height, 200);

    let aspect_ratio = framebuffer_width as f32 / framebuffer_height as f32;
    let start_time = Instant::now();
    let mut last_frame = Instant::now();
    let mut warp_planet_index = 0;
    let mut frame_count = 0;
    let mut fps_timer = Instant::now();
    let mut fps_counter = 0;

    println!("=== Iniciando renderizado ===\n");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let current_time = Instant::now();
        let delta_time = (current_time - last_frame).as_secs_f32();
        last_frame = current_time;
        let elapsed = start_time.elapsed().as_secs_f32();

        fps_counter += 1;
        if fps_timer.elapsed().as_secs() >= 1 {
            println!("FPS: {}", fps_counter);
            fps_counter = 0;
            fps_timer = Instant::now();
        }

        camera.update(&window, delta_time, &planets);

        if window.is_key_pressed(Key::F, minifb::KeyRepeat::No) {
            warp_planet_index = (warp_planet_index + 1) % planets.len();
            camera.warp_to(planets[warp_planet_index].position, 100.0);
        }

        for planet in &mut planets {
            planet.update(delta_time);
        }

        light.position = Vector3::new(
            planets[0].position.x,
            planets[0].position.y,
            planets[0].position.z,
        );

        framebuffer.clear();
        skybox.render(&mut framebuffer);

        let camera_target = camera.position + camera.get_forward() * 10.0;
        let view_matrix = create_view_matrix(camera.position, camera_target, camera.get_up());
        let projection_matrix = create_projection_matrix(PI / 3.0, aspect_ratio, 0.1, 2000.0);
        let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);

        for planet in &planets {
            if planet.orbit_radius > 0.0 {
                let orbit_uniforms = Uniforms {
                    model_matrix: Mat4::identity(),
                    view_matrix,
                    projection_matrix,
                    viewport_matrix,
                    time: elapsed,
                };
                render_orbit(&mut framebuffer, &orbit_uniforms, planet.orbit_radius, 32);
            }
        }

        for planet in planets.iter() {
            let model_matrix = create_model_matrix(planet.position, planet.scale, planet.rotation);
            let uniforms = Uniforms {
                model_matrix,
                view_matrix,
                projection_matrix,
                viewport_matrix,
                time: elapsed,
            };
            render(&mut framebuffer, &uniforms, &planet.vertex_array, &light, planet.shader_type);
        }

        let ship_offset = camera.get_forward() * 15.0 + camera.get_right() * -3.0 + camera.get_up() * -2.0;
        let ship_position = camera.position + ship_offset;
        let ship_rotation = Vec3::new(-camera.pitch, camera.yaw + PI, 0.0);
        let ship_model = create_model_matrix(ship_position, 2.5, ship_rotation);
        
        let ship_uniforms = Uniforms {
            model_matrix: ship_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time: elapsed,
        };
        
        render(&mut framebuffer, &ship_uniforms, &ywing_vertices, &light, PlanetShaderType::Terra);

        window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).ok();

        std::thread::sleep(frame_delay);
        frame_count += 1;
    }
    
    println!("\n=== Programa terminado - {} frames ===", frame_count);
}