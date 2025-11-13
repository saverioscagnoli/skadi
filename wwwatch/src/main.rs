mod info;
mod payloads;

use crate::payloads::WorkspacePayload;
use clap::Parser;
use serde::Serialize;
use std::collections::HashSet;
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
    #[arg(
        short,
        long,
        help = "Queries cpu info every <interval> milliseconds",
        default_value_t = false
    )]
    cpu: bool,
    #[arg(
        short,
        long,
        help = "Queries memory info every <interval> milliseconds",
        default_value_t = false
    )]
    mem: bool,

    #[arg(
        short,
        long,
        help = "Queries disk info every <interval> milliseconds",
        default_value_t = false
    )]
    disk: bool,
    #[arg(
        short,
        long,
        help = "Queries network info every <interval> milliseconds",
        default_value_t = false
    )]
    network: bool,
    #[arg(
        short,
        long,
        help = "Interval in milliseconds, used to query information about the system",
        default_value_t = 1000
    )]
    interval: u64,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
enum Op {
    Workspaces,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut event_list = Vec::new();
    let mut connection = Connection::new()?;
    let mut current_workspace = String::new();
    let mut workspace_cache = HashSet::new();

    info::query_info(args.cpu, args.mem, args.disk, args.network, args.interval);

    if args.workspaces {
        event_list.push(EventType::Workspace);
        // Initial check to workspaces

        let workspaces = connection.get_workspaces()?;

        for ws in workspaces {
            if ws.focused {
                current_workspace = ws.name.clone();
            }
            let num = ws.num;
            workspace_cache.insert((num, ws.name));
        }

        // Sort workspaces by num
        let mut sorted_workspaces: Vec<_> = workspace_cache.iter().cloned().collect();
        sorted_workspaces.sort_by_key(|(num, _)| *num);

        let payload = WorkspacePayload {
            op: Op::Workspaces,
            current: &current_workspace,
            total: &sorted_workspaces,
        };

        println!("{}", serde_json::to_string(&payload).unwrap());
    }

    // One connection for events
    let events = connection.subscribe(&event_list)?;

    // Listen for events
    for event in events.map_while(Result::ok) {
        match event {
            Event::Workspace(ws_event) => match ws_event.change {
                WorkspaceChange::Focus => {
                    if let Some(current_ws) = ws_event.current
                        && let Some(name) = current_ws.name
                    {
                        let num = current_ws.num.unwrap_or(-1);
                        workspace_cache.insert((num, name.clone()));
                        current_workspace = name;
                    }

                    // Sort workspaces by num
                    let mut sorted_workspaces: Vec<_> = workspace_cache.iter().cloned().collect();
                    sorted_workspaces.sort_by_key(|(num, _)| *num);

                    let payload = WorkspacePayload {
                        op: Op::Workspaces,
                        current: &current_workspace,
                        total: &sorted_workspaces,
                    };

                    println!("{}", serde_json::to_string(&payload)?);
                }

                WorkspaceChange::Empty => {
                    if let Some(current_ws) = ws_event.current
                        && let Some(name) = current_ws.name
                    {
                        let num = current_ws.num.unwrap_or(-1);
                        workspace_cache.remove(&(num, name));
                    }

                    // Sort workspaces by num
                    let mut sorted_workspaces: Vec<_> = workspace_cache.iter().cloned().collect();
                    sorted_workspaces.sort_by_key(|(num, _)| *num);

                    let payload = WorkspacePayload {
                        op: Op::Workspaces,
                        current: &current_workspace,
                        total: &sorted_workspaces,
                    };

                    println!("{}", serde_json::to_string(&payload).unwrap());
                }

                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}
