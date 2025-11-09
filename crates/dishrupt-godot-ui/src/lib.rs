#![feature(box_as_ptr)]
#![feature(push_mut)]

mod guard;
mod manager;
mod node;
mod pool;
mod registry;
mod root;
mod vnode;

pub use guard::VNodeGuard;
pub use manager::GuiManager;
pub use node::{UINode, UITree};
pub use pool::{PooledContainer, SharedPooledContainer};
pub use registry::{Gui, GuiCommands, GuiRegistry};
pub use root::UIRoot;
pub use vnode::VNode;
