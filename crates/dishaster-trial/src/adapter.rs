use dishaster_models as models;
use dishaster_views as views;

pub trait ToView {
    type View;

    fn to_view(&self) -> Self::View;
}

pub trait ToViewWithId {
    type View;

    fn to_view_with_id(&self, index: usize) -> Self::View;
}

impl ToViewWithId for models::TrialSpeech {
    type View = views::TrialSpeech;

    fn to_view_with_id(&self, id: usize) -> Self::View {
        views::TrialSpeech {
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
            models::TrialSpeechItem::Text(t) => views::TrialSpeechItem::Text(t.clone()),
            models::TrialSpeechItem::Keyword(k) => views::TrialSpeechItem::Keyword(k.clone()),
            models::TrialSpeechItem::LineBreak => views::TrialSpeechItem::LineBreak,
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
