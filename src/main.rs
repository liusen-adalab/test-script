use std::{io, path::Path, str::FromStr};

use cmd_lib::*;
use structopt::StructOpt;

const COIN_URL: &str = "http://119.91.143.43:29999/NoahPool/coin-distribution.git";
const POOL_URL: &str = "http://119.91.143.43:29999/MinerPool/backend-pools.git";
const IRON_URL: &str = "https://github.com/iron-fish/ironfish.git";

const SESSION: &str = "fish";
const WIN_POOL: &str = "pool";
const WIN_NODE: &str = "node";
const WIN_SERVICE: &str = "service";
const BIN_DIR: &str = "./bin";

#[derive(StructOpt)]
#[structopt(name = "dd_test", about = "Get disk read bandwidth.")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Sub,
}

#[derive(StructOpt)]
enum Sub {
    Restart,
    Update {
        #[structopt(short, default_value = "all")]
        name: Code,
    },
    SetTmux,
}

enum Code {
    Pool,
    Gate,
    All,
    Distribute,
}

impl FromStr for Code {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pool" => Ok(Code::Pool),
            "gate" => Ok(Code::Gate),
            "all" => Ok(Code::All),
            "coin" | "distribute" => Ok(Code::Distribute),
            _ => Err("not support"),
        }
    }
}

impl Code {
    fn to_string(&self) -> &'static str {
        match self {
            Code::Pool => "pool",
            Code::Gate => "gate",
            Code::All => "all",
            Code::Distribute => "distribution",
        }
    }
}

fn setup_dirs() -> CmdResult {
    let exists = |path: &'static str| -> bool { Path::new(path).exists() };
    let codes = [
        Code::Pool.to_string(),
        Code::Gate.to_string(),
        Code::Distribute.to_string(),
    ];
    for code in codes {
        if !exists(code) {
            run_cmd!(
                git submodule add $POOL_URL $code
                git submodule update --init --recursive
            )?;
        }
    }
    Ok(())
}

fn update_code(code: Code) -> CmdResult {
    let name = code.to_string();
    run_cmd!(
        cd $name;
        git pull origin test;
        cargo build --release;
    )?;

    std::env::set_current_dir(name)?;
    let bin_name = get_crate_name()?;
    run_cmd!(
        mv ./target/release/$bin_name ../$BIN_DIR/$name
    )?;
    std::env::set_current_dir("../")?;
    Ok(())
}

fn get_crate_name() -> Result<String, io::Error> {
    let output = run_fun!(
        cargo metadata --format-version=1 --no-deps
    )?;

    let mut output = output.split(":");
    println!("{}", output.next().unwrap());
    while output.next() != Some(r#"[{"name""#) {}
    let mut name = output
        .next()
        .unwrap()
        .to_string()
        .split(",")
        .next()
        .unwrap()
        .to_string();
    name.remove(0);
    name.remove(name.len() - 1);
    Ok(name)
}

/// .
///
/// # Examples
///
/// tmux structure
/// {
///     "pool": {
///         "pane0": "pool",
///         "pane1": "gate",
///         "pane2": "coin-distribute",
///         "pane3": "none"
///     },
///     "node": {
///         "pane0": "node0",
///         "pane1": "node1",
///         "pane2": "node2"
///     },
///     "sevice": {
///         "pane0": "redis-server"
///     }
/// }
fn setup_tmux() -> CmdResult {
    // setup session
    // ignore error
    let _ = run_cmd!(
        // kill previous session
        tmux kill-session -t $SESSION;
        tmux new-session -d -s $SESSION;
        info "session setted";
    );
    // build window for pool service
    run_cmd!(
        tmux rename-window -t "fish:0" $WIN_POOL;
        tmux splitw -h -p 50;
        tmux splitw -v -p 70;
        tmux selectp -t 0;
        tmux splitw -v -p 50;
        info "win for pool";
    )?;

    // build window for blockchain network
    run_cmd!(
        tmux new-window -t $SESSION:1 -n $WIN_NODE;
        tmux splitw -h -p 50;
        tmux splitw -v -p 20;
    )?;

    // build window for middleware service
    run_cmd!(
        tmux new-window -t $SESSION:2 -n $WIN_SERVICE;
    )?;

    Ok(())
}

fn run_in_tmux(bin: Code) -> CmdResult {
    let bin_name = bin.to_string();
    // run
    run_cmd!(
        tmux select-window -t $SESSION:$WIN_SERVICE;
        tmux selectp -t 0;
        tmux send-keys $BIN_DIR/$bin_name C-m
    )?;
    Ok(())
}

fn main() -> CmdResult {
    use_builtin_cmd!(echo, info);
    init_builtin_logger();

    let opt = Opt::from_args();
    match opt.cmd {
        Sub::Restart => {
            // update code
            update_code(Code::All)?;
            setup_tmux()?;
            // run in tmux
        }
        Sub::Update { name } => {
            update_code(name)?;
        }
        Sub::SetTmux => {
            setup_tmux()?;
        }
    }

    Ok(())
}

#[test]
fn test() {
    println!("{}", Path::new("target").exists());
}
