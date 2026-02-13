use godot::{
    classes::{CanvasItem, Node},
    prelude::*,
};

use crate::VNode;

pub trait UITree: VNode {
    fn root(&self) -> &UINode;

    fn root_mut(&mut self) -> &mut UINode;

    fn ready(&mut self) {}

    /// Called when the tree is shown
    fn on_enable(&mut self) {}

    /// Called when the tree is hidden
    fn on_disable(&mut self) {}

    fn is_visible(&self) -> bool {
        self.root().is_visible()
    }

    fn set_visible(&mut self, visible: bool) {
        self.root_mut().set_visible(visible)
    }

    fn show(&mut self) {
        if !self.is_active() {
            self.set_active(true);
            self.on_enable();
        }
    }

    fn hide(&mut self) {
        if self.is_active() {
            self.on_disable();
            self.set_active(false);
        }
    }
}

impl<T: UITree> VNode for T {
    fn gd(&self) -> &Gd<CanvasItem> {
        &self.root().0
    }

    fn gd_mut(&mut self) -> &mut Gd<CanvasItem> {
        &mut self.root_mut().0
    }

    fn set_active(&mut self, active: bool) {
        self.root_mut().set_active(active)
    }

    fn is_active(&self) -> bool {
        self.root().is_active()
    }

    fn set_parent(&self, parent: &mut dyn VNode) {
        self.root().set_parent(parent)
    }

    fn add_child(&mut self, child: &dyn VNode) {
        self.root_mut().add_child(child);
    }

    fn detach(&self) {
        let root = &self.root().0;
        if let Some(mut parent) = root.get_parent() {
            parent.remove_child(root);
        }
    }

    fn free(&mut self) {
        let root = &mut self.root_mut().0;
        if root.is_instance_valid() {
            root.queue_free()
        }
    }
}

#[derive(Clone)]
pub struct UINode(pub Gd<CanvasItem>);

impl UINode {
    pub fn node2d(&self) -> Gd<Node2D> {
        self.0.clone().cast()
    }

    pub fn dup(&self) -> Self {
        Self(Gd::duplicate_node(&self.0))
    }

    pub fn child<T>(&self, path: &str) -> Gd<T>
    where
        T: Inherits<Node>,
    {
        self.0.try_get_node_as(path).unwrap_or_else(|| {
            if let Some(node) = self.0.get_node_or_null(path) {
                panic!(
                    "There is no node of type {ty} at path `{path}`, but node of type {ty2}",
                    ty = T::class_id(),
                    ty2 = node.get_class()
                )
            } else {
                panic!(
                    "There is no node of type {ty} at path `{path}`",
                    ty = T::class_id()
                )
            }
        })
    }

    pub fn child_ui(&self, path: &str) -> UINode {
        UINode(self.0.get_node_as(path))
    }
    /*
       pub fn child_of_type<T>(&self) -> Option<Gd<T>>
       where
           T: GodotClass + Inherits<Node>,
       {
           self.0.get_node_of_type()
       }

       pub fn descendant_of_type<T>(&self) -> Option<Gd<T>>
       where
           T: GodotClass + Inherits<Node>,
       {
           self.0.get_descendant_of_type()
       }

       pub fn child_ui_of_type<T>(&self) -> UINode
       where
           T: GodotClass + Inherits<Node> + Inherits<CanvasItem>,
       {
           UINode(
               self.0
                   .get_node_of_type::<T>()
                   .expect("get child of type")
                   .upcast(),
           )
       }
    */
    pub fn is_visible(&self) -> bool {
        self.0.is_visible()
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.0.set_visible(visible);
    }
}

impl VNode for UINode {
    fn gd(&self) -> &Gd<CanvasItem> {
        &self.0
    }

    fn gd_mut(&mut self) -> &mut Gd<CanvasItem> {
        &mut self.0
    }

    fn set_active(&mut self, active: bool) {
        self.set_visible(active);
        self.0.set_process(active);
        self.0.set_process_input(active);
    }

    fn is_active(&self) -> bool {
        self.0.is_processing()
    }

    fn set_parent(&self, parent: &mut dyn VNode) {
        parent.gd_mut().add_child(&self.0);
    }

    fn add_child(&mut self, child: &dyn VNode) {
        self.0.add_child(child.gd());
    }

    fn detach(&self) {
        let root = &self.0;
        if let Some(mut parent) = root.get_parent() {
            parent.remove_child(root);
        }
    }

    fn free(&mut self) {
        if self.0.is_instance_valid() {
            self.0.queue_free()
        }
    }
}

unsafe impl Send for UINode {}
unsafe impl Sync for UINode {}
