use std::ops::{Deref, DerefMut};

use super::vnode::VNode;

pub struct VNodeGuard<T: VNode>(T);

impl<T: VNode> From<T> for VNodeGuard<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: VNode> Deref for VNodeGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: VNode> DerefMut for VNodeGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: VNode> Drop for VNodeGuard<T> {
    fn drop(&mut self) {
        self.free()
    }
}
