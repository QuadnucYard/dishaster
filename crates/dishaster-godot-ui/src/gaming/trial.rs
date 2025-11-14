use dishaster_views::{
    TrialIntro, TrialParticipantAppearance, TrialResponseKind, TrialResponseOption, TrialSpeech,
    TrialSpeechItem, TrialStatement,
};
use dishrupt_core::prelude::*;
use godot::{classes::AnimationPlayer, prelude::*};

use crate::prelude::*;

const INTRO_TIME: f32 = 2.0;
const LEFT_SPEECH_INTERVAL: f32 = 2.0;
const RIGHT_SPEECH_INTERVAL: f32 = 2.0;
const COUNTDOWN_TIME: f32 = 3.0;

#[derive(Debug, Default)]
struct State {
    pub time: f32,
    pub phase: Phase,
    pub inner_speech_text: EcoString,
    pub fade_time: f32,
    /// Remaining speeches in the sequence (for multi-turn diner dialogue)
    pub speech_sequence: Vec<TrialSpeech>,
}

#[derive(Debug, Default)]
enum Phase {
    #[default]
    Idle,
    Intro,
    LeftSpeaking,
    RightSpeaking,
}

enum Side {
    Left,
    Right,
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

    state: State,
}

impl TrialGui {
    pub fn intro(&mut self, e: TrialIntro) {
        self.state = State {
            phase: Phase::Intro,
            ..Default::default()
        };

        self.left.set_appearance(&e.left);
        self.right.set_appearance(&e.right);
        self.left.content.set_visible(false);
        self.right.content.set_visible(false);
        self.time_progress.set_visible(false);
        self.thought.set_visible(false);

        // Play intro animation
        self.anim_player.play_ex().name("intro").done();
        self.anim_player.seek(0.0);
    }

    pub fn left_speak(&mut self, statement: TrialStatement) {
        // Start with the first speech in the sequence
        let speech_sequence = statement.speech_sequence;
        if speech_sequence.is_empty() {
            panic!("Speech sequence should not be empty");
        }

        self.right.set_visible(false);
        self.left.set_visible(true);

        self.state = State {
            phase: Phase::LeftSpeaking,
            speech_sequence,
            ..Default::default()
        };
        self.display_next_speech(Side::Left);
    }

    pub fn right_speak(&mut self, statement: TrialStatement) {
        // Start with the first speech in the sequence
        let speech_sequence = statement.speech_sequence;
        if speech_sequence.is_empty() {
            panic!("Speech sequence should not be empty");
        }

        self.left.set_visible(false);
        self.right.set_visible(true);

        self.state = State {
            phase: Phase::RightSpeaking,
            speech_sequence,
            ..Default::default()
        };
        self.display_next_speech(Side::Right);
    }

    fn display_next_speech(&mut self, side: Side) {
        let state: &mut State = &mut self.state;

        let next_speech = state.speech_sequence.remove(0);
        let next_text = speech_to_bbcode(&next_speech);

        let speech_gui = match side {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        };
        speech_gui.set_speech(&next_speech.appearance, &next_text);

        // Update state for next speech
        state.time = 0.0;
        state.fade_time = estimate_fade_time(&next_speech.text);
        state.inner_speech_text = next_text;

        // Hide thought panel if it was open
        self.thought.set_visible(false);
    }

    pub fn show_response_candidates(&mut self, options: Vec<TrialResponseOption>) {
        let state = &mut self.state;
        if let Phase::LeftSpeaking = state.phase {
            // Show the thought panel with the options
            self.thought.set_options(&options);
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
            let meta = meta.to::<GString>().to_string();
            let parts = meta.split_once('-').expect("invalid meta format");
            cmd.push_req(GameRequest::TrialCheckKeyword {
                speech_id: parts.0.parse().unwrap(),
                keyword_index: parts.1.parse().unwrap(),
            });
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
        let state = &mut self.state;

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

                // Check if current speech has finished displaying
                if state.time <= state.fade_time {
                    return;
                }

                // Check if there are more speeches in the sequence
                if state.speech_sequence.is_empty() {
                    // All speeches done, show countdown
                    self.time_progress.set_visible(true);
                    self.time_progress
                        .set_value((state.time - state.fade_time) / COUNTDOWN_TIME);
                    if state.time > state.fade_time + COUNTDOWN_TIME {
                        state.phase = Phase::Idle;
                        cmd.push_req(GameRequest::TrialTimeout);
                    }
                    return;
                }

                // Move to next speech after a short interval
                if state.time <= state.fade_time + LEFT_SPEECH_INTERVAL {
                    return;
                }

                // Display next speech
                self.display_next_speech(Side::Left);
            }
            Phase::RightSpeaking => {
                self.right
                    .content
                    .set_text(&faded(&state.inner_speech_text, state.time));

                // Check if current speech has finished displaying
                if state.time <= state.fade_time {
                    return;
                }

                // Check if there are more speeches in the sequence
                if state.speech_sequence.is_empty() {
                    // All speeches done
                    if state.time > state.fade_time + RIGHT_SPEECH_INTERVAL {
                        state.phase = Phase::Idle;
                        cmd.push_req(GameRequest::TrialResponseDone);
                    }
                    return;
                }

                // Move to next speech after a short interval
                if state.time <= state.fade_time + RIGHT_SPEECH_INTERVAL {
                    return;
                }

                // Display next speech
                self.display_next_speech(Side::Right);
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
                let meta = format!("{}-{}", speech.index, keyword_index);
                buffer.push_str(&format!("[url={meta}][b][color=dark_orchid][font_size=90]{k}[/font_size][/color][/b][/url]"));
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
        let (kind_msg, kind_color) = match option.kind {
            TrialResponseKind::Agreement => ("trial-agreement", Color::from_rgb(0.0, 0.23, 0.06)),
            TrialResponseKind::Objection => ("trial-objection", Color::from_rgb(0.23, 0.02, 0.02)),
            TrialResponseKind::Perjury => ("trial-perjury", Color::from_rgb(0.06, 0.08, 0.41)),
            TrialResponseKind::Question => ("trial-question", Color::from_rgb(0.39, 0.18, 0.01)),
        };
        self.kind_label.set_text(&tr!(kind_msg));
        self.kind_label
            .gd()
            .add_theme_color_override("font_outline_color", kind_color);
    }
}

#[ui_tree_api]
impl UITree for TrialThoughtOptionItem {}
