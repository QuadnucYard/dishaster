use super::vnode::VNode;
use crate::{UINode, UITree};

pub struct PooledContainer<T>
where
    T: UITree,
{
    container: UINode,
    template: UINode,

    /// Storage: inactive -> active. It does not necessarily reflect the child order.
    items: Vec<T>,
    available_num: usize,
}

impl<T> PooledContainer<T>
where
    T: UITree,
{
    pub fn new(container: UINode, mut template: UINode) -> Self {
        template.set_active(false);
        Self {
            container,
            template,
            items: Default::default(),
            available_num: 0,
        }
    }

    pub fn clear(&mut self) {
        for item in &mut self.items[self.available_num..] {
            item.set_active(false);
        }
        self.available_num = self.items.len();
    }

    pub fn is_empty(&self) -> bool {
        self.items.len() == self.available_num
    }
}

impl<T> PooledContainer<T>
where
    T: UITree + From<UINode>,
{
    /// Get a new item from the pool. It ensures the node is added to the container.
    pub fn get(&mut self) -> &mut T {
        let item = if self.available_num == 0 {
            // create new one
            self.items.push(self.template.dup().into());
            let item = self.items.last_mut().unwrap();
            item.set_parent(&mut self.container);
            item
        } else {
            // reuse existing one
            self.available_num -= 1;
            let item = &mut self.items[self.available_num];
            self.container.0.move_child(&item.root().0, -1);
            item
        };
        item.set_active(true);
        item
    }

    /// A variant version, but the container of self is just for storage.
    pub fn get_to(&mut self, container: &mut UINode) -> &mut T {
        let item = if self.available_num == 0 {
            // create new one
            self.items.push(self.template.dup().into());
            let item = self.items.last_mut().unwrap();
            item.set_parent(container);
            item
        } else {
            // reuse existing one
            self.available_num -= 1;
            let item = &mut self.items[self.available_num];
            item.detach();
            item.set_parent(container);
            item
        };
        item.set_active(true);
        item
    }
}

/// A variant version, but the container of self is just for storage.
pub struct SharedPooledContainer<T>
where
    T: VNode,
{
    template: UINode,

    /// Storage: inactive -> active. It does not necessarily reflect the child order.
    items: Vec<T>,
    available_num: usize,
}

impl<T> SharedPooledContainer<T>
where
    T: UITree,
{
    pub fn new(mut template: UINode) -> Self {
        template.set_active(false);
        Self {
            template,
            items: Default::default(),
            available_num: 0,
        }
    }

    pub fn clear(&mut self) {
        for item in &mut self.items[self.available_num..] {
            item.set_active(false);
        }
        self.available_num = self.items.len();
    }

    pub fn is_empty(&self) -> bool {
        self.items.len() == self.available_num
    }
}

impl<T> SharedPooledContainer<T>
where
    T: VNode + From<UINode>,
{
    pub fn get_to(&mut self, container: &mut UINode) -> &mut T {
        let item = if self.available_num == 0 {
            // create new one
            self.items.push(self.template.dup().into());
            let item = self.items.last_mut().unwrap();
            item.set_parent(container);
            item
        } else {
            // reuse existing one
            self.available_num -= 1;
            let item = &mut self.items[self.available_num];
            item.detach();
            item.set_parent(container);
            item
        };
        item.set_active(true);
        item
    }
}
