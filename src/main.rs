use std::{io, str::FromStr};

use cmd_lib::*;
use structopt::StructOpt;

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
    // ignore error
    let _ = run_cmd!(
        tmux kill-session -t $SESSION;
    );
    run_cmd!(
        tmux new-session -d -s $SESSION;
        info "session setted";
    )?;
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
        tmux select-window -t $SESSION:$WIN_POOL
    )?;

    Ok(())
}

fn run_in_tmux(bin: Code) -> CmdResult {
    let bin_name = bin.to_string();
    let run = |pane: u8| -> CmdResult {
        run_cmd!(
            tmux select-window -t $SESSION:$WIN_POOL;
            tmux selectp -t $pane;
            tmux send-keys $BIN_DIR/$bin_name C-m
        )
    };
    // run
    match bin {
        Code::Pool => run(0)?,
        Code::Distribute => {
            run(1)?;
        }
        Code::Gate => {
            run(2)?;
        }
        Code::All => {
            run_in_tmux(Code::Distribute)?;
            run_in_tmux(Code::Pool)?;
            run_in_tmux(Code::Gate)?;
        }
    }
    Ok(())
}

fn set_config_env() {
    std::env::set_var(
        "DISTRIBUTION_CONFIG",
        BIN_DIR.to_string() + "/config-distribution.toml",
    );
}

fn main() -> CmdResult {
    use_builtin_cmd!(echo, info);
    init_builtin_logger();

    let opt = Opt::from_args();
    set_config_env();
    match opt.cmd {
        Sub::Restart => {
            setup_tmux()?;
            // run in tmux
            run_in_tmux(Code::Distribute)?;
        }
        Sub::SetTmux => {
            setup_tmux()?;
        }
    }

    Ok(())
}

#[test]
fn test() {}
