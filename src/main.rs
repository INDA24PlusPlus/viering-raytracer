extern crate nalgebra_glm as glm;
use glm::{dot, vec3, Vec3};
use minifb::{Window, WindowOptions};
use minifb_fonts::*;
use rand::Rng;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::time::Instant;

mod camera;
use crate::camera::Camera;

mod utils;
use crate::utils::*;

/*
    note: whats left to do (the important stuff at least)
    - distance calculation for lighting, some kind of light energy dropoff on hit
    - fix noise in the image
    - add a nicer way to place objects, maybe a config file
    - add something like egui to make it easier to change settings
    - make keyboard input work when the game is slow af
*/

trait GameObject: Sync + Send {
    fn hit(&self, ray: &Ray) -> Option<HitData>;
    fn material(&self) -> &Material;
    fn pos(&self) -> glm::Vec3;
}

#[derive(Clone)]
struct Sphere {
    pos: glm::Vec3,
    radius: f32,
    material: Material,
}

impl GameObject for Sphere {
    fn hit(&self, ray: &Ray) -> Option<HitData> {
        let origin = ray.origin - self.pos;
        let a = dot(&ray.dir, &ray.dir);
        let b = 2.0 * dot(&origin, &ray.dir);
        let c = dot(&origin, &origin) - self.radius * self.radius;

        let disc = b * b - 4.0 * a * c;
        if disc < 0.0 {
            return None;
        }

        let t = (-b - disc.sqrt()) / (2.0 * a);
        if t < 0.0 {
            return None;
        }

        Some(HitData {
            point: ray.origin + ray.dir * t,
            t,
            object: Box::new(self.clone()),
        })
    }

    fn material(&self) -> &Material {
        &self.material
    }

    fn pos(&self) -> glm::Vec3 {
        self.pos
    }
}

#[derive(Clone)]
struct Plane {
    pos: glm::Vec3,
    normal: glm::Vec3,
    material: Material,
}

impl GameObject for Plane {
    fn hit(&self, ray: &Ray) -> Option<HitData> {
        let d = dot(&self.normal, &ray.dir);
        if d.abs() < 1e-6 {
            return None;
        }

        let t = dot(&(self.pos - ray.origin), &self.normal) / d;
        if t < 0.0 {
            return None;
        }

        Some(HitData {
            point: ray.origin + ray.dir * t,
            t,
            object: Box::new(self.clone()),
        })
    }

    fn material(&self) -> &Material {
        &self.material
    }

    fn pos(&self) -> glm::Vec3 {
        self.pos
    }
}

#[derive(Clone)]
struct Material {
    color: glm::Vec3,
    emission: f32,
    reflectiveness: f32,
}

struct World {
    objects: Vec<Box<dyn GameObject>>,
    unlit: bool,
    frame_averaging: bool,
    max_bounces: usize,
    num_samples: usize,
}

struct HitData {
    point: glm::Vec3,
    t: f32, // for distance calculation later on
    object: Box<dyn GameObject>,
}

struct Ray {
    origin: glm::Vec3,
    dir: glm::Vec3,
}

impl Ray {
    fn new(origin: glm::Vec3, dir: glm::Vec3) -> Self {
        Ray { origin, dir }
    }
}

impl World {
    fn hit(&self, ray: &Ray) -> Option<HitData> {
        let mut smallest_t = f32::MAX;
        let mut closest_hit: Option<HitData> = None;

        for object in &self.objects {
            if let Some(hit) = object.hit(ray) {
                if hit.t < smallest_t {
                    smallest_t = hit.t;
                    closest_hit = Some(hit);
                }
            }
        }

        closest_hit
    }
}

fn raytrace(world: &World, ray: Ray) -> glm::Vec3 {
    let mut ray_color = vec3_from(1.0);
    let mut light = vec3_from(0.0);
    let mut curr_ray = ray;

    let mut rng = rand::thread_rng();

    for _ in 0..(world.max_bounces) {
        let hit = world.hit(&curr_ray);

        if let Some(hit) = hit {
            curr_ray.origin = hit.point;
            let normal = glm::normalize(&(hit.point - hit.object.pos()));

            curr_ray.dir = if rng.gen::<f32>() < hit.object.material().reflectiveness {
                curr_ray.dir - 2.0 * glm::dot(&curr_ray.dir, &normal) * normal
            } else {
                glm::normalize(&(normal + vec3_rand_unit()))
            };

            ray_color = glm::vec3(
                ray_color.x * hit.object.material().color.x,
                ray_color.y * hit.object.material().color.y,
                ray_color.z * hit.object.material().color.z,
            );

            if world.unlit {
                return ray_color;
            }

            // note: something about light dropoff based on distance

            light += glm::vec3(
                hit.object.material().emission * hit.object.material().color.x,
                hit.object.material().emission * hit.object.material().color.y,
                hit.object.material().emission * hit.object.material().color.z,
            )
            .component_mul(&ray_color);

            light = glm::vec3(light.x.min(1.0), light.y.min(1.0), light.z.min(1.0));
        } else {
            break;
        }
    }

    light
}

fn main() {
    let mut window = Window::new(
        "shurdatracer",
        800,
        700,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();
    let mut buffer: Vec<u32> = vec![0; window.get_size().0 * window.get_size().1];

    let mut camera = Camera::new(
        glm::vec3(0.0, 0.0, 2.0),
        glm::vec3(0.0, 0.0, -1.0),
        glm::vec3(0.0, 1.0, 0.0),
        0.005,
        3.0,
        10.0,
        45.0,
        0.1,
        100.0,
    );

    let mut world = World {
        objects: vec![
            Box::new(Sphere {
                pos: glm::vec3(0.0, -10.0, 0.0),
                radius: 10.0,
                material: Material {
                    color: glm::vec3(0.6, 0.0, 0.8),
                    emission: 0.0,
                    reflectiveness: 0.5,
                },
            }),
            Box::new(Sphere {
                pos: glm::vec3(1.0, 1.0, 0.0),
                radius: 1.0,
                material: Material {
                    color: glm::vec3(1.0, 1.0, 1.0),
                    emission: 0.0,
                    reflectiveness: 0.8,
                },
            }),
            Box::new(Sphere {
                pos: glm::vec3(-2.0, 1.0, 0.0),
                radius: 1.0,
                material: Material {
                    color: glm::vec3(0.0, 0.6, 0.0),
                    emission: 0.0,
                    reflectiveness: 0.0,
                },
            }),
            Box::new(Sphere {
                pos: glm::vec3(1.0, 2.2, -4.0),
                radius: 0.7,
                material: Material {
                    color: glm::vec3(1.0, 0.4, 0.0),
                    emission: 0.0,
                    reflectiveness: 0.0,
                },
            }),
            Box::new(Sphere {
                pos: glm::vec3(1.5, 4.0, -20.0),
                radius: 10.0,
                material: Material {
                    color: glm::vec3(1.0, 1.0, 1.0),
                    emission: 1.0,
                    reflectiveness: 0.0,
                },
            }),
            Box::new(Plane {
                pos: glm::vec3(0.0, -5.0, 0.0),
                normal: glm::vec3(0.0, 1.0, 0.0),
                material: Material {
                    color: glm::vec3(1.0, 0.0, 0.0),
                    emission: 0.0,
                    reflectiveness: 0.0,
                },
            }),
        ],
        unlit: true, // start in unlit to make it easier to position camera
        frame_averaging: true,
        max_bounces: 3,
        num_samples: 30,
    };

    let mut font_renderer =
        font6x8::new_renderer(window.get_size().0, window.get_size().1, 0xff_ff_ff);

    let mut last_time = Instant::now();

    let mut update_frame_time_timer = 0.0;
    let mut frame_time = 0.0;

    let mut last_window_size = window.get_size();

    let mut last_render: Vec<Vec3> =
        vec![vec3(0.0, 0.0, 0.0); window.get_size().0 * window.get_size().1];
    let mut rendered_frames = 0;

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        let window_size = window.get_size();

        // delta time calculation
        let now = Instant::now();
        let delta_time = now.duration_since(last_time).as_secs_f32();
        last_time = now;

        // note: fix input polling by using another thread and saving a list of pressed keys
        // note 2: bruh window isnt thread safe

        // window resizing logic
        if last_window_size != window_size {
            buffer = vec![0; window_size.0 * window_size.1];
            last_window_size = window_size;
            font_renderer =
                font6x8::new_renderer(window.get_size().0, window.get_size().1, 0xff_ff_ff);
        }

        // update camera
        camera.update(&window, delta_time);

        // allow toggling unlit mode
        if window.is_key_pressed(minifb::Key::U, minifb::KeyRepeat::No) {
            world.unlit = !world.unlit;
            camera.was_dirty = true;
        }

        // logic for resetting frame averaging
        if camera.was_dirty {
            rendered_frames = 0;
            last_render = vec![vec3(0.0, 0.0, 0.0); window.get_size().0 * window.get_size().1];
        }

        // update frame time
        update_frame_time_timer += delta_time;
        if update_frame_time_timer >= 1.0 {
            update_frame_time_timer = 0.0;
            frame_time = delta_time;
        }

        // render scene
        let samples = if world.unlit { 1 } else { world.num_samples };

        let mut out: Vec<glm::Vec3> = vec![vec3_from(0.0); window_size.0 * window_size.1];
        out.par_iter_mut().enumerate().for_each(|(i, color)| {
            // multiple samples
            let mut total_color = vec3_from(0.0);
            for _ in 0..samples {
                total_color += raytrace(&world, Ray::new(camera.pos, camera.ray_dirs[i]));
            }
            total_color /= samples as f32;

            // frame averaging
            if world.frame_averaging {
                let weight = 1.0 / (rendered_frames + 1) as f32;
                *color = last_render[i] * (1.0 - weight) + total_color * weight
            } else {
                *color = total_color
            }
        });

        // apply color
        for (i, color) in out.iter().enumerate() {
            let red = (color.x * 255.0) as u32;
            let green = (color.y * 255.0) as u32;
            let blue = (color.z * 255.0) as u32;
            buffer[i] = (red << 16) | (green << 8) | blue;

            last_render[i] = *color;
        }

        // render ui
        // background
        for x in 0..128 {
            for y in 0..46 {
                buffer[y * window_size.0 + x] = 0x3a_3a_3a;
            }
        }

        // text
        font_renderer.draw_text(
            &mut buffer,
            10,
            10,
            format!(
                "Fps: {}\nFrame time: {}ms\nUnlit: {}",
                (1.0 / frame_time) as i32,
                (frame_time * 1000.0) as i32,
                world.unlit
            )
            .as_str(),
        );

        // update the frame buffer
        window
            .update_with_buffer(&buffer, window_size.0, window_size.1)
            .unwrap();

        rendered_frames += 1;
    }
}
