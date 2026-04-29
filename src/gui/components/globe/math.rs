use eframe::egui::Pos2;


pub fn lat_lon_to_vec3(lat: f32, lon: f32) -> [f32; 3] {
    let lat_rad = lat.to_radians();
    let lon_rad = lon.to_radians();
    [
        lat_rad.cos() * lon_rad.sin(),
        lat_rad.sin(),
        lat_rad.cos() * lon_rad.cos(),
    ]
}

pub fn rotate(p: [f32; 3], yaw: f32, pitch: f32) -> [f32; 3] {
    // Rotate around Y axis (yaw)
    let x1 = p[0] * yaw.cos() + p[2] * yaw.sin();
    let y1 = p[1];
    let z1 = -p[0] * yaw.sin() + p[2] * yaw.cos();

    // Rotate around X axis (pitch)
    let x2 = x1;
    let y2 = y1 * pitch.cos() - z1 * pitch.sin();
    let z2 = y1 * pitch.sin() + z1 * pitch.cos();

    [x2, y2, z2]
}

pub fn project(p: [f32; 3], center: Pos2, radius: f32) -> Pos2 {
    Pos2::new(center.x + p[0] * radius, center.y - p[1] * radius)
}
