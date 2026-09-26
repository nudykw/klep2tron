//! `ktrl` — command-line client for KTRL (Klep2tron Control Server).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use ktrl::Client;

#[derive(Parser)]
#[command(
    name = "ktrl",
    version,
    about = "Talk to a running Klep2tron client/editor over KTRL"
)]
struct Cli {
    /// Base URL of the control server.
    #[arg(long, global = true, env = "KLEP_CONTROL_URL", default_value = "http://127.0.0.1:15703")]
    url: String,

    /// Bearer token, when the server requires one.
    #[arg(long, global = true, env = "KLEP_CONTROL_TOKEN")]
    token: Option<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Server identity and API version.
    Version,
    /// Full game/map state.
    State,
    /// Capture a PNG screenshot.
    Shot {
        /// `primary` (default), `camera:<id>` or `rtt:<id>`.
        #[arg(long)]
        view: Option<String>,
        /// Output file (`-` for stdout).
        #[arg(short, long, default_value = "/tmp/ktrl_shot.png")]
        out: String,
    },
    /// List UI widgets (and their on-screen rects).
    Ui {
        /// Case-insensitive substring filter.
        #[arg(long)]
        label: Option<String>,
    },
    /// Read recent log lines (warnings/errors included).
    Logs {
        /// Only entries newer than this sequence number (incremental polling).
        #[arg(long)]
        since: Option<u64>,
        #[arg(long, default_value_t = 100)]
        tail: usize,
        /// Minimum severity: `info`, `warn` or `error`.
        #[arg(long)]
        level: Option<String>,
    },
    /// Stream live events (`log`, `state`, `panic`) until interrupted.
    Events {
        /// Only print this event kind.
        #[arg(long)]
        kind: Option<String>,
    },
    /// Entity hierarchy.
    Tree {
        #[arg(long, default_value_t = 4)]
        depth: u32,
        #[arg(long)]
        root: Option<u32>,
    },
    /// One entity: components, transform, asset ids.
    Entity { id: u32 },
    /// Mesh of an entity: attributes, counts, topology.
    Mesh { id: u32 },
    /// Material of an entity.
    Material { id: u32 },
    /// Run an action (StartEditor, Undo, SetTile, …).
    Action {
        name: String,
        /// Extra fields, e.g. `--set x=3 --set tt=WedgeN` (value parsed as JSON).
        #[arg(long = "set", value_name = "KEY=VALUE")]
        set: Vec<String>,
    },
    /// Place a tile (convenience wrapper over the `SetTile` action).
    SetTile {
        x: u32,
        z: u32,
        #[arg(long)]
        h: Option<i32>,
        #[arg(long = "type")]
        tt: Option<String>,
    },
    /// Spawn a fixture entity (plain entity, survives map rebuilds).
    Spawn {
        #[arg(long, default_value = "cube")]
        mesh: String,
        #[arg(long)]
        name: Option<String>,
        /// Translation as `x,y,z`.
        #[arg(long)]
        pos: Option<String>,
        #[arg(long)]
        scale: Option<String>,
        /// Euler rotation in degrees as `x,y,z`.
        #[arg(long = "rot")]
        rot: Option<String>,
        /// sRGB color `r,g,b[,a]` (0..1).
        #[arg(long)]
        color: Option<String>,
    },
    /// Despawn an entity by index.
    Despawn { id: u32 },
    /// Edit an entity's transform (position/scale/rotation).
    Move {
        id: u32,
        #[arg(long)]
        pos: Option<String>,
        #[arg(long)]
        scale: Option<String>,
        #[arg(long = "rot")]
        rot: Option<String>,
        /// Treat `--pos`/`--scale` as deltas.
        #[arg(long)]
        relative: bool,
    },
    /// Inject a key (`tap`/`press`/`release`).
    Key {
        key: String,
        #[arg(long, default_value = "tap")]
        action: String,
    },
    /// Type text into the focused field (`\n`, `\t`, `\u{8}` for Enter/Tab/Backspace).
    Text { text: String },
    /// Click a UI button by label.
    Click { label: String },
    /// Hold a UI button hovered.
    Hover { label: String },
    /// Clear the forced hover.
    Unhover,
    /// Pause or resume virtual time.
    Pause {
        #[arg(long)]
        off: bool,
    },
    /// Advance exactly N frames (blocks until rendered).
    Step {
        #[arg(long, default_value_t = 1)]
        frames: u32,
    },
    /// Run a JSON scenario of steps (see scenario.rs docs for the format).
    Scenario {
        file: PathBuf,
        #[arg(long)]
        verbose: bool,
    },
    /// Record a PNG sequence by stepping frame by frame.
    Record {
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 120)]
        frames: u32,
        /// Write one frame every N frames.
        #[arg(long, default_value_t = 1)]
        every: u32,
        #[arg(long)]
        view: Option<String>,
        #[arg(long, default_value = "frame")]
        prefix: String,
    },
}

fn pretty(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

fn parse_set(pairs: &[String]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for pair in pairs {
        let (key, raw) = pair.split_once('=').unwrap_or((pair.as_str(), "true"));
        let value = serde_json::from_str(raw)
            .unwrap_or_else(|_| serde_json::Value::String(raw.to_string()));
        map.insert(key.to_string(), value);
    }
    serde_json::Value::Object(map)
}

/// Parse `x,y,z` (or any comma-separated numbers) into exactly `expected` floats.
fn parse_floats(spec: &str, expected: usize) -> Option<Vec<f32>> {
    let parts: Vec<f32> = spec
        .split(',')
        .map(|part| part.trim().parse::<f32>().ok())
        .collect::<Option<Vec<_>>>()?;
    (parts.len() == expected).then_some(parts)
}

fn parse_color(spec: &str) -> Option<Vec<f32>> {
    let mut parts = parse_floats(spec, 3).or_else(|| parse_floats(spec, 4))?;
    if parts.len() == 3 {
        parts.push(1.0);
    }
    Some(parts)
}

fn run(client: &Client, cmd: Cmd) -> ktrl::Result<()> {
    match cmd {
        Cmd::Version => println!("{}", pretty(&client.version()?)),
        Cmd::State => println!("{}", pretty(&client.state()?)),
        Cmd::Shot { view, out } => {
            let bytes = client.screenshot(view.as_deref())?;
            if out == "-" {
                use std::io::Write;
                std::io::stdout().write_all(&bytes)?;
            } else {
                std::fs::write(&out, &bytes)?;
                println!("{} ({} bytes)", out, bytes.len());
            }
        }
        Cmd::Ui { label } => println!("{}", pretty(&client.ui_query(label.as_deref())?)),
        Cmd::Logs { since, tail, level } => {
            println!("{}", pretty(&client.logs(since, tail, level.as_deref())?))
        }
        Cmd::Events { kind } => {
            eprintln!("streaming /events (ctrl-c to stop)");
            client.events(|event_kind, data| {
                if kind.as_deref().map_or(true, |want| want == event_kind) {
                    println!("[{event_kind}] {data}");
                }
            })?;
        }
        Cmd::Tree { depth, root } => {
            println!("{}", pretty(&client.scene_tree(depth, root)?))
        }
        Cmd::Entity { id } => println!("{}", pretty(&client.entity(id)?)),
        Cmd::Mesh { id } => println!("{}", pretty(&client.mesh(id)?)),
        Cmd::Material { id } => println!("{}", pretty(&client.material(id)?)),
        Cmd::Action { name, set } => {
            let extra = if set.is_empty() { None } else { Some(parse_set(&set)) };
            println!("{}", client.action(&name, extra)?);
        }
        Cmd::SetTile { x, z, h, tt } => {
            println!("{}", client.set_tile(x, z, h, tt.as_deref())?)
        }
        Cmd::Spawn { mesh, name, pos, scale, rot, color } => {
            let mut args = serde_json::Map::new();
            args.insert("mesh".into(), mesh.into());
            if let Some(name) = name {
                args.insert("name".into(), name.into());
            }
            if let Some(v) = pos.as_deref().and_then(|s| parse_floats(s, 3)) {
                args.insert("translation".into(), v.into());
            }
            if let Some(v) = scale.as_deref().and_then(|s| parse_floats(s, 3)) {
                args.insert("scale".into(), v.into());
            }
            if let Some(v) = rot.as_deref().and_then(|s| parse_floats(s, 3)) {
                args.insert("rotation_euler_deg".into(), v.into());
            }
            if let Some(v) = color.as_deref().and_then(parse_color) {
                args.insert("color".into(), v.into());
            }
            println!("{}", client.spawn_entity(serde_json::Value::Object(args))?);
        }
        Cmd::Despawn { id } => println!("{}", client.despawn_entity(id)?),
        Cmd::Move { id, pos, scale, rot, relative } => {
            let mut args = serde_json::Map::new();
            if let Some(v) = pos.as_deref().and_then(|s| parse_floats(s, 3)) {
                args.insert("translation".into(), v.into());
            }
            if let Some(v) = scale.as_deref().and_then(|s| parse_floats(s, 3)) {
                args.insert("scale".into(), v.into());
            }
            if let Some(v) = rot.as_deref().and_then(|s| parse_floats(s, 3)) {
                args.insert("rotation_euler_deg".into(), v.into());
            }
            if relative {
                args.insert("relative".into(), true.into());
            }
            println!("{}", client.set_transform(id, serde_json::Value::Object(args))?);
        }
        Cmd::Key { key, action } => println!("{}", client.key(&key, &action)?),
        Cmd::Text { text } => println!("{}", client.text(&text)?),
        Cmd::Click { label } => println!("{}", client.ui_click(&label, "click")?),
        Cmd::Hover { label } => println!("{}", client.ui_click(&label, "hover")?),
        Cmd::Unhover => println!("{}", client.ui_click("", "unhover")?),
        Cmd::Pause { off } => println!("{}", client.pause(!off)?),
        Cmd::Step { frames } => println!("{}", client.step(frames)?),
        Cmd::Scenario { file, verbose } => {
            let scenario = ktrl::scenario::Scenario::load(&file)?;
            if let Some(name) = &scenario.name {
                eprintln!("scenario: {name}");
            }
            let steps = ktrl::scenario::run(client, &scenario, verbose)?;
            eprintln!("scenario ok ({steps} steps)");
        }
        Cmd::Record { out, frames, every, view, prefix } => {
            let files = ktrl::scenario::record(
                client,
                &out,
                &prefix,
                frames,
                every,
                view.as_deref(),
            )?;
            println!("wrote {} frames to {}", files.len(), out.display());
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let client = Client::new(cli.url, cli.token);
    match run(&client, cli.cmd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ktrl: {e}");
            ExitCode::FAILURE
        }
    }
}
