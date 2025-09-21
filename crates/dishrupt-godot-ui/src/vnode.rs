use godot::{classes::CanvasItem, obj::Gd};

pub trait VNode {
    fn gd(&self) -> &Gd<CanvasItem>;

    fn gd_mut(&mut self) -> &mut Gd<CanvasItem>;

    fn set_active(&mut self, active: bool);

    fn is_active(&self) -> bool;

    fn set_parent(&self, parent: &mut dyn VNode);

    fn add_child(&mut self, child: &dyn VNode);

    fn detach(&self);

    fn free(&mut self);
}
