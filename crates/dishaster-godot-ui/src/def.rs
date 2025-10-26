use dishrupt_godot::display::assets;
use dishrupt_godot_ui::*;
use godot::{
    classes::{Node, PackedScene},
    prelude::*,
};

use crate::*;

macro_rules! create_gui {
    ($t: ty, $path: literal) => {{ <$t>::new(UINode(load_ui($path))) }};
}

macro_rules! register_gui {
    ($r: ident, $($t: ty => $path: literal),* $(,)?) => {
        $( $r.register(create_gui!($t, $path)); )*
    };
}

pub fn register_guis(registry: &mut GuiRegistry) {
    // NOTE: the following are registered by declaration order.
    register_gui!(registry,
        StartMenuUI => "start/start_menu",
        GamingLayout => "gaming/layout",
        DishPricePopup => "gaming/price_editor",
        SettlementGui => "gaming/settlement",
        TimeStatsGui => "gaming/time_stats",
        TrialGui => "gaming/trial",
    );
}

fn load_ui<T>(path: &str) -> Gd<T>
where
    T: Inherits<Node>,
{
    load::<PackedScene>(&format!("{}{path}.tscn", assets::GUI)).instantiate_as()
}
