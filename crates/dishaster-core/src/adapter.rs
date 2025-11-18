//! Adapter traits and implementations for converting between models and views.

use crate::{models, views};

/// Trait for converting a model to its corresponding view.
pub trait ToView {
    /// The corresponding view type.
    type View;

    /// Convert the model to its corresponding view.
    fn to_view(&self) -> Self::View;
}

/// Trait for converting a view to its corresponding model.
pub trait ToModel {
    /// The corresponding model type.
    type Model;

    /// Convert the view to its corresponding model.
    fn to_model(&self) -> Self::Model;
}

impl ToView for models::PricingMethod {
    type View = views::PricingMethod;

    fn to_view(&self) -> Self::View {
        match *self {
            Self::PerPortion(v) => Self::View::PerPortion(v),
            Self::ByWeight(v) => Self::View::ByWeight(v),
        }
    }
}

impl ToModel for views::PricingMethod {
    type Model = models::PricingMethod;

    fn to_model(&self) -> Self::Model {
        match *self {
            Self::PerPortion(v) => Self::Model::PerPortion(v),
            Self::ByWeight(v) => Self::Model::ByWeight(v),
        }
    }
}

impl ToView for models::FeedbackTopic {
    type View = views::FeedbackTopic;

    fn to_view(&self) -> Self::View {
        match *self {
            Self::Appeal => Self::View::Appeal,
            Self::Queue => Self::View::Queue,
            Self::Tableware => Self::View::Tableware,
            Self::Quality => Self::View::Quality,
            Self::Price => Self::View::Price,
            Self::Hygiene => Self::View::Hygiene,
            Self::Taste => Self::View::Taste,
            Self::Hunger => Self::View::Hunger,
            Self::Praise => Self::View::Praise,
            Self::Crab => Self::View::Crab,
        }
    }
}

impl ToModel for views::FeedbackTopic {
    type Model = models::FeedbackTopic;

    fn to_model(&self) -> Self::Model {
        match *self {
            Self::Appeal => Self::Model::Appeal,
            Self::Queue => Self::Model::Queue,
            Self::Tableware => Self::Model::Tableware,
            Self::Quality => Self::Model::Quality,
            Self::Price => Self::Model::Price,
            Self::Hygiene => Self::Model::Hygiene,
            Self::Taste => Self::Model::Taste,
            Self::Hunger => Self::Model::Hunger,
            Self::Praise => Self::Model::Praise,
            Self::Crab => Self::Model::Crab,
        }
    }
}

impl ToView for models::Appearance {
    type View = views::Appearance;

    fn to_view(&self) -> Self::View {
        Self::View {
            head: self.head.to_view(),
            upper_garment: self.upper_garment.to_view(),
            lower_garment: self.lower_garment.to_view(),
            hands: self.hands.to_view(),
            shoes: self.shoes.to_view(),
        }
    }
}

impl ToView for models::BodyPart {
    type View = views::BodyPart;

    fn to_view(&self) -> Self::View {
        Self::View {
            variant: self.variant,
            color_transform: self.color_transform,
        }
    }
}
