use noexcept_basic::*;

#[test]
fn smoke_noexcept_add() {
    assert_eq!(noexcept_add(10, 20), 30, "noexcept_add(10, 20) should return 30");
    assert_eq!(noexcept_add(-5, 5), 0, "noexcept_add(-5, 5) should return 0");
    assert_eq!(noexcept_add(0, 0), 0, "noexcept_add(0, 0) should return 0");
}

#[test]
fn smoke_noexcept_multiply() {
    assert_eq!(noexcept_multiply(6, 7), 42, "noexcept_multiply(6, 7) should return 42");
    assert_eq!(noexcept_multiply(-3, 4), -12, "noexcept_multiply(-3, 4) should return -12");
}

#[test]
fn smoke_conditional_abs() {
    assert_eq!(conditional_abs(-42), 42, "conditional_abs(-42) should return 42");
    assert_eq!(conditional_abs(42), 42, "conditional_abs(42) should return 42");
    assert_eq!(conditional_abs(0), 0, "conditional_abs(0) should return 0");
}

#[test]
fn smoke_noexcept_mover() {
    let mover = noexcept_mover_new(100);
    assert_eq!(mover.get_value(), 100, "NoexceptMover initial value should be 100");
}

#[test]
fn smoke_noexcept_mover_move() {
    use hicc::AbiClass;
    let mut mover1 = noexcept_mover_new(200);
    let mover2 = unsafe { noexcept_mover_move(&mover1.as_mut_ptr()) };
    assert_eq!(mover2.get_value(), 200, "moved NoexceptMover value should be 200");
}
