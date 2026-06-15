// 012_class_volatile 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「含 volatile 数据成员的命名空间类」默认生成的
// hicc 骨架。volatile 寄存器以普通 const/非 const 访问器暴露（hicc 不支持
// volatile-this 方法），默认构造派生 make_unique 工厂；本示例无需手写补全。

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
