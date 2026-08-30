//! Project automation tasks.
//!
//! Run via `cargo xtask <command>`.

use std::{
    fs,
    fs::File,
    io::{Cursor, ErrorKind},
    net::{Ipv4Addr, SocketAddrV4},
    path::{Component, Path, PathBuf},
    process::{self, Command},
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tiny_http::{Header, Response, Server, StatusCode};
use xshell::{Shell, cmd};

/// Bind address used by `serve-dist`.
const DIST_HOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
/// Default port used by `serve-dist`.
const DEFAULT_DIST_PORT: u16 = 8000;

/// Command line interface for the `xtask` helper.
#[derive(Debug, Parser)]
#[command(name = "xtask")]
struct Cli {
    /// Task to run.
    #[command(subcommand)]
    command: CommandName,
}

/// Supported automation commands.
#[derive(Debug, Subcommand)]
enum CommandName {
    /// Web build and serve tasks.
    #[command(subcommand)]
    Web(WebCommand),
}

/// Web build and serve commands.
#[derive(Debug, Subcommand)]
enum WebCommand {
    /// Install toolchain requirements for building and serving `scurve-web`.
    Setup,
    /// Launch the wasm development server for `scurve-web`.
    Serve,
    /// Build the production web bundle into `dist/`.
    Build,
    /// Serve the `dist/` directory on `http://127.0.0.1:<port>`.
    ServeDist {
        /// Port to bind.
        #[arg(default_value_t = DEFAULT_DIST_PORT)]
        port: u16,
    },
}

/// Common repository paths computed relative to the `xtask` crate.
#[derive(Debug, Clone)]
struct RepoPaths {
    /// Repository root directory.
    root: PathBuf,
    /// `dist/` output directory.
    dist: PathBuf,
    /// Web dev index HTML used by `wasm-server-runner`.
    dev_index_html: PathBuf,
    /// Raw wasm output produced by `cargo build --profile wasm-release`.
    raw_wasm: PathBuf,
}

impl RepoPaths {
    /// Discover repository paths from `CARGO_MANIFEST_DIR`.
    fn discover() -> Result<Self> {
        let xtask_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = xtask_dir
            .parent()
            .context("xtask crate must live at <repo>/xtask")?
            .to_path_buf();

        Ok(Self {
            dist: root.join("dist"),
            dev_index_html: root
                .join("crates")
                .join("scurve-gui")
                .join("assets")
                .join("index.html"),
            raw_wasm: root
                .join("target")
                .join("wasm32-unknown-unknown")
                .join("wasm-release")
                .join("scurve-web.wasm"),
            root,
        })
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        process::exit(1);
    }
}

/// Dispatch the selected `xtask` command.
fn run() -> Result<()> {
    let cli = Cli::parse();
    let paths = RepoPaths::discover()?;

    match cli.command {
        CommandName::Web(cmd) => match cmd {
            WebCommand::Setup => web_setup(&paths),
            WebCommand::Serve => web_serve(&paths),
            WebCommand::Build => web_build(&paths),
            WebCommand::ServeDist { port } => web_serve_dist(&paths, port),
        },
    }
}

/// Create a verbose shell rooted at the repository root.
fn repo_shell(paths: &RepoPaths) -> Result<Shell> {
    let sh = Shell::new()?;
    sh.change_dir(&paths.root);
    Ok(sh)
}

/// Install toolchain requirements for building and serving `scurve-web`.
fn web_setup(paths: &RepoPaths) -> Result<()> {
    let sh = repo_shell(paths)?;

    println!("Refreshing web toolchain requirements...");
    cmd!(sh, "rustup target add wasm32-unknown-unknown").run()?;
    cmd!(sh, "cargo install -f wasm-server-runner").run()?;
    cmd!(sh, "cargo install -f wasm-bindgen-cli").run()?;

    println!();
    println!("Setup complete.");
    println!(
        "If `cargo xtask web serve` reports a wasm-bindgen schema mismatch, rerun this command."
    );
    println!(
        "The dev server uses `wasm-server-runner`, which embeds its own wasm-bindgen support."
    );
    println!();
    println!("Next steps:");
    println!("  cargo xtask web serve   # Start development server");
    println!("  cargo xtask web build   # Build optimized bundle into dist/");

    Ok(())
}

/// Run the wasm dev server for `scurve-web`.
fn web_serve(paths: &RepoPaths) -> Result<()> {
    let sh = repo_shell(paths)?;

    if !paths.dev_index_html.is_file() {
        anyhow::bail!(
            "missing dev index html at {}; expected a checked-in file",
            paths.dev_index_html.display()
        );
    }

    sh.set_var(
        "WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML",
        &paths.dev_index_html,
    );
    cmd!(
        sh,
        "cargo run --target wasm32-unknown-unknown --bin scurve-web"
    )
    .run()?;

    Ok(())
}

/// Build the production web bundle into `dist/`.
fn web_build(paths: &RepoPaths) -> Result<()> {
    let sh = repo_shell(paths)?;

    println!("Building scurve-web (wasm-release)...");
    cmd!(
        sh,
        "cargo build --target wasm32-unknown-unknown --bin scurve-web --profile wasm-release"
    )
    .run()?;

    ensure_raw_wasm_exists(paths)?;
    println!("WASM raw size:\n{}", describe_file(&paths.raw_wasm)?);

    prepare_dist(paths)?;

    if has_command("wasm-bindgen")? {
        wasm_bindgen(paths)?;
        optimize_wasm(paths)?;
        write_dist_index(paths, production_index_html())?;
    } else {
        emit_fallback_bundle(paths)?;
    }

    println!();
    println!("Build complete. Deploy the contents of dist/ via any static web server.");
    println!("Included artifacts:");
    for path in sorted_files(&paths.dist)? {
        println!("{}", describe_file(&path)?);
    }
    println!();
    println!("Next steps:");
    println!("  cargo xtask web serve-dist 8000");

    Ok(())
}

/// Serve the built web bundle from `dist/`.
fn web_serve_dist(paths: &RepoPaths, port: u16) -> Result<()> {
    ensure_dist_ready(paths)?;

    let addr = SocketAddrV4::new(DIST_HOST, port);
    let server =
        Server::http(addr).map_err(|err| anyhow::anyhow!("failed to bind to {addr}: {err}"))?;

    println!(
        "Serving dist/ on http://{}:{} (Ctrl+C to stop)",
        DIST_HOST, port
    );

    for request in server.incoming_requests() {
        handle_dist_request(paths, request)?;
    }

    Ok(())
}

/// Ensure the raw wasm artifact exists after a build.
fn ensure_raw_wasm_exists(paths: &RepoPaths) -> Result<()> {
    if paths.raw_wasm.is_file() {
        return Ok(());
    }

    anyhow::bail!(
        "expected wasm artifact at {}, but it does not exist",
        paths.raw_wasm.display()
    );
}

/// Create or clean the `dist/` output directory.
fn prepare_dist(paths: &RepoPaths) -> Result<()> {
    if paths.dist.exists() {
        println!("Removing existing {} ...", paths.dist.display());
        fs::remove_dir_all(&paths.dist).with_context(|| {
            format!(
                "failed to remove existing dist dir {}",
                paths.dist.display()
            )
        })?;
    }

    fs::create_dir_all(&paths.dist)
        .with_context(|| format!("failed to create dist dir {}", paths.dist.display()))?;
    Ok(())
}

/// Run `wasm-bindgen` for the compiled web artifact.
fn wasm_bindgen(paths: &RepoPaths) -> Result<()> {
    let sh = repo_shell(paths)?;
    let dist = &paths.dist;
    let raw_wasm = &paths.raw_wasm;
    cmd!(
        sh,
        "wasm-bindgen --target web --no-typescript --out-dir {dist} --out-name scurve-web {raw_wasm}"
    )
    .run()?;
    Ok(())
}

/// Optimize the `wasm-bindgen` output using `wasm-opt` when available.
fn optimize_wasm(paths: &RepoPaths) -> Result<()> {
    let bg_wasm = paths.dist.join("scurve-web_bg.wasm");
    if !bg_wasm.is_file() {
        println!("wasm-bindgen output missing; skipping wasm-opt.");
        return Ok(());
    }

    if !has_command("wasm-opt")? {
        println!("wasm-opt not found; skipping additional optimization.");
        return Ok(());
    }

    let sh = repo_shell(paths)?;
    cmd!(sh, "wasm-opt -Oz -o {bg_wasm} {bg_wasm}").run()?;
    Ok(())
}

/// Copy the raw wasm artifact into `dist/` and emit a fallback `index.html`.
fn emit_fallback_bundle(paths: &RepoPaths) -> Result<()> {
    println!("wasm-bindgen not found; creating fallback bundle.");

    let raw_out = paths.dist.join("scurve-web.wasm");
    fs::copy(&paths.raw_wasm, &raw_out).with_context(|| {
        format!(
            "failed to copy raw wasm from {} to {}",
            paths.raw_wasm.display(),
            raw_out.display()
        )
    })?;

    write_dist_index(paths, fallback_index_html())?;
    Ok(())
}

/// Ensure `dist/index.html` exists before serving.
fn ensure_dist_ready(paths: &RepoPaths) -> Result<()> {
    let index_html = paths.dist.join("index.html");
    if index_html.is_file() {
        return Ok(());
    }

    anyhow::bail!("dist/index.html not found. Run `cargo xtask web build` first.");
}

/// Serve a single request from the `dist/` directory.
fn handle_dist_request(paths: &RepoPaths, request: tiny_http::Request) -> Result<()> {
    let Some(rel_path) = sanitize_request_path(request.url()) else {
        request.respond(not_found_response())?;
        return Ok(());
    };

    let path = paths.dist.join(rel_path);
    if !path.is_file() {
        request.respond(not_found_response())?;
        return Ok(());
    }

    let file = File::open(&path)
        .with_context(|| format!("failed to open dist file {}", path.display()))?;
    let mut response = Response::from_file(file);

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let content_type = Header::from_bytes("Content-Type", mime.essence_str())
        .map_err(|()| anyhow::anyhow!("invalid content type"))?;
    response.add_header(content_type);

    request.respond(response)?;
    Ok(())
}

/// Map a URL path into a safe `dist/`-relative filesystem path.
fn sanitize_request_path(url: &str) -> Option<PathBuf> {
    let trimmed = url.trim_start_matches('/');
    let requested = if trimmed.is_empty() {
        "index.html"
    } else {
        trimmed
    };

    let mut out = PathBuf::new();
    for component in Path::new(requested).components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }

    Some(out)
}

/// Build a 404 response.
fn not_found_response() -> Response<Cursor<Vec<u8>>> {
    Response::from_string("Not Found").with_status_code(StatusCode(404))
}

/// Return `true` when `name` is found on `$PATH`.
fn has_command(name: &str) -> Result<bool> {
    match Command::new(name).arg("--help").output() {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err).with_context(|| format!("failed to probe for {name}")),
    }
}

/// Write `dist/index.html` with the supplied contents.
fn write_dist_index(paths: &RepoPaths, html: &str) -> Result<()> {
    let index = paths.dist.join("index.html");
    fs::write(&index, html).with_context(|| format!("failed to write {}", index.display()))?;
    Ok(())
}

/// Return an iterator of files in `dir`, sorted by filename.
fn sorted_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let entries =
        fs::read_dir(dir).with_context(|| format!("failed to read directory {}", dir.display()))?;

    let mut files = Vec::new();
    for entry in entries {
        let path = entry
            .with_context(|| format!("failed to read directory entry in {}", dir.display()))?
            .path();
        if path.is_file() {
            files.push(path);
        }
    }

    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(files)
}

/// Describe a file size and basename.
fn describe_file(path: &Path) -> Result<String> {
    let metadata =
        fs::metadata(path).with_context(|| format!("failed to stat file {}", path.display()))?;
    let size = human_size(metadata.len());
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    Ok(format!("{size} \t{name}"))
}

/// Render a byte count as a human-friendly string.
fn human_size(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut amount = bytes as f64;

    for unit in units {
        if amount < 1024.0 || unit == units[units.len() - 1] {
            return format!("{amount:.1} {unit}");
        }
        amount /= 1024.0;
    }

    format!("{amount:.1} PB")
}

/// HTML used for the production web bundle.
fn production_index_html() -> &'static str {
    r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>spacecurve — Web</title>
  <link rel="icon" href="data:," />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <style>
    html, body { margin: 0; padding: 0; height: 100%; overflow: hidden; background: #2c3e50; font-family: Arial, sans-serif; }
    canvas { display: block; width: 100vw; height: 100vh; border: 2px solid #34495e; border-radius: 8px; box-shadow: 0 4px 8px rgba(0,0,0,.3); }
    .loading { position: absolute; bottom: 16px; left: 50%; transform: translateX(-50%); color: white; font-size: 1.1em; }
  </style>
</head>
<body>
  <canvas id="bevy"></canvas>
  <div class="loading" id="loading">Loading...</div>

  <script type="module">
    import init from './scurve-web.js';
    init().then(() => {
      const loading = document.getElementById('loading');
      if (loading) loading.style.display = 'none';
    }).catch(console.error);
  </script>
</body>
</html>
"#
}

/// HTML used when `wasm-bindgen` is not installed.
fn fallback_index_html() -> &'static str {
    r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>spacecurve — Web (fallback)</title>
  <link rel="icon" href="data:," />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
</head>
<body>
  <p>Fallback build created. To produce a working web bundle, install wasm-bindgen CLI:</p>
  <pre>cargo install wasm-bindgen-cli</pre>
  <p>Then re-run <code>cargo xtask web build</code>.</p>
  <p>Raw wasm artifact is at <code>./dist/scurve-web.wasm</code>.</p>
</body>
</html>
"#
}
