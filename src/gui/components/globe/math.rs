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

pub fn inverse_rotate(p: [f32; 3], yaw: f32, pitch: f32) -> [f32; 3] {
    // Undo pitch (rotate around X by -pitch)
    let x1 = p[0];
    let y1 = p[1] * pitch.cos() + p[2] * pitch.sin();
    let z1 = -p[1] * pitch.sin() + p[2] * pitch.cos();
    // Undo yaw (rotate around Y by -yaw)
    [
        x1 * yaw.cos() - z1 * yaw.sin(),
        y1,
        x1 * yaw.sin() + z1 * yaw.cos(),
    ]
}

pub fn project(p: [f32; 3], center: Pos2, radius: f32) -> Pos2 {
    Pos2::new(center.x + p[0] * radius, center.y - p[1] * radius)
}

/// Screen cursor → object-space world point on the unit sphere.
/// Clamps to sphere edge if cursor is outside the globe circle.
pub fn unproject(cursor: Pos2, center: Pos2, radius: f32, yaw: f32, pitch: f32) -> [f32; 3] {
    let lx = (cursor.x - center.x) / radius;
    let ly = (cursor.y - center.y) / radius;
    let r = (lx * lx + ly * ly).sqrt();
    let (lx, ly) = if r > 1.0 { (lx / r, ly / r) } else { (lx, ly) };
    let lz = (1.0 - lx * lx - ly * ly).max(0.0).sqrt();
    inverse_rotate([lx, -ly, lz], yaw, pitch)
}
