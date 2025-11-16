use crate::{models, views};

pub trait ToView {
    type View;

    fn to_view(&self) -> Self::View;
}

pub trait ToModel {
    type Model;

    fn to_model(&self) -> Self::Model;
}

impl ToView for models::PricingMethod {
    type View = views::PricingMethod;

    fn to_view(&self) -> Self::View {
        match *self {
            models::PricingMethod::PerPortion(v) => views::PricingMethod::PerPortion(v),
            models::PricingMethod::ByWeight(v) => views::PricingMethod::ByWeight(v),
        }
    }
}

impl ToModel for views::PricingMethod {
    type Model = models::PricingMethod;

    fn to_model(&self) -> Self::Model {
        match *self {
            views::PricingMethod::PerPortion(v) => models::PricingMethod::PerPortion(v),
            views::PricingMethod::ByWeight(v) => models::PricingMethod::ByWeight(v),
        }
    }
}

impl ToView for models::FeedbackTopic {
    type View = views::FeedbackTopic;

    fn to_view(&self) -> Self::View {
        match *self {
            models::FeedbackTopic::Appeal => views::FeedbackTopic::Appeal,
            models::FeedbackTopic::Queue => views::FeedbackTopic::Queue,
            models::FeedbackTopic::Tableware => views::FeedbackTopic::Tableware,
            models::FeedbackTopic::Quality => views::FeedbackTopic::Quality,
            models::FeedbackTopic::Price => views::FeedbackTopic::Price,
            models::FeedbackTopic::Hygiene => views::FeedbackTopic::Hygiene,
            models::FeedbackTopic::Taste => views::FeedbackTopic::Taste,
            models::FeedbackTopic::Hunger => views::FeedbackTopic::Hunger,
            models::FeedbackTopic::Praise => views::FeedbackTopic::Praise,
        }
    }
}

impl ToModel for views::FeedbackTopic {
    type Model = models::FeedbackTopic;

    fn to_model(&self) -> Self::Model {
        match *self {
            views::FeedbackTopic::Appeal => models::FeedbackTopic::Appeal,
            views::FeedbackTopic::Queue => models::FeedbackTopic::Queue,
            views::FeedbackTopic::Tableware => models::FeedbackTopic::Tableware,
            views::FeedbackTopic::Quality => models::FeedbackTopic::Quality,
            views::FeedbackTopic::Price => models::FeedbackTopic::Price,
            views::FeedbackTopic::Hygiene => models::FeedbackTopic::Hygiene,
            views::FeedbackTopic::Taste => models::FeedbackTopic::Taste,
            views::FeedbackTopic::Hunger => models::FeedbackTopic::Hunger,
            views::FeedbackTopic::Praise => models::FeedbackTopic::Praise,
        }
    }
}

impl ToView for models::Appearance {
    type View = views::Appearance;

    fn to_view(&self) -> Self::View {
        views::Appearance {
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
        views::BodyPart {
            variant: self.variant,
            color_transform: self.color_transform,
        }
    }
}
