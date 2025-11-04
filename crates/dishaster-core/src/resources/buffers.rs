use dishaster_interface::*;

use crate::prelude::*;

pub type EventQueue = MessageQueue<SimEvent>;
pub type ResponseQueue = MessageQueue<SimResponse>;
