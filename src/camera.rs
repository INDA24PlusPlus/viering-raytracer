use minifb::Window;

pub struct Camera {
    pub view: glm::Mat4,
    pub proj: glm::Mat4,
    pub inv_view: glm::Mat4,
    pub inv_proj: glm::Mat4,

    pub fov: f32,
    pub near_clip: f32,
    pub far_clip: f32,

    pub pos: glm::Vec3,
    pub forward: glm::Vec3,
    pub up: glm::Vec3,

    pub ray_dirs: Vec<glm::Vec3>,

    pub sensitivity: f32,
    pub speed: f32,
    pub fast_speed: f32,

    last_mouse_pos: glm::Vec2,
    last_size: glm::Vec2,

    dirty: bool,

    pub was_dirty: bool,
}

impl Camera {
    pub fn new(
        pos: glm::Vec3,
        forward: glm::Vec3,
        up: glm::Vec3,
        sensitivity: f32,
        move_speed: f32,
        fast_move_speed: f32,
        fov: f32,
        near_clip: f32,
        far_clip: f32,
    ) -> Self {
        Self {
            view: glm::Mat4::from_element(1.0),
            inv_view: glm::Mat4::from_element(1.0),
            proj: glm::Mat4::from_element(1.0),
            inv_proj: glm::Mat4::from_element(1.0),
            speed: move_speed,
            fast_speed: fast_move_speed,
            ray_dirs: vec![],
            dirty: true,
            was_dirty: false,
            last_mouse_pos: glm::vec2(999999.0, 999999.0),
            last_size: glm::vec2(0.0, 0.0),
            pos,
            forward,
            up,
            sensitivity,
            fov,
            near_clip,
            far_clip,
        }
    }

    pub fn update(&mut self, window: &Window, dt: f32) {
        // mouse stuff
        let current_mouse_pos = glm::vec2(
            window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap().0,
            window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap().1,
        );

        if self.last_mouse_pos == glm::vec2(999999.0, 999999.0) {
            self.last_mouse_pos = current_mouse_pos;
        }

        let mouse_delta = current_mouse_pos - self.last_mouse_pos;
        self.last_mouse_pos = current_mouse_pos;

        // camera calculations
        let right = glm::normalize(&glm::cross(&self.forward, &self.up));

        // window resized?
        let window_size = glm::vec2(window.get_size().0 as f32, window.get_size().1 as f32);
        if window_size != self.last_size {
            self.dirty = true;
            self.last_size = window_size;
        }

        // movement
        if window.get_mouse_down(minifb::MouseButton::Right) {
            let speed = if window.is_key_down(minifb::Key::LeftShift) {
                self.fast_speed
            } else {
                self.speed
            };

            // keyboard input
            if window.is_key_down(minifb::Key::W) {
                self.pos += self.forward * speed * dt;
                self.dirty = true;
            }
            if window.is_key_down(minifb::Key::S) {
                self.pos -= self.forward * speed * dt;
                self.dirty = true;
            }
            if window.is_key_down(minifb::Key::A) {
                self.pos -= right * speed * dt;
                self.dirty = true;
            }
            if window.is_key_down(minifb::Key::D) {
                self.pos += right * speed * dt;
                self.dirty = true;
            }
            if window.is_key_down(minifb::Key::Q) {
                self.pos -= self.up * speed * dt;
                self.dirty = true;
            }
            if window.is_key_down(minifb::Key::E) {
                self.pos += self.up * speed * dt;
                self.dirty = true;
            }

            // mouse movement
            if mouse_delta.x != 0.0 || mouse_delta.y != 0.0 {
                let yaw_delta = mouse_delta.x * self.sensitivity;
                let pitch_delta = mouse_delta.y * self.sensitivity;

                let q = glm::quat_normalize(&glm::quat_cross(
                    &glm::quat_angle_axis(-pitch_delta, &right),
                    &glm::quat_angle_axis(-yaw_delta, &self.up),
                ));

                self.forward = glm::quat_rotate_vec3(&q, &self.forward);
                self.dirty = true;
            }
        }

        // update camera
        self.was_dirty = self.dirty;
        if self.dirty {
            self.dirty = false;

            // note: should probably differentiate between what has changed
            // as view, proj and ray_dirs don't always need to be updated

            // calc view matrix
            self.view = glm::look_at(&self.pos, &(self.pos + self.forward), &self.up);
            self.inv_view = glm::inverse(&self.view);

            // calc proj matrix
            let rad = self.fov.to_radians();
            self.proj = glm::perspective_fov(
                rad,
                window_size.x,
                window_size.y,
                self.near_clip,
                self.far_clip,
            );
            self.inv_proj = glm::inverse(&self.proj);

            self.ray_dirs.resize(
                (window_size.x * window_size.y) as usize,
                glm::vec3(0.0, 0.0, 0.0),
            );

            // calc ray directions
            for y in 0..(window_size.y as usize) {
                for x in 0..(window_size.x as usize) {
                    let mut coord =
                        glm::vec2((x as f32) / window_size.x, 1.0 - (y as f32) / window_size.y);

                    coord.x = coord.x * 2.0 - 1.0;
                    coord.y = coord.y * 2.0 - 1.0;

                    let target = &self.inv_proj * glm::vec4(coord.x, coord.y, 1.0, 1.0);

                    let p1 = glm::normalize(&(target.xyz() / target.w));
                    let p2 = &self.inv_view * glm::vec4(p1.x, p1.y, p1.z, 0.0);

                    self.ray_dirs[y * (window_size.x as usize) + x] = p2.xyz();
                }
            }
        }
    }
}
