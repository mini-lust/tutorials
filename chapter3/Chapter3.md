# 序列化和反序列化

本章主要内容是 Message trait 和 Protocol 定义，以及 Message trait 的实现。

## 0x00 Thrift 协议编码

### 0x00.0 消息体编解码
以下以 GetUserRequest 为例（参考 `lib.rs`）。

其 IDL 定义是：
```
struct GetUserRequest {
    1: i32 user_id,
    2: string user_name,
    3: bool is_male,
}
```

我们为其写的对应序列化函数：
```rust
protocol.write_struct_begin(&TStructIdentifier {
    name: "GetUserRequest".to_string(),
})?;

// user_id
protocol.write_field_begin(&TFieldIdentifier {
    name: Some("user_id".to_string()),
    field_type: TType::I32,
    id: Some(1),
})?;
protocol.write_i32(self.user_id)?;
protocol.write_field_end()?;

// user_name
protocol.write_field_begin(&TFieldIdentifier {
    name: Some("user_name".to_string()),
    field_type: TType::String,
    id: Some(2),
})?;
protocol.write_string(&self.user_name)?;
protocol.write_field_end()?;

// is_male
protocol.write_field_begin(&TFieldIdentifier {
    name: Some("is_male".to_string()),
    field_type: TType::Bool,
    id: Some(3),
})?;
protocol.write_bool(self.is_male)?;
protocol.write_field_end()?;

protocol.write_field_stop()?;
protocol.write_struct_end()?;
Ok(())
```

可以看出，通常我们需要写一下字段头来标明后续数据的类型和长度，再做后续读写数据的流程。
并且在数据写入结束时要写入结束标记。

我们可以对照着看一下序列化的数据：
```rust
GetUserRequest {
    user_id: 7,
    user_name: "ChiHai".to_string(),
    is_male: true,
}

// \x06\x00\x01\x00\x00\x00\x07\x08\x00\x02\x00\x00\x00\x06\x43\x68
// \x69\x48\x61\x69\x02\x00\x03\x01\x00
```

1. write_struct_begin: 在 Binary Protocol 实现中其实什么都没做。
2. write_field_begin: 
   a. 先写 1byte 的 field type，对应 I32 是 `\x06`。
   b. 再写 i16 类型(2byte)的 field id，在我们 case 中是 1，即 `\x00\x01`。
3. write_i32: 写入 4 字节的 i32 数据 7，对应`\x00\x00\x00\x07`。
4. write_field_end: 在 Binary Protocol 实现中其实什么都没做。
5. write_field_begin: field type String = `\x08`；field id i16(2) = `\x00\x02`。
6. write_string: 
   a. 先写 string 长度 i32 6: `\x00\x00\x00\x06`。
   b. 写 string 内容 ChiHai: `\x43\x68\x69\x48\x61\x69`。
7. write_field_end: 在 Binary Protocol 实现中其实什么都没做。
8. write_field_begin: field type Bool = `\x02`；field id i16(3) = `\x00\x03`。
9. write_bool: true = `\x01`。
10. write_field_end: 啥也不做。
11. write_field_stop: `\x00`。
12: write_struct_end: 啥也不做。


解码时（参考 `lib.rs`）：
```rust
let mut output = Self::default();
protocol.read_struct_begin()?;

loop {
    let ident = protocol.read_field_begin()?;
    if ident.field_type == TType::Stop {
        break;
    }
    match ident.id {
        Some(1) => {
            ttype_comparing(ident.field_type, TType::I32)?;

            // read i32
            let content = protocol.read_i32()?;
            output.user_id = content;
        }
        Some(2) => {
            ttype_comparing(ident.field_type, TType::String)?;

            // read string
            let content = protocol.read_string()?;
            output.user_name = content;
        }
        Some(3) => {
            ttype_comparing(ident.field_type, TType::Bool)?;

            // read bool
            let content = protocol.read_bool()?;
            output.is_male = content;
        }
        _ => {
            protocol.skip(ident.field_type)?;
        }
    }
    protocol.read_field_end()?;
}

protocol.read_struct_end()?;
Ok(output)
```

### 0x00.1 衍生协议头
主要说说 Framed 协议头，这个是用的比较多的。当然其他自定义的协议头都可以套。

Framed 协议头是在 Thrift 消息的基础上，在整个消息的前面加 4 字节的长度（u32 小端）。

这个长度信息十分有用，比如要做一个网关代理，只需要双方约定并通过自定义消息头传递必要元信息，显然是不关心消息内容的。
如果没办法提前知道消息长度，只能去逐字段解析并跳过 field，这个是很大的无用 cost。

## 0x01 Message 和 Protocol
序列化 & 反序列化，顾名思义是将消息和二进制互相转换的过程。在 Rust 中我们如何优雅地抽象消息序列化过程呢？

Thrift 规范里写明了如何定义 Message，但是这个仅仅是定义，不涉及序列化的细节，如并不指定一个 i32 要如何表示；
还有一个概念是 Protocol，Protocol 指定了如何具体将消息序列化 & 反序列化。

所以这里有两个概念，一个是 Message，对应了消息的定义；一个是 Protocol，对应了消息的序列化协议。

在 Thrift 中，Protocol 通常有 2 种：Binary 和 Compact。
Binary 协议一般由长度+数据构成，Compact 协议在 Binary 的基础上加上了压缩，对于小数字较多的数据包压缩性能较好。

设想一下，如果要定义一个结构体（以 User 为例）的编解码方式（编解码方式可以有多种），可以怎么写？
1. 定义两个包装：`Binary<T>` 和 `Compact<T>`，并为 `Binary<User>` 和 `Compact<User>` 实现 encode/decode 方法
2. 将 Binary 和 Compact 实现为 Protocol，为 User 实现 `encode_with_protocol<P: Protocol>(p: P)` 和 `decode_with_protocol<P: Protocol>(p: P)`

显然，后者耦合性更低，将消息的编解码流程（这可以是递归的）和具体的 Protocol 解耦，消息（实现了 Message trait）在编解码时使用 Protocol 作为读写方式。

事实上，在 [官方](https://github.com/apache/thrift/blob/master/lib/rs/README.md) 的实现中也都是使用的类似的 Protocol 抽象。

### 流式解码 & 非流式解码
在官方版本中，Protocol 的实现封装了底层 Transport（Read+Write），可以直接操作连接上的 IO 实现流式解码，可以直接异步读写结构体。

这样做的好处是可以适配裸的 Thrift 协议，可以在不预先获知长度的情况下解码；
但是这样做最大的问题是性能不佳：如果数据陆陆续续地缓慢到来，则会多很多读写的系统调用。
一次性读取足够的数据再解码或者编码后一次性写入会减少 syscall 的次数。

由于在协议头里携带 Body 长度信息是一个十分常见且应当的做法（上文提到的 Framed 就是官方的一个版本），在很多场景下可以更高效（如转发场景下可以做到不解包），
本项目不支持裸的 Thrift 协议，至少需要 Framed 或其他能够确定消息长度的协议头。

所以在 Lust 和本项目中，Protocol 只持有 MutBuf 的引用，所有的读写全部是对其操作，而不是网络 IO。

最终我们的 Message Trait 定义（参考 `message.rs`）：
```rust
trait Message {
    fn encode<T: TOutputProtocol>(
        &self,
        cx: &MsgContext,
        protocol: &mut T,
    ) -> Result<(), Error>;

    fn decode<T: TInputProtocol>(
        cx: &mut MsgContext,
        protocol: &mut T,
    ) -> Result<Self, Error>;
}

pub trait TInputProtocol {
    type Error;
    /// Read the beginning of a Thrift message.
    fn read_message_begin(&mut self) -> Result<TMessageIdentifier, Self::Error>;
    /// Read the end of a Thrift message.
    fn read_message_end(&mut self) -> Result<(), Self::Error>;
    /// Read the beginning of a Thrift struct.
    fn read_struct_begin(&mut self) -> Result<Option<TStructIdentifier>, Self::Error>;
    /// Read the end of a Thrift struct.
    fn read_struct_end(&mut self) -> Result<(), Self::Error>;

    // more functions...
}

pub trait TOutputProtocol {
    type Error;

    /// Write the beginning of a Thrift message.
    fn write_message_begin(&mut self, identifier: &TMessageIdentifier) -> Result<(), Self::Error>;
    /// Write the end of a Thrift message.
    fn write_message_end(&mut self) -> Result<(), Self::Error>;
    /// Write the beginning of a Thrift struct.
    fn write_struct_begin(&mut self, identifier: &TStructIdentifier) -> Result<(), Self::Error>;
    /// Write the end of a Thrift struct.
    fn write_struct_end(&mut self) -> Result<(), Self::Error>;

    // more functions...
}
```
Message 的 encode 和 decode 接收 protocol 作为参数，将序列化、反序列化的过程实现为对 protocol 的操作（protocol 内部持有 BufMut 的引用，负责将对其的操作转换为对 BufMut 二进制形式的读写）。

我们将为一些生成的结构体实现 Message。

## 0x02 实现 Binary Protocol
关于 Trait 定义不用自己写太多，全部 Copy 自官方代码（见本章代码的 `protocol.rs` 和 `error.rs`，不贴了）。

之后是实现 Binary Protocol。
首先从官方代码 fork 一下到 `binary.rs`，之后按照我们前面的说明修改，将流式实现改为持有 buffer 引用读写 BufMut。

然后是手写一些 IDL 生成结构体的实现。主要是为它们实现 Message trait，这部分主要在 `lib.rs` 里。
在后面章节中，手写的应该生成的代码也会全放在 `lib.rs` 里。

（另外本代码里修了一个官方代码的 message header strict 判断的 bug，并向官方提了 [PR](https://github.com/apache/thrift/pull/2412) ）

至此，我们可以基于 Message 接口对 IDL 定义的结构体按 Binary Protocol 做序列化 & 反序列化，并且有测试能够验证我们的结果是正确的。

### 0x02.0 零拷贝优化
另外一个后续可以考虑优化的地方是：当读 String 的时候，我们会先调用 read_bytes 将数据拷贝至 `Vec<u8>`，然后再使用其构建 String（传递了所有权无拷贝）。
虽然构建 String 没有拷贝，但是 read_bytes 的时候我们做了一次拷贝。

一个优化方案是，我们将 BufMut 中的字符串部分切下并 freeze 得到 Buf（内部实现是 Arc，有所有权），然后使用它来构建 String（带所有权）并返回给用户。
当这个字符串 drop，这个 buffer 会回到池子中。但唯一的问题在于，原生的 String 内部是 Vec。
所以如果不用一些 unsafe 的骚操作（关于这部分[做了一下尝试](https://gist.github.com/ihciah/4e1d602891c22ddf82cdb81549ba9d41)，感兴趣可以参考），我们就只能另外搞出一个类型来替代 String。但是这样使用起来就不太用户友好。
目前这个优化暂时搁置，后续可以另行考虑。

## 0x03 MsgContext
看到 Context 这个词你可能会想到 golang 里的 Context。没错，它在这里的设计用途其实和 golang 里你写的 context 是一样的。

因为我们框架内部在处理请求时，后面的处理会依赖于前面的结果，所以利用 Context 透传这些信息，就会使这个过程比较容易。

目前我们 MsgContext 中是空的，后续有需求会加入新的字段。

你可能想问：为什么非要在 encode 和 decode 接口上传递 MsgContext 参数？
这是因为我们想让 普通的结构体（类似 Protobuf 做的事情）和 RPC 请求（类似 gRPC 做的事情）的序列化和反序列化共用这个 Message 抽象，我们需要读写 RPC 元信息。
下一章里我们会详细说明。

## 参考链接
Thrift 的一些概念: https://thrift.apache.org/docs/concepts.html
