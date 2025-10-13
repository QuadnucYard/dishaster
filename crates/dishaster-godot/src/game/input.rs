use dishaster_core::commands::SimCommand;
use dishrupt_core::prelude::*;
use dishrupt_godot::input::{event::MouseButtonEvent, listener::GodotInputEvent};
use dishrupt_godot_scene::SceneContext;
use godot::{
    classes::{Area2D, Node, Node2D, PhysicsPointQueryParameters2D},
    global::MouseButton,
    prelude::*,
};

use crate::game::{DayPhase, Game};

impl Game {
    pub fn process_input(&mut self, ctx: &mut SceneContext, event: GodotInputEvent) {
        #[allow(clippy::single_match)]
        match event {
            GodotInputEvent::Button(e) => {
                if e.button == MouseButton::LEFT && !e.pressed {
                    if self.phase == DayPhase::Preparation
                        && let Some(pickable) = self.pick(e.position)
                    {
                        #[allow(mutable_transmutes)]
                        let pickable: &mut dyn Pickable = unsafe { std::mem::transmute(pickable) };
                        pickable.on_click(ctx, &e);
                        return;
                    }

                    let canvas_pos = screen_to_canvas(&self.root, e.position);
                    let sim_pos = self.to_map_pos(canvas_pos);
                    godot_print!("click map： {canvas_pos} {sim_pos}");
                    self.sim_runner
                        .send_command(SimCommand::QueryDistance(sim_pos));
                }
            }
            _ => {}
        }
    }

    fn pick(&self, position: Vector2) -> Option<&dyn Pickable> {
        get_pickable_under_mouse(&self.root.clone().cast(), position, |area| {
            self.get_pickable_of(&area)
        })
    }

    fn get_pickable_of(&self, gd: &Gd<Area2D>) -> Option<&dyn Pickable> {
        let instance_id = gd.instance_id_unchecked();
        self.dc
            .dishes
            .values()
            .find(|t| t.collider_instance_id() == instance_id)
            .map(|t| -> &dyn Pickable { t })
    }

    fn to_map_pos(&self, pos: Vector2) -> Vec2 {
        self.stage
            .display_context()
            .to_simulation_space(pos - self.stage_origin)
    }
}

fn get_pickable_under_mouse<'a>(
    gd: &Gd<Node2D>,
    view_pos: Vector2,
    mapper: impl FnMut(Gd<Area2D>) -> Option<&'a dyn Pickable>,
) -> Option<&'a dyn Pickable> {
    let canvas_pos = screen_to_canvas(&gd.clone().upcast(), view_pos);

    let mut query = PhysicsPointQueryParameters2D::new_gd();
    query.set_position(canvas_pos);
    query.set_collide_with_areas(true);

    let mut space_state = gd.get_world_2d().unwrap().get_direct_space_state().unwrap();

    let result = space_state.intersect_point_ex(&query).done();
    godot_print!("pick result: {:?}", result);
    result
        .iter_shared()
        .map(|x| x.at("collider").to())
        .filter_map(mapper)
        .max_by_key(|x| x.z_index())
}

fn screen_to_canvas(root: &Gd<Node>, screen_pos: Vector2) -> Vector2 {
    root.get_viewport()
        .unwrap()
        .get_canvas_transform()
        .affine_inverse()
        * screen_pos
}

#[allow(unused_variables)]
pub trait Pickable {
    fn collider_instance_id(&self) -> InstanceId;

    /// The z-index for sorting order detection in events
    fn z_index(&self) -> i32 {
        0
    }

    fn on_click(&mut self, ctx: &mut SceneContext, event: &MouseButtonEvent) {}
}
