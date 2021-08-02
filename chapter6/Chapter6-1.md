# Parser
我们需要能够理解用户传入的 thrift IDL，并根据其生成出 rust 结构和其他辅助代码。

本小节会写一个 thrift parser，目标是输入 thrift 文件内容，输出一个便于使用的结构体。

## 0x00 试着写一个简单的 Const Int Parser 吧！
我们使用 [nom](https://github.com/Geal/nom) 处理 BNF 文法描述的 IDL。

为了熟悉 Nom 的使用方式，我们从一个 Demo 入手。

Demo 的目标是能够解析：
1. Identifier: 和 Thrift 定义一样，`( Letter | '_' ) ( Letter | Digit | '.' | '_' )*`
2. IntConstant: 和 Thrift 定义一样，`('+' | '-')? Digit+`
3. IntConstantExpr: Thrift 定义中的 const 包括了多种类型，简便起见我们定义了这么一个描述：`'const' Identifier '=' IntConstant`

开始写 Parser 吧！本节的代码在 `thrift-parser/examples/int_constant.rs` 中。

### IntConstant
我们将 Identifier 定义为：
```rust
#[derive(Debug, Clone, Copy)]
pub struct IntConstant(pub i64);
```

解析函数可以这么写：
```rust
// Parse a int constant value like "+123", "123", "-123".
// IntConstant     ::=  ('+' | '-')? Digit+
fn parse_int_constant(input: &str) -> IResult<&str, IntConstant> {
    map_res(
        recognize(tuple((opt(alt((tag("-"), tag("+")))), digit1))),
        |d_str| -> Result<IntConstant, std::num::ParseIntError> {
            let d = FromStr::from_str(d_str)?;
            Ok(IntConstant(d))
        },
    )(input)
}
```

解释一下：
1. `tag` 表示匹配给定的字符串
2. `opt` 表示可有可无
3. `digit1` 表示至少一个数字字符
4. `tuple` 表示顺序匹配多个
5. `alt` 表示顺序尝试多个，返回第一个成功的匹配
6. `recognize` 表示匹配后将处理前的原始字符串返回
7. `map_res` 表示将第一个参数的返回丢给第二个参数处理，第二个参数的返回签名是 Result

由于数字前面的 `+` 或者 `-` 最多一个，所以我们首先用 `alt((tag("-"), tag("+")))` 去匹配这两个符号中的一个。

又因为这俩符号可有可无，我们直接使用 opt 即可表示：`opt(alt((tag("-"), tag("+"))))`。

之后直接匹配至少一个的数字字符（`digit1`），然后将这两段匹配表达为一个匹配序列：`tuple((opt(alt((tag("-"), tag("+")))), digit1))`。
注：使用 `pair` 也是可以的。

前面匹配之后会返回这两个匹配的结果，但是我们并不在乎这两段是什么，我们在乎的是匹配到的原始字符串（带符号的数字字符串），所以我们使用 `recognize` 包装前面的表达式。

我们需要将这个字符串 parse 为数字。由于这个过程可能会出错，所以我们将错误抛出，使用 `map_res` 将匹配到的原始字符串输入第二个函数处理。

### Identifier
我们将 Identifier 定义为：
```rust
#[derive(Debug, Clone)]
pub struct Identifier(pub String);
```
简便起见，其内部包含了一个 String。如果是出于性能，可以用 Cow。

解析函数：
```rust
// Parse a identifier like "my_age", "my_salary", "my_name".
// Identifier      ::=  ( Letter | '_' ) ( Letter | Digit | '.' | '_' )*
// Note: Identifier is not strictly following the BNF above!
// Instead, "_" and "_123" are not allowed since in rust they are invalid parameter names.
fn parse_identifier(input: &str) -> IResult<&str, Identifier> {
    map(
        recognize(tuple((
            opt(cchar('_')),
            satisfy(|c| c.is_ascii_alphabetic()),
            take_while(|c: char| c.is_ascii_alphanumeric() || c == '.' || c == '_'),
        ))),
        |ident: &str| -> Identifier { Identifier(ident.to_string()) },
    )(input)
}
```
用到的算子和前面的差不多。

`satisfy` 会使用传入的判断函数匹配单个字符；`take_while` 会顺序判断字符直到不满足输入条件。

注意：由于 `_` 和 `_123` 这种变量在 rust 里不可用，所以我们加强了一下限制，不允许这类虽然符合 IDL 规范但是不太适合直接用的写法。

### IntConstantExpr
定义包括 Identifier 和 IntConstant 两部分：
```rust
#[derive(Debug, Clone)]
pub struct IntConstantExpr {
    pub name: Identifier,
    pub value: IntConstant,
}
```

解析函数：
```rust
// Parse a int const expr like "const my_age = +24", "const my_salary = 0".
// Note: This is not thrift definition, it's just for demo.
// IntConstant           ::=  'const' Identifier '=' IntConstant
fn parse_int_constant_expr(input: &str) -> IResult<&str, IntConstantExpr> {
    map(
        tuple((
            tag("const"),
            preceded(space0, parse_identifier),
            preceded(space0, tag("=")),
            preceded(space0, parse_int_constant),
        )),
        |(_, name, _, value)| -> IntConstantExpr { IntConstantExpr { name, value } },
    )(input)
}
```
类似之前的逻辑，这里不同之处在于多个空格：`space0` 可以匹配可有可无的任意多个空格。
我们可以使用 `preceded` 将 `space0` 忽略掉，只返回第二个匹配。

最后使用 map 来忽略不重要的 `const` 和 `=`，并使用 name 和 value 组装 IntConstantExpr 结构体。

看到这里，是不是觉得这堆算子有点多有点乱？如果有点晕的话，可以参考文末的 Nom Combinators 文档链接。

# 0x01 Thrift Parsing
本节将解析 Thrift 文件。

如果你对 Thrift 结构定义有疑惑，可以参考本系列第二章和官方文档，我们的解析将尽量按照官方文档来。

ps:
1. Thrift 官方实现实际上支持了 annotation，但是没有文档化。简便起见，我们不做支持。
2. 本小节附带的代码将是一个新项目，因为它独立于框架代码。

## 0x01.0 Parser Trait

按照 [Thrift IDL 定义](https://thrift.apache.org/docs/idl) ，顶层结构（直接包括在 Docuemnt 中的结构）包括 Header 和 Definition：
- Header: `Include`, `CppInclude`, `Namespace`
- Definition: `Const`, `Typedef`, `Enum`, `Senum`, `Struct`, `Union`, `Exception`, `Service`

我们可以将这些 Thrift 类型直接定义为 Rust 类型，这部分类似第一节 demo 的实现。

区别于第一节 demo 中的例子，我们可以直接为这个类型实现 parse 方法。作为约束，我们可以定义 Parser Trait：
```rust
use nom::IResult;

pub trait Parser: Sized {
    fn parse(input: &str) -> IResult<&str, Self>;
}
```

## 0x01.1 基本类型
对应文档：[link](https://thrift.apache.org/docs/idl#basic-definitions) ，对应代码在 `basic.rs`。

Literal:
```rust
// Literal         ::=  ('"' [^"]* '"') | ("'" [^']* "'")
#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct Literal<'a>(&'a str);

impl<'a> Parser<'a> for Literal<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            alt((
                delimited(cchar('"'), take_until("\""), cchar('"')),
                delimited(cchar('\''), take_until("'"), cchar('\'')),
            )),
            Self,
        )(input)
    }
}
```

Identifier 类似 demo 中的代码，不再展开。

ListSeparator:
```rust
// ListSeparator   ::=  ',' | ';'
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct ListSeparator;

impl<'a> Parser<'a> for ListSeparator {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(one_of(",;"), |_: char| Self)(input)
    }
}
```

注释和分隔符(分隔符包含注释和空格)：
```rust
// 1. The line begins with // or #
// 2. The content between /* and */
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Comment<'a>(&'a str);

impl<'a> Parser<'a> for Comment<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            alt((
                preceded(tag("//"), take_till(|c| c == '\n')),
                preceded(cchar('#'), take_till(|c| c == '\n')),
                delimited(tag("/*"), take_until("*/"), tag("*/")),
            )),
            Self,
        )(input)
    }
}

// 1. Comment
// 2. Space
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct Separator;

impl<'a> Parser<'a> for Separator {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            many1(alt((map(Comment::parse, |_| ()), map(multispace1, |_| ())))),
            |_| Self,
        )(input)
    }
}
```

## 0x01.2 常量类型
对应文档：[link](https://thrift.apache.org/docs/idl#constant-values) ，对应代码在 `constant.rs`。

IntConstant 类似 demo 中的代码，不再展开。

DoubleConstant:

(我们额外利用 `float_cmp` 包实现了近似相等的判断)
```rust
#[derive(derive_newtype::NewType, Debug, Copy, Clone)]
pub struct DoubleConstant(f64);

// DoubleConstant  ::=  ('+' | '-')? Digit* ('.' Digit+)? ( ('E' | 'e') IntConstant )?
impl<'a> Parser<'a> for DoubleConstant {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map_res(
            recognize(tuple((
                opt(alt((cchar('-'), cchar('+')))),
                digit0,
                opt(pair(cchar('.'), digit1)),
                opt(pair(alt((cchar('E'), cchar('e'))), IntConstant::parse)),
            ))),
            |d_str| -> Result<Self, std::num::ParseFloatError> {
                let d = FromStr::from_str(d_str)?;
                Ok(Self(d))
            },
        )(input)
    }
}

impl PartialEq for DoubleConstant {
    fn eq(&self, other: &Self) -> bool {
        float_cmp::approx_eq!(f64, self.0, other.0)
    }
}
```
虽然我们的 Double 可以正常 Parse，但是当我们 Parse List 或 Map 时：
1. 优先尝试 Parse 为 Int：如 `1.1` 会被 parse 为 `i64(1)` 和 `.1`，后者会导致整个匹配失败。
2. 优先尝试 Parse 为 Double：`1.1` 会正常 parse 为 `f64(1.1)`，但 `1` 会 parse 为 `f64(1.1)`，这是不符合预期的。

所以我们这里在处理 double 数据时，如果这个数据可以被 parse 为 int，则抛出错误，让上层继续尝试使用 int 匹配：

```rust
// Double except int: If the double is indeed a int, it will fail!
impl DoubleConstant {
    fn parse2(input: &str) -> IResult<&str, Self> {
        map_res(
            recognize(tuple((
                opt(alt((cchar('-'), cchar('+')))),
                digit0,
                opt(pair(cchar('.'), digit1)),
                opt(pair(alt((cchar('E'), cchar('e'))), IntConstant::parse)),
            ))),
            |d_str| -> Result<Self, std::num::ParseFloatError> {
                if !d_str.contains('.') && !d_str.contains('e') && !d_str.contains('E') {
                    return Err(f64::from_str("").unwrap_err());
                }
                let d = FromStr::from_str(d_str)?;
                Ok(Self(d))
            },
        )(input)
    }
}
```

ConstList 和 ConstMap 解析代码类似，需要考虑一些 corner case，会稍复杂一些，不贴了。

最后我们为总的 enum ConstValue 写匹配：
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue<'a> {
    Identifier(Identifier<'a>),
    Literal(Literal<'a>),
    Double(DoubleConstant),
    Int(IntConstant),
    List(ConstList<'a>),
    Map(ConstMap<'a>),
}

impl<'a> Parser<'a> for ConstValue<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(Identifier::parse, ConstValue::Identifier),
            map(Literal::parse, ConstValue::Literal),
            map(DoubleConstant::parse2, ConstValue::Double),
            map(IntConstant::parse, ConstValue::Int),
            map(ConstList::parse, ConstValue::List),
            map(ConstMap::parse, ConstValue::Map),
        ))(input)
    }
}
```
这里的顺序比较重要，Double 要放在 Int 前面（因为我们的 DoubleConstant::parse2 会在匹配到 Int 时失败，这时会尝试后续匹配；
而如果 Int 在前，则会匹配到 `0.123` 的整数部分，导致后续匹配失败继而整个失败）。

## 0x01.3 其他类型
之后我们按照官方 IDL 定义中的分类，分别定义 Typedef, Enum, Struct, Union, Service, Document 等类型并实现 parse 方法。

额外说明一下 FieldType 这个定义：
```rust
pub enum FieldType<'a> {
    Identifier(Identifier<'a>),
    Bool,
    Byte,
    I8,
    I16,
    I32,
    I64,
    Double,
    String,
    Binary,
    Map(Box<FieldType<'a>>, Box<FieldType<'a>>),
    Set(Box<FieldType<'a>>),
    List(Box<FieldType<'a>>),
}
```
由于 thrift 中类型是可以嵌套的，所以我们需要包一个 Box 否则 rust 无法推断类型大小。


我们的 Parser 已经发布在 [https://crates.io/crates/thrift-parser](https://crates.io/crates/thrift-parser) 。

# 参考链接
- [Nom Combinators](https://github.com/Geal/nom/blob/master/doc/choosing_a_combinator.md)
- [字节范编码器](https://github.com/ihciah/byte-style-encoder)
