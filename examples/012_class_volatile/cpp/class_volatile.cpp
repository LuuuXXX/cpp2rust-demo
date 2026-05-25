#include "class_volatile.h"
#include <iostream>
#include <cstdint>

// HardwareDevice class implementations
HardwareDevice::HardwareDevice() : status_reg(0xA5A5A5A5), data_reg(0), config_reg(0) {}

HardwareDevice::~HardwareDevice() {}

volatile uint32_t HardwareDevice::readStatus() volatile {
    return status_reg;
}

volatile uint32_t HardwareDevice::readData() volatile {
    return data_reg;
}

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

// FFI wrapper functions
struct HardwareDevice* hardware_device_new(void) {
    return new HardwareDevice();
}

void hardware_device_delete(struct HardwareDevice* self) {
    delete self;
}

uint32_t hardware_device_read_status(volatile struct HardwareDevice* self) {
    return self->readStatus();
}

uint32_t hardware_device_read_data(volatile struct HardwareDevice* self) {
    return self->readData();
}

void hardware_device_init(struct HardwareDevice* self) {
    self->init();
}

void hardware_device_reset(struct HardwareDevice* self) {
    self->reset();
}
