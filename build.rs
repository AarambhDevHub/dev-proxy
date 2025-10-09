use std::path::Path;
use std::process::Command;

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn main() {
    println!("cargo:rerun-if-changed=../ui/src");
    println!("cargo:rerun-if-changed=../ui/static");
    println!("cargo:rerun-if-changed=../ui/package.json");

    // Check if npm is installed
    if !command_exists("npm") {
        eprintln!("\n❌ ERROR: npm is not installed or not in PATH");
        eprintln!("\nPlease install Node.js and npm:");
        eprintln!("  - Ubuntu/Debian: sudo apt install nodejs npm");
        eprintln!("  - Fedora: sudo dnf install nodejs npm");
        eprintln!("  - Arch: sudo pacman -S nodejs npm");
        eprintln!("  - macOS: brew install node");
        eprintln!("  - Or download from: https://nodejs.org/\n");
        std::process::exit(1);
    }

    let ui_dir = Path::new("../ui");

    if !ui_dir.exists() {
        eprintln!("\n❌ ERROR: ui directory not found");
        std::process::exit(1);
    }

    // Check if node_modules exists
    let node_modules = ui_dir.join("node_modules");
    if !node_modules.exists() {
        println!("📦 Installing UI dependencies...");
        let install = Command::new("npm")
            .current_dir(ui_dir)
            .args(&["install"])
            .output()
            .expect("Failed to run npm install");

        if !install.status.success() {
            eprintln!("\n❌ npm install failed:");
            eprintln!("{}", String::from_utf8_lossy(&install.stderr));
            std::process::exit(1);
        }
        println!("✓ Dependencies installed");
    }

    // Run svelte-kit sync to generate .svelte-kit directory
    println!("🔄 Running SvelteKit sync...");
    let sync = Command::new("npm")
        .current_dir(ui_dir)
        .args(&["run", "sync"])
        .output()
        .expect("Failed to run svelte-kit sync");

    if !sync.status.success() {
        eprintln!("\n⚠️  SvelteKit sync warning (continuing anyway):");
        eprintln!("{}", String::from_utf8_lossy(&sync.stderr));
    }

    // Build SvelteKit project
    println!("🔨 Building UI...");
    let output = Command::new("npm")
        .current_dir(ui_dir)
        .args(&["run", "build"])
        .output()
        .expect("Failed to run npm build");

    if !output.status.success() {
        eprintln!("\n❌ SvelteKit build failed:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(1);
    }

    // Verify build directory exists
    let build_path = ui_dir.join("build");
    if !build_path.exists() || !build_path.is_dir() {
        eprintln!("\n❌ ERROR: ui/build directory does not exist after build");
        std::process::exit(1);
    }

    println!("✓ UI build complete!");
}
