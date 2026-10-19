pub mod condition;
pub mod delay;
pub mod fs_read;
pub mod fs_write;
pub mod http_get;
pub mod json_parse;
pub mod log_output;
pub mod text_clean;

use crate::registry::NodeRegistry;
use std::sync::Arc;

pub fn register_all(registry: &mut NodeRegistry) {
    registry.register(Arc::new(http_get::HttpGetNode));
    registry.register(Arc::new(log_output::LogOutputNode));
    registry.register(Arc::new(fs_read::FsReadNode));
    registry.register(Arc::new(fs_write::FsWriteNode));
    registry.register(Arc::new(json_parse::JsonParseNode));
    registry.register(Arc::new(text_clean::TextCleanNode));
    registry.register(Arc::new(condition::ConditionNode));
    registry.register(Arc::new(delay::DelayNode));
}
