# 012_class_volatile - volatile 成员函数

## C++ 特性

本示例展示 C++ volatile 成员函数的 FFI 映射。

## C++ 代码

### class_volatile.h

```cpp
// volatile 成员函数 - 读取可能随时改变的硬件寄存器
uint32_t hardware_device_read_status(volatile struct HardwareDevice* self);
uint32_t hardware_device_read_data(volatile struct HardwareDevice* self);
```

### volatile 成员函数实现

```cpp
uint32_t hardware_device_read_status(volatile struct HardwareDevice* self) {
    // volatile 确保每次都真正读取硬件寄存器
    // 编译器不能将这个调用优化为只读一次
    return self->status_reg;
}
```

## volatile 限定符与 FFI

### C++ volatile

volatile 限定符用于硬件寄存器或其他可能意外改变的状态：

```cpp
class HardwareDevice {
    volatile uint32_t status_reg;  // 硬件状态寄存器
    volatile uint32_t data_reg;    // 数据寄存器
    uint32_t config_reg;           // 配置寄存器（可以缓存）
};
```

volatile 的含义：
1. **每次访问都必须读取**：编译器不能缓存值
2. **可能随时改变**：硬件DMA、中断服务程序可能修改
3. **顺序不能重排**：volatile 读写不能被编译器重排

### FFI 映射

| C++ 函数 | FFI 声明 |
|----------|----------|
| `uint32_t read() volatile` | `uint32_t read(volatile Class*)` |

关键点：`volatile` 修饰的是 `Class*` 指针指向的对象。

## Rust FFI 代码

```rust
// volatile 成员函数
#[cpp(func = "uint32_t hardware_device_read_status(volatile struct HardwareDevice*)")]
unsafe fn hardware_device_read_status(self_: *mut HardwareDevice) -> u32;
```

Rust 中不需要特殊的 volatile 限定符，因为：
- Rust 的 raw pointer 访问本身就是一次性的
- 编译器不会优化掉独立的指针解引用
- 但需要在 FFI 边界明确标注 volatile 的存在

## 关键点

### volatile vs const

| 限定符 | 含义 | 用途 |
|--------|------|------|
| const | 不能修改 | API 契约 |
| volatile | 可能意外改变 | 硬件/并发 |

### volatile 成员函数

```cpp
class Device {
    uint32_t read() volatile;  // 承诺不修改，且状态可能变
};
```

### FFI 注意事项

1. **volatile 修饰指针指向的对象**：`volatile Class*` 而不是 `Class* volatile`
2. **Rust 端**：通过每次独立读取来模拟 volatile 语义
3. **硬件访问**：通常与内存映射 I/O (mmio) 结合使用

## 总结

1. **volatile 成员函数**：用于硬件寄存器等可能意外改变的状态
2. **FFI 映射**：`this` 参数需要 `volatile` 限定符
3. **Rust 类型安全**：volatile 语义通过文档和每次独立读取来保持
## 运行结果

```
Reading volatile hardware registers (values may change):
  Read 0: status=0x12345678, data=0x00000000
  Read 1: status=0x12345678, data=0x00000000
  Read 2: status=0x12345678, data=0x00000000
  Read 3: status=0x12345678, data=0x00000000
  Read 4: status=0x12345678, data=0x00000000

Rust FFI: volatile qualifier requires volatile pointer in C
Note: In C, volatile on the pointed-to object matters for hardware registers
```
