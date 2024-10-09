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
