use dishaster_interface::*;
use dishaster_ui_protocol::UiCommand;
use dishrupt_core::prelude::*;
use dishrupt_godot_input::event::{GodotInputEvent, MouseButtonEvent};
use godot::{
    classes::{Area2D, Node, Node2D, PhysicsPointQueryParameters2D},
    global::{Key, MouseButton},
    prelude::*,
};

use crate::Game;

impl Game {
    pub fn process_input(&mut self, event: GodotInputEvent) {
        match event {
            GodotInputEvent::Button(e) => {
                if e.button == MouseButton::LEFT && !e.pressed {
                    if self.try_process_picking(&e).is_some() {
                        return;
                    }

                    let canvas_pos = screen_to_canvas(&self.root, e.position);
                    let sim_pos = self.to_map_pos(canvas_pos);
                    godot_print!("click map： {canvas_pos} {sim_pos}");
                    self.sim_runner.send_query(SimQuery::Distance(sim_pos));
                }
            }
            GodotInputEvent::Key(key) => {
                if !key.pressed {
                    return;
                }

                if key.keycode == Key::F1 {
                    self.set_dev_enabled(!self.dev_features_enabled);
                }

                // Only process dev feature keys if enabled
                if !self.dev_features_enabled {
                    return;
                }

                self.process_key_dev(key.keycode);
            }
            _ => {}
        }
    }

    fn process_key_dev(&mut self, keycode: Key) {
        match keycode {
            Key::Q => {
                if let Some(&diner) = self.pres.agents.keys().next() {
                    godot_print!("DEV: Starting trial for diner {:?}", diner);
                    self.ui_commands
                        .push(UiCommand::TrialStart { diner, topic: None });
                }
            }
            Key::H => {
                let message = "This is a hint triggered by pressing the H key.".into();
                self.ui_commands.push(UiCommand::ShowHint { message });
            }
            Key::I => {
                use dishaster_views::{PsychImpactView, ReputationView, TrialImpactView};
                godot_print!("DEV: Triggering random trial impact");
                let test_impact = TrialImpactView {
                    psych_impact: Some(PsychImpactView {
                        mood_delta: 0.15,
                        trust_delta: 0.08,
                        patience_delta: 3.0,
                    }),
                    reputation_impact: Some(ReputationView {
                        reputation: 65.0,
                        reputation_delta: 2.5,
                        fsri: 5.0,
                        food_quality: 75.0,
                    }),
                };
                self.ui_commands
                    .push(UiCommand::TrialImpact(Box::new(test_impact)));
            }

            // Page Up: Increase reputation by 5
            Key::PAGEUP => {
                godot_print!("DEV: Increasing reputation by 5");
                self.send_sim_command(SimCommand::DevAdjustReputation(5.0));
            }
            // Page Down: Decrease reputation by 5
            Key::PAGEDOWN => {
                godot_print!("DEV: Decreasing reputation by 5");
                self.send_sim_command(SimCommand::DevAdjustReputation(-5.0));
            }

            Key::N => {
                godot_print!("Dev: Trigger inspector visit");
                self.send_sim_command(SimCommand::DevInspectorVisit(false));
            }
            Key::M => {
                godot_print!("Dev: Trigger inspector visit (fail)");
                self.send_sim_command(SimCommand::DevInspectorVisit(true));
            }

            Key::C => {
                godot_print!("Dev: Trigger crab");
                self.send_sim_command(SimCommand::DevCrab);
            }

            _ => {}
        }
    }

    fn try_process_picking(&mut self, e: &MouseButtonEvent) -> Option<()> {
        let pickable = self.pick(e.position)?;

        #[allow(mutable_transmutes)]
        let pickable: &mut dyn Pickable = unsafe { std::mem::transmute(pickable) };
        let ctx = &mut PickingContext {
            cmds: &mut self.ui_commands,
        };
        pickable.on_click(ctx, e);

        Some(())
    }

    fn pick(&self, position: Vector2) -> Option<&dyn Pickable> {
        get_pickable_under_mouse(&self.root.clone().cast(), position, |area| {
            self.get_pickable_of(&area)
        })
    }

    fn get_pickable_of(&self, gd: &Gd<Area2D>) -> Option<&dyn Pickable> {
        let instance_id = gd.instance_id_unchecked();
        self.get_pickables()
            .find(|t| t.collider_instance_id() == instance_id)
    }

    fn get_pickables(&self) -> impl Iterator<Item = &dyn Pickable> {
        (self.pres.dishes.values())
            .map(|t| -> &dyn Pickable { t })
            .chain(
                (self.pres.agents.values())
                    .filter_map(|t| t.feedback.as_ref())
                    .map(|t| -> &dyn Pickable { t }),
            )
            .chain((self.pres.dispensers.values()).map(|t| -> &dyn Pickable { t }))
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

    let mut space_state = gd.get_world_2d()?.get_direct_space_state()?;

    let result = space_state.intersect_point_ex(&query).done();
    godot_print!("pick result: {:?}", result);
    result
        .iter_shared()
        .map(|x| x.at("collider").to())
        .filter_map(mapper)
        .filter(|p| p.is_active())
        .max_by_key(|p| p.z_index())
}

fn screen_to_canvas(root: &Gd<Node>, screen_pos: Vector2) -> Vector2 {
    root.get_viewport()
        .expect("failed to get viewport")
        .get_canvas_transform()
        .affine_inverse()
        * screen_pos
}

pub struct PickingContext<'a> {
    pub cmds: &'a mut Vec<UiCommand>,
}

#[allow(unused_variables)]
pub trait Pickable {
    fn collider_instance_id(&self) -> InstanceId;

    /// Check if this pickable is currently active
    fn is_active(&self) -> bool {
        true
    }

    /// The z-index for sorting order detection in events
    fn z_index(&self) -> i32 {
        0
    }

    fn on_click(&mut self, ctx: &mut PickingContext, event: &MouseButtonEvent) {}
}
