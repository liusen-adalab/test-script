use std::{env, str::FromStr, thread, time::Duration};

use cmd_lib::{
    log::{info, warn},
    *,
};
use dotenv::dotenv;
use structopt::StructOpt;

const SESSION_FISH: &str = "fish";
const SESSION_NODE: &str = "node";
const WIN_POOL: &str = "pool";
const WIN_NODE: &str = "node";
const WIN_SERVICE: &str = "service";
const WIN_MINER: &str = "miner";

const BIN_DIR: &str = "bin";

#[derive(StructOpt)]
#[structopt(name = "dd_test", about = "Get disk read bandwidth.")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Sub,
}

#[derive(StructOpt)]
enum Sub {
    /// 重启全部矿池相关的程序
    Restart,
    /// 建立 tmux 框架
    SetTmux,
    /// 关闭全部矿池相关程序
    Kill {
        /// 是否关闭区块链网络
        code: Option<String>,
    },
    /// 更新代码
    /// 支持 pool, gate, coin, all, self
    Update { code: Vec<Code> },
}

enum Code {
    Pool,
    Gate,
    All,
    Miner,
    Distribute,
    Me,
}

impl FromStr for Code {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pool" => Ok(Code::Pool),
            "gate" => Ok(Code::Gate),
            "all" => Ok(Code::All),
            "coin" | "distribute" => Ok(Code::Distribute),
            "self" | "me" => Ok(Code::Me),
            _ => Err("not support"),
        }
    }
}

impl Code {
    fn to_string(&self) -> &'static str {
        match self {
            Code::Pool => "fish-pool",
            Code::Gate => "pool-gate",
            Code::All => "all",
            Code::Distribute => "coin-distribution",
            Code::Miner => "noah-miner",
            Code::Me => "self",
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
///         "pane1": "null"
///     }
/// }
fn setup_tmux() -> CmdResult {
    // ignore error
    let _ = run_cmd!(
        tmux kill-session -t $SESSION_FISH;
    );
    run_cmd!(
        tmux new-session -d -s $SESSION_FISH;
        info "session setted";
    )?;
    // build window for pool service
    run_cmd!(
        tmux rename-window -t $SESSION_FISH:0 $WIN_POOL;
        tmux splitw -h -p 50;
        tmux splitw -v -p 70;
        tmux selectp -t 0;
        tmux splitw -v -p 50;
        info "win for pool";
    )?;

    // build window for miners
    run_cmd!(
        tmux new-window -t $SESSION_FISH:1 -n $WIN_MINER;
    )?;

    // build window for middleware service
    run_cmd!(
        tmux new-window -t $SESSION_FISH:2 -n $WIN_SERVICE;
        tmux splitw -h -p 50;
    )?;

    // build session for blockchain network
    let node_dir = std::env::var("IRON_DIR").unwrap();
    let cd_to_node = "cd ".to_string() + &node_dir;
    let result = run_cmd!(
        tmux new-session -d -s $SESSION_NODE;
        tmux rename-window -t $SESSION_NODE:0 $WIN_NODE;
        tmux splitw -h -p 50;
        tmux splitw -v -p 100;
        tmux splitw -v -p 300;

        tmux send-keys -t $SESSION_NODE:$WIN_NODE.0 $cd_to_node C-m;
        tmux send-keys -t $SESSION_NODE:$WIN_NODE.0 "clear" C-m;
        tmux send-keys -t $SESSION_NODE:$WIN_NODE.1 $cd_to_node C-m;
        tmux send-keys -t $SESSION_NODE:$WIN_NODE.1 "clear" C-m;
        tmux send-keys -t $SESSION_NODE:$WIN_NODE.2 $cd_to_node C-m;
        tmux send-keys -t $SESSION_NODE:$WIN_NODE.2 "clear" C-m;
        tmux send-keys -t $SESSION_NODE:$WIN_NODE.3 $cd_to_node C-m;
        tmux send-keys -t $SESSION_NODE:$WIN_NODE.3 "clear" C-m;
    );
    if let Err(_) = result {
        warn!("blockchain network has been setup");
    }

    Ok(())
}

fn run_in_tmux(bin: Code) -> CmdResult {
    let bin_name = bin.to_string();
    let run = |pane: u8| -> CmdResult {
        run_cmd!(
            tmux send-keys -t $SESSION_FISH:$WIN_POOL.$pane $BIN_DIR/$bin_name C-m;
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
            run_in_tmux(Code::Miner)?;
        }
        Code::Miner => {
            let cmd = std::env::var("MINER_CMD").unwrap();
            run_cmd!(
                tmux send-keys  -t $SESSION_FISH:$WIN_MINER.0 $cmd C-m;
            )?;
        }
        Code::Me => {
            panic!("don't run `setup` in tmux");
        }
    }
    Ok(())
}

fn run_service() -> CmdResult {
    run_cmd!(
        tmux send-keys -t $SESSION_FISH:$WIN_SERVICE.1 "redis-server" C-m;
    )?;

    Ok(())
}

fn run_chain() -> CmdResult {
    let node1 = std::env::var("NODE1").unwrap();
    let node2 = std::env::var("NODE2").unwrap();
    let node3 = std::env::var("NODE3").unwrap();
    run_cmd!(
        tmux send-keys -t $SESSION_NODE:$WIN_NODE.1 $node1 C-m;
        tmux send-keys -t $SESSION_NODE:$WIN_NODE.2 $node2 C-m;
        tmux send-keys -t $SESSION_NODE:$WIN_NODE.3 $node3 C-m;
    )?;

    info!("new chain");

    Ok(())
}

fn update(code: &Code) -> CmdResult {
    let code_name = code.to_string();
    let dir = match code {
        Code::Pool => std::env::var("POOL_DIR").unwrap(),
        Code::Gate => std::env::var("GATE_DIR").unwrap(),
        Code::Miner => std::env::var("MINER_DIR").unwrap(),
        Code::Distribute => std::env::var("DISTRIBUTE_DIR").unwrap(),
        Code::Me => {
            let self_dir = env::var("SELF").unwrap();
            let cargo_home = env::var("HOME").unwrap() + "/.cargo/bin";
            let target = self_dir.clone() + "/target/release/setup";
            let env_path = self_dir.clone() + "/.env";
            let env_target = env::var("HOME").unwrap() + "/.env";
            return run_cmd!(
                cd $self_dir;
                git pull;
                cargo build --release;
                mv $target $cargo_home;
                rm -f $env_target;
                cp $env_path $env_target
            );
        }
        Code::All => {
            update(&Code::Distribute)?;
            update(&Code::Gate)?;
            update(&Code::Miner)?;
            update(&Code::Pool)?;
            return Ok(());
        }
    };
    let cur = std::env::current_dir().unwrap();
    let bin_dir = cur.join(BIN_DIR);
    let bin_name = bin_dir.join(code_name);
    run_cmd!(
        cd $dir;
        cargo build --release --bin $code_name;
        rm -f $bin_name;
        cp target/release/$code_name $bin_dir;
    )
}

fn main() -> CmdResult {
    use_builtin_cmd!(echo, info);
    init_builtin_logger();

    dotenv().ok();
    let self_dir = env::var("SELF").unwrap();
    env::set_current_dir(self_dir).unwrap();

    let opt = Opt::from_args();
    match opt.cmd {
        Sub::Restart => {
            restart()?;
        }
        Sub::SetTmux => {
            setup_tmux()?;
        }
        Sub::Kill { code } => {
            if run_cmd!(tmux kill-session -t $SESSION_FISH;).is_err() {
                info!("session {} not exist", SESSION_FISH);
            } else {
                info!("tmux session killed: {}", SESSION_FISH);
            };

            if code == Some("all".to_string()) {
                if run_cmd!(tmux kill-session -t $SESSION_NODE;).is_err() {
                    info!("session {} not exist", SESSION_NODE);
                } else {
                    info!("tmux session killed: {}", SESSION_NODE);
                };
                let node_db_dirs = std::env::var("NODE_DATA").unwrap();
                let node_db_dirs: Vec<&str> = node_db_dirs.split(" ").collect();
                // delete blockchain data
                for dir in node_db_dirs {
                    match run_cmd!(rm -rf $dir) {
                        Ok(_) => {
                            info!("iron data directories {} deleted", dir);
                        }
                        Err(err) => {
                            warn!("failed to delete iron data: {}", err);
                        }
                    }
                }
            }
        }
        Sub::Update { code } => {
            for code in code {
                update(&code)?;
                info!("code {} updated", code.to_string());
            }
        }
    }

    Ok(())
}

fn restart() -> Result<(), std::io::Error> {
    setup_tmux()?;
    info!("tmux built");
    run_chain()?;
    info!("chain running");
    run_service()?;
    info!("middleware running");
    run_in_tmux(Code::All)?;
    info!("pool services running");
    info!("attaching tmux...");
    thread::sleep(Duration::from_secs(1));
    run_cmd!(
        tmux select-window -t $SESSION_FISH:$WIN_POOL;
        tmux a -t $SESSION_FISH;
    )?;
    Ok(())
}

#[test]
fn test() {
    dotenv().ok();
}
