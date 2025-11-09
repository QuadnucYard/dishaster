use dishrupt_core::prelude::*;

/// Map of parameters for localization formatting
#[derive(Debug, Clone)]
pub struct ParamsMap(pub Vec<(EcoString, ParamValue)>);

/// Macro to create a ParamsMap easily
#[macro_export]
macro_rules! params {
    ( $($key:ident => $value: expr),* $(,)? ) => {
        {
            let mut vec = Vec::new();
            $(
                vec.push((stringify!($key).into(), $value.into()));
            )*
            ParamsMap(vec)
        }
    };
}

/// Parameter value used for localization formatting
#[derive(Debug, Clone)]
pub enum ParamValue {
    /// Integer value
    Int(i64),
    /// Floating-point value
    Float(f64),
    /// String value
    String(EcoString),
}

macro_rules! impl_from_value {
    ($ty:ty as $casted:ty => $variant:ident) => {
        impl From<$ty> for ParamValue {
            fn from(value: $ty) -> Self {
                Self::$variant(value as $casted)
            }
        }
    };
    ($ty:ty => $variant:ident) => {
        impl From<$ty> for ParamValue {
            fn from(value: $ty) -> Self {
                Self::$variant(value.into())
            }
        }
    };
}

impl_from_value!(i8 => Int);
impl_from_value!(i16 => Int);
impl_from_value!(i32 => Int);
impl_from_value!(i64 => Int);
impl_from_value!(u8 => Int);
impl_from_value!(u16 => Int);
impl_from_value!(u32 => Int);
impl_from_value!(u64 as i64 => Int);
impl_from_value!(usize as i64 => Int);
impl_from_value!(f32 => Float);
impl_from_value!(f64 => Float);
impl_from_value!(&str => String);
impl_from_value!(EcoString => String);
