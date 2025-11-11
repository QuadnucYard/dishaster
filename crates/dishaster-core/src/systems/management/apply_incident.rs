use crate::{events::DispatchManagement, systems::prelude::*};

pub fn register_management_incident_systems(world: &mut World) {
    macro_rules! add_observers {
        ($($system: ident),* $(,)?) => {
            $( world.add_observer($system); )*
        };
    }

    add_observers!(apply_mislabel_price,);
}

fn apply_mislabel_price(
    event: On<DispatchManagement<MislabelPriceModel>>,
    mut dish_query: Query<(Entity, &mut Dish)>,
    mut rng: ResMut<WorldRng>,
    mut events: ResMut<EventQueue>,
) {
    let model = &event.0;
    for &rate in &model.overpriced_rates {
        // Choose a dish to mislabel
        let Some((dish_entity, mut dish)) = dish_query.iter_mut().choose(&mut rng) else {
            continue;
        };

        let (original_price, new_price) = match &mut dish.pricing {
            PricingMethod::PerPortion(v) | PricingMethod::ByWeight(v) => {
                let original_price = *v;
                let new_price = original_price * (1.0 + rate);
                *v = new_price;
                (original_price, new_price)
            }
        };

        log::info!(
            "Mislabeling price for dish {:?}: {:.2} -> {:.2}",
            dish.model_id,
            original_price,
            new_price
        );

        events.push(SimEvent::DishPriceChanged {
            entity: dish_entity.to_entity_id(),
            new_pricing: dish.pricing.to_view(),
        });
    }
}
