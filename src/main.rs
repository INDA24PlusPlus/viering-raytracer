extern crate nalgebra_glm as glm;
use minifb::{Window, WindowOptions};
use minifb_fonts::*;
use std::time::Instant;

mod camera;
use crate::camera::Camera;

struct Sphere {
    //pos: glm::Vec3,
    radius: f32,
    color: glm::Vec3,
}

fn raytrace(ray_origin: &glm::Vec3, ray_dir: &glm::Vec3) -> glm::Vec4 {
    // base color
    let sphere = Sphere {
        //pos: glm::vec3(0.0, 0.0, -1.5),
        radius: 0.5,
        color: glm::vec3(1.0, 0.4, 0.0),
    };

    // SPHERE CALC START
    let a = glm::dot(&ray_dir, &ray_dir);
    let b = 2.0 * glm::dot(&ray_origin, &ray_dir);
    let c = glm::dot(&ray_origin, &ray_origin) - sphere.radius * sphere.radius;

    let disc = b * b - 4.0 * a * c;

    if disc < 0.0 {
        // temp atmosphere
        return glm::vec4(0.0, 0.0, 0.2, 1.0);
    }

    let t = (-b - disc.sqrt()) / (2.0 * a); // closest intersect point

    let hit_point = ray_origin + ray_dir * t;
    let normal = glm::normalize(&hit_point);

    let light_direction = glm::normalize(&glm::vec3(-1.0, -1.0, -1.0));
    let light = glm::dot(&normal, &-light_direction).max(0.0);
    // SPHERE CALC END

    glm::vec4(
        sphere.color.x * light,
        sphere.color.y * light,
        sphere.color.z * light,
        1.0,
    )
}

fn main() {
    let mut window = Window::new(
        "raytracer",
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
        45.0,
        0.1,
        100.0,
    );

    let mut font_renderer =
        font6x8::new_renderer(window.get_size().0, window.get_size().1, 0xff_ff_ff);

    let mut last_time = Instant::now();

    let mut update_fps_timer = 0.0;
    let mut fps = 0;

    let mut last_window_size = window.get_size();
    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        let window_size = window.get_size();

        // delta time calculation
        let now = Instant::now();
        let delta_time = now.duration_since(last_time).as_secs_f32();
        last_time = now;

        // window resizing logic
        if last_window_size != window_size {
            buffer = vec![0; window_size.0 * window_size.1];
            last_window_size = window_size;
            font_renderer =
                font6x8::new_renderer(window.get_size().0, window.get_size().1, 0xff_ff_ff);
        }

        // update camera
        camera.update(&window, delta_time);

        // update fps
        update_fps_timer += delta_time;
        if update_fps_timer >= 1.0 {
            update_fps_timer = 0.0;
            fps = (1.0 / delta_time) as i32;
        }

        // render scene
        for y in 0..window_size.1 {
            for x in 0..window_size.0 {
                let ray_dir = camera.ray_dirs[x + y * window_size.0];

                let color = raytrace(&camera.pos, &ray_dir);

                let red = (color.x * 255.0) as u32;
                let green = (color.y * 255.0) as u32;
                let blue = (color.z * 255.0) as u32;

                buffer[y * window_size.0 + x] = (red << 16) | (green << 8) | blue;
            }
        }

        // render ui
        font_renderer.draw_text(&mut buffer, 10, 10, format!("FPS: {}", fps).as_str());

        // update the frame buffer
        window
            .update_with_buffer(&buffer, window_size.0, window_size.1)
            .unwrap();
    }
}
