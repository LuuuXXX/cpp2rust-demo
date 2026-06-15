use enum_class::*;

#[test]
fn smoke_operation_result_error() {
    let mut result = operation_result_new();
    result.set_error(ERROR_INVALID_INPUT);
    assert_eq!(result.get_error(), ERROR_INVALID_INPUT, "error code should be ERROR_INVALID_INPUT");

    result.set_error(ERROR_NOT_FOUND);
    assert_eq!(result.get_error(), ERROR_NOT_FOUND, "error code should be ERROR_NOT_FOUND");
}

#[test]
fn smoke_operation_result_state() {
    let mut result = operation_result_new();
    result.set_state(STATE_RUNNING);
    assert_eq!(result.get_state(), STATE_RUNNING, "state should be STATE_RUNNING");

    result.set_state(STATE_PAUSED);
    assert_eq!(result.get_state(), STATE_PAUSED, "state should be STATE_PAUSED");
}

#[test]
fn smoke_flags() {
    let mut result = operation_result_new();
    result.set_flags(FLAG_READ | FLAG_WRITE);
    let flags = result.get_flags();
    assert_eq!(flags, FLAG_READ | FLAG_WRITE, "flags should be READ|WRITE");
    assert!(has_flag(flags, FLAG_READ) != 0, "should contain FLAG_READ");
    assert!(has_flag(flags, FLAG_WRITE) != 0, "should contain FLAG_WRITE");
    assert!(has_flag(flags, FLAG_EXECUTE) == 0, "should not contain FLAG_EXECUTE");
}

#[test]
fn smoke_combine_flags() {
    let combined = combine_flags(FLAG_READ, FLAG_EXECUTE);
    assert_eq!(combined, FLAG_READ | FLAG_EXECUTE, "combine_flags should merge two flags");
}

#[test]
fn smoke_error_code_name() {
    assert_eq!(error_code_name(0), "None");
    assert_eq!(error_code_name(1), "InvalidInput");
    assert_eq!(error_code_name(3), "NotFound");
    assert_eq!(error_code_name(99), "Unknown");
}

#[test]
fn smoke_state_name() {
    assert_eq!(state_name(0), "Idle");
    assert_eq!(state_name(1), "Running");
    assert_eq!(state_name(2), "Paused");
    assert_eq!(state_name(3), "Stopped");
    assert_eq!(state_name(99), "Unknown");
}
