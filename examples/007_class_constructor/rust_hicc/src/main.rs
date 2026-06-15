use class_constructor::*;

fn main() {
    let p1 = point_new_2(3, 4);
    println!("Point 1: ({}, {})", p1.get_x(), p1.get_y());
    println!("  Magnitude: {}", p1.get_magnitude());
    println!("  Angle: {}", p1.get_angle());

    println!("\nRust FFI: Constructor pattern works!");
}
