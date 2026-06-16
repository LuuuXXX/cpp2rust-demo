//! 012_class_volatile: volatile 数据成员（命名空间类 + make_unique 工厂）。
//!
//! hicc 模式：volatile 寄存器经普通 const 访问器（`read_status`/`read_data` → `&self`）
//! 读取，配置方法（`init`/`reset` → `&mut self`）可变；默认构造派生 make_unique 工厂。
//! hicc 不支持 `volatile`-this 限定的成员函数，故访问器采用非 volatile-this 写法，
//! 是 hicc 直出下的地道做法。本示例 `lib.rs` 与工具默认支架一致。

hicc::cpp! {
    #include "class_volatile.h"
}

hicc::import_class! {
    #[cpp(class = "class_volatile_ns::HardwareDevice")]
    pub class HardwareDevice {
        #[cpp(method = "uint32_t read_status() const")]
        pub fn read_status(&self) -> u32;

        #[cpp(method = "uint32_t read_data() const")]
        pub fn read_data(&self) -> u32;

        #[cpp(method = "void init()")]
        pub fn init(&mut self);

        #[cpp(method = "void reset()")]
        pub fn reset(&mut self);

        pub fn new() -> Self { hardware_device_new() }
    }
}

hicc::import_lib! {
    #![link_name = "class_volatile"]

    #[cpp(func = "std::unique_ptr<class_volatile_ns::HardwareDevice> hicc::make_unique<class_volatile_ns::HardwareDevice>()")]
    pub fn hardware_device_new() -> HardwareDevice;
}
