# clash-tui
clash终端面板

# 编译
1. 安装rust环境
2. 进入源代码目录：

    `cd clash-tui`
3. 编译:

    `cargo build --release`
4. 编译后的可执行程序位于:
 
    `target/release`

# 配置
如果服务器地址为:`127.0.0.1:9090`且没有`secret`。则不需要配置。

配置文件为`clash-tui.ini`，放在程序所在目录或启动目录。

格式为:

```ini
host=127.0.0.1:9090
key=123456
```
其中：

`host`为`clash`配置的`external-controller`

`key`为`clash`配置的`secret`

# 功能
支持以下功能 
- 查看代理
- 测速
- 选择节点
- 查看日志
- 查看连接

# 使用说明
启动后，进入查看代理界面

按键
- 上、下：选择当前代理
- Enter：查看代理中的节点
- L：日志界面
- C：链接界面
 
界面最下行有按键说明

# 界面效果
![image](https://raw.githubusercontent.com/nybuxtsui/clash-tui/refs/heads/main/doc/pic1.png)
