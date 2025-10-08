use anyhow::Result;
use std::thread;
use structopt::StructOpt;

mod config;
mod http_layer;
mod mock;
mod proxy;
mod recorder;
mod storage;
mod ui;

use config::ProxyConfig;

#[derive(Debug, StructOpt)]
#[structopt(name = "dev-proxy", about = "Development proxy with traffic recording")]
struct Opt {
    #[structopt(short, long, default_value = "8080")]
    port: u16,

    #[structopt(short, long, default_value = "3000")]
    ui_port: u16,

    #[structopt(long, default_value = "9090")]
    internal_port: u16,

    #[structopt(short, long, default_value = "http://localhost:8000")]
    upstream: String,

    #[structopt(short, long)]
    record: bool,
}

fn main() -> Result<()> {
    env_logger::init();

    let opt = Opt::from_args();

    let config = ProxyConfig {
        proxy_port: opt.internal_port,
        ui_port: opt.ui_port,
        upstream_url: opt.upstream,
        recording_enabled: opt.record,
    };

    println!("🚀 Starting Dev Proxy...");
    println!("   Proxy: http://0.0.0.0:{}", opt.port);
    println!("   UI:    http://0.0.0.0:{}", config.ui_port);
    println!("   Upstream: {}", config.upstream_url);
    println!(
        "   Recording: {}",
        if config.recording_enabled {
            "enabled"
        } else {
            "disabled"
        }
    );

    let storage = storage::Storage::new();
    let mock_manager = mock::MockManager::new(); // Add this

    // Start UI server in a separate thread with its own runtime
    let ui_storage = storage.clone();
    let ui_mock_manager = mock_manager.clone(); // Add this
    let ui_port = config.ui_port;
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            if let Err(e) = ui::start_ui_server(ui_port, ui_storage, ui_mock_manager).await {
                eprintln!("UI server error: {}", e);
            }
        });
    });

    let http_storage = storage.clone();
    let http_mock_manager = mock_manager.clone();
    let http_port = opt.port;
    let pingora_port = opt.internal_port;
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            if let Err(e) = http_layer::start_http_layer(
                http_port,
                pingora_port,
                http_storage,
                http_mock_manager,
            )
            .await
            {
                eprintln!("HTTP layer error: {}", e);
            }
        });
    });

    // Give UI server time to start
    thread::sleep(std::time::Duration::from_millis(200));

    // Start proxy server on main thread (Pingora creates its own runtime)
    proxy::start_proxy_server(config, storage, mock_manager)
}
