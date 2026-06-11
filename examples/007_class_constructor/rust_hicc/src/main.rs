use class_constructor::*;

fn main() {
    // Use Cartesian coordinates constructor
    let p1 = point_new_xy(3, 4);
    println!("Point 1: ({}, {})", p1.get_x(), p1.get_y());
    println!("  Magnitude: {}", p1.get_magnitude());
    println!("  Angle: {}", p1.get_angle());

    println!();

    // Use polar coordinates constructor
    let p2 = point_new_polar(5.0, 0.0);
    println!("Point 2: ({}, {})", p2.get_x(), p2.get_y());
    println!("  Magnitude: {}", p2.get_magnitude());
    println!("  Angle: {}", p2.get_angle());

    println!("\nRust FFI: Multiple constructors work!");
}
