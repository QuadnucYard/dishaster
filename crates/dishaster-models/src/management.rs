//! Model definitions for Dishaster management decision/incident system
//!
//! We do not directly persist these models in save data, so they are placed
//! in `dishaster-models` instead of `dishaster-save-models`.

mod decisions;
mod incidents;

pub use self::{decisions::*, incidents::*};
use crate::prelude::*;

macro_rules! define_sum_model {
    ($name:ident { $( $variant:ident ),* $(,)? }) => { paste::paste! {
        // #[allow(missing_docs)]
        // #[derive(Debug, Clone, Deserialize)]
        // pub enum $name {
        //     $(
        //         $variant ( [<$variant Model> ]),
        //     )*
        // }

        // impl HasId for $name {
        //     fn id(&self) -> &ModelId {
        //         match self {
        //             $(
        //                 Self::$variant(m) => &m.id,
        //             )*
        //         }
        //     }
        // }

        // Template struct for the event
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Deserialize)]
        pub struct [<$name Template>] {
            /// Unique identifier for this event
            pub id: ModelId,
            /// Weight for random selection
            pub weight: u32,
            /// Icon representing the event
            pub icon: SpriteRef,
            /// Definition of the specific event
            pub def: [<$name TemplateDef>],
        }

        // Sum type for the event definitions
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Deserialize)]
        pub enum [<$name TemplateDef>] {
            $(
                $variant ( [<$variant Template> ]),
            )*
        }

        impl HasId for [<$name Template>] {
            fn id(&self) -> &ModelId {
                &self.id
            }
        }

        // For the instantiated models, we directly use the sum type, as the ID does not matter.
        #[allow(missing_docs)]
        #[derive(Debug, Clone)]
        pub enum [<$name Model>] {
            $(
                $variant ( [<$variant Model> ]),
            )*
        }
    }};
}

define_sum_model!(ManagementIncident {
    MislabelPrice,
    AttractionChange,
    TemporaryCrowd,
});

define_sum_model!(ManagementDecision {
    AddTables,
    RemoveTables,
    DisarrangeTables,
    OpenWindow,
    CloseWindow,
    ChangeWindowService,
    PlayMusic,
    AdvertiseCampaign,
    AddMotivationalSlogan,
    AddLuxuryDish,
});
