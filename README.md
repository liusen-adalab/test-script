## Usage
基本操作流程
1. 关闭所有程序： setup kill all
2. 重启所有程序：setup start
3. clt-W 进入 miner 窗口，直接回车运行 miner

- 更新程序（git 拉取 test 分支后编译）： setup updata [pool | gate | miner | coin | self | all ]

## TODO
- [ ] build system environment at first time running
    - [ ] nvm curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.3/install.sh | bash
    - [ ] npm 16.x
    - [ ] yarn and install ironfish
    - [ ] rust
    - [ ] sudo apt install libmysqlclient-dev
    - [ ] sudo apt install pkg-config
- [x] use git to update
- [x] more convenient tmux layout
- [x] support to run in any directory

## Bug to Fix
- [x] miner crash at restart
- [ ] determine if running in tmux