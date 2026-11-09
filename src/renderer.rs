//! CPU renderer: every pixel is calculated by Rust code on the CPU.

use image::{Rgba, RgbaImage};

use crate::math::Vec3;
use crate::shader::{Shader, SurfaceSample};

pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    depth: Vec<f32>,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
            depth: vec![f32::INFINITY; width * height],
        }
    }

    pub fn clear(&mut self, color: Vec3) {
        let packed = pack_color(color);
        self.pixels.fill(packed);
        self.depth.fill(f32::INFINITY);
    }

    pub fn save_png(&self, path: &str) -> Result<(), image::ImageError> {
        let mut image = RgbaImage::new(self.width as u32, self.height as u32);

        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.pixels[y * self.width + x];
                let red = ((color >> 16) & 0xff) as u8;
                let green = ((color >> 8) & 0xff) as u8;
                let blue = (color & 0xff) as u8;
                image.put_pixel(x as u32, y as u32, Rgba([red, green, blue, 255]));
            }
        }

        image.save(path)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Ray {
    origin: Vec3,
    direction: Vec3,
}

fn ray_for_pixel(frame: &FrameBuffer, x: usize, y: usize) -> Ray {
    let aspect_ratio = frame.width as f32 / frame.height as f32;
    let screen_x = ((x as f32 + 0.5) / frame.width as f32 * 2.0 - 1.0) * aspect_ratio;
    let screen_y = 1.0 - (y as f32 + 0.5) / frame.height as f32 * 2.0;

    Ray {
        origin: Vec3::new(screen_x, screen_y, 3.0),
        direction: Vec3::new(0.0, 0.0, -1.0),
    }
}

fn write_depth_tested(frame: &mut FrameBuffer, x: usize, y: usize, depth: f32, color: Vec3) {
    let index = y * frame.width + x;
    if depth < frame.depth[index] {
        frame.depth[index] = depth;
        frame.pixels[index] = pack_color(color);
    }
}

/// Renders a sphere with any shader implementation.
pub fn render_shader(frame: &mut FrameBuffer, shader: &dyn Shader) {
    frame.clear(Vec3::new(0.008, 0.012, 0.035));

    render_shader_object(frame, shader, Vec3::ZERO, 1.0);
}

/// Renders a sphere and a smaller procedural moon in front of it.
pub fn render_shader_with_moon(
    frame: &mut FrameBuffer,
    shader: &dyn Shader,
    moon_shader: &dyn Shader,
) {
    render_shader(frame, shader);

    // The moon is a second sphere, placed slightly toward the camera so that
    // the depth buffer also handles the occasional overlap with the Earth.
    let moon_center = Vec3::new(-1.03, 0.55, 0.85);
    render_shader_object(frame, moon_shader, moon_center, 0.25);
}

fn render_shader_object(frame: &mut FrameBuffer, shader: &dyn Shader, center: Vec3, radius: f32) {
    for y in 0..frame.height {
        for x in 0..frame.width {
            let ray = ray_for_pixel(frame, x, y);
            let Some((surface_point, normal)) = intersect_sphere(ray, center, radius) else {
                continue;
            };

            let light_direction = Vec3::new(-0.6, 0.8, 1.0).normalized();
            let view_direction = (ray.origin - surface_point).normalized();
            let diffuse = normal.dot(light_direction).max(0.0);
            let rim = 1.0 - normal.dot(view_direction).max(0.0);
            let latitude = normal.y * 0.5 + 0.5;
            let longitude = normal.x.atan2(normal.z) / (std::f32::consts::TAU) + 0.5;

            let sample = SurfaceSample {
                // The main sphere keeps its original surface coordinates;
                // the moon uses normalized local coordinates so its shader
                // does not depend on its position in the scene.
                position: if radius == 1.0 { surface_point } else { normal },
                normal,
                view_direction,
                light_direction,
                diffuse,
                rim,
                latitude,
                longitude,
            };

            let depth = ray.origin.z - surface_point.z;
            write_depth_tested(frame, x, y, depth, shader.shade(sample));
        }
    }
}

/// Renders a shader and then adds Saturn's separate procedural ring model.
pub fn render_shader_with_saturn_rings(frame: &mut FrameBuffer, shader: &dyn Shader) {
    render_shader(frame, shader);

    for y in 0..frame.height {
        for x in 0..frame.width {
            render_saturn_ring_at_pixel(frame, x, y, ray_for_pixel(frame, x, y));
        }
    }
}

fn intersect_sphere(ray: Ray, center: Vec3, radius: f32) -> Option<(Vec3, Vec3)> {
    let sphere_to_ray = ray.origin - center;
    let b = sphere_to_ray.dot(ray.direction);
    let c = sphere_to_ray.dot(sphere_to_ray) - radius * radius;
    let discriminant = b * b - c;

    if discriminant < 0.0 {
        return None;
    }

    let near_distance = -b - discriminant.sqrt();
    if near_distance < 0.0 {
        return None;
    }

    let surface_point = ray.origin + ray.direction * near_distance;
    let normal = (surface_point - center).normalized();
    Some((surface_point, normal))
}

fn render_saturn_ring_at_pixel(frame: &mut FrameBuffer, x: usize, y: usize, ray: Ray) {
    let ring_normal = Vec3::new(0.28, 0.88, 0.22).normalized();
    let denominator = ray.direction.dot(ring_normal);

    if denominator.abs() < 0.0001 {
        return;
    }

    let intersection_distance = -ray.origin.dot(ring_normal) / denominator;
    if intersection_distance < 0.0 {
        return;
    }

    let ring_point = ray.origin + ray.direction * intersection_distance;
    let radius = ring_point.length();
    let inner_radius = 1.18;
    let outer_radius = 1.72;

    if radius < inner_radius || radius > outer_radius {
        return;
    }

    let band = ((radius - inner_radius) * 26.0).sin() * 0.5 + 0.5;
    let base_color = Vec3::new(0.48, 0.37, 0.24);
    let highlight_color = Vec3::new(0.92, 0.82, 0.62);
    let ring_color = base_color * (1.0 - band * 0.55) + highlight_color * (band * 0.55);

    write_depth_tested(frame, x, y, intersection_distance, ring_color * 0.9);
}

fn pack_color(color: Vec3) -> u32 {
    let color = color.clamp(0.0, 1.0);
    let red = (color.x * 255.0) as u32;
    let green = (color.y * 255.0) as u32;
    let blue = (color.z * 255.0) as u32;
    (red << 16) | (green << 8) | blue
}
