pub trait Depth {
    type Next: Depth;
}

pub trait Depth0_4 {}
pub trait Depth1_4 {}
pub trait Depth2_4 {}
pub trait Depth3_4 {}
pub trait Depth4_4 {}
pub trait Depth0_3 {}
pub trait Depth0_2 {}
pub trait Depth0_1 {}
pub trait Depth0_0 {}

pub struct Depth0;
impl Depth for Depth0 {
    type Next = Depth1;
}
impl Depth0_4 for Depth0 {}
impl Depth0_3 for Depth0 {}
impl Depth0_2 for Depth0 {}
impl Depth0_1 for Depth0 {}
impl Depth0_0 for Depth0 {}

pub struct Depth1;
impl Depth for Depth1 {
    type Next = Depth2;
}
impl Depth0_4 for Depth1 {}
impl Depth0_3 for Depth1 {}
impl Depth0_2 for Depth1 {}
impl Depth0_1 for Depth1 {}
impl Depth1_4 for Depth1 {}

pub struct Depth2;
impl Depth for Depth2 {
    type Next = Depth3;
}
impl Depth0_4 for Depth2 {}
impl Depth0_3 for Depth2 {}
impl Depth0_2 for Depth2 {}
impl Depth1_4 for Depth2 {}
impl Depth2_4 for Depth2 {}

pub struct Depth3;
impl Depth for Depth3 {
    type Next = Depth4;
}
impl Depth0_4 for Depth3 {}
impl Depth0_3 for Depth3 {}
impl Depth1_4 for Depth3 {}
impl Depth2_4 for Depth3 {}
impl Depth3_4 for Depth3 {}

pub struct Depth4;
impl Depth for Depth4 {
    type Next = Depth5;
}
impl Depth0_4 for Depth4 {}
impl Depth1_4 for Depth4 {}
impl Depth2_4 for Depth4 {}
impl Depth3_4 for Depth4 {}
impl Depth4_4 for Depth4 {}

pub struct Depth5;
impl Depth for Depth5 {
    type Next = Depth5;
}
