#![feature(box_as_ptr)]
#![feature(push_mut)]

mod guard;
mod node;
mod pool;
mod provider;
mod registry;
mod root;
mod vnode;

pub use guard::VNodeGuard;
pub use node::{UINode, UITree};
pub use pool::{PooledContainer, SharedPooledContainer};
pub use provider::AssetProvider;
pub use registry::{Gui, GuiCommands, GuiRegistry};
pub use root::UiRoot;
pub use vnode::VNode;
