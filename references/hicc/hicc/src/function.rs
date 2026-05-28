use std::marker::PhantomData;

///
/// 辅助`Function`的实现.
///
pub trait FunctionType<T> {
    type CallFunction;
}

struct FunctionImpl<T, M = T>(T, PhantomData<M>);

impl<T> FunctionImpl<T, T> {
    fn new(val: T) -> Self {
        Self(val, PhantomData)
    }
}

/// 对应`c++`的`std::function<R(...)>`类型.
///
/// 可和`Fn`相互转换
///
#[repr(C)]
#[derive(Debug)]
pub struct Function<T: FunctionType<T>, M = T> {
    call: T::CallFunction,
    destroy: extern "C" fn(*const ()),
    ctx: *const (),
    _mark: PhantomData<M>,
}

impl<T: FunctionType<T>, M> Drop for Function<T, M> {
    fn drop(&mut self) {
        (self.destroy)(self.ctx);
    }
}

unsafe impl<T: FunctionType<T>, M> Send for Function<T, M> {}

macro_rules! function_declare {
    ($retn: ident) => {
        function_declare!($retn,);
    };
    ($retn: ident, $($arg: ident : $ty: ident),*) => {
        impl<HiccF, $retn, $($ty),*> FunctionImpl<HiccF, ($retn, $($ty),*)>
        where
            HiccF: Fn($($ty),*) -> $retn,
        {
            extern "C" fn call(this: *const (), $($arg: $ty),*) -> $retn {
                let this = unsafe { &mut *this.cast::<Self>().cast_mut() };
                (this.0)($($arg),*)
            }
            extern "C" fn destroy(this: *const ()) {
                let _ = unsafe { Box::from_raw(this.cast::<Self>().cast_mut()) };
            }
        }

        impl<$retn, $($ty),*> FunctionType<fn($($ty),*) -> $retn> for fn($($ty),*) -> $retn {
            type CallFunction = extern "C" fn(*const (), $($ty),*) -> $retn;
        }

        impl<HiccF, $retn, $($ty),*> From<HiccF> for Function<fn($($ty),*) -> $retn>
        where
            HiccF: Fn($($ty),*) -> $retn,
        {
            fn from(val: HiccF) -> Self {
                Self {
                    call: FunctionImpl::<HiccF, ($retn, $($ty),*)>::call,
                    destroy: FunctionImpl::<HiccF, ($retn, $($ty),*)>::destroy,
                    ctx: Box::into_raw(Box::new(FunctionImpl::new(val))).cast::<()>(),
                    _mark: PhantomData,
                }
            }
        }

        impl<$retn, $($ty),*> Function<fn($($ty),*) -> $retn> {
            pub fn into(self) -> impl Fn($($ty),*) -> $retn {
                let this = ::std::sync::Arc::new(self);
                move |$($arg),*| (this.call)(this.ctx, $($arg),*)
            }
        }
    }
}

function_declare!(R);
function_declare!(R, p: T);
function_declare!(R, p1: T1, p2: T2);
function_declare!(R, p1: T1, p2: T2, p3: T3);
function_declare!(R, p1: T1, p2: T2, p3: T3, p4: T4);
function_declare!(R, p1: T1, p2: T2, p3: T3, p4: T4, p5: T5);
function_declare!(R, p1: T1, p2: T2, p3: T3, p4: T4, p5: T5, p6: T6);
function_declare!(R, p1: T1, p2: T2, p3: T3, p4: T4, p5: T5, p6: T6, p7: T7);
function_declare!(R, p1: T1, p2: T2, p3: T3, p4: T4, p5: T5, p6: T6, p7: T7, p8: T8);
function_declare!(R, p1: T1, p2: T2, p3: T3, p4: T4, p5: T5, p6: T6, p7: T7, p8: T8, p9: T9);
function_declare!(R, p1: T1, p2: T2, p3: T3, p4: T4, p5: T5, p6: T6, p7: T7, p8: T8, p9: T9, p10: T10);
function_declare!(R, p1: T1, p2: T2, p3: T3, p4: T4, p5: T5, p6: T6, p7: T7, p8: T8, p9: T9, p10: T10, p11: T11);
function_declare!(R, p1: T1, p2: T2, p3: T3, p4: T4, p5: T5, p6: T6, p7: T7, p8: T8, p9: T9, p10: T10, p11: T11, p12: T12);
function_declare!(R, p1: T1, p2: T2, p3: T3, p4: T4, p5: T5, p6: T6, p7: T7, p8: T8, p9: T9, p10: T10, p11: T11, p12: T12, p13: T13);
function_declare!(R, p1: T1, p2: T2, p3: T3, p4: T4, p5: T5, p6: T6, p7: T7, p8: T8, p9: T9, p10: T10, p11: T11, p12: T12, p13: T13, p14: T14);
function_declare!(R, p1: T1, p2: T2, p3: T3, p4: T4, p5: T5, p6: T6, p7: T7, p8: T8, p9: T9, p10: T10, p11: T11, p12: T12, p13: T13, p14: T14, p15: T15);
