# 命令行压缩文件并运行代码

本项目演示了如何使用终端命令行进行文件压缩，并运行相应的代码。你可以通过以下操作来实现：

## 压缩文件

在终端中使用以下命令来压缩文件：

```bash
-f <要压缩的文件或目录> compressed.zip
or 
-z <选择压缩方式（zip,tar）> <要压缩的文件或目录>  <保存的地址>
or
-h 你可以查看更多的帮助
```

### zip方法压缩时，可能会有bug，无法将压缩后的文件放回源文件夹，建议使用tar

该命令将会创建一个名为 \`compressed.zip\` 的压缩文件，其中 \`<要压缩的文件或目录>\` 是你想要压缩的文件或目录的路径。

## 运行代码

在运行代码之前，确保你已经安装了 Rust 编程语言以及 Cargo 包管理器。

### 构建可执行程序

在终端中使用以下命令构建可执行程序：

```bash
cargo build --release
```

这将会在项目目录下生成一个名为 \`target/release/main.exe\` 的可执行程序。

### 运行代码

在终端中使用以下命令运行代码：

```bash
cargo run
或者直接下载zip.exe去直接使用
```

这将会编译并运行代码。请确保在代码中已经定义了 \`main\` 函数。

## 贡献

如果你希望贡献代码或改进本项目，请先进行以下操作：

1. Fork 本项目
2. 在你的本地克隆项目：\`git clone https://github.com/hacker1477/compress.git\`
3. 进入项目目录：\`cd 项目名称\`
4. 运行代码：\`cargo run\`

请确保你已经安装了 Rust 编程语言以及 Cargo 包管理器。

5. 在你的本地进行修改、添加新功能或修复错误
6. 将修改推送到你的 GitHub 仓库：\`git push origin master\`
7. 创建一个 Pull 请求，向本项目的 \`master\` 分支提交你的修改

## 联系我们

如果您有任何问题或建议，请提交问题或发送电子邮件至zzj01262022@163.com。

感谢您使用我们的服务!

## 许可证

本项目使用 [MIT 许可证](LICENSE)。
