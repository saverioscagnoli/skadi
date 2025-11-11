use serde::Serialize;
use swayipc::Connection;

use crate::Op;

#[derive(Debug, Clone, Serialize)]
pub struct WorkspacePayload {
    pub op: Op,
    pub current: u8,
    pub total: Vec<u8>,
}

impl WorkspacePayload {
    pub fn new(conn: &mut Connection) -> Self {
        let mut s = Self {
            op: Op::Workspaces,
            current: 0,
            total: Vec::new(),
        };

        let workspaces = conn.get_workspaces().expect("Failed to get workspaces");

        for w in &workspaces {
            let name = w
                .name
                .parse::<u8>()
                .expect("Workspace name is not a number");

            if w.focused {
                s.current = name;
            }

            s.total.push(name);
        }

        s
    }
}
