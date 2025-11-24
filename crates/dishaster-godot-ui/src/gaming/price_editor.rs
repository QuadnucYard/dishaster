use std::cell::OnceCell;

use dishaster_views::{DishPriceView, PricingMethod};
use dishrupt_core::EntityId;

use crate::prelude::*;

struct PopupState {
    entity: EntityId,
    dish_name: String,
    original_mode: PricingMode,
    original_value: f32,
    current_mode: PricingMode,
    parsed_value: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PricingMode {
    PerPortion,
    ByWeight,
}

impl std::fmt::Display for PricingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PricingMode::PerPortion => f.write_str(&tr!("price-by-portion-display.label")),
            PricingMode::ByWeight => f.write_str(&tr!("price-by-weight-display.label")),
        }
    }
}

/// Interactive popup that lets the player tweak dish pricing before service.
#[derive(UITree)]
#[ui_tree]
pub struct DishPricePopup {
    commands: OnceCell<GuiCommands>,

    #[child("%DishNameLabel")]
    dish_name_label: LabelA,

    #[child("%OriginalValue")]
    original_value_label: LabelA,
    #[child("%OriginalMode")]
    original_mode_label: LabelA,
    #[child("%PriceInput")]
    price_input: LineEditA,
    #[child("%ModeButton")]
    mode_btn: TextButtonA,

    #[child("%ConfirmButton")]
    confirm_btn: ButtonA,
    #[child("%CancelButton")]
    cancel_btn: ButtonA,

    state: Option<PopupState>,

    pub enabled: bool,
}

#[ui_tree_api]
impl UITree for DishPricePopup {}

impl Gui for DishPricePopup {
    fn start(&mut self, commands: GuiCommands, _provider: AssetProvider) {
        let _ = self.commands.set(commands.clone());

        let cmd = commands.clone();
        self.price_input.on_text_change.connect(move |text| {
            cmd.push_cmd(move |gui| {
                gui.get_mut::<DishPricePopup>().handle_price_input(text);
            });
        });

        let cmd = commands.clone();
        self.mode_btn.on_click.connect(move || {
            cmd.push_cmd(|gui| {
                gui.get_mut::<DishPricePopup>().handle_mode_switch();
            });
        });

        let cmd = commands.clone();
        self.confirm_btn.on_click.connect(move || {
            cmd.push_cmd(|gui| {
                gui.get_mut::<DishPricePopup>().emit_apply();
            });
        });

        let cmd = commands.clone();
        self.cancel_btn.on_click.connect(move || {
            cmd.push_cmd(|gui| {
                gui.get_mut::<DishPricePopup>().hide();
            });
        });
    }
}

impl DishPricePopup {
    /// Populate the popup with the supplied pricing snapshot.
    pub fn set_view(&mut self, view: &DishPriceView) {
        let (original_mode, original_value) = match view.original_price {
            PricingMethod::PerPortion(v) => (PricingMode::PerPortion, v),
            PricingMethod::ByWeight(v) => (PricingMode::ByWeight, v),
        };
        let (current_mode, current_value) = match view.current_price {
            PricingMethod::PerPortion(v) => (PricingMode::PerPortion, v),
            PricingMethod::ByWeight(v) => (PricingMode::ByWeight, v),
        };

        let state = PopupState {
            entity: view.entity,
            dish_name: view.dish_name.clone(),
            original_mode,
            original_value,
            current_mode,
            parsed_value: Some(current_value),
        };

        self.dish_name_label.set_text(&state.dish_name);
        self.original_value_label
            .set_text(&format_price(state.original_mode, state.original_value));
        self.original_mode_label
            .set_text(&state.original_mode.to_string());

        self.price_input.set_text(&format!("{:.1}", current_value));
        self.mode_btn.set_text(&current_mode.to_string());
        self.price_input.grab_focus();

        self.confirm_btn.set_enabled(true);

        self.state = Some(state);
    }

    fn handle_price_input(&mut self, text: String) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        state.parsed_value = parse_price(&text);
        self.confirm_btn.set_enabled(state.parsed_value.is_some());
    }

    fn handle_mode_switch(&mut self) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        state.current_mode = match state.current_mode {
            PricingMode::PerPortion => PricingMode::ByWeight,
            PricingMode::ByWeight => PricingMode::PerPortion,
        };
        self.mode_btn.set_text(&state.current_mode.to_string());
    }

    fn emit_apply(&mut self) {
        let Some(state) = self.state.as_ref() else {
            return;
        };
        let Some(value) = state.parsed_value else {
            return;
        };
        let cmd = self.commands.get_mut().expect("commands not set");
        cmd.push_req(GameRequest::ApplyDishPrice {
            dish: state.entity,
            method: match state.current_mode {
                PricingMode::PerPortion => PricingMethod::PerPortion(value),
                PricingMode::ByWeight => PricingMethod::ByWeight(value),
            },
        });
        self.hide();
    }
}

fn parse_price(text: &str) -> Option<f32> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let value: f32 = trimmed.parse().ok()?;
    if value.is_finite() && value >= 0.0 {
        Some(value)
    } else {
        None
    }
}

fn format_price(mode: PricingMode, value: f32) -> String {
    match mode {
        PricingMode::PerPortion => format!("¥{value:.1}"),
        PricingMode::ByWeight => format!("¥{value:.1}/kg"),
    }
}
