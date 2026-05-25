#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// 内联函数
inline int min(int a, int b) {
    return a < b ? a : b;
}

inline int max(int a, int b) {
    return a > b ? a : b;
}

// 普通函数（用于对比）
int min_v2(int a, int b);
int max_v2(int a, int b);

#ifdef __cplusplus
}
#endif
