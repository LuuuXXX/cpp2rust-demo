#pragma once
#include <cstdint>
#include <iostream>

namespace class_volatile_ns {

// 演示 volatile 成员变量：寄存器以 `volatile` 修饰，禁止编译器缓存/优化掉读取，
// 保证每次访问都真实读内存（模拟内存映射硬件寄存器）。
//
// 注：hicc 不支持 `volatile`-this 限定的成员函数（方法指针类型不匹配），故访问器
// 采用普通 const/非 const 方法读取 volatile 数据成员，是 hicc 直出下的地道写法。
class HardwareDevice {
public:
    HardwareDevice()
        : status_reg_(0xA5A5A5A5u), data_reg_(0u), config_reg_(0u) {
        std::cout << "HardwareDevice() ctor" << std::endl;
    }
    ~HardwareDevice() {
        std::cout << "~HardwareDevice() dtor" << std::endl;
    }

    // 读取 volatile 寄存器（只读 → &self）。
    uint32_t read_status() const { return status_reg_; }
    uint32_t read_data() const { return data_reg_; }

    // 配置（可变 → &mut self）。
    void init() {
        config_reg_ = 0x00000001u;
        status_reg_ = 0x12345678u;
        data_reg_ = 0u;
    }
    void reset() {
        status_reg_ = 0xA5A5A5A5u;
        data_reg_ = 0u;
        config_reg_ = 0u;
    }

private:
    volatile uint32_t status_reg_;
    volatile uint32_t data_reg_;
    uint32_t config_reg_;
};

} // namespace class_volatile_ns
