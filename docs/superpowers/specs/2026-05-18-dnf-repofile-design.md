# DNF/YUM Repository Configuration File Format Specification

> 来源: DNF Configuration Reference (dnf.readthedocs.io), libdnf 源码 (rpm-software-management/libdnf dnf-4-master branch)

---

## 1. 文件格式

### 1.1 基本结构

DNF/YUM `.repo` 文件采用 **INI 格式**，由以下元素组成：

- `[section]` — 段声明，方括号包裹
- `key=value` — 键值对，每行一个
- `# comment` — 以 `#` 开头的注释行
- 空行 — 被保留

### 1.2 Section 类型

| 类型 | 说明 | 数量 |
|------|------|------|
| `[main]` | 全局配置，定义默认值 | 0 或 1 |
| `[repo-id]` | 仓库配置 | 0 到多个 |

### 1.3 Repo ID 字符约束

仓库 ID（`[repo-id]` 中方括号内的名称）允许的字符：

```
A-Z a-z 0-9 - _ . :
```

来源定义：
> libdnf: `ConfigParser.hpp` — "Repo id consists of the following characters: `[A-Za-z0-9-_.:]`"

在所有已加载的 `.repo` 文件中，repo ID 必须**全局唯一**。重复的 ID 会导致后者覆盖前者或报错。

### 1.4 注释和空行

- 注释以 `#` 开头，可出现在行首或行尾
- 空行和注释在 parse → render round-trip 中应被保留
- 注释在 section 头部之前（header comments）和 entry 之后（inline comments）都需要保留

libdnf 实现参考:
> libdnf: `ConfigParser.hpp` — "IniParser preserve order of items. Comments and empty lines are kept."

### 1.5 `include=` 指令

`.repo` 文件可包含 `include=` 指令来引用其他配置文件或远程 URL：

```ini
include=https://example.com/extra.repo
include=/etc/yum.repos.d/extra.repo
```

`include=` 可出现在文件级别（在任何 section 之前）或 section 内部。

> 注：本库的 `RepoFile::parse()` 将 `include=` 行保留在 raw_entries 中（不做自动加载），由调用者决定是否解析。`ReposDir::load()` 加载整个目录，天然覆盖所有文件，无需 include 指令。

### 1.6 文件编码

UTF-8，无 BOM。

---

## 2. 选项：完整清单与类型

以下清单来源于 DNF Configuration Reference，按作用域分类。

### 2.1 布尔值语法

所有布尔选项接受以下写法（大小写不敏感：内部转 lowercase 后匹配）：

| 规范值 | 含义 |
|--------|------|
| `1`, `yes`, `true`, `on` | 真 |
| `0`, `no`, `false`, `off` | 假 |

> 来源: `OptionBool.cpp` — `"for (auto & ch : value) ch = std::tolower(ch);"` 然后分别匹配 `trueValues` 和 `falseValues` 两个数组。
> `OptionBool.hpp` — `defTrueValues = {"1", "yes", "true", "on"}`, `defFalseValues = {"0", "no", "false", "off"}`

### 2.2 列表语法

列表类型选项的值用**空格**或**逗号**分隔。同一 key 可多次出现来追加多值：

```ini
baseurl=http://repo1.example.com/$basearch/
baseurl=http://repo2.example.com/$basearch/
```

或：

```ini
baseurl=http://repo1.example.com/$basearch/, http://repo2.example.com/$basearch/
```

> 来源: DNF Configuration Reference — "List: strings separated by space or comma characters"

### 2.3 数值类型

| 类型 | 说明 |
|------|------|
| `integer` | 整数，可选范围约束（min/max） |
| `storage size` | 字节数，可选单位 `k`、`M`、`G` |
| `time` | 秒数，部分选项接受 `-1` 或 `never` 表示永不过期 |

> 来源: `OptionNumber.hpp` — 支持 `int32_t`, `uint32_t`, `int64_t`, `uint64_t`, `float`，带 min/max 约束

### 2.4 String 类型

纯字符串，无转换。部分选项支持正则校验（icase 可选）。

> 来源: `OptionString.hpp` — `"fromString: return value"` (identity transform)

---

## 3. 选项清单

### 3.1 仅 Repo 段可用（Repo-Only Options）

这些选项**不能**在 `[main]` 中设置，只能出现在 `[repo-id]` 段中。

| 选项 | 类型 | 默认值 | 约束 |
|------|------|--------|------|
| `name` | string | repo ID | 人类可读名称 |
| `baseurl` | list of URLs | `[]` | URL 列表，按序 failover |
| `mirrorlist` | URL string | `None` | 单个 URL |
| `metalink` | URL string | `None` | 单个 URL |
| `gpgkey` | list of strings | 空 | GPG 密钥文件 URL 列表 |
| `enabled` | boolean | `True` | 是否启用 |
| `priority` | integer | `99` | 越小优先级越高 |
| `cost` | integer | `1000` | 相对访问成本（同优先级时比较） |
| `module_hotfixes` | boolean | `False` | 禁用模块 RPM 过滤 |
| `type` | string | — | 元数据类型：`rpm-md`（别名：`rpm`, `repomd`, `rpmmd`, `yum`, `YUM`） |
| `mediaid` | string | — | 媒体 ID（来源: libdnf ConfigRepo.hpp，未出现在 conf_ref） |
| `enabled_metadata` | list of strings | — | 按元数据类型启用（来源: libdnf ConfigRepo.hpp，未出现在 conf_ref） |

**最小配置**：repo ID + (`baseurl` | `mirrorlist` | `metalink`) 三者至少一个。

### 3.2 `[main]` 与 Repo 段均可使用

在 `[main]` 中设置作为默认值，在 repo 段中可覆盖。

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `gpgcheck` | boolean | `False` | 是否对**包**进行 GPG 签名检查 |
| `repo_gpgcheck` | boolean | `False` | 是否对**仓库元数据**进行 GPG 签名检查 |
| `localpkg_gpgcheck` | boolean | `False` | 是否对本地包进行 GPG 检查 |
| `bandwidth` | storage size | — | 总带宽限制，单位 `k`/`M`/`G` |
| `deltarpm` | boolean | `False` | 启用 delta RPM |
| `deltarpm_percentage` | integer | `75` | 最大 delta 相对大小 |
| `enablegroups` | boolean | `True` | 允许包组 |
| `excludepkgs` | list (glob) | `[]` | 排除的包（glob 模式） |
| `includepkgs` | list (glob) | `[]` | 仅包含的包（glob 模式） |
| `fastestmirror` | boolean | `False` | 使用最快镜像 |
| `ip_resolve` | enum | either | `4`/`IPv4` 或 `6`/`IPv6` |
| `max_parallel_downloads` | integer | `3` | 最大并发下载数（最大 20） |
| `metadata_expire` | time (seconds) | 48h (172800) | 元数据过期时间；`-1` 或 `never` = 永不过期 |
| `minrate` | storage size | `1000` | 低速阈值（bytes/sec），单位 `k`/`M`/`G` |
| `password` | string | 空 | HTTP Basic Auth 密码 |
| `proxy` | URL string | 空 | 代理服务器 URL；空字符串或 `_none_` 禁用 |
| `proxy_username` | string | 空 | 代理用户名 |
| `proxy_password` | string | 空 | 代理密码 |
| `proxy_auth_method` | enum | `any` | `basic`, `digest`, `negotiate`, `ntlm`, `digest_ie`, `ntlm_wb`, `none`, `any` |
| `proxy_sslcacert` | path | 空 | 代理 SSL CA 证书路径 |
| `proxy_sslverify` | boolean | `True` | 验证代理 SSL 证书 |
| `proxy_sslclientcert` | path | 空 | 代理 SSL 客户端证书路径 |
| `proxy_sslclientkey` | path | 空 | 代理 SSL 客户端密钥路径 |
| `retries` | integer | `10` | 每文件重试次数（`0` = 无限） |
| `skip_if_unavailable` | boolean | `False` | 不可用时静默跳过 |
| `sslcacert` | path | 空 | SSL CA 证书路径 |
| `sslverify` | boolean | `True` | 验证远程 SSL 证书 |
| `sslverifystatus` | boolean | `False` | OCSP stapling 验证 |
| `sslclientcert` | path | 空 | SSL 客户端证书路径 |
| `sslclientkey` | path | 空 | SSL 客户端密钥路径 |
| `throttle` | storage size/% | `0` (无) | 下载限速，绝对值或 `bandwidth` 的百分比 |
| `timeout` | time (seconds) | `30` | 连接超时 |
| `username` | string | 空 | HTTP Basic Auth 用户名 |
| `user_agent` | string | 自动生成 | User-Agent 头 |
| `countme` | boolean | `False` | 每周 metalink GET 带计数标记 |

### 3.3 仅 `[main]` 段可用

这些选项在 repo 段中**无效**，仅用于全局配置。

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `allow_vendor_change` | boolean | `True` | 允许供应商变更 |
| `arch` | string | 自动检测 | CPU 架构 |
| `assumeno` | boolean | `False` | 所有提示自动选 No |
| `assumeyes` | boolean | `False` | 所有提示自动选 Yes |
| `autocheck_running_kernel` | boolean | `True` | 自动检查运行中内核 |
| `basearch` | string | 自动检测 | 基础架构 |
| `best` | boolean | `False` | 优先最高版本，无则失败 |
| `cachedir` | path string | 发行版相关 | 缓存目录 |
| `cacheonly` | boolean | `False` | 仅使用缓存 |
| `check_config_file_age` | boolean | `True` | 配置文件变更时过期元数据 |
| `clean_requirements_on_remove` | boolean | `True` | 删除时清理无用依赖 |
| `config_file_path` | path string | `/etc/dnf/dnf.conf` | 主配置文件路径 |
| `debuglevel` | integer | `2` | 调试级别 0-10 |
| `debug_solver` | boolean | `False` | 创建 solver 调试数据 |
| `defaultyes` | boolean | `False` | 默认 Yes 但仍提示 |
| `diskspacecheck` | boolean | `True` | 事务前磁盘空间检查 |
| `errorlevel` | integer | `3` | 错误级别 0-10（已废弃） |
| `exclude_from_weak` | list | `[]` | 排除弱依赖 |
| `exclude_from_weak_autodetect` | boolean | `True` | 自动检测未满足的弱依赖 |
| `exit_on_lock` | boolean | `False` | 锁被持有时立即退出 |
| `gpgkey_dns_verification` | boolean | `False` | DNSSEC 密钥验证 |
| `group_package_types` | list | `default, mandatory` | 包组类型 |
| `ignorearch` | boolean | `False` | 允许安装与当前架构不兼容的包 |
| `installonlypkgs` | list | kernel 等 | 仅安装不升级的包 |
| `installonly_limit` | integer | `3` | 保留版本数（最小 2，0 = 无限制） |
| `installroot` | absolute path | — | 安装根路径 |
| `install_weak_deps` | boolean | `True` | 安装弱依赖 |
| `keepcache` | boolean | `False` | 保留下载的包 |
| `logdir` | path | `/var/log` | 日志目录 |
| `logfilelevel` | integer | `9` | 日志级别 0-10 |
| `log_compress` | boolean | `False` | 压缩轮转日志 |
| `log_rotate` | integer | `4` | 日志轮转数（0 = 不轮转） |
| `log_size` | storage size | `1 MB` | 单个日志文件大小上限 |
| `metadata_timer_sync` | time | `3 hours` | 元数据定时同步 |
| `module_obsoletes` | boolean | `False` | 应用模块 obsoletes |
| `module_platform_id` | string | — | 格式 `$name:$stream` |
| `module_stream_switch` | boolean | `False` | 允许切换已启用的模块流 |
| `multilib_policy` | enum | `best` | `best` 或 `all` |
| `obsoletes` | boolean | `True` | 启用 obsoletes 处理 |
| `optional_metadata_types` | list | 空 | 支持值：`filelists` |
| `persistence` | string | `auto` | 持久化模式（`auto`/`transient`/`persist`），bootc 系统适用 |
| `persistdir` | path | `/var/lib/dnf` | 持久数据目录 |
| `plugins` | boolean | `True` | 启用插件 |
| `pluginconfpath` | list | `/etc/dnf/plugins` | 插件配置目录 |
| `pluginpath` | list | — | 插件搜索目录 |
| `protected_packages` | list | dnf 等 | 受保护包 |
| `protect_running_kernel` | boolean | `True` | 保护运行中的内核 |
| `releasever` | string | 自动检测 | `$releasever` 的取值来源 |
| `reposdir` | list | — | `.repo` 文件搜索目录 |
| `rpmverbosity` | string | `info` | RPM 详细级别 |
| `strict` | boolean | `True` | 严格模式 |
| `tsflags` | list | — | 事务标志 |
| `upgrade_group_objects_upgrade` | boolean | `True` | 自动升级组 |
| `usr_drift_protected_paths` | list | — | 防 drift 保护路径 |
| `varsdir` | list | `/etc/dnf/vars`, `/etc/yum/vars` | 变量定义目录 |
| `zchunk` | boolean | `True` | zchunk 压缩元数据 |

---

## 4. 变量系统

### 4.1 内置变量

| 变量 | 说明 | 来源 |
|------|------|------|
| `$arch` | CPU 架构（如 `x86_64`） | 不可被用户覆盖 |
| `$basearch` | 基础架构（如 `x86_64`） | 不可被用户覆盖 |
| `$releasever` | 发行版版本号 | 从 RPMDB 获取 |
| `$releasever_major` | `$releasever` 第一个 `.` 之前的部分 | 自动派生 |
| `$releasever_minor` | `$releasever` 第一个 `.` 之后的部分 | 自动派生 |

> 来源: `ConfigParser.cpp` — `splitReleasever()` 按 `.` 拆分

### 4.2 变量引用语法

- `$varname` — 简单引用
- `${varname}` — 花括号引用
- `${varname:-word}` — 默认值扩展：变量为空或未设置时使用 `word`
- `${varname:+word}` — 替代值扩展：变量已设置且非空时使用 `word`

word 本身可递归包含变量引用，最大嵌套深度 **32 层**。

> 来源: `ConfigParser.cpp` — `substitute_expression()` 函数，`MAXIMUM_EXPRESSION_DEPTH = 32`

### 4.3 用户自定义变量

- 环境变量：`DNF_VAR_<NAME>=value`（去掉前缀后作为变量名）
- 文件：`/etc/dnf/vars/<name>` 和 `/etc/yum/vars/<name>`（文件内容作为变量值）
- 兼容变量：`DNF0`–`DNF9`（已废弃）

---

## 5. 解析与序列化行为（来源：libdnf 源码）

### 5.1 顺序保持

> libdnf: `ConfigParser.hpp` — 使用 `PreserveOrderMap<std::string, PreserveOrderMap<std::string, std::string>>` 作为内部容器

section 和 entry 都按照文件中出现顺序保存。

### 5.2 注释存储

> libdnf: `ConfigParser.hpp` — 注释行存储为合成 key `"#" + std::to_string(++itemNumber)`，与真正的 key-value 交替排列，保证位置正确

### 5.3 原始行保留

> libdnf: `ConfigParser.hpp` — `rawItems: std::map<std::string, std::string>` 存储未替换变量之前的原始行
> key 格式：`section + ']' + key`（选项），或仅 `section`（段头）

### 5.4 写入时格式保留

> libdnf: `ConfigParser.cpp` — 写入时找到旧的 rawItem，复用其 `=` 位置和格式，替换 value 部分：
> `"oldRawItem.substr(0, keyAndDelimLength) + value + '\n'"`

### 5.5 布尔值解析

> libdnf: `OptionBool.cpp` — 输入 lowercase 后分别匹配 falseValues 和 trueValues 数组，未匹配则抛出 `InvalidValue`

---

## 6. libdnf 类型系统映射

libdnf 使用 C++ 模板类为每个选项提供类型安全的值存储和解析：

| C++ 类型 | 对应的 DNF 类型 | 内部存储 |
|----------|----------------|---------|
| `OptionBool` | boolean | `bool` |
| `OptionString` | string | `std::string` |
| `OptionStringList` | list | `std::vector<std::string>` |
| `OptionNumber<int32_t>` | integer | `int32_t` |
| `OptionNumber<uint32_t>` | unsigned integer | `uint32_t` |
| `OptionNumber<float>` | float (用于 throttle 百分比) | `float` |
| `OptionEnum<std::string>` | enum (ip_resolve 等) | `std::string` |
| `OptionSeconds` | time | `uint32_t` |
| `OptionChild<T>` | 继承/委托 | 从 `ConfigMain` 获取默认值 |

---

## 7. Rust 类型模型设计

### 7.1 设计原则

1. **忠实于官方规范** — 每个选项的类型、默认值、约束与 DNF Configuration Reference 一致
2. **Rust 原生** — 利用代数类型系统、所有权模型、trait 抽象，不平移 C++ 范式
3. **非法状态不可表达** — URL 用 `url::Url`、路径用 `Utf8PathBuf`、枚举用 `enum`、裸 `String` 不出现在公开 API
4. **Round-trip 保真** — parse → render 不丢失注释、空行、section/entry 顺序
5. **变量不自动展开** — 变量保留原文，展开由调用者控制

### 7.2 社区 Crate 依赖

| Crate | 用途 |
|-------|------|
| `indexmap` | `IndexMap` — section 保序 |
| `url` | `Url` — 所有 URL 字段的类型 |
| `camino` | `Utf8PathBuf` — 所有文件路径字段的类型 |
| `nutype` | 带校验的 newtype 宏 |
| `derive_more` | newtype 样板 derive（`Display`、`AsRef`、`Deref`、`From`） |
| `thiserror` | Error 类型 |

**不引入**: `serde`（库不强制序列化格式）、`winnow`/`nom`（INI 格式简单，手写解析器更精确可控）、`nonempty`（非空由 `nutype` 的 `not_empty` 或 `min_entries` 处理）

### 7.3 模块架构

```
src/
  lib.rs           — pub use
  error.rs         — Error 类型大全
  types.rs         — 所有 newtype、enum、值类型
  repo.rs          — Repo 结构体 + 只读 API
  repofile.rs      — RepoFile 结构体 + parse / render
  mainconfig.rs    — MainConfig 结构体
  builder.rs       — RepoBuilder（创建/修改的 Builder 模式）
  validate.rs      — 校验规则
  diff.rs          — Diff 引擎
  variables.rs     — 变量检测与展开
```

### 7.4 值类型定义（`types.rs`）

#### 7.4.1 标识符

```rust
/// Repo ID — [repo-id] 中方括号内的名称
/// 允许字符: [A-Za-z0-9-_.:]
/// 来源: libdnf ConfigParser.hpp
#[nutype(
    sanitize(trim),
    validate(not_empty, regex = r"^[A-Za-z0-9\-_.:]+$"),
    derive(
        Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
        Display, AsRef, Deref, FromStr,
    ),
)]
pub struct RepoId(String);

/// 人类可读的仓库名称
#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct RepoName(String);

/// HTTP 认证用户名
#[nutype(
    sanitize(trim),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct Username(String);

/// HTTP 认证密码
#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct Password(String);

/// 代理认证用户名
#[nutype(
    sanitize(trim),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct ProxyUsername(String);

/// 代理认证密码
#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct ProxyPassword(String);

/// User-Agent 字符串
#[nutype(
    sanitize(trim),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct UserAgent(String);

/// 模块平台 ID，格式 $name:$stream
#[nutype(
    sanitize(trim),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct ModulePlatformId(String);
```

#### 7.4.2 数值类型

每个选项独立 newtype，互不混淆。默认值来自官方规范。

```rust
/// 仓库优先级，默认 99，越小越高
/// 来源: DNF conf_ref — integer, default 99
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 99),
    default = 99,
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, Deref, From),
)]
pub struct Priority(i32);

/// 相对访问成本，默认 1000
/// 来源: DNF conf_ref — integer, default 1000
#[nutype(
    validate(greater_or_equal = 0),
    default = 1000,
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, Deref, From),
)]
pub struct Cost(i32);

/// 重试次数，默认 10。0 = 无限
/// 来源: DNF conf_ref — integer, default 10
#[nutype(
    validate(greater_or_equal = 0),
    default = 10,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct Retries(u32);

/// 连接超时，默认 30 秒
/// 来源: DNF conf_ref — time in seconds, default 30
#[nutype(
    validate(greater_or_equal = 0),
    default = 30,
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, Deref, From),
)]
pub struct TimeoutSeconds(u32);

/// Delta RPM 百分比，默认 75，0 禁用
/// 来源: DNF conf_ref — integer, default 75
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 100),
    default = 75,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct DeltaRpmPercentage(u32);

/// 最大并发下载数，默认 3，最大 20
/// 来源: DNF conf_ref — integer, default 3
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 20),
    default = 3,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct MaxParallelDownloads(u32);

/// 调试级别 0-10，默认 2
/// 来源: DNF conf_ref — integer, default 2
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 10),
    default = 2,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct DebugLevel(u8);

/// 日志级别 0-10，默认 9
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 10),
    default = 9,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct LogLevel(u8);

/// installonly_limit，0=无限制，默认 3，值 1 明确不允许
/// 来源: DNF conf_ref — "Min 2; 0 = unlimited; Value 1 is explicitly not allowed"
#[nutype(
    validate(greater_or_equal = 0, predicate = |x| x != 1),
    default = 3,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct InstallOnlyLimit(u32);

/// 日志轮转数，0 不轮转，默认 4
#[nutype(
    validate(greater_or_equal = 0),
    default = 4,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct LogRotate(u32);

/// 元数据定时同步（秒），0=禁用，默认 10800 (3h)
#[nutype(
    validate(greater_or_equal = 0),
    default = 10800,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct MetadataTimerSync(u32);

/// 错误输出级别（已废弃），范围 0-10，默认 3
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 10),
    default = 3,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, From),
)]
pub struct ErrorLevel(u8);
```

#### 7.4.3 复合值类型

```rust
/// 存储大小，支持 k/M/G 单位后缀，内部归一化为字节
/// 来源: DNF conf_ref — storage size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageSize(pub u64);

/// throttle 值：绝对大小或 bandwidth 的百分比
/// 来源: DNF conf_ref — storage size/% (absolute or percentage of bandwidth)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Throttle {
    Absolute(StorageSize),
    Percent(u8),  // 0-100
}

/// 元数据过期时间
/// 来源: DNF conf_ref — time in seconds, -1/never = never
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataExpire {
    Duration(u64),  // 默认 172800 (48h)
    Never,
}
```

#### 7.4.4 布尔值

```rust
/// DNF 布尔值，精确匹配 libdnf OptionBool 行为
///
/// 解析输入（大小写不敏感，内部 lowercase 后匹配）:
///   True:  "1", "True", "true", "Yes", "yes", "On", "on"
///   False: "0", "False", "false", "No", "no", "Off", "off"
///
/// 序列化输出: "1" / "0"
///
/// 来源: libdnf OptionBool.hpp/cpp
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DnfBool {
    True,
    False,
}

impl DnfBool {
    pub fn parse(s: &str) -> Result<Self, ParseBoolError>;
}

impl Display for DnfBool { /* → "1" / "0" */ }
impl From<bool> for DnfBool { /* true→True, false→False */ }
impl From<DnfBool> for bool { /* True→true, False→false */ }
```

#### 7.4.5 枚举类型

```rust
/// 代理设置：未设置、明确禁用（`_none_`/空）、或指定 URL
/// 来源: DNF conf_ref — proxy: URL string, empty or "_none_" disables
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxySetting {
    /// 代理未在文件中设置
    Unset,
    /// 显式禁用代理（值为 `_none_` 或空字符串）
    Disabled,
    /// 代理 URL
    Url(Url),
}

/// IP 地址族
/// 来源: DNF conf_ref — "4"/"IPv4" or "6"/"IPv6"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpResolve { V4, V6 }

/// 代理认证方式
/// 来源: DNF conf_ref — proxy_auth_method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyAuthMethod { Any, None_, Basic, Digest, Negotiate, Ntlm, DigestIe, NtlmWb }

/// 仓库元数据类型
/// 来源: DNF conf_ref — "rpm-md" (canonical)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoMetadataType { RpmMd }

/// multilib_policy
/// 来源: DNF conf_ref
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultilibPolicy { Best, All }

/// 持久化模式
/// 来源: DNF conf_ref
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Persistence { Auto, Transient, Persist }

/// RPM 详细级别
/// 来源: DNF conf_ref — rpmverbosity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpmVerbosity { Critical, Emergency, Error, Warn, Info, Debug }

/// [main] tsflags 允许的值
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TsFlag { NoScripts, Test, NoTriggers, NoDocs, JustDb, NoContexts, NoCaps, NoCrypto, Deploops, NoPlugins }
```

### 7.5 核心数据结构

#### 7.5.1 `Repo` — 一个仓库 section 的完整类型化表示

```rust
use url::Url;
use camino::Utf8PathBuf;

/// 一个 [repo-id] section 的类型化、完整表示
///
/// 每个字段的类型由 DNF 规范规定。Option 表示该选项未在文件中出现；
/// 与 libdnf 不同，我们不区分 "未设置" 和 "设为默认值"——都由 Option 表示。
/// 字段按 DNF 规范分组，repo-only 在前，shared 在后。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    // ===== repo-only 字段 =====
    pub id: RepoId,
    pub name: Option<RepoName>,
    pub baseurl: Vec<Url>,
    pub mirrorlist: Option<Url>,
    pub metalink: Option<Url>,
    pub gpgkey: Vec<String>,                  // URL 或本地路径
    pub enabled: Option<DnfBool>,
    pub priority: Option<Priority>,
    pub cost: Option<Cost>,
    pub module_hotfixes: Option<DnfBool>,
    pub metadata_type: Option<RepoMetadataType>,
    pub mediaid: Option<String>,             // 无类型约束的自由字符串
    pub enabled_metadata: Vec<String>,        // 无类型约束的自由字符串列表

    // ===== shared (可在 [main] 或 [repo] 中) =====
    pub excludepkgs: Vec<String>,            // glob 模式
    pub includepkgs: Vec<String>,            // glob 模式
    pub gpgcheck: Option<DnfBool>,
    pub repo_gpgcheck: Option<DnfBool>,
    pub localpkg_gpgcheck: Option<DnfBool>,
    pub skip_if_unavailable: Option<DnfBool>,
    pub deltarpm: Option<DnfBool>,
    pub deltarpm_percentage: Option<DeltaRpmPercentage>,
    pub enablegroups: Option<DnfBool>,
    pub fastestmirror: Option<DnfBool>,
    pub countme: Option<DnfBool>,

    pub bandwidth: Option<StorageSize>,
    pub throttle: Option<Throttle>,
    pub minrate: Option<StorageSize>,

    pub retries: Option<Retries>,
    pub timeout: Option<TimeoutSeconds>,
    pub max_parallel_downloads: Option<MaxParallelDownloads>,
    pub metadata_expire: Option<MetadataExpire>,

    pub ip_resolve: Option<IpResolve>,

    pub sslverify: Option<DnfBool>,
    pub sslverifystatus: Option<DnfBool>,
    pub sslcacert: Option<Utf8PathBuf>,
    pub sslclientcert: Option<Utf8PathBuf>,
    pub sslclientkey: Option<Utf8PathBuf>,

    pub proxy: ProxySetting,
    pub proxy_username: Option<ProxyUsername>,
    pub proxy_password: Option<ProxyPassword>,
    pub proxy_auth_method: Option<ProxyAuthMethod>,
    pub proxy_sslverify: Option<DnfBool>,
    pub proxy_sslcacert: Option<Utf8PathBuf>,
    pub proxy_sslclientcert: Option<Utf8PathBuf>,
    pub proxy_sslclientkey: Option<Utf8PathBuf>,

    pub username: Option<Username>,
    pub password: Option<Password>,
    pub user_agent: Option<UserAgent>,

    // ===== 未知选项（保真通道） =====
    pub extras: IndexMap<String, Vec<String>>,
}
```

**设计决策**：
- `Url` 用于 `baseurl`, `mirrorlist`, `metalink`, `proxy`（必须是合法 URL）
- `gpgkey` 保持 `Vec<String>` — 支持 URL 和本地路径（如 `/etc/pki/rpm-gpg/KEY`）
- `Utf8PathBuf` 用于所有证书/密钥路径
- `Username`/`Password`/`ProxyUsername`/`ProxyPassword` 用 newtype 防止混淆
- `mediaid` 保持 `String` — 无格式约束
- `extras` 保留所有无法识别的 key-value，确保 round-trip 不丢数据

#### 7.5.2 `MainConfig` — `[main]` section 的类型化表示

```rust
/// [main] section 的类型化表示
///
/// 仅包含 main-only 选项。Shared 选项不出现在此——它们在每个 Repo 中独立管理。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainConfig {
    // ---- 系统标识 ----
    pub arch: Option<String>,
    pub basearch: Option<String>,
    pub releasever: Option<String>,

    // ---- 路径 ----
    pub cachedir: Option<Utf8PathBuf>,
    pub persistdir: Option<Utf8PathBuf>,
    pub logdir: Option<Utf8PathBuf>,
    pub config_file_path: Option<Utf8PathBuf>,
    pub installroot: Option<Utf8PathBuf>,
    pub reposdir: Vec<Utf8PathBuf>,
    pub varsdir: Vec<Utf8PathBuf>,
    pub pluginconfpath: Vec<Utf8PathBuf>,
    pub pluginpath: Vec<Utf8PathBuf>,

    // ---- 数值 ----
    pub debuglevel: Option<DebugLevel>,
    pub logfilelevel: Option<LogLevel>,
    pub log_rotate: Option<LogRotate>,
    pub log_size: Option<StorageSize>,
    pub installonly_limit: Option<InstallOnlyLimit>,
    pub errorlevel: Option<ErrorLevel>,            // 已废弃
    pub metadata_timer_sync: Option<MetadataTimerSync>,

    // ---- 布尔 ----
    pub allow_vendor_change: Option<DnfBool>,
    pub assumeno: Option<DnfBool>,
    pub assumeyes: Option<DnfBool>,
    pub autocheck_running_kernel: Option<DnfBool>,
    pub best: Option<DnfBool>,
    pub cacheonly: Option<DnfBool>,
    pub check_config_file_age: Option<DnfBool>,
    pub clean_requirements_on_remove: Option<DnfBool>,
    pub debug_solver: Option<DnfBool>,
    pub defaultyes: Option<DnfBool>,
    pub diskspacecheck: Option<DnfBool>,
    pub exclude_from_weak_autodetect: Option<DnfBool>,
    pub exit_on_lock: Option<DnfBool>,
    pub gpgkey_dns_verification: Option<DnfBool>,
    pub ignorearch: Option<DnfBool>,
    pub install_weak_deps: Option<DnfBool>,
    pub keepcache: Option<DnfBool>,
    pub log_compress: Option<DnfBool>,
    pub module_obsoletes: Option<DnfBool>,
    pub module_stream_switch: Option<DnfBool>,
    pub obsoletes: Option<DnfBool>,
    pub plugins: Option<DnfBool>,
    pub protect_running_kernel: Option<DnfBool>,
    pub strict: Option<DnfBool>,
    pub upgrade_group_objects_upgrade: Option<DnfBool>,
    pub zchunk: Option<DnfBool>,

    // ---- 列表 ----
    pub installonlypkgs: Vec<String>,
    pub protected_packages: Vec<String>,
    pub exclude_from_weak: Vec<String>,
    pub group_package_types: Vec<String>,
    pub optional_metadata_types: Vec<String>,
    pub tsflags: Vec<TsFlag>,
    pub usr_drift_protected_paths: Vec<String>,

    // ---- 枚举 ----
    pub multilib_policy: Option<MultilibPolicy>,
    pub persistence: Option<Persistence>,
    pub rpmverbosity: Option<RpmVerbosity>,
    pub module_platform_id: Option<ModulePlatformId>,

    // ---- 未知 ----
    pub extras: IndexMap<String, Vec<String>>,
}
```

#### 7.5.3 `RepoFile` — 完整的 `.repo` 文件表示

```rust
use indexmap::IndexMap;

/// 一个完整的 .repo 文件的解析后表示
///
/// 这是库的核心类型：parse 产出它，render 消费它。
/// 同时持有类型化数据和格式化元数据，保证 round-trip 保真。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoFile {
    /// 第一个 section 之前的注释和空行
    pub preamble: Vec<String>,

    /// [main] section（如果存在）
    pub main: Option<SectionBlock<MainConfig>>,

    /// 所有 [repo-id] section，按文件顺序
    pub repos: IndexMap<RepoId, SectionBlock<Repo>>,
}

/// 一个 section 的类型化数据 + 格式化元数据
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionBlock<T> {
    /// 该 section 之前的注释/空行
    pub header_comments: Vec<String>,
    /// 类型化数据
    pub data: T,
    /// 每个已知字段的 inline comment，key 为字段的规范名
    pub item_comments: IndexMap<String, String>,
    /// 已知字段在文件中的出现顺序（用于 render 时保持顺序）
    pub item_order: Vec<String>,
    /// 无法识别的条目（原始 key=value + 注释），保持文件顺序
    pub raw_entries: Vec<RawEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEntry {
    pub key: String,
    pub value: String,
    pub inline_comment: Option<String>,
    pub leading_comments: Vec<String>,
}
```

### 7.6 Builder 模式（`builder.rs`）

```rust
/// 通过 Builder 模式创建 Repo，默认值与 DNF 规范一致
let repo = Repo::builder(RepoId::try_new("myrepo")?)
    .name(RepoName::try_new("My Repository")?)
    .baseurl("https://example.com/repo/$basearch/".parse()?)
    .gpgcheck(DnfBool::True)
    .gpgkey("https://example.com/RPM-GPG-KEY")
    .priority(Priority::try_new(50)?)
    .build();

// 修改已有 Repo
let modified = Repo::builder_from(existing_repo)
    .enabled(DnfBool::False)
    .build();
```

### 7.7 `parse` / `render` API

```rust
impl RepoFile {
    /// 从字符串解析一个 .repo 文件
    ///
    /// 返回类型化的 RepoFile，同时保留所有注释和未知条目。
    /// 变量保留原文不展开。
    pub fn parse(input: &str) -> Result<Self, ParseError>;

    /// 渲染为字符串
    ///
    /// 输出保真：注释、空行、section/entry 顺序与输入一致。
    pub fn render(&self) -> String;
}
```

### 7.8 关键架构差异（vs libdnf C++）

| libdnf C++ | 本库 Rust |
|-----------|----------|
| 两层：`ConfigParser`(string map) + `Option<T>`(typed wrapper) | 一层：`RepoFile` 直接持有类型化结构 + 格式元数据 |
| `OptionChild<T>` 继承链 | `SectionBlock<T>` 泛型容器，`Option<T>` 表示未设置 |
| `OptionBool` + `OptionString` + `OptionNumber` 继承体系 | 每个字段用 `nutype` newtype 或标准 Rust 类型 |
| 优先级别 (`Priority::DEFAULT/COMMANDLINE/RUNTIME`) | 不建模优先级——这是调度器概念，不属于格式库 |
| 解析、替换、访问混在一起 | `parse()` 解析、`variables::expand()` 展开、`Repo` 方法访问，职责分离 |
| `std::map<std::string, std::string>` 存 raw 值 | `SectionBlock<T>` 存类型化 `T` + `raw_entries` 存未知项

---

## 8. 完整 API 设计

三级 API 覆盖"大中小"全部操作场景。

### 8.1 大 — `ReposDir`：目录级管理

对应 `/etc/yum.repos.d/` 目录操作：加载、遍历、搜索、跨文件校验、回写。

```rust
/// 管理一个 .repo 文件目录
///
/// 加载目录中所有 *.repo 文件，提供统一的查询和修改接口。
#[derive(Debug)]
pub struct ReposDir {
    path: PathBuf,
    files: IndexMap<String, RepoFile>,
}

impl ReposDir {
    // ===== 生命周期 =====

    /// 从目录路径加载所有 *.repo 文件
    ///
    /// 非 *.repo 文件忽略，解析失败的文件收集在返回的 Errors 中。
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LoadErrors>;

    /// 重新加载（目录内文件可能已被外部修改）
    pub fn reload(&mut self) -> Result<(), LoadErrors>;

    /// 将所有已修改文件写回磁盘
    pub fn save_all(&self) -> Result<(), Vec<(String, io::Error)>>;

    /// 将指定文件写回磁盘
    pub fn save(&self, filename: &str) -> Result<(), io::Error>;

    // ===== 文件级操作 =====

    /// 文件列表（按文件名排序）
    pub fn file_names(&self) -> Vec<&str>;

    /// 获取一个文件的只读引用
    pub fn get_file(&self, filename: &str) -> Option<&RepoFile>;

    /// 获取一个文件的可变引用
    pub fn get_file_mut(&mut self, filename: &str) -> Option<&mut RepoFile>;

    /// 添加或替换一个文件（内存操作，不立刻写入磁盘）
    pub fn set_file(&mut self, filename: &str, file: RepoFile);

    /// 移除一个文件（内存 + 删除磁盘文件）
    pub fn remove_file(&mut self, filename: &str) -> Result<Option<RepoFile>, io::Error>;

    /// 创建一个新文件
    pub fn create_file(&mut self, filename: &str) -> &mut RepoFile;

    // ===== 跨文件查询 =====

    /// 在所有文件中搜索一个 repo
    pub fn find_repo(&self, id: &RepoId) -> Option<(&str, &Repo)>;

    /// 查找包含指定 repo 的文件名
    pub fn file_for_repo(&self, id: &RepoId) -> Option<&str>;

    /// 所有 repo 的合并视图（跨文件，处理 ID 重复）
    pub fn all_repos(&self) -> Vec<&Repo>;

    /// repo 总数
    pub fn repo_count(&self) -> usize;

    /// 遍历所有 repo
    pub fn iter_repos(&self) -> impl Iterator<Item = (&str, &Repo)>;

    // ===== 校验 =====

    /// 校验所有文件：跨文件重复 ID、每个 repo 合法性
    pub fn validate(&self) -> ValidationReport;
}
```

### 8.2 中 — `RepoFile`：单文件管理

单个 `.repo` 文件的 CRUD，对应 `RepoFile` 结构体。

```rust
impl RepoFile {
    // ===== 构造 =====

    /// 空文件
    pub fn new() -> Self;

    /// 从字符串解析
    pub fn parse(input: &str) -> Result<Self, ParseError>;

    /// 从文件路径加载
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ParseError>;

    // ===== 渲染 =====

    /// 渲染为字符串
    pub fn render(&self) -> String;

    /// 写入文件
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), io::Error>;

    // ===== [main] 管理 =====

    pub fn main(&self) -> Option<&SectionBlock<MainConfig>>;
    pub fn main_mut(&mut self) -> Option<&mut SectionBlock<MainConfig>>;
    pub fn set_main(&mut self, config: MainConfig);
    pub fn remove_main(&mut self);

    // ===== Repo 增删改查 =====

    /// 获取 repo 的数据，带 SectionBlock 元数据
    pub fn get(&self, id: &RepoId) -> Option<&SectionBlock<Repo>>;
    pub fn get_mut(&mut self, id: &RepoId) -> Option<&mut SectionBlock<Repo>>;

    /// 添加 repo（ID 已存在则返回错误）
    pub fn add(&mut self, repo: Repo) -> Result<(), AddRepoError>;

    /// 插入或替换 repo（覆盖 SectionBlock 数据，保留 header_comments）
    pub fn set(&mut self, repo: Repo);

    /// 移除 repo
    pub fn remove(&mut self, id: &RepoId) -> Option<SectionBlock<Repo>>;

    /// 是否包含指定 repo
    pub fn contains(&self, id: &RepoId) -> bool;

    /// repo 数量
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;

    // ===== 迭代 =====

    /// 只读遍历所有 (RepoId, &SectionBlock<Repo>)
    pub fn iter(&self) -> impl Iterator<Item = (&RepoId, &SectionBlock<Repo>)>;
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&RepoId, &mut SectionBlock<Repo>)>;
    pub fn repo_ids(&self) -> impl Iterator<Item = &RepoId>;

    // ===== 校验 =====

    /// 校验文件：所有 repo 的必填项、类型正确性、一致性
    pub fn validate(&self) -> ValidationReport;

    // ===== 合并 =====

    /// 合并另一个 RepoFile 的内容到此文件
    /// [main]：other 的选项覆盖 self（Option 字段：Some 覆盖 None）
    /// repo：重复 ID 用 other 的 SectionBlock 覆盖，不重复的追加
    pub fn merge(&mut self, other: RepoFile);
}
```

### 8.3 小 — `Repo` & `MainConfig`：选项级操作

#### 8.3.1 字段操作

`Repo` 和 `MainConfig` 的字段是公开的 Rust 结构体字段。读/写直接通过字段访问：

```rust
// 读
let prio = repo.priority;   // Option<Priority>
let urls = &repo.baseurl;   // &Vec<Url>
let gpgs = &repo.gpgkey;    // &Vec<String>

// 修改
repo.name = Some(RepoName::try_new("My Repo")?);
repo.gpgcheck = Some(DnfBool::True);
repo.baseurl.push("https://example.com/repo/".parse()?);
repo.priority = Some(Priority::try_new(50)?);

// 移除一个选项（恢复为"未设置"）
repo.timeout = None;
```

#### 8.3.2 选项存在性与原始值

```rust
impl Repo {
    /// 列出所有被显式设置为非 None 的选项名
    pub fn set_options(&self) -> Vec<&'static str>;

    /// 某个已知选项是否被显式设置
    pub fn has_option(&self, key: &str) -> bool;

    /// 获取 extras 中的原始值（已知选项通过字段直接访问）
    pub fn extra_value(&self, key: &str) -> Option<&Vec<String>>;
}

impl<T> SectionBlock<T> {
    /// 通过 raw_entries 获取原始字符串值（包括未知和已知选项的未解析形式）
    pub fn raw_value(&self, key: &str) -> Option<&str>;
}
```

#### 8.3.3 选项校验

```rust
impl Repo {
    /// 校验单个 repo
    ///
    /// 检查: 至少一个 URL 来源、互斥约束、值范围、必填项
    pub fn validate(&self) -> ValidationReport;
}

impl MainConfig {
    /// 校验 [main] section
    ///
    /// 检查: installonly_limit ≠ 1、debuglevel/logfilelevel 范围、路径有效性
    pub fn validate(&self) -> ValidationReport;
}
```

#### 8.3.4 URL 来源便捷方法

```rust
impl Repo {
    /// URL 来源类型（解析互斥关系）
    ///
    /// 如果三者都没设置返回 None
    pub fn url_source(&self) -> Option<UrlSource>;
}

pub enum UrlSource {
    BaseUrl(Vec<Url>),
    MirrorList(Url),
    Metalink(Url),
}
```

#### 8.3.5 Builder

```rust
impl Repo {
    /// 创建 Builder
    pub fn builder(id: RepoId) -> RepoBuilder;

    /// 基于已有 Repo 创建 Builder（用于修改）
    pub fn builder_from(existing: &Repo) -> RepoBuilder;
}

/// Repo 的 Builder，每个 setter 返回 Self 支持链式调用
#[derive(Debug)]
pub struct RepoBuilder { /* ... */ }

impl RepoBuilder {
    pub fn build(self) -> Repo;

    // 链式 setter —— 覆盖所有字段
    pub fn name(mut self, v: RepoName) -> Self;
    pub fn baseurl(mut self, v: Url) -> Self;         // 追加
    pub fn baseurls(mut self, v: Vec<Url>) -> Self;   // 设置全部
    pub fn mirrorlist(mut self, v: Url) -> Self;
    pub fn metalink(mut self, v: Url) -> Self;
    pub fn enabled(mut self, v: DnfBool) -> Self;
    pub fn gpgcheck(mut self, v: DnfBool) -> Self;
    pub fn priority(mut self, v: Priority) -> Self;
    // ... 每个字段对应一个 setter
    pub fn extra(mut self, key: &str, value: &str) -> Self;
}
```

---

## 9. 校验（`validate.rs`）

### 9.1 三层校验

| 级别 | 函数 | 检查内容 |
|------|------|---------|
| 小 | `Repo::validate()` | 必填、URL 来源互斥、值范围、GPG key 与 gpgcheck 一致性 |
| 小 | `MainConfig::validate()` | installonly_limit ≥ 2（非 1）、debuglevel 0-10 |
| 中 | `RepoFile::validate()` | section ID 唯一性、所有 Repo 的 validate() |
| 大 | `ReposDir::validate()` | 跨文件 repo ID 重复、所有文件的 validate() |

### 9.2 校验报告

```rust
/// 校验结果
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// 错误级别的问题（必须修复）
    pub errors: Vec<ValidationIssue>,
    /// 警告级别的问题（不阻止使用）
    pub warnings: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn is_ok(&self) -> bool { self.errors.is_empty() }
    pub fn has_issues(&self) -> bool { !self.errors.is_empty() || !self.warnings.is_empty() }
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub level: IssueLevel,
    /// 所属 repo ID（跨文件校验时包含文件名）
    pub location: IssueLocation,
    pub field: Option<String>,
    pub message: String,
}

pub enum IssueLevel { Error, Warning }

pub enum IssueLocation {
    File(String),
    Repo(RepoId),
    Main,
}
```

### 9.3 校验规则清单

| 规则 | 级别 | 说明 |
|------|------|------|
| name 非空 | Error | 名称不能为空字符串 |
| baseurl/mirrorlist/metalink 至少一个 | Error | repo 至少需要一个 URL 来源 |
| baseurl 与 mirrorlist/metalink 互斥 | Warning | 同时设置可能导致 DNF 行为不确定 |
| gpgkey 存在但 gpgcheck=False | Warning | GPG key 设置了但未启用检查 |
| priority 范围 1-99 | Error | 超出范围无效 |
| cost ≥ 0 | Error | 负值无效 |
| max_parallel_downloads ≤ 20 | Error | 超出最大值 |
| installonly_limit ≠ 1 | Error | 值 1 不允许（main-only） |
| 跨文件 repo ID 重复 | Error | 两个文件定义了相同的 repo ID |

---

## 10. Diff（`diff.rs`）

```rust
/// 比较两个 RepoFile
pub fn diff_files(a: &RepoFile, b: &RepoFile) -> FileDiff;

/// 比较两个 Repo
pub fn diff_repos(a: &Repo, b: &Repo) -> RepoDiff;

/// 比较两个 MainConfig
pub fn diff_main(a: &MainConfig, b: &MainConfig) -> ConfigDiff;

#[derive(Debug, Clone)]
pub struct FileDiff {
    /// [main] 的变更
    pub main_changes: Option<ConfigDiff>,
    /// 被添加的 repo ID
    pub repos_added: Vec<RepoId>,
    /// 被移除的 repo ID
    pub repos_removed: Vec<RepoId>,
    /// 被修改的 repo 及其变更
    pub repos_modified: IndexMap<RepoId, RepoDiff>,
    /// 未变化的 repo ID
    pub repos_unchanged: Vec<RepoId>,
    /// 总体：是否有变化
    pub has_changes: bool,
}

#[derive(Debug, Clone)]
pub struct RepoDiff {
    /// 被修改的选项: key, (old_raw, new_raw)
    pub changed: Vec<(String, String, String)>,
    /// 被添加的选项: key, value
    pub added: Vec<(String, String)>,
    /// 被移除的选项: key, value
    pub removed: Vec<(String, String)>,
    /// 是否有变化
    pub has_changes: bool,
}

#[derive(Debug, Clone)]
pub struct ConfigDiff {
    pub changed: Vec<(String, String, String)>,
    pub added: Vec<(String, String)>,
    pub removed: Vec<(String, String)>,
    pub has_changes: bool,
}
```

---

## 11. 变量展开（`variables.rs`）

```rust
/// 展开字符串中的 DNF 变量
///
/// 支持格式:
///   $var, ${var}, ${var:-default}, ${var:+alt}
/// 递归深度上限 32（与 libdnf 一致）
pub fn expand_variables(
    input: &str,
    vars: &HashMap<String, String>,
) -> Result<String, ExpandError>;

/// 检测字符串中包含的变量名（不展开）
pub fn detect_variables(input: &str) -> Vec<String>;
```

---

## 12. 模块依赖关系

```
                    ┌──────────────┐
                    │   types.rs   │ (零内部依赖)
                    └──────┬───────┘
           ┌───────────────┼───────────────┐
           ▼               ▼               ▼
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │   repo.rs    │ │ mainconfig   │ │ variables.rs │
    │  Repo 结构体  │ │   .rs        │ │  (零依赖)     │
    └──────┬───────┘ └──────┬───────┘ └──────────────┘
           │                │
           ▼                ▼
    ┌──────────────┐ ┌──────────────┐
    │  builder.rs  │ │  validate.rs │
    │  RepoBuilder │ │  校验引擎     │
    └──────┬───────┘ └──────────────┘
           │
           ▼
    ┌──────────────┐      ┌──────────────┐
    │  repofile.rs │──────│   diff.rs    │
    │RepoFile结构体 │      │  Diff 引擎    │
    └──────┬───────┘      └──────────────┘
           │
           ▼
    ┌──────────────┐
    │  reposdir.rs │
    │ ReposDir 管理 │
    └──────────────┘
```

- `types.rs` — 零内部依赖（只依赖 crate 外部：url, camino, nutype, derive_more）
- `repo.rs` — 依赖 `types`
- `mainconfig.rs` — 依赖 `types`
- `builder.rs` — 依赖 `repo`, `types`
- `repofile.rs` — 依赖 `repo`, `mainconfig`, `types`
- `reposdir.rs` — 依赖 `repofile`
- `validate.rs` — 依赖 `repo`, `mainconfig`, `types`
- `diff.rs` — 依赖 `repo`, `mainconfig`, `types`
- `variables.rs` — 零内部依赖
