# Query Common Group Chats Bot  
一个 Telegram 机器人 + UserBot 组合，用来统计并查询「多个群」的共同成员。

你可以随时在配置文件里增删目标群，或指定额外管理员。

---

功能亮点
- 支持 无限量群 的共同成员查询  
- 通过 Bot 命令 交互，安全、易用  
- 后台使用 UserBot 拉取完整成员列表，数据更准确  
- 支持 多管理员，可在运行时热加载配置  
- 纯 Rust 实现，体积小、启动快、无外部依赖数据库

---

## 准备条件
1. 一个 Telegram Bot（通过 @BotFather 创建）  
2. 一个 普通 Telegram 账号（充当 UserBot，用于拉取成员列表）  
3. 已安装 Rust 1.73+ 与 Cargo

---

## 快速开始

### 1. 克隆并进入项目

```bash
git clone https://github.com/<your-repo>/query-common-group-chats-bot.git
cd query-common-group-chats-bot
```

### 2. 第一次运行（自动生成模板配置）

```bash
cargo run --release
```

首次运行会在 `./` 目录下生成两个文件：
- `config.toml`        —— 目标群、管理员、Bot Token 等  
- `userbot.session`    —— UserBot 登录后持久化的会话文件
- `bot.session`    —— Bot 登录后持久化的会话文件

> 注意：首次运行时终端会提示你用 UserBot 接收验证码，按提示完成即可。

### 3. 修改配置
打开 `config.toml`，按需填写或修改：

```toml
groups = []
admins = []
```

保存后 无需重启，配置会在下次命令时自动热重载。

---

## 使用方式

命令	说明	
`/addadmin <uid>`	新增管理员（仅超级管理员可用）	
`/addgroup <uid>`	新增群聊（仅超级管理员可用）	

---

## 常见问题（FAQ）

### 1. 如何获取群聊 ID？

   群聊 ID

### 2. UserBot 被封禁怎么办？

   删除 `userbot.session` 重新登录即可；若因频繁拉群成员被限，请降低查询频率或仅在工作时段执行。

### 3. 配置文件格式错误导致无法启动？

   不用担心，会自动生成新的模板。

### 4. 如何持久化日志？

   设置环境变量 `RUST_LOG=info` 并重定向输出：  
   
```bash
   RUST_LOG=info cargo run --release > bot.log 2>&1
   ```

---

开发 & 贡献
欢迎 PR / Issue！