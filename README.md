# Atomic Language

Atomic 是一门静态类型、多范式的编程语言，编译器用 Rust 编写，基于 LLVM 后端，支持 JIT 即时编译与 AOT 原生编译。

## 特性

- **静态类型** — 结构化类型推断，支持泛型、类型别名、扩展方法
- **模式匹配** — 穷尽性 when 表达式，支持枚举/结构体/字面量/范围/类型模式
- **一等函数** — Lambda 表达式，隐式 `it` 参数，函数引用
- **集合类型** — List、Set、Map 及丰富的内置操作
- **协程** — 轻量级 Task/Stream，支持异步通信
- **错误处理** — `?` 传播运算符，Option/Result 类型
- **模块系统** — import/export，选择性导入，模块别名
- **FFI** — `external` 关键字，支持调用 C 函数
- **多目标** — 支持 linux-x64、linux-arm64、windows-x64、wasm（需 LLVM 21+）

## 快速开始

### 安装

```bash
# 需要 Rust 工具链和 LLVM 21+
cargo build --release
```

### Hello World

```atomic
// hello.at
fun main() {
    println("Hello, World!")
}
```

```bash
atomic run hello.at
```

### 变量与类型

```atomic
val x = 42             // 不可变
var y = "hello"        // 可变

val z: Int = 10        // 带类型标注
val name: String = "Atomic"
```

### 函数

```atomic
fun add(a: Int, b: Int): Int = a + b

fun greet(name: String): String {
    return "Hello, " + name
}

// 泛型函数
fun <T> identity(x: T): T = x
```

### Lambda

```atomic
val double = { x -> x * 2 }          // 显式参数
val triple = { it * 3 }               // 隐式 it 参数

List[1, 2, 3].map({ it * 2 })          // 作为参数传递
```

### 模式匹配

```atomic
when x {
    0 -> "zero"
    1 -> "one"
    else -> "many"
}

// 枚举解构
when result {
    Ok(value) -> value
    Err(msg) -> "Error: " + msg
}
```

### 枚举

```atomic
enum Option[T] {
    Some(T),
    None
}

enum Result[T, E] {
    Ok(T),
    Err(E)
}
```

### 结构体

```atomic
type Point = {x: Int, y: Int}

val p = {x = 10, y = 20}            // 字面量构造
val {x, y} = p                        // 解构
val px = p.x                          // 字段访问
```

### For 循环

```atomic
// 遍历
for item in List[1, 2, 3] {
    println(item)
}

// for 表达式（收集结果）
val squares = for x in 1..5 { x * x }

// 条件循环
var i = 0
for i < 10 {
    i = i + 1
}

// 无限循环
for {
    println("loop")
}
```

### 错误处理

```atomic
val x? = parse_int("123")      // 错误传播
val y = result?                 // 后缀 try 运算符

val safe = maybe?.field         // 安全字段访问
val result = obj?.method(arg)   // 安全方法调用
```

### 字符串插值

```atomic
val name = "World"
val msg = "Hello, ${name}!"
```

### 集合

```atomic
val list = List[1, 2, 3]
val set = Set["a", "b", "c"]
val map = Map["key": "value", "count": 42]

val filtered = list.filter({ it > 1 })
val mapped = list.map({ it * 2 })
```

### 协程

```atomic
val task = launch {
    // 异步任务
    val (ch, send) = stream()
    // 通过 channel 通信
}
val result = task.wait()
```

### FFI

```atomic
external fun printf(format: String, ...): Int
external type FileHandle
```

## 命令行

```bash
# 运行 Atomic 程序
atomic run file.at

# 编译为可执行文件
atomic build file.at -o output

# 类型检查（不运行）
atomic run file.at --check

# 优化编译
atomic run file.at -O 3

# 指定目标平台
atomic run file.at --target wasm

# 输出 IR/汇编
atomic build file.at --emit ir
atomic build file.at --emit asm

# 创建新项目
atomic init my_project
```

## 项目结构

```
my_project/
├── atom.toml       # 项目配置
├── src/
│   └── main.atom   # 入口文件
└── tests/
```

### atom.toml

```toml
[project]
name = "my_project"
version = "0.1.0"
authors = ["Your Name <email@example.com>"]

[dependencies]

[build]
optimize = true

[profile.release]
opt_level = 3
lto = true
```

## 编译器架构

```
源文件 (.at/.atom)
  → Lexer (lexer.rs)         词法分析
  → Parser (parser.rs)       Pratt 解析器生成 AST
  → TypeChecker (typecheck.rs) 类型检查与推断
  → Codegen (codegen/)        LLVM IR 生成
  → 目标代码 (.o / 可执行文件 / JIT)
```

## 从源码构建

```bash
# 依赖: Rust 1.70+, LLVM 21+
git clone https://github.com/TetraploidHuman/atomic-lang.git
cd atomic-lang
cargo build --release
```

## 许可证

MIT License

## 示例

`examples/` 目录包含 70+ 示例文件：

| 示例 | 说明 |
|------|------|
| `hello.at` | Hello World |
| `fizzbuzz.at` | FizzBuzz |
| `lambda.at` | Lambda 表达式 |
| `enum.at` | 枚举与模式匹配 |
| `struct.at` | 结构体 |
| `for_loop.at` | For 循环 |
| `coroutine.at` | 协程 |
| `map_filter.at` | 高阶函数 |
| `generic_fun.at` | 泛型函数 |
| `import.at` | 模块导入 |
