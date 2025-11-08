use dishaster_views::CreditsView;
use dishrupt_l10n_godot::tr;

use crate::prelude::*;

#[derive(UITree)]
#[ui_tree]
pub struct CreditsGui {
    #[child("%CreditsContent")]
    content: RichLabelA,
    #[child("%BackButton")]
    back_btn: ButtonA,
}

#[ui_tree_api]
impl UITree for CreditsGui {}

impl Gui for CreditsGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.back_btn.on_click.connect(move || {
            cmd.push_req(AppRequest::BackToMenu);
        });
    }
}

impl CreditsGui {
    pub fn set_view(&mut self, view: CreditsView) {
        let content = Self::format_credits(&view);
        self.content.set_text(&content);
    }

    fn format_credits(view: &CreditsView) -> String {
        let mut result = String::new();

        // Overall center alignment
        result.push_str("[center]");
        // Header
        result.push_str(&format!(
            "[b][font_size=32]{}[/font_size][/b]\n",
            tr!("credits-header")
        ));

        // Each section
        for section in &view.sections {
            let section_title = tr!(&section.title);
            result.push_str(&format!("[b]{}[/b]", section_title));
            for entry in &section.entries {
                result.push_str("[br]");
                for (i, item) in entry.iter().enumerate() {
                    if i > 0 {
                        result.push_str("        ");
                    }
                    result.push_str(item);
                }
            }
            result.push('\n');
        }

        // Footer
        result.push_str(&format!("[b]{}[/b]", tr!("credits-footer")));
        result.push_str("[/center]");

        result
    }
}
