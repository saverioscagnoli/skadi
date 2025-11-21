use crate::payload::OpCode;
use crate::payload::Payload;
use crate::payload::SerializePrint;
use crate::payload::WorkspacesPayload;
use std::collections::HashSet;
use swayipc::Event;
use swayipc::WorkspaceChange;
use swayipc::{Connection, EventType};

pub async fn listen(workspaces: bool) {
    let mut conn = Connection::new().expect("Failed to enstablish sway ipc connection");
    let mut event_list = Vec::new();
    let mut focused_workspace_index = 0;
    let mut workspace_cache = HashSet::new();

    if workspaces {
        event_list.push(EventType::Workspace);

        // First check for workspaces
        let workspaces = match conn.get_workspaces() {
            Ok(workspaces) => workspaces,
            Err(e) => {
                eprintln!("Failed te get workspaces: {}", e);
                return;
            }
        };

        for ws in &workspaces {
            if ws.focused {
                focused_workspace_index = ws.num;
            }

            workspace_cache.insert((ws.num, ws.name.clone()));
        }

        let mut workspaces = workspace_cache.iter().cloned().collect::<Vec<_>>();
        workspaces.sort_by_key(|(num, _)| *num);

        let payload = Payload {
            op: OpCode::Workspaces,
            data: WorkspacesPayload {
                focused: focused_workspace_index,
                workspaces,
            },
        };

        payload.print();
    }

    if event_list.is_empty() {
        return;
    }

    let events = match conn.subscribe(&event_list) {
        Ok(events) => events,
        Err(e) => {
            eprintln!("Failed to subscribe to events: {}", e);
            return;
        }
    };

    for event in events.map_while(Result::ok) {
        match event {
            Event::Workspace(ws_event) => match ws_event.change {
                WorkspaceChange::Focus => {
                    if let Some(focused_ws) = ws_event.current
                        && let Some(name) = focused_ws.name.clone()
                    {
                        let num = focused_ws.num.unwrap_or(-1);

                        workspace_cache.insert((num, name));
                        focused_workspace_index = num;
                    }

                    let mut workspaces = workspace_cache.iter().cloned().collect::<Vec<_>>();
                    workspaces.sort_by_key(|(num, _)| *num);

                    let payload = Payload {
                        op: OpCode::Workspaces,
                        data: WorkspacesPayload {
                            focused: focused_workspace_index,
                            workspaces,
                        },
                    };

                    payload.print();
                }

                WorkspaceChange::Empty => {
                    if let Some(focused_ws) = ws_event.current
                        && let Some(name) = focused_ws.name.clone()
                    {
                        let num = focused_ws.num.unwrap_or(-1);

                        workspace_cache.insert((num, name));
                        focused_workspace_index = num;
                    }

                    let mut workspaces = workspace_cache.iter().cloned().collect::<Vec<_>>();
                    workspaces.sort_by_key(|(num, _)| *num);

                    let payload = Payload {
                        op: OpCode::Workspaces,
                        data: WorkspacesPayload {
                            focused: focused_workspace_index,
                            workspaces,
                        },
                    };

                    payload.print();
                }

                _ => {}
            },

            _ => {}
        }
    }
}
