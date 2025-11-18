use dishaster_models as models;
use dishaster_views::{self as views, SpeechId};

pub trait ToView {
    type View;

    fn to_view(&self) -> Self::View;
}

pub trait ToViewWithId {
    type View;

    fn to_view_with_id(&self, index: SpeechId) -> Self::View;
}

impl ToViewWithId for models::TrialSpeech {
    type View = views::TrialSpeech;

    fn to_view_with_id(&self, id: SpeechId) -> Self::View {
        Self::View {
            id,
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
            Self::Text(t) => Self::View::Text(t.clone()),
            Self::Keyword(k) => Self::View::Keyword(k.clone()),
            Self::LineBreak => Self::View::LineBreak,
        }
    }
}

impl ToView for models::TrialResponseKind {
    type View = views::TrialResponseKind;

    fn to_view(&self) -> Self::View {
        match self {
            Self::Agreement => Self::View::Agreement,
            Self::Objection => Self::View::Objection,
            Self::Perjury => Self::View::Perjury,
            Self::Question => Self::View::Question,
        }
    }
}

impl ToView for models::TrialParticipantAppearance {
    type View = views::TrialParticipantAppearance;

    fn to_view(&self) -> Self::View {
        Self::View {
            emotion: self.emotion,
            gesture: self.gesture,
        }
    }
}
