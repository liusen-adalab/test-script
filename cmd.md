## Miner
```
./noah-miner -t 30 -a 0afd09bca4d35a73e03840263da4a92e69bb99103d89d673fb93256cd4075ffac34d11c6a1872947c9138a -p ssl://www.noahpool.xyz:9034 -b 1000000 --coin=iron

./noah-miner --coin=iron --address 0afd09bca4d35a73e03840263da4a92e69bb99103d89d673fb93256cd4075ffac34d11c6a1872947c9138a --pool tcp://54.90.34.95:9034 --thread-count=1

```

## Mysql
mysql --host=175.178.172.159 --user=root --password=Hack1233211234567 noah

## Screen
```
screen -d -r miner 进去指定窗口
ctrl + a +d 退出当前窗口
screen -R newscreen
```

## Iron
- config
    - yarn start config:show --datadir ~/.ironfish-1
    - yarn start config:set enableRpcTcp true
- miner
    - yarn start miners:start yarn start chain:show --datadir ~/.ironfish-1 -t 1
- chain
    - yarn start chain:show --datadir ~/.ironfish-1
    - yarn start blocks:show --datadir ~/.ironfish-1 <HASH>
- genesis
    - yarn start  chain:genesisblock --difficulty=60000000 --datadir ~/.ironfish-1 
    - yarn start  chain:genesisblock --difficulty=131072 --datadir ~/.ironfish-1 