use dishaster_models::{TrialIntro, TrialParticipantAppearance, TrialSpeech};
use godot::{builtin::GString, classes::RegEx, obj::NewGd};

use crate::{prelude::*, req::GameRequest};

#[derive(UITree)]
#[ui_tree]
pub struct TrialGui {
    #[subtree("%Left")]
    left: TrialSpeechGui,
    #[subtree("%Right")]
    right: TrialSpeechGui,

    #[child("%TimeProgress")]
    time_progress: ProgressBarA,

    state: Option<State>,
}

impl TrialGui {
    pub fn intro(&mut self, e: TrialIntro) {
        self.state = Some(State {
            time: 0.0,
            phase: Phase::Intro,
        });

        self.left.set_appearance(&e.left);
        self.right.set_appearance(&e.right);
        self.left.content.set_visible(false);
        self.right.content.set_visible(false);
        self.time_progress.set_visible(false);

        // todo: use animation
        self.left.set_visible(true);
        self.right.set_visible(false);
    }

    pub fn left_speak(&mut self, speech: TrialSpeech) {
        if let Some(s) = self.state.as_mut() {
            s.phase = Phase::LeftSpeaking;
            s.time = 0.0;
        }
        self.left.set_speech(speech);
        self.left.set_visible(true);
        self.right.set_visible(false);
    }

    pub fn right_speak(&mut self, speech: TrialSpeech) {
        if let Some(s) = self.state.as_mut() {
            s.phase = Phase::RightSpeaking;
            s.time = 0.0;
        }
        self.right.set_speech(speech);
        self.right.set_visible(true);
        self.left.set_visible(false);
    }
}

#[ui_tree_api]
impl UITree for TrialGui {}

impl Gui for TrialGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.left.content.on_meta_click.connect(move |meta| {
            godot::global::godot_print!("Left content meta clicked: {:?}", meta);
            let keyword = meta.to::<GString>();
            cmd.push_req(GameRequest::TrialChooseKeyword(keyword.to_string().into()));
        });
    }

    fn process(&mut self, cmd: GuiCommands, delta: f64) {
        let Some(state) = &mut self.state else {
            return;
        };

        state.time += delta as f32;

        match state.phase {
            Phase::Intro => {
                if state.time < 1.0 {
                    self.left.set_visible(true);
                    self.right.set_visible(false);
                } else if state.time < 2.0 {
                    // switch to right
                    self.left.set_visible(false);
                    self.right.set_visible(true);
                } else {
                    state.phase = Phase::Idle;
                    self.left.set_visible(false);
                    self.right.set_visible(false);
                    // wait for providing left content
                    cmd.push_req(GameRequest::TrialIntroDone);
                }
            }
            Phase::RightSpeaking => {
                if state.time > 3.0 {
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
    pub fn set_speech(&mut self, speech: TrialSpeech) {
        self.set_appearance(&speech.appearance);
        self.content.set_text(&speech_to_bbcode(&speech.text));
        self.content.set_visible(true);
        self.set_visible(true);
    }

    pub fn set_appearance(&mut self, appearance: &TrialParticipantAppearance) {
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

#[derive(Debug, Default)]
struct State {
    pub time: f32,
    pub phase: Phase,
}

#[derive(Debug, Default)]
enum Phase {
    #[default]
    Idle,
    Intro,
    LeftSpeaking,
    RightSpeaking,
}

fn speech_to_bbcode(text: &str) -> String {
    let mut pattern = RegEx::new_gd();
    pattern.compile(r"\[(.+?)\]");
    let res = pattern
        .sub_ex(
            text,
            "[url=\"$1\"][b][color=dark_orchid]$1[/color][/b][/url]",
        )
        .all(true)
        .done();
    let res = res.replacen("\\", "[br]");
    res.to_string()
}
