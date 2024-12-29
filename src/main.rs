extern crate nalgebra_glm as glm;
use glm::{vec4, Vec4};
use minifb::{Window, WindowOptions};
use minifb_fonts::*;
use rand::Rng;
use std::time::Instant;

mod camera;
use crate::camera::Camera;

struct Sphere {
    pos: glm::Vec3,
    radius: f32,
    color: glm::Vec3,
    emission: f32,
}

struct World {
    objects: Vec<Sphere>,
    unlit: bool,
    max_bounces: usize,
    pixel_sample_count: usize,
}

// temporarily yoinked from chatgpt because i wanna work on other parts of the raytracer rn
fn random_hemisphere_direction(normal: glm::Vec3) -> glm::Vec3 {
    let mut rng = rand::thread_rng();
    let u: f32 = rng.gen();
    let v: f32 = rng.gen();

    let theta = 2.0 * std::f32::consts::PI * u;
    let phi = v.acos();

    let x = phi.sin() * theta.cos();
    let y = phi.sin() * theta.sin();
    let z = phi.cos();

    let local_dir = glm::vec3(x, y, z);

    let tangent = if normal.y.abs() < 1e-4 {
        glm::normalize(&glm::cross(&normal, &glm::vec3(0.0, 1.0, 0.0)))
    } else {
        glm::normalize(&glm::cross(&normal, &glm::vec3(1.0, 0.0, 0.0)))
    };
    let bitangent = glm::cross(&normal, &tangent);

    let world_dir = local_dir.x * tangent + local_dir.y * bitangent + local_dir.z * normal;

    glm::normalize(&world_dir)
}

fn raytrace(world: &World, ray_origin: &glm::Vec3, ray_dir: &glm::Vec3) -> glm::Vec4 {
    let mut smallest_t = f32::MAX;
    let mut selected_sphere: Option<&Sphere> = None;

    let mut ray_color = glm::vec4(1.0, 1.0, 1.0, 1.0);
    let mut light = glm::vec4(0.0, 0.0, 0.0, 1.0);

    let mut curr_ray_origin = *ray_origin;
    let mut curr_ray_dir = *ray_dir;
    let mut hit_light_source = false;

    for i in 0..(world.max_bounces) {
        for sphere in &world.objects {
            let origin = &(curr_ray_origin - sphere.pos);

            let a = glm::dot(&curr_ray_dir, &curr_ray_dir);
            let b = 2.0 * glm::dot(&origin, &curr_ray_dir);
            let c = glm::dot(&origin, &origin) - sphere.radius * sphere.radius;

            let disc = b * b - 4.0 * a * c;
            if disc < 0.0 {
                continue;
            }

            let t = (-b - disc.sqrt()) / (2.0 * a); // closest intersect point

            if t >= 0.0 && t < smallest_t {
                smallest_t = t;
                selected_sphere = Some(sphere);
            }
        }

        if let Some(sphere) = selected_sphere {
            // note: need to handle light dropoff and surfaces absorbing light

            let origin = &(curr_ray_origin - sphere.pos);

            let hit_point = origin + curr_ray_dir * smallest_t;
            let normal = glm::normalize(&hit_point);

            curr_ray_origin = hit_point;
            curr_ray_dir = random_hemisphere_direction(normal);

            if ray_color == glm::vec4(0.0, 0.0, 0.0, 1.0) {
                ray_color = glm::vec4(sphere.color.x, sphere.color.y, sphere.color.z, 1.0);
            } else {
                ray_color = glm::vec4(
                    ray_color.x * sphere.color.x,
                    ray_color.y * sphere.color.y,
                    ray_color.z * sphere.color.z,
                    1.0,
                );
            }

            if world.unlit {
                return ray_color;
            }

            let emitted_light = glm::vec4(
                sphere.emission * sphere.color.x,
                sphere.emission * sphere.color.y,
                sphere.emission * sphere.color.z,
                1.0,
            );

            light += emitted_light.component_mul(&ray_color);

            if sphere.emission > 0.0 {
                hit_light_source = true;
            }
        } else {
            // note: below snippet is to see difference between dark sphere and atmosphere
            // return glm::vec4(0.0, 0.0, 0.2, 1.0);

            break;
        }
    }

    if hit_light_source {
        light
    } else {
        glm::vec4(0.0, 0.0, 0.0, 1.0)
    }
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
        0.5,
        3.0,
        45.0,
        0.1,
        100.0,
    );

    let mut world = World {
        objects: vec![
            Sphere {
                pos: glm::vec3(0.0, -10.0, 0.0),
                radius: 10.0,
                color: glm::vec3(0.6, 0.0, 0.8),
                emission: 0.0,
            },
            Sphere {
                pos: glm::vec3(0.0, 0.5, 0.0),
                radius: 0.5,
                color: glm::vec3(0.8, 0.0, 0.2),
                emission: 0.0,
            },
            Sphere {
                pos: glm::vec3(-2.0, 1.0, 0.0),
                radius: 1.0,
                color: glm::vec3(0.0, 0.6, 0.0),
                emission: 0.0,
            },
            Sphere {
                pos: glm::vec3(1.0, 2.2, -4.0),
                radius: 0.7,
                color: glm::vec3(1.0, 0.6, 0.0),
                emission: 0.0,
            },
            Sphere {
                pos: glm::vec3(2.0, 4.0, -10.0),
                radius: 5.0,
                color: glm::vec3(1.0, 1.0, 1.0),
                emission: 1.0,
            },
        ],
        unlit: true,
        max_bounces: 10,
        pixel_sample_count: 10,
    };

    let mut font_renderer =
        font6x8::new_renderer(window.get_size().0, window.get_size().1, 0xff_ff_ff);

    let mut last_time = Instant::now();

    let mut update_frame_time_timer = 0.0;
    let mut frame_time = 0.0;

    let mut last_window_size = window.get_size();

    let mut last_render: Vec<Vec4> =
        vec![vec4(0.0, 0.0, 0.0, 1.0); window.get_size().0 * window.get_size().1];
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

        // allow toggling unlit mode
        if window.is_key_pressed(minifb::Key::U, minifb::KeyRepeat::No) {
            world.unlit = !world.unlit;
        }

        // update camera
        camera.update(&window, delta_time);

        if camera.was_dirty {
            rendered_frames = 0;
            last_render = vec![vec4(0.0, 0.0, 0.0, 1.0); window.get_size().0 * window.get_size().1];
        }

        // update frame time
        update_frame_time_timer += delta_time;
        if update_frame_time_timer >= 1.0 {
            update_frame_time_timer = 0.0;
            frame_time = delta_time;
        }

        // render scene
        let samples = if world.unlit {
            1
        } else {
            world.pixel_sample_count
        };

        for y in 0..window_size.1 {
            for x in 0..window_size.0 {
                let id = y * window_size.0 + x;

                // multiple samples
                let mut total_color = glm::vec4(0.0, 0.0, 0.0, 0.0);
                for _ in 0..(samples) {
                    total_color += raytrace(&world, &camera.pos, &camera.ray_dirs[id]);
                }
                total_color /= samples as f32;

                let weight = 1.0 / (rendered_frames + 1) as f32;
                let average = last_render[id] * (1.0 - weight) + total_color * weight;

                let red = (average.x * 255.0) as u32;
                let green = (average.y * 255.0) as u32;
                let blue = (average.z * 255.0) as u32;

                buffer[id] = (red << 16) | (green << 8) | blue;

                last_render[id] = average;
            }
        }

        // render ui
        font_renderer.draw_text(
            &mut buffer,
            10,
            10,
            format!(
                "FPS: {}\nFrame time: {:.2}ms",
                (1.0 / frame_time) as i32,
                frame_time * 1000.0
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
