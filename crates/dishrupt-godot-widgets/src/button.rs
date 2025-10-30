use godot::classes::{BaseButton, Button, Texture2D, TextureButton};

use super::{ControlA, prelude::*};

#[ui_element(BaseButton, base = ControlA)]
pub struct ButtonA {
    pub on_click: Signal<()>,
    pub on_toggle: Signal<(bool,)>,
}

impl ButtonA {
    pub fn new(gd: Gd<BaseButton>) -> Self {
        let on_click: Signal<()> = Signal::new();

        let on_click_handle = on_click.get_emit_handle();
        gd.signals().pressed().connect(move || {
            on_click_handle.emit();
        });

        let on_toggle: Signal<(bool,)> = Signal::new();
        let on_toggle_handle = on_toggle.get_emit_handle();
        gd.signals().toggled().connect(move |toggled| {
            on_toggle_handle.emit(toggled);
        });

        Self {
            on_click,
            on_toggle,
            base: ControlA::new(gd.clone().upcast()),
            gd,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.gd.is_disabled()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.gd.set_disabled(!enabled);
    }

    pub fn is_pressed(&self) -> bool {
        self.gd.is_pressed()
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.gd.set_pressed_no_signal(pressed)
    }
}

impl Drop for ButtonA {
    fn drop(&mut self) {
        if self.gd.is_instance_valid() {
            self.gd.clear_connections("pressed");
            self.gd.clear_connections("toggled");
        }
    }
}

#[ui_element(Button, base = ControlA)]
pub struct TextButtonA {
    pub on_click: Signal<()>,
}

impl TextButtonA {
    pub fn new(gd: Gd<Button>) -> Self {
        let on_click: Signal<()> = Signal::new();

        let on_click_handle = on_click.get_emit_handle();
        gd.signals().pressed().connect(move || {
            on_click_handle.emit();
        });

        Self {
            on_click,
            base: ControlA::new(gd.clone().upcast()),
            gd,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.gd.is_disabled()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.gd.set_disabled(!enabled);
    }

    pub fn set_text(&mut self, text: &str) {
        self.gd.set_text(text);
    }

    pub fn get_text(&self) -> String {
        self.gd.get_text().into()
    }

    pub fn set_icon(&mut self, icon: Gd<Texture2D>) {
        self.gd.set_button_icon(&icon);
    }

    pub fn set_icon_opt(&mut self, icon: Option<Gd<Texture2D>>) {
        if let Some(icon) = icon {
            self.gd.set_button_icon(&icon);
        } else {
            self.gd.set_button_icon(Gd::null_arg());
        }
    }
}

impl Drop for TextButtonA {
    fn drop(&mut self) {
        if self.gd.is_instance_valid() {
            self.gd.clear_connections("pressed");
        }
    }
}

#[ui_element(TextureButton, base = ControlA)]
pub struct TextureButtonA {
    pub on_click: Signal<()>,
}

impl TextureButtonA {
    pub fn new(gd: Gd<TextureButton>) -> Self {
        let on_click: Signal<()> = Signal::new();

        let on_click_handle = on_click.get_emit_handle();
        gd.signals().pressed().connect(move || {
            on_click_handle.emit();
        });

        Self {
            on_click,
            base: ControlA::new(gd.clone().upcast()),
            gd,
        }
    }
}

impl Drop for TextureButtonA {
    fn drop(&mut self) {
        if self.gd.is_instance_valid() {
            self.gd.clear_connections("pressed");
        }
    }
}
