# osu2malody-store

让 [Malody V](https://mugzone.net/) 客户端可以直接浏览、搜索和下载 osu!mania 谱面。

基于 Rust + [rosu-v2](https://github.com/MaxOhn/rosu-v2) 构建。

[English](README.md)

## 致谢

本项目灵感来源于 [flben233/OsuToMalodyServer](https://github.com/flben233/OsuToMalodyServer)，感谢原作者的杰出工作。

## 目录

- [使用](#使用)
- [配置](#配置)
- [下载](#下载)
- [构建](#构建)
- [贡献](#贡献)
- [许可证](#许可证)

## 使用

0. 在 [osu! 账户设置](https://osu.ppy.sh/home/account/edit#new-oauth-application) 创建 OAuth 应用（回调 URL 可留空）。

1. 从 [Releases](https://github.com/cheriu/Osu2MalodyBridge/releases) 下载对应平台的压缩包。

2. 解压，编辑 `application.yml` 填入 osu! API 凭证：

   ```yaml
   malody:
     osu:
       clientID: 12345
       clientSecret: 你的 osu-oauth-密钥
   ```

   也可通过环境变量设置（无需修改配置文件）：
   ```bash
   export OSU_CLIENT_ID=12345
   export OSU_CLIENT_SECRET=你的 osu-oauth-密钥
   ```

3. 运行服务器：
   - **Linux**: `./osu2malody-store`
   - **Windows**: `osu2malody-store.exe`

4. 在 Malody V 中，进入 **设置 → 谱面商店 → 自定义服务器**，填入 `http://你的服务器IP:8081`。

5. 开始游玩！

## 配置

所有配置项均在 `application.yml` 中，可通过环境变量覆盖。

```yaml
server:
  port: 8081                        # 监听端口
  # bind_address: "::"              # 绑定地址，默认双栈
  # tls_cert: /path/to/cert.pem     # HTTPS 证书
  # tls_key: /path/to/key.pem       # HTTPS 密钥

malody:
  server:
    api: 202310                     # 服务器 API 版本
    min: 202310                     # 最低客户端 API 版本
    welcome: "Welcome!"             # 欢迎语
    tmp: /tmp/Osu2Malody            # .osz 缓存目录
    # verify_client_auth: false     # 是否验证客户端 uid/key 签名
  osu:
    clientID:                       # osu! OAuth Client ID
    clientSecret:                   # osu! OAuth Client Secret
```

### HTTPS

同时设置 `server.tls_cert` 和 `server.tls_key` 即可启用 HTTPS。

### 客户端验证

当 `verify_client_auth: true` 时，服务器会验证 Malody 客户端发来的 `uid`、`key`、`api` 参数的 RSA 签名，拒绝伪造或篡改的请求。

## 下载

[Releases](https://github.com/cheriu/Osu2MalodyBridge/releases)

提供 Linux (x86_64) 和 Windows (x86_64) 预编译包。

## 构建

```bash
git clone https://github.com/cheriu/Osu2MalodyBridge.git
cd Osu2MalodyBridge/
cargo build --release
# 二进制位于: target/release/osu2malody-store
```

## 贡献

欢迎提交 [Issue](https://github.com/cheriu/Osu2MalodyBridge/issues/new) 或 Pull Request。

## 许可证

[MIT](LICENSE)
