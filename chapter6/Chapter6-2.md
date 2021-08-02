# 过程宏

参考前一章节的手写版生成代码部分，我们根据 IDL 中定义的结构，一对一地生成了结构体和其对应的 Message 实现，以及辅助的匿名结构体和 Client、Server 代码。

占比相当多的代码是 Message 实现。这部分我们尝试利用过程宏（确切说是继承式过程宏）来实现。

## 0x00 派生过程宏
过程宏分为三种：(ref: #2)
- 派生宏（Derive macro）：用于结构体（struct）、枚举（enum）、联合（union）类型，可为其实现函数或特征（Trait）。
- 属性宏（Attribute macro）：用在结构体、字段、函数等地方，为其指定属性等功能。如标准库中的`#[inline]`、`#[derive(...)]`等都是属性宏。 
- 函数式宏（Function-like macro）：用法与普通的规则宏类似，但功能更加强大，可实现任意语法树层面的转换功能。

要写一个派生宏，我们需要新创建一个 crate，并在 Cargo.toml 里指定 `proc-macro = true`。

这里我们创建 `mini-lust-macros` 并在其 `lib.rs` 中实现这个宏：
```rust
#[proc_macro_derive(Message, attributes(mini_lust))]
pub fn message(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // ...
}
```

我们可以通过 input 参数拿到 `TokenStream`，这个 TokenStream 即被 derive 的 struct、enum 或 union 的代码。
函数最终返回 TokenStream，即额外生成的代码。

我们往往要根据输入结构做代码生成，比如我们要实现类似 Debug 宏，就要知道这个结构是 struct 还是 enum 还是 union，以及有哪些 field，它们的类型都是什么。

那么显然我们需要 parse 这个结构定义。我们可以利用 `syn::parse_macro_input!` 得到这个解析后的结果。

## 0x01 attribute 提取
在我们的目标场景中，我们希望直接在生成的结构体上 derive 即可得到对应的 Message 实现。

由于我们的 Message 实现必须知道每个 field 的 id，而生成结构体里不可能包含这个信息。所以我们可以利用 attribute 来提供这个信息。

```rust
#[derive(::mini_lust_macros::Message)]
pub struct Friend {
    #[mini_lust(field_id = 1, required = "true", field_type = "i32")]
    id: i32,
}
```

目标生成代码：

```rust
impl ::mini_lust_chap6::Message for Friend {
    fn encode<T: ::mini_lust_chap6::TOutputProtocol>(
        &self,
        cx: &::mini_lust_chap6::MsgContext,
        protocol: &mut T,
    ) -> ::mini_lust_chap6::Result<()> {
        protocol.write_struct_begin(&::mini_lust_chap6::TStructIdentifier {
            name: "Friend".to_string(),
        })?;
        let inner = &self.id;
        protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
            name: Some("id".to_string()),
            field_type: ::mini_lust_chap6::TType::I32,
            id: Some(1i16),
        })?;
        protocol.write_i32(*inner)?;
        protocol.write_field_end()?;
        protocol.write_field_stop()?;
        protocol.write_struct_end()?;
        Ok(())
    }
    fn decode<T: ::mini_lust_chap6::TInputProtocol>(
        cx: &mut ::mini_lust_chap6::MsgContext,
        protocol: &mut T,
    ) -> ::mini_lust_chap6::Result<Self> {
        let mut field_id = None;
        protocol.read_struct_begin()?;
        loop {
            let ident = protocol.read_field_begin()?;
            if ident.field_type == ::mini_lust_chap6::TType::Stop {
                break;
            }
            match ident.id {
                Some(1i16) => {
                    ::mini_lust_chap6::ttype_comparing(
                        ident.field_type,
                        ::mini_lust_chap6::TType::I32,
                    )?;
                    let content = protocol.read_i32()?;
                    field_id = Some(content);
                }
                _ => {
                    protocol.skip(ident.field_type)?;
                }
            }
            protocol.read_field_end()?;
        }
        protocol.read_struct_end()?;
        let output = Self {
            id: field_id.ok_or_else(|| {
                ::mini_lust_chap6::new_protocol_error(
                    ::mini_lust_chap6::ProtocolErrorKind::InvalidData,
                    "field id is required",
                )
            })?,
        };
        Ok(output)
    }
}
```

提取字段的 attribute 并不复杂，我们可以直接从 syn 解析到的结构中拿到 fields，并从 field 的 attrs 中读到所有的 attribute，并过滤出带有我们 mini_lust 标记的属性，再自行解析出来。

而这个相对通用的过程有一个叫 [darling](https://crates.io/crates/darling) 的库可以快速帮我们完成。

我们定义 Receiver 结构体：
```rust
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(mini_lust))]
pub(crate) struct Receiver {
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub data: ast::Data<EnumReceiver, FieldReceiver>,
}
```
其中，EnumReceiver 对应 enum 情况下的接收器，FieldReceiver 对应 struct 情况下的接收器。

例如，我们可以定义 FieldReceiver：
```rust
#[derive(Debug, FromField)]
#[darling(attributes(mini_lust))]
pub(crate) struct FieldReceiver {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,

    pub field_type: String,
    pub field_id: i32,
    #[darling(default)]
    pub required: Required,
}
```

之后我们就可以将 parse 后的结构丢进去继续解析：
```rust
let parsed = syn::parse_macro_input!(input as syn::DeriveInput);
let receiver = Receiver::from_derive_input(&parsed);
```

这时我们便能直接拿到解析后的 field_type 和 field_id 等标记。

## 0x02 生成代码
与 rust 编译器打交道的时候，我们接收和返回的是 `proc_macro::TokenStream`。

但是这个接口和实现过于简单，不易使用，我们一般会将其转换为 `proc_macro2::TokenStream` 使用。

当生成代码时，我们可以通过 quote 库来快速包装代码：
```rust
quote::quote! {
    impl XXX for YYY {
        // ...
    }
}
```
这时便会生成 `impl` 代码对应的 TokenStream。

最后通过 `.into()` 可以快速将 `proc_macro2::TokenStream` 转换为需要的 `proc_macro::TokenStream`。

我们通过：
```rust
let ts2 = quote::quote! {
    impl ::mini_lust_chap6::Message for #name #generics {
        fn encode<T: ::mini_lust_chap6::TOutputProtocol>(&self, cx: &::mini_lust_chap6::MsgContext, protocol: &mut T) -> ::mini_lust_chap6::Result<()> {
            #tok_enc
        }

        fn decode<T: ::mini_lust_chap6::TInputProtocol>(cx: &mut ::mini_lust_chap6::MsgContext, protocol: &mut T) -> ::mini_lust_chap6::Result<Self> {
            #tok_dec
        }
    }
};
proc_macro::TokenStream::from(ts2)
```
生成并返回代码。详细的生成细节可以参考本章附的代码 mini-lust-macros。

## 0x03 测试
我们写好了 derive 宏之后要如何测试呢？可以利用 `cargo-expand` 来输出宏展开后的代码。

```bash
cargo install cargo-expand
```

之后我们可以使用 `cargo expand` 来展开代码。你可以尝试在本章附的 demo-derive-macro 包下执行这个命令。

至此，我们利用过程宏实现了对带 attribute 的结构体的 Message 实现生成；并通过 cargo expand 验证展开后的代码。

## 参考链接
1. [如何编写一个过程宏(proc-macro)](https://dengjianping.github.io/2019/02/28/%E5%A6%82%E4%BD%95%E7%BC%96%E5%86%99%E4%B8%80%E4%B8%AA%E8%BF%87%E7%A8%8B%E5%AE%8F(proc-macro).html)
2. [Rust过程宏入门](https://zhuanlan.zhihu.com/p/342408254)