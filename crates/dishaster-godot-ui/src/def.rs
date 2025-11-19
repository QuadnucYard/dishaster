use dishrupt_asset::{AssetCatalog, AssetKind, ResourceLocator};
use dishrupt_godot_ui::*;
use godot::{
    classes::{Node, PackedScene},
    prelude::*,
};

use crate::*;

pub fn register_guis(registry: &mut GuiRegistry, catalog: &AssetCatalog) {
    macro_rules! create_gui {
        ($t: ty, $path: literal) => {{ <$t>::new(UINode(load_ui($path, catalog))) }};
    }

    macro_rules! register_gui {
        ($($t: ty => $path: literal),* $(,)?) => {
            $( registry.register(create_gui!($t, $path)); )*
        };
    }

    // NOTE: the following are registered and displayed by declaration order.
    register_gui!(
        StartMenuGui => "start/start_menu",
        CreditsGui => "start/credits",
        EndingGalleryGui => "start/ending_gallery",
        GamingLayout => "gaming/layout",
        TimeStatsGui => "gaming/time_stats",
        ReputationGui => "gaming/reputation",
        DishPricePopup => "gaming/price_editor",
        SettlementGui => "gaming/settlement",
        ManageDecisionGui => "gaming/decision_selection",
        ManageIncidentGui => "gaming/incident_notification",
        InspectorResultGui => "gaming/inspector_result",
        EndingGui => "gaming/ending",
        TrialGui => "gaming/trial",
        TrialImpactGui => "gaming/trial_impact",
        HintNotification => "gaming/hint",
        TutorialGui => "gaming/tutorial",
    );
}

fn load_ui<T>(path: &str, catalog: &AssetCatalog) -> Gd<T>
where
    T: Inherits<Node>,
{
    let Ok(ResourceLocator::Uri(uri)) = catalog.resolve(AssetKind::Gui, path) else {
        godot_error!("Failed to resolve GUI asset: {path}");
        panic!("Failed to resolve GUI asset: {path}");
    };
    load::<PackedScene>(&uri).instantiate_as()
}
