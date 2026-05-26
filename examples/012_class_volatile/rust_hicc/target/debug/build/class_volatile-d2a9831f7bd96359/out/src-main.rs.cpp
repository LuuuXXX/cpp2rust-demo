#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <cstdint>
    #include <iostream>

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

    HardwareDevice* hardware_device_new() {
        return new HardwareDevice();
    }

    void hardware_device_delete(HardwareDevice* self) {
        delete self;
    }

    uint32_t hardware_device_read_status(HardwareDevice* self) {
        return self->readStatus();
    }

    uint32_t hardware_device_read_data(HardwareDevice* self) {
        return self->readData();
    }

    void hardware_device_init(HardwareDevice* self) {
        self->init();
    }

    void hardware_device_reset(HardwareDevice* self) {
        self->reset();
    }
#line 68
 struct HardwareDevice_68;
#line 68
namespace hicc { template<> struct MethodsType<HardwareDevice, void> { typedef HardwareDevice_68 methods_type; }; }
#line 68
 struct HardwareDevice_68 {
#line 68
typedef HardwareDevice Self; typedef void SelfContainer; typedef HardwareDevice_68 SelfMethods;
#line 70
static void _hicc_test_70() { void (Self::* _70)() = &Self::init; (void)_70; }
#line 70
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::init));
#line 73
static void _hicc_test_73() { void (Self::* _73)() = &Self::reset; (void)_73; }
#line 73
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::reset));
#line 68
};
#line 79
EXPORT_METHODS_BEG(class_volatile) {
#line 83
static void _hicc_test_83() { HardwareDevice* (* _83)() = &hardware_device_new; (void)_83; }
#line 83
EXPORT_METHOD_IN(void, ExportMethods, ((HardwareDevice* (*)())&hardware_device_new));
#line 86
static void _hicc_test_86() { void (* _86)(HardwareDevice* self) = &hardware_device_delete; (void)_86; }
#line 86
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(HardwareDevice* self))&hardware_device_delete));
#line 89
static void _hicc_test_89() { uint32_t (* _89)(HardwareDevice* self) = &hardware_device_read_status; (void)_89; }
#line 89
EXPORT_METHOD_IN(void, ExportMethods, ((uint32_t (*)(HardwareDevice* self))&hardware_device_read_status));
#line 92
static void _hicc_test_92() { uint32_t (* _92)(HardwareDevice* self) = &hardware_device_read_data; (void)_92; }
#line 92
EXPORT_METHOD_IN(void, ExportMethods, ((uint32_t (*)(HardwareDevice* self))&hardware_device_read_data));
#line 95
static void _hicc_test_95() { void (* _95)(HardwareDevice* self) = &hardware_device_init; (void)_95; }
#line 95
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(HardwareDevice* self))&hardware_device_init));
#line 98
static void _hicc_test_98() { void (* _98)(HardwareDevice* self) = &hardware_device_reset; (void)_98; }
#line 98
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(HardwareDevice* self))&hardware_device_reset));
#line 79
} EXPORT_METHODS_END();

