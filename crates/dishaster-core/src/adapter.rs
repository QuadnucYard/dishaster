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

impl ToView for models::TrialSpeech {
    type View = views::TrialSpeech;

    fn to_view(&self) -> Self::View {
        views::TrialSpeech {
            text: self.text.clone(),
            items: self.items.iter().map(|item| item.to_view()).collect(),
            appearance: self.appearance.to_view(),
        }
    }
}

impl ToView for models::TrialSpeechItem {
    type View = views::TrialSpeechItem;

    fn to_view(&self) -> Self::View {
        match self {
            models::TrialSpeechItem::Text(t) => views::TrialSpeechItem::Text(t.clone()),
            models::TrialSpeechItem::Keyword(k) => views::TrialSpeechItem::Keyword(k.clone()),
            models::TrialSpeechItem::LineBreak => views::TrialSpeechItem::LineBreak,
        }
    }
}

impl ToView for models::TrialResponse {
    type View = views::TrialResponse;

    fn to_view(&self) -> Self::View {
        views::TrialResponse {
            kind: self.kind.to_view(),
            summary: self.summary.clone(),
            content: self.content.to_view(),
        }
    }
}

impl ToView for models::TrialResponseKind {
    type View = views::TrialResponseKind;

    fn to_view(&self) -> Self::View {
        match self {
            models::TrialResponseKind::Agreement => views::TrialResponseKind::Agreement,
            models::TrialResponseKind::Objection => views::TrialResponseKind::Objection,
            models::TrialResponseKind::Perjury => views::TrialResponseKind::Perjury,
            models::TrialResponseKind::Question => views::TrialResponseKind::Question,
        }
    }
}

impl ToView for models::TrialParticipantAppearance {
    type View = views::TrialParticipantAppearance;

    fn to_view(&self) -> Self::View {
        views::TrialParticipantAppearance {
            emotion: self.emotion,
            gesture: self.gesture,
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
