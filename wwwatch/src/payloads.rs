use crate::Op;
use indexmap::IndexSet;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WorkspacePayload<'a> {
    pub op: Op,
    pub current: &'a String,
    pub total: &'a IndexSet<String>,
}
