use dishaster_views::{
    TrialIntro, TrialParticipantAppearance, TrialResponseKind, TrialResponseOption, TrialSpeech,
    TrialSpeechItem, TrialStatement,
};
use dishrupt_core::prelude::*;
use godot::{classes::AnimationPlayer, prelude::*};

use crate::{prelude::*, req::GameRequest};

const INTRO_TIME: f32 = 2.0;
const COUNTDOWN_TIME: f32 = 3.0;

#[derive(Debug, Default)]
struct State {
    pub time: f32,
    pub phase: Phase,
    pub inner_speech_text: EcoString,
    pub fade_time: f32,
    pub options: Vec<Vec<TrialResponseOption>>,
}

#[derive(Debug, Default)]
enum Phase {
    #[default]
    Idle,
    Intro,
    LeftSpeaking,
    RightSpeaking,
}

#[derive(UITree)]
#[ui_tree]
pub struct TrialGui {
    #[subtree("%Left")]
    left: TrialSpeechGui,
    #[subtree("%Right")]
    right: TrialSpeechGui,

    #[child("%TimeProgress")]
    time_progress: ProgressBarA,

    #[node("%AnimationPlayer")]
    anim_player: Gd<AnimationPlayer>,

    #[subtree("%Thought")]
    thought: TrialThoughtGui,

    state: Option<State>,
}

impl TrialGui {
    pub fn intro(&mut self, e: TrialIntro) {
        self.state = Some(State {
            phase: Phase::Intro,
            ..Default::default()
        });

        self.left.set_appearance(&e.left);
        self.right.set_appearance(&e.right);
        self.left.content.set_visible(false);
        self.right.content.set_visible(false);
        self.time_progress.set_visible(false);

        // Play intro animation
        self.anim_player.play_ex().name("intro").done();
        self.anim_player.seek(0.0);
    }

    pub fn left_speak(&mut self, statement: TrialStatement) {
        let speech = statement.speech;
        // The speech may contain keywords for the left side.
        let inner_text = speech_to_bbcode(&speech);
        self.right.set_visible(false);
        self.left.set_speech(&speech.appearance, &inner_text);
        self.left.set_visible(true);
        self.state = Some(State {
            phase: Phase::LeftSpeaking,
            time: 0.0,
            inner_speech_text: inner_text,
            fade_time: estimate_fade_time(&speech.text),
            options: statement.options,
        });
    }

    pub fn right_speak(&mut self, speech: TrialSpeech) {
        let inner_text = speech_to_bbcode(&speech);
        self.left.set_visible(false);
        self.right.set_speech(&speech.appearance, &inner_text);
        self.right.set_visible(true);
        self.state = Some(State {
            phase: Phase::RightSpeaking,
            time: 0.0,
            inner_speech_text: inner_text,
            fade_time: estimate_fade_time(&speech.text),
            options: Vec::new(),
        });
        self.time_progress.set_visible(false);
    }

    pub fn check_keyword(&mut self, keyword_index: usize) {
        if let Some(state) = &self.state
            && let Phase::LeftSpeaking = state.phase
            && let Some(options) = state.options.get(keyword_index)
        {
            self.thought.set_options(options);
            self.thought.set_visible(true);
        }
    }

    pub fn back_from_thought(&mut self) {
        self.thought.set_visible(false);
    }

    pub fn finish_thought(&mut self) {
        self.thought.set_visible(false);
        self.right.set_visible(false);
    }
}

#[ui_tree_api]
impl UITree for TrialGui {}

impl Gui for TrialGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.left.content.on_meta_click.connect(move |meta| {
            godot_print!("Left content meta clicked: {:?}", meta);
            let keyword_index = meta.to::<GString>().to_string().parse().unwrap_or_default();
            cmd.push_req(GameRequest::TrialCheckKeyword(keyword_index));
        });

        let cmd = commands.clone();
        self.thought.back_button.on_click.connect(move || {
            cmd.push_req(GameRequest::TrialBackFromThought);
        });

        self.thought.set_visible(false);

        let cmd = commands.clone();
        self.thought.on_select_option.connect(move |corpus_index| {
            godot_print!("Thought option selected: {}", corpus_index);
            cmd.push_req(GameRequest::TrialRespond(corpus_index));
        });
    }

    fn process(&mut self, cmd: GuiCommands, delta: f64) {
        let Some(state) = &mut self.state else {
            return;
        };

        state.time += delta as f32;

        match state.phase {
            Phase::Intro => {
                if state.time > INTRO_TIME {
                    state.phase = Phase::Idle;
                    self.left.set_visible(false);
                    self.right.set_visible(false);
                    // wait for providing left content
                    cmd.push_req(GameRequest::TrialIntroDone);
                }
            }
            Phase::LeftSpeaking => {
                self.left
                    .content
                    .set_text(&faded(&state.inner_speech_text, state.time));
                if state.time > state.fade_time {
                    self.time_progress.set_visible(true);
                    self.time_progress
                        .set_value((state.time - state.fade_time) / COUNTDOWN_TIME);
                    if state.time > state.fade_time + COUNTDOWN_TIME {
                        state.phase = Phase::Idle;
                        cmd.push_req(GameRequest::TrialTimeout);
                    }
                }
            }
            Phase::RightSpeaking => {
                self.right
                    .content
                    .set_text(&faded(&state.inner_speech_text, state.time));
                if state.time > state.fade_time + 1.0 {
                    state.phase = Phase::Idle;
                    cmd.push_req(GameRequest::TrialResponseDone);
                }
            }
            _ => { /* TODO */ }
        }
    }
}

#[derive(UITree)]
#[ui_tree]
pub struct TrialSpeechGui {
    // #[child("Face")]
    // face: TextureRectA,
    // #[child("Face/Gesture")]
    // gesture: TextureRectA,
    #[child("FaceAlt")]
    face_alt: LabelA,
    #[child("FaceAlt/GestureAlt")]
    gesture_alt: LabelA,
    #[child("Content")]
    content: RichLabelA,
}

impl TrialSpeechGui {
    pub(self) fn set_speech(&mut self, appearance: &TrialParticipantAppearance, text: &str) {
        self.set_appearance(appearance);
        self.content.set_text(&faded(text, 0.0));
        self.content.set_visible(true);
        self.set_visible(true);
    }

    pub(self) fn set_appearance(&mut self, appearance: &TrialParticipantAppearance) {
        self.face_alt.set_text(&appearance.emotion.to_string());
        if let Some(gesture) = appearance.gesture {
            self.gesture_alt.set_text(&gesture.to_string());
            self.gesture_alt.set_visible(true);
        } else {
            self.gesture_alt.set_visible(false);
        }
    }
}

#[ui_tree_api]
impl UITree for TrialSpeechGui {}

fn speech_to_bbcode(speech: &TrialSpeech) -> EcoString {
    let mut buffer = EcoString::with_capacity(speech.text.len());
    let mut keyword_index = 0;
    for item in &speech.items {
        match item {
            TrialSpeechItem::Text(t) => {
                buffer.push_str(t);
            }
            TrialSpeechItem::Keyword(k) => {
                buffer.push_str(&format!("[url={keyword_index}][b][color=dark_orchid][font_size=90]{k}[/font_size][/color][/b][/url]"));
                keyword_index += 1;
            }
            TrialSpeechItem::LineBreak => {
                buffer.push_str("[br]");
            }
        }
    }
    buffer
}

const FADE_SPEED: f32 = 20.0;
const FADE_LEN: i32 = 10;
const FADE_PRE_TIME: f32 = FADE_LEN as f32 / FADE_SPEED;

fn estimate_fade_time(text: &str) -> f32 {
    let char_count = text.chars().count();
    FADE_PRE_TIME + char_count as f32 / FADE_SPEED
}

fn faded(text: &str, time: f32) -> String {
    let (start, length) = if time < FADE_PRE_TIME {
        (0.0, time * FADE_SPEED)
    } else {
        ((time - FADE_PRE_TIME) * FADE_SPEED, FADE_LEN as f32)
    };
    format!("[fade start={start} length={length}]{text}[/fade]")
}

#[derive(UITree)]
#[ui_tree]
struct TrialThoughtGui {
    #[new(root.child_ui("%OptionButtons"), root.child_ui("%OptionButtonTemplate"))]
    options: PooledContainer<TrialThoughtOptionItem>,

    #[child("%BackButton")]
    back_button: ButtonA,

    on_select_option: signals2::Signal<(usize,)>,
}

impl TrialThoughtGui {
    fn set_options(&mut self, options: &[TrialResponseOption]) {
        self.options.clear();
        for option in options {
            let item = self.options.get();
            item.set_option(option);

            let on_select_option_handle = self.on_select_option.get_emit_handle();
            item.option_button.on_click.clear();
            let corpus_index = option.corpus_index;
            item.option_button.on_click.connect(move || {
                on_select_option_handle.emit(corpus_index);
            });
        }
    }
}

#[ui_tree_api]
impl UITree for TrialThoughtGui {}

#[derive(UITree)]
#[ui_tree]
pub struct TrialThoughtOptionItem {
    #[child("OptionButton")]
    option_button: TextButtonA,
    #[child("KindLabel")]
    kind_label: LabelA,
}

impl TrialThoughtOptionItem {
    pub fn set_option(&mut self, option: &TrialResponseOption) {
        self.option_button.set_text(&option.summary);
        self.kind_label.set_text(match option.kind {
            TrialResponseKind::Agreement => "赞同",
            TrialResponseKind::Objection => "反对",
            TrialResponseKind::Perjury => "伪证",
            TrialResponseKind::Question => "疑问",
        });
    }
}

#[ui_tree_api]
impl UITree for TrialThoughtOptionItem {}
