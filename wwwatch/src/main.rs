mod info;
mod notifications;
mod payload;
mod wm;

use clap::Parser;

#[derive(Debug, Clone, clap::Parser)]
struct InfoArgs {
    #[arg(long, default_value_t = false)]
    cpu: bool,

    #[arg(long, default_value_t = false)]
    mem: bool,

    #[arg(long, default_value_t = false)]
    disks: bool,

    #[arg(long, default_value_t = false)]
    net: bool,

    #[arg(long, default_value_t = false)]
    battery: bool,

    #[arg(long, default_value_t = 2000)]
    interval: u64,
}

#[derive(Debug, Clone, clap::Parser)]
struct Args {
    #[arg(long, default_value_t = false)]
    workspaces: bool,

    #[clap(flatten)]
    info: InfoArgs,

    #[arg(long, default_value_t = false)]
    notifications: bool,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let args = Args::parse();
    let mut handles = Vec::new();

    if args.workspaces {
        // TODO: Detect wm
        let handle = tokio::spawn(async move { wm::sway::listen(args.workspaces).await });
        handles.push(handle);
    }

    if args.info.cpu || args.info.mem || args.info.disks || args.info.net || args.info.battery {
        let handle = tokio::spawn(async move { info::poll(&args.info).await });
        handles.push(handle);
    }

    if args.notifications {
        let handle = tokio::spawn(async move {
            notifications::listen()
                .await
                .expect("Failed to start notification daemon");
        });
        handles.push(handle);
    }

    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("Error: {}", e);
        }
    }
}
