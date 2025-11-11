mod payloads;

use crate::payloads::WorkspacePayload;
use clap::Parser;
use serde::Serialize;
use std::error::Error;
use swayipc::{Connection, Event, EventType, WorkspaceChange};

#[derive(Debug, clap::Subcommand)]
enum Command {
    Workspace,
}

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(
        short,
        long,
        help = "Subscribe to workspace events",
        default_value_t = false
    )]
    workspaces: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
enum Op {
    Workspaces,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut events_list = Vec::new();
    let mut event_connection = Connection::new()?;

    if args.workspaces {
        events_list.push(EventType::Workspace);

        let payload = WorkspacePayload::new(&mut event_connection);
        println!("{:?}", serde_json::to_string(&payload)?);
    }

    // One connection for events
    let events = event_connection.subscribe(&events_list)?;

    // Separate connection for queries
    let mut query_connection = Connection::new()?;

    // Listen for events
    for event in events.map_while(Result::ok) {
        match event {
            Event::Workspace(ws_event) => {
                if matches!(ws_event.change, WorkspaceChange::Focus) {
                    let mut payload = WorkspacePayload {
                        op: Op::Workspaces,
                        current: 0,
                        total: Vec::new(),
                    };

                    if let Some(current) = ws_event.current {
                        payload.current =
                            current.name.and_then(|n| n.parse::<u8>().ok()).unwrap_or(0);
                    }

                    // Use the dedicated query connection
                    let workspaces = query_connection.get_workspaces()?;
                    payload.total = workspaces
                        .iter()
                        .filter_map(|ws| ws.name.parse::<u8>().ok())
                        .collect();

                    println!("{:?}", serde_json::to_string(&payload)?);
                }
            }
            _ => {}
        }
    }

    Ok(())
}
