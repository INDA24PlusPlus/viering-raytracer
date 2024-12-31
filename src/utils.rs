use rand::Rng;

pub fn vec3_from(x: f32) -> glm::Vec3 {
    glm::vec3(x, x, x)
}

pub fn vec3_rand_unit() -> glm::Vec3 {
    let mut rng = rand::thread_rng();

    let theta = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
    let phi = rng.gen_range(0.0..std::f32::consts::PI);

    glm::vec3(phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos())
}
