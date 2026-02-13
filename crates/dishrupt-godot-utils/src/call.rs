//! Utilities for creating Godot Callables from Rust functions.

use godot::{
    builtin::{Callable, Variant},
    meta::FromGodot,
};

/// Creates a `Callable` from a function that takes a slice of `Variant` references.
pub fn make_callable<F>(name: &str, mut func: F) -> Callable
where
    F: 'static + FnMut(&[&Variant]),
{
    Callable::from_fn(name.to_string(), move |vargs| {
        func(vargs);
    })
}

macro_rules! impl_connect_fn {
    ($name:ident; $fn_name: ident; $($args:ident)*; $($indiced:literal)*) => {
        /// Trait for connecting a signal to a local function with specific argument types.
        pub trait $name {
            /// Connects a signal to a local function.
            fn $fn_name<F, $($args,)*>(&mut self, signal: &str, func: F)
            where
                F: 'static + FnMut($($args,)*),
                $($args: FromGodot,)*
            ;
        }


        impl $name for godot::classes::Object {
            fn $fn_name<F, $($args,)*>(&mut self, signal: &str, mut func: F)
            where
                F: 'static + FnMut($($args,)*),
                $($args: FromGodot,)*
            {
                self.connect(
                    signal,
                    #[allow(unused_variables)]
                    &Callable::from_fn(signal.to_string(), move |vargs| {
                        func($(vargs[$indiced].to(),)*);
                    }),
                );
            }
        }
    };
}

impl_connect_fn!(ConnectLocalFn0; connect_fn_0; ; );
impl_connect_fn!(ConnectLocalFn1; connect_fn_1; T0; 0);
impl_connect_fn!(ConnectLocalFn2; connect_fn_2; T0 T1; 0 1);
