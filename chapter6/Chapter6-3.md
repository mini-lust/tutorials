# 代码生成
在前面两个小节，我们实现了 thrift 的 parser 和 Message derive 宏。

我们需要写一个生成器把它们串起来，并生成结构体定义和其余代码（有 derive 宏可以省去在本步生成 Message 实现的麻烦）。

在我们的设计中，用户会在代码里通过 `build.rs` 来触发代码生成，然后在需要使用的地方使用 `include!` 宏来引入生成代码。

## 0x00 最简 demo
如果我们只考虑最简单的一个情况：用户需要生成一个 thrift 文件，且该文件并未 include 别的文件。

这时我们的 Builder 需要做的事情是：1. 读取这个文件 2. Parse 这个文件 3. 使用 Parse 后的结果生成代码 4. 将代码写入目标文件

我们先跳过第 3 步，将整个 Builder 完成（参考 demo-generator）。

```rust
pub struct SimpleBuilder {
    file: Option<std::path::PathBuf>,
}

impl SimpleBuilder {
    pub fn new() -> Self {
        Self {file: None}
    }

    pub fn with_file<P: Into<std::path::PathBuf>>(mut self, p: P) -> Self {
        self.file = Some(p.into());
        self
    }

    pub fn build(self) {
        let idl = std::fs::read_to_string(self.file.expect("idl path must be specified")).unwrap();
        let (_, document) = thrift_parser::document::Document::parse(&idl).unwrap();

        // TODO: document -> code
        let code = quote! {
            pub fn demo() -> String {
                "DEMO".to_string()
            }
        };

        // We will get OUT_DIR when build. However, in test the env not exists, so we use
        // ${CARGO_MANIFEST_DIR}/target. It's not a precise path.
        let output_dir = std::env::var("OUT_DIR")
            .unwrap_or_else(|_| std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/target");
        std::fs::create_dir(&output_dir);
        let mut output_path = std::path::PathBuf::from(output_dir);
        output_path.push("gen.rs");
        let mut output_file = std::fs::File::create(output_path).unwrap();
        output_file.write(code.to_string().as_ref()).unwrap();
    }
}

#[test]
fn simple_build()  {
    let mut idl_path =
        std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
    idl_path.extend(vec!["thrift", "demo.thrift"]);
    SimpleBuilder::new().with_file(idl_path).build();
}
```

运行后，我们会在 `gen.rs` 中找到生成代码： `pub fn demo () -> String { "DEMO" . to_string () }` 。

## 0x01 通过 Build 脚本自动由 IDL 生成代码
前一小节的代码十分简单，但事实上代码生成并没有这么简单！我们有一些问题需要想清楚。

### 0x01.0 层级的问题
别忘了，thrift 中支持通过 include 引入其他 thrift 文件。

有两个需要考虑的问题：
1. IDL 目录层级区分：IDL 文件中的依赖路径 和 生成代码中的依赖路径
2. 多个 Thrift 及其共同依赖：我们不能重复为一个 thrift 文件生成代码

thrift 中通过 namespace 指定了生成后的 mod 结构，而 thrift 文件中的 include 仅仅表示相对于本文件的被依赖文件的路径。

### 0x01.1 节点表示
因为 thrift 文件是相互依赖的一个树形结构，很常规的一个思路就是将每个文件抽象为树中的一个节点。它包含了从文件内容 parse 出的结果 Document。
```rust
pub struct Node {
    // document content, we read and parse it from file
    document: Document,
}
```

#### namespace 问题
前面提到了，我们可以在 thrift 文件中指定 namespace，通过 namespace 我们得知了生成后的代码的层次结构。

如 namespace 为 `a::b::c`，那么我们会将代码生成到：
```rust
pub mod a { pub mod b { pub mod c { 
    // here 
}}}
```

通常情况下我们可以从 Document 里得到这个 namespace。那如果 IDL 里没有指定 namespace 呢？我们就只能用文件名作为 namespace 了。

这样也说明仅仅依靠 parse 出来的结果是不够的。所以我们额外在 Node 定义中加入这个字段：
```rust
pub struct Node {
    // document content, we read and parse it from file
    document: Document,
    // namespace is defined in IDL file like "a.b.c", if no namespace found, the
    // filename in snake case is used.
    namespace: String,
}
```

#### 相对路径问题
thrift 文件中的 include 路径并非相对与当前工作目录，而是相对于当前 thrift 文件。

而这个路径显然仅仅从 Document 结构是拿不到的，因为 thrift 文件内容不可能包含它当前所在的路径信息。

所以我们需要继续扩展 Node 定义：
```rust
pub struct Node {
    // document content, we read and parse it from file
    document: Document,
    // file_abs is IDL file abs path
    file_abs: PathBuf,
    // namespace is defined in IDL file like "a.b.c", if no namespace found, the
    // filename in snake case is used.
    namespace: String,
}
```

此时，Node 已经包含了足够的信息，可以去生成它以及它所依赖的文件。

### 0x01.2 成环检测
如果用户给了一个有环的 thrift 结构，比起生成器卡死，我们更希望能够报错。

我们可以在遍历生成这棵树的时候，带一个可写的 Vec 来记录当前所在的路径。
每当走到一个新的节点，我们会先检查当前节点有没有出现在走过的路上，如果有，说明成环，报错退出。

为了维护这个路径，我们可以在进入新节点时 push 新节点名字；在离开时 pop 掉。

### 0x01.3 避免重复生成
如果多个文件都依赖了同一个文件，就会导致重复遍历。这时我们不希望重复生成，因为这样一定会导致命名冲突。

我们可以在遍历生成这棵树的时候，带一个可写的 HashSet 来记录所有走过的节点。
每当走到一个新节点，除了上一步的成环检查，我们还需要在这之后检查是否已生成过。如果已经生成过该节点，则直接跳过即可。

在离开节点时，将上一步 pop 掉的该节点名加入 HashSet。

### 0x01.4 为节点生成代码
由于前面提到的两个问题，我们需要在生成时带上可写的 generated 和 generating，并带一个可写的 output 方便递归写。
```rust
impl Node {
   /// Generate token recursively to output.
   pub fn generate(
      &self,
      generated: &mut HashSet<PathBuf>,
      generating: &mut Vec<PathBuf>,
      output: &mut TokenStream,
   ) -> GenerateResult<()> {
      // generating 检查（判断成环）
      // generated 检查（判断重复生成）
      // 标记为 generating
      // 读依赖并构造所有依赖 Node
      // 所有依赖 Node.generate(generated, generating, output)
      // 本文件生成
      // 取消 generating 标记并标记为 generated
   }
}
```

## 0x02 自动生成结构体

### 0x02.0 信息传递
我们在 Node 定义里补充了 2 个 Document 里可能没有的信息：file_abs 和 namespace。

file_abs 用于在 generate 实现中读取依赖文件，namespace 是要提供给 Document 用于生成代码的。
除此之外，需要提供给 Document 的信息还有它所有 include 对应的 namespace。

所以我们定义了一个结构体：
```rust
pub struct CodeGenContext {
    // namespaces of all files that it contains, eg: a::b::c, c::d::e
    includes: Vec<String>,
    // namespaces, eg: a::b::C -> [a, b, c]
    namespaces: Vec<String>,
}
```

并定义了一个 trait 用于生成代码：
```rust
pub trait CodeGenWithContext {
    fn gen_token(&self, cx: &CodeGenContext) -> CodeGenResult<TokenStream> {
        let mut stream = TokenStream::new();
        let _ = self.write_token(cx, &mut stream)?;
        Ok(stream)
    }
    fn write_token(&self, cx: &CodeGenContext, output: &mut TokenStream) -> CodeGenResult<()>;
}
```

之后我们为 Document 实现这个 trait 即可。

### 0x02.1 CodeGenWithContext 实现
我们将 Document 的所有 include 生成为 `pub use XXX`；
并为所有的 struct 和 service 调用各自的生成函数；
最后将已经生成的代码包装进它所在的 namespace：`pub mod A {pub mod B {pub moc C {}}}`。

```rust
impl CodeGenWithContext for Document {
    fn write_token(&self, cx: &CodeGenContext, output: &mut TokenStream) -> CodeGenResult<()> {
        let mut generated = TokenStream::new();

        // generate include
        // We may not use includes of self since they are the file system path instead of
        // their namespace.
        // So the includes is set with CodeGenContext.
        for inc in cx.includes.iter() {
            let parts = inc
                .split("::")
                .map(|p| quote::format_ident!("{}", p))
                .collect::<Vec<_>>();
            generated.extend(quote::quote! {
                pub use #(#parts)::*;
            })
        }

        // generate struct
        for stut in self.structs.iter() {
            let _ = stut.write_token(&mut generated)?;
        }

        // generate service
        for service in self.services.iter() {
            let _ = service.write_token(&mut generated)?;
        }

        // generate namespaces, it will wrap the generated above.
        // We may not use namespaces of self since we only want to use scope rs or *.
        // Also, if no namespace exists, we want to use the file stem and self does not
        // know it.
        // So the namespace is set with CodeGenContext.
        for m in cx.namespaces.iter().rev() {
            let ident = quote::format_ident!("{}", m);
            generated = quote::quote! {
                pub mod #ident {
                    #generated
                }
            }
        }
        // write to output
        output.extend(generated);
        Ok(())
    }
}
```

### 0x02.2 struct 和 service 生成
struct 生成主要是生成所有字段和 annotation。

service 需要生成一些相关的匿名结构体和客户端、服务端结构以及其实现。

本部分代码较多且较为机械，不贴了。
