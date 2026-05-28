use std::convert::From;
use std::fmt;
use std::mem::ManuallyDrop;
use std::str;

/// 保存`c++`异常信息
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ExceptionInfo([u8; 64]);

impl ExceptionInfo {
    fn new(info: &str) -> Self {
        let mut this = Self([0_u8; 64]);
        let len = info.len().min(this.0.len());
        this.0[..len].copy_from_slice(&info.as_bytes()[..len]);
        this.0[63] = 0;
        this
    }
    pub fn what(&self) -> &str {
        if let Some(slice) = self.0.splitn(2, |num| *num == 0).next() {
            let mut size = slice.len();
            while size > 0 {
                if let Ok(err) = str::from_utf8(&slice[..size]) {
                    return err;
                }
                size -= 1;
            }
        }
        "unknown exception"
    }
}
impl fmt::Debug for ExceptionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.what().fmt(f)
    }
}

#[repr(C)]
union ExceptionValue<T> {
    except: ExceptionInfo,
    value: ManuallyDrop<T>,
}

/// 对应`c++`接口抛出的异常
///
/// 需要和`c++`侧的`EXPORT_EXCEPT_METHOD/EXPORT_EXCEPT_MEMBER_METHOD`配套使用.
///
/// 可转换为`Result<T, ExceptionInfo>`.
///
#[repr(C)]
#[must_use]
pub struct Exception<T> {
    value: ExceptionValue<T>,
    has_except: bool,
}

impl<T> Exception<T> {
    fn new_value(value: T) -> Self {
        Self {
            has_except: false,
            value: ExceptionValue {
                value: ManuallyDrop::new(value),
            },
        }
    }

    fn new_except(info: &str) -> Self {
        Self {
            has_except: true,
            value: ExceptionValue {
                except: ExceptionInfo::new(info),
            },
        }
    }

    pub fn ok(self) -> Result<T, ExceptionInfo> {
        if !self.has_except {
            let mut this = ManuallyDrop::new(self);
            Ok(unsafe { ManuallyDrop::take(&mut this.value.value) })
        } else {
            Err(unsafe { self.value.except })
        }
    }
}

impl<T> Drop for Exception<T> {
    fn drop(&mut self) {
        if !self.has_except {
            unsafe { ManuallyDrop::drop(&mut self.value.value) };
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Exception<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has_except {
            unsafe { write!(f, "Err({:?})", self.value.except) }
        } else {
            unsafe { write!(f, "Ok({:?})", &*self.value.value) }
        }
    }
}

impl<T, E: fmt::Debug> From<Result<T, E>> for Exception<T> {
    fn from(val: Result<T, E>) -> Self {
        match val {
            Ok(val) => Self::new_value(val),
            Err(e) => {
                let s = format!("{e:?}");
                Self::new_except(&s)
            }
        }
    }
}
