use inheritance_single::*;

fn show(s: &hicc_std::string) -> String {
    let cs = unsafe { std::ffi::CStr::from_ptr(s.c_str()) };
    cs.to_str().unwrap().to_string()
}

fn main() {
    let animal = Animal::new(hicc_std::string::from(c"Generic"));
    let dog = Dog::new(hicc_std::string::from(c"Buddy"));

    println!("{}: {}", show(&animal.name()), show(&animal.speak()));
    println!("dog speak: {}", show(&dog.speak()));
    println!("dog bark : {}", show(&dog.bark()));
}
