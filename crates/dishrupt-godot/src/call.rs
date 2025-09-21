use godot::{
    builtin::{Callable, Variant},
    meta::FromGodot,
};

pub fn make_callable<F>(name: &str, mut func: F) -> Callable
where
    F: 'static + FnMut(&[&Variant]),
{
    Callable::from_local_fn(name, move |vargs| {
        func(vargs);
        Ok(Variant::nil())
    })
}

macro_rules! impl_connect_local_fn {
    ($name:ident; $fn_name: ident; $($args:ident)*; $($indiced:literal)*) => {

        pub trait $name {
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
                    &Callable::from_local_fn(signal, move |vargs| {
                        func($(vargs[$indiced].to(),)*);
                        Ok(Variant::nil())
                    }),
                );
            }
        }
    };
}

impl_connect_local_fn!(ConnectLocalFn0; connect_local_fn_0; ; );
impl_connect_local_fn!(ConnectLocalFn1; connect_local_fn_1; T0; 0);
impl_connect_local_fn!(ConnectLocalFn2; connect_local_fn_2; T0 T1; 0 1);
