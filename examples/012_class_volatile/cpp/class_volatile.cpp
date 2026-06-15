#include "class_volatile.h"
#include <iostream>
#include <cstdint>

HardwareDevice::HardwareDevice() : status_reg(0xA5A5A5A5), data_reg(0), config_reg(0) {}

HardwareDevice::~HardwareDevice() {}

void HardwareDevice::init() {
    config_reg = 0x00000001;
    status_reg = 0x12345678;
    data_reg = 0;
}

void HardwareDevice::reset() {
    status_reg = 0xA5A5A5A5;
    data_reg = 0;
    config_reg = 0;
}
