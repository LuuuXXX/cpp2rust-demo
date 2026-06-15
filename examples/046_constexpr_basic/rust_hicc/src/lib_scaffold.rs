#[allow(unused_imports)]
use crate::*;

hicc::cpp! {
#include <cstddef>
#include "constexpr_basic.h"
using ConstexprPoint = example::ConstexprPoint;
}

hicc::import_class! {
#[cpp(class = "ConstexprPoint")]
pub class ConstexprPoint {
#[cpp(method = "int manhattan_distance() const")]
pub fn manhattan_distance(&self) -> i32;
}
}

hicc::import_lib! {
#![link_name = "constexpr_basic"]

class ConstexprPoint;

#[cpp(func = "std::unique_ptr<ConstexprPoint> std::make_unique<ConstexprPoint>(int, int)")]
pub fn constexpr_point_new_2(x: i32, y: i32) -> ConstexprPoint;
}
