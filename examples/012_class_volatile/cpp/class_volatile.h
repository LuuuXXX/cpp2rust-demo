#pragma once

#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

struct HardwareDevice;

struct HardwareDevice* hardware_device_new(void);
void hardware_device_delete(struct HardwareDevice* self);

// volatile 成员函数 - 读取可能随时改变的硬件寄存器
uint32_t hardware_device_read_status(volatile struct HardwareDevice* self);
uint32_t hardware_device_read_data(volatile struct HardwareDevice* self);

// 非 volatile 成员函数 - 配置
void hardware_device_init(struct HardwareDevice* self);
void hardware_device_reset(struct HardwareDevice* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
class HardwareDevice {
    volatile uint32_t status_reg;
    volatile uint32_t data_reg;
    uint32_t config_reg;
public:
    HardwareDevice();
    ~HardwareDevice();
    volatile uint32_t readStatus() volatile;
    volatile uint32_t readData() volatile;
    void init();
    void reset();
};

#endif
