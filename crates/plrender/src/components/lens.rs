// @TODO Implement lens correction

struct LensCorrectionComponent {
    camera_matrix: [[f32; 3]; 3],
    distortion_coefficients: [f32; 5],
}
