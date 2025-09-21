#![feature(box_as_ptr)]

pub mod elem;
mod guard;
mod manager;
mod node;
mod pool;
mod registry;
mod root;
mod vnode;

pub use dishrupt_godot_ui_macros::*;
pub use guard::VNodeGuard;
pub use manager::GuiManager;
pub use node::{UINode, UITree};
pub use pool::{PooledContainer, SharedPooledContainer};
pub use registry::{Gui, GuiCommands, GuiRegistry, GuiRequest};
pub use root::UIRoot;
pub use vnode::VNode;
