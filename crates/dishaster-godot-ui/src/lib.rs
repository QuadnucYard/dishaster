mod def;
mod gaming;
mod start;

use std::{borrow::Cow, collections::HashMap};

pub use def::register_guis;
use dishaster_views::{ParamValue, ParamsMap};
use dishrupt_core::prelude::EcoString;
pub use gaming::*;
pub use start::*;

mod prelude {
    pub use dishaster_ui_protocol::*;
    pub use dishrupt_godot_ui::*;
    pub use dishrupt_godot_ui_macros::*;
    pub use dishrupt_godot_widgets::*;
    pub use dishrupt_l10n_godot::tr;
    pub use signals2::*;

    pub use crate::ToFluent;
}

use dishrupt_l10n_godot::fluent::{FluentValue, types::FluentNumber};

pub trait ToFluent<'s> {
    type Output;

    fn to_fluent(&'s self) -> Self::Output;
}

impl<'s> ToFluent<'s> for ParamsMap {
    type Output = HashMap<EcoString, FluentValue<'s>>;

    fn to_fluent(&'s self) -> HashMap<EcoString, FluentValue<'s>> {
        let mut map = HashMap::new();
        for (k, v) in self.0.iter() {
            map.insert(k.clone(), v.to_fluent());
        }
        map
    }
}

impl<'s> ToFluent<'s> for ParamValue {
    type Output = FluentValue<'s>;

    fn to_fluent(&'s self) -> FluentValue<'s> {
        match self {
            ParamValue::String(s) => FluentValue::String(Cow::Borrowed(s.as_str())),
            ParamValue::Int(i) => {
                FluentValue::Number(FluentNumber::new(*i as f64, Default::default()))
            }
            ParamValue::Float(f) => FluentValue::Number(FluentNumber::new(*f, Default::default())),
        }
    }
}
