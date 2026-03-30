use super::Point;

/// The AAA 1D Spline Engine.
pub fn smooth_spline(points: &[Point], smoothing_level: f32) -> Vec<Point> {
    if points.len() < 3 || smoothing_level <= 0.0 { 
        return points.to_vec(); 
    }
    
    let steps = (smoothing_level * 10.0).max(1.0) as usize;
    let mut smoothed = Vec::with_capacity(points.len() * steps);
    smoothed.push(points[0]);

    for i in 0..(points.len() - 1) {
        let p0 = if i == 0 { &points[0] } else { &points[i - 1] };
        let p1 = &points[i]; 
        let p2 = &points[i + 1];
        let p3 = if i + 2 < points.len() { &points[i + 2] } else { &points[i + 1] };

        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            let t2 = t * t; 
            let t3 = t2 * t;

            let interpolate = |v0: f32, v1: f32, v2: f32, v3: f32| -> f32 {
                0.5 * ((2.0 * v1) + (-v0 + v2) * t + (2.0 * v0 - 5.0 * v1 + 4.0 * v2 - v3) * t2 + (-v0 + 3.0 * v1 - 3.0 * v2 + v3) * t3)
            };

            smoothed.push(Point { 
                x: interpolate(p0.x, p1.x, p2.x, p3.x), 
                y: interpolate(p0.y, p1.y, p2.y, p3.y), 
                pressure: interpolate(p0.pressure, p1.pressure, p2.pressure, p3.pressure) 
            });
        }
    }
    smoothed
}