#pragma once

#include <cstdint>

class HardwareDevice {
    volatile uint32_t status_reg;
    volatile uint32_t data_reg;
    uint32_t config_reg;
public:
    HardwareDevice();
    ~HardwareDevice();
    void init();
    void reset();
};
