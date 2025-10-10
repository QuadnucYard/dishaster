use std::iter::FromIterator;

use dishaster_core::snapshots::CrowdFieldDebugSnapshot;
use dishrupt_core::prelude::*;
use dishrupt_godot::{bind::IntoGodot, display::DisplayContext2D};
use godot::{
    classes::{Image, ImageTexture, Node2D, Sprite2D, canvas_item::TextureFilter, image::Format},
    prelude::*,
};

const Z_INDEX: i32 = 8;

/// Renders the crowd navigation cost field as a heatmap sprite for debugging.
pub struct CrowdDebugOverlay {
    sprite: Gd<Sprite2D>,
    texture: Gd<ImageTexture>,
    image: Option<Gd<Image>>,
    pixel_buffer: Vec<u8>,
    image_size: IVec2,
}

impl CrowdDebugOverlay {
    pub fn new(mut root: Gd<Node2D>) -> Self {
        let mut sprite = Sprite2D::new_alloc();
        sprite.set_name("CrowdDebugHeatmap");
        sprite.set_centered(false);
        sprite.set_z_index(Z_INDEX);
        sprite.set_visible(false);
        sprite.set_texture_filter(TextureFilter::NEAREST);
        root.add_child(&sprite);

        let texture = ImageTexture::new_gd();
        sprite.set_texture(&texture);

        Self {
            sprite,
            texture,
            image: None,
            pixel_buffer: Vec::new(),
            image_size: Default::default(),
        }
    }

    pub fn present(&mut self, snapshot: Option<&CrowdFieldDebugSnapshot>, ctx: &DisplayContext2D) {
        let Some(snapshot) = snapshot else {
            self.sprite.set_visible(false);
            return;
        };

        if snapshot.costs.is_empty() {
            self.sprite.set_visible(false);
            return;
        }

        let dimensions = snapshot.dimensions;
        if dimensions.x == 0 || dimensions.y == 0 {
            self.sprite.set_visible(false);
            return;
        }

        let width = dimensions.x;
        let height = dimensions.y;

        let image_dims = IVec2::new(width as i32, height as i32);
        if self.image_size != image_dims {
            self.image = Some(
                Image::create(image_dims.x, image_dims.y, false, Format::RGBA8)
                    .expect("failed to allocate crowd heatmap image"),
            );
            self.image_size = image_dims;
        }

        let image = self
            .image
            .as_mut()
            .expect("heatmap image must be allocated at this point");

        update_buffer(&mut self.pixel_buffer, snapshot);
        let byte_array = PackedByteArray::from_iter(self.pixel_buffer.iter().copied());
        image.set_data(
            width as i32,
            height as i32,
            false,
            Format::RGBA8,
            &byte_array,
        );
        self.texture.set_image(&*image); // IMPORTANT: Update the texture with the new image data

        let scale = ctx.view_scale.xy() * snapshot.cell_size;
        self.sprite.set_scale(scale.into_godot());

        let min_coord = snapshot.origin;
        let top_left_world = min_coord.as_vec2() * snapshot.cell_size;
        let top_left_display = ctx.to_display_space(top_left_world.extend(0.0));
        self.sprite.set_position(top_left_display);
        self.sprite.set_visible(true);
    }
}

fn update_buffer(pixel_buffer: &mut Vec<u8>, snapshot: &CrowdFieldDebugSnapshot) {
    let required_len = snapshot.dimensions.x * snapshot.dimensions.y * 4;
    if pixel_buffer.len() != required_len {
        pixel_buffer.resize(required_len, 0);
    }
    pixel_buffer.fill(0);

    let clamp = |value: f32| -> u8 { (value.clamp(0.0, 1.0) * 255.0).round() as u8 };

    // Define a simple RGBA color bar: blue for low intensity, red for high intensity
    const BASE_ALPHA: f32 = 0.05;
    const LOW_COLOR: Color = Color::from_rgba(0.0, 0.0, 1.0, BASE_ALPHA);
    const HIGH_COLOR: Color = Color::from_rgba(1.0, 0.0, 0.0, BASE_ALPHA + 0.3);

    for (i, &cost) in snapshot.costs.iter().enumerate() {
        if cost <= 0.0 {
            continue;
        }
        let intensity = (cost.sqrt() / 5.0).min(1.0);

        let color = LOW_COLOR.lerp(HIGH_COLOR, intensity as f64);

        let idx = i * 4;
        pixel_buffer[idx] = clamp(color.r);
        pixel_buffer[idx + 1] = clamp(color.g);
        pixel_buffer[idx + 2] = clamp(color.b);
        pixel_buffer[idx + 3] = clamp(color.a);
    }
}
