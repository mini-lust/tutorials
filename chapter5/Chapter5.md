# 客户端和服务端实现

直到上一章我们已经实现了 **消息编解码**、**编解码和连接合并提供可读写消息的 Transport**，并定义好了 Thrift Error 和编解码过程透传的 MsgContext。

本章的目标是基于前面手写代码，实现一个能用的客户端和服务端（无服务发现和 LB，需要手动指定目的地）。

新增的东西主要是客户端和服务端的实现，以及连接的抽象。

## 0x00 拨出连接抽象
主要代码在 `connection.rs` 中。

![](../asserts/images/make-connection.png)

### 0x00.0 BoxedIo
通常情况下我们创建的连接是 TcpStream 和 UnixStream，它们都实现了 AsyncRead 和 AsyncWrite。

那么我们可以利用 Trait Object 机制将其抽象为 BoxedIo 结构体（这部分参考了 Tonic 的实现）：
```rust
pub trait Io: AsyncWrite + AsyncRead + Send + 'static {}
pub struct BoxedIo(Pin<Box<dyn Io>>);
```

然后我们可以为所有实现了 `AsyncRead + AsyncWrite` 的结构体实现 Io：
```rust
impl<T> Io for T where T: AsyncRead + AsyncWrite + Send + 'static {}
```

BoxedIo 实现了 AsyncRead 和 AsyncWrite。
```rust
impl BoxedIo {
    pub fn new<I: Io>(io: I) -> BoxedIo {
        BoxedIo(Box::pin(io))
    }
}

impl AsyncRead for BoxedIo {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for BoxedIo {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
```

有了这个连接的抽象，我们就可以为动态确定的类型实现 MakeConnection（下一小节会提到）。
[MakeConnection](https://docs.rs/tower/0.4.8/tower/make/trait.MakeConnection.html) 是 tower 的一个 Trait。
我们实现符合特定要求的 Service 即可实现 MakeConnection。

### 0x00.1 SocketOrUnix
这里我们将地址包装为一个 Enum：
```rust
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SocketOrUnix {
    Socket(SocketAddr),
    #[cfg(unix)]
    Unix(PathBuf),
}
```

然后为这个地址实现 MakeConnection:
```rust
pub struct DefaultMakeConnection;

impl Service<SocketOrUnix> for DefaultMakeConnection {
    type Response = BoxedIo;
    type Error = std::io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: SocketOrUnix) -> Self::Future {
        match req {
            SocketOrUnix::Socket(addr) => {
                Box::pin(FutureExt::map(tokio::net::TcpStream::connect(addr), |r| {
                    r.map(BoxedIo::new)
                }))
            }
            #[cfg(unix)]
            SocketOrUnix::Unix(path) => {
                Box::pin(FutureExt::map(tokio::net::UnixStream::connect(path), |r| {
                    r.map(BoxedIo::new)
                }))
            }
        }
    }
}
```

当然，对于可以编译期确定地址类型的，我们可以利用泛型做静态分发提升性能，所以也为 SocketAddr 和 PathBuf 实现了 MakeConnection：
```rust
impl Service<SocketAddr> for DefaultMakeConnection {
    type Response = tokio::net::TcpStream;
    type Error = std::io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: SocketAddr) -> Self::Future {
        Box::pin(tokio::net::TcpStream::connect(req))
    }
}

#[cfg(unix)]
impl Service<PathBuf> for DefaultMakeConnection {
    type Response = tokio::net::UnixStream;
    type Error = std::io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: PathBuf) -> Self::Future {
        Box::pin(tokio::net::UnixStream::connect(req))
    }
}
```
至此，我们为 DefaultMakeConnection 实现了传入 SocketAddr/PathBuf/SocketOrUnix 创建连接的 Service。

### 0x00.2 MakeTransport

![](../asserts/images/make-transport.png)

有了 `MakeConnection` 和 `MakeCodec`，我们就可以先调用 `MakeConnection` 创建一个连接，再调用 `MakeCodec` 创建 Codec；
最后将连接和 Codec 合并为 Transport。

我们把这个过程封装成 MakeTransport。

为什么做这个封装？一个原因是为了更好的封装性，客户端直接 `mt.make_transport(target)` 即可拿到可以直接读写结构体的 Sink + Stream；
另一个更重要的原因是这样方便我们做连接池的抽象。

通常我们连接池里缓存的是连接，但是由于是确定的，我们可以直接将这个 Transport 缓存下来。
通过一个 PoolMakeTransport 结构体包装 FramedMakeTransport，实现 MakeTransport trait，我们就可以做到无感知的连接复用。

PoolMakeTransport 目前先不实现，将在后续章节加进去。

## 0x02 监听地址和拨入连接的抽象
主要代码在 `server.rs` 中。

Server 侧的事情主要是 bind 地址并接收连接，之后和 Codec 一起生成 Transport（类似客户端侧的 MakeTransport），然后读写 Transport 并调用用户定义逻辑。

![](../asserts/images/incoming.png)

### 0x01.0 Listenable
Listenable 顾名思义，是对可监听地址的抽象。我们可以这么定义这个 trait（由于 bind 不是热路径，所以我们直接用了 async_trait）：
```rust
#[async_trait::async_trait]
pub trait Listenable {
    type Conn: AsyncRead + AsyncWrite + Send + Unpin + 'static;
    type Stream: Stream<Item = io::Result<Self::Conn>> + Unpin;

    async fn bind(&self) -> io::Result<Self::Stream>;
}
```

显然 SocketAddr 是可监听的，我们为 SocketAddr 实现这个接口：
```rust
#[async_trait::async_trait]
impl Listenable for SocketAddr {
    type Conn = TcpStream;
    type Stream = TcpListenerStream;

    async fn bind(&self) -> io::Result<Self::Stream> {
        let listener = tokio::net::TcpListener::bind(self).await?;
        Ok(TcpListenerStream::new(listener))
    }
}
```
同理，我们也可以为 PathBuf 实现这个接口，其对应的 Stream 是 UnixStream。

### 0x01.1 Incoming
Incoming 是指拨入连接的 Transport。它可以由 Listenable bind 后返回的 listen_stream 和 codec 构建得到。

Incoming 实现了 Stream，本质上是包装了 listen Stream 并对接收到的连接包装 codec 变成 transport。

```rust
#[pin_project::pin_project]
pub struct Incoming<LS, MC> {
    #[pin]
    listener_stream: LS,
    make_codec: MC,
}

impl<LS, MC> Incoming<LS, MC> {
    #[allow(unused)]
    pub fn new(listener_stream: LS, make_codec: MC) -> Self {
        Self {
            listener_stream,
            make_codec,
        }
    }
}

impl<LS, MC> Stream for Incoming<LS, MC>
where
    LS: TryStream,
    LS::Ok: AsyncRead + AsyncWrite,
    MC: MakeCodec,
{
    type Item = Result<Framed<LS::Ok, MC::Codec>, LS::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut listener_stream = this.listener_stream;
        let codec = this.make_codec;
        match ready!(listener_stream.as_mut().try_poll_next(cx)) {
            Some(Ok(conn)) => {
                let f = Framed::new(conn, codec.make_codec());
                Poll::Ready(Some(Ok(f)))
            }
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}
```

## 0x02 服务端逻辑
主要代码在 `server.rs` 和 `lib.rs` 中。

前面提到，服务端拿到 Transport 后，会拿 Item 并调用用户定义 Handler，等待 Handler 响应后将结果丢回去。

由于最内层是用户逻辑，所以为了之后实现各种中间件方便，我们比较好的方案是将用户 Handler 包装为 Service，并最终由 Server 去包装这个 Service，提供 serve 函数供用户使用。

![](../asserts/images/server.png)

## 0x02.0 从 IDL 生成 Trait
关于用户 Handler 的实现，我们在代码生成时肯定是要提供一个 Trait 的（生成代码）：
```rust
#[async_trait::async_trait]
pub trait ItemService {
    async fn get_user(
        &self,
        req: GetUserRequest,
        shuffle: bool,
    ) -> ApplicationResult<AnonymousItemServiceGetUserResult>;
}
```
之后用户需要实现这个 Trait 并将用户结构传入 ItemServiceServer 的 new 方法。

## 0x02.1 使用生成结构包装用户实现结构
然后包装为 ItemServiceServer（生成代码）：

这一步主要目的是包装成 Tower Service，便于在外面包装中间件处理。
```rust
pub struct ItemServiceServer<S> {
    inner: Arc<S>,
}

impl<S> ItemServiceServer<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl<S> Service<(MsgContext, ApplicationResult<AnonymousItemServiceRequest>)>
for ItemServiceServer<S>
where
    S: ItemService + Send + Sync + 'static,
{
    // Option since we may or may not return due to call or oneway
    type Response = Option<(MsgContext, ApplicationResult<AnonymousItemServiceResponse>)>;
    type Error = crate::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(
        &mut self,
        req: (MsgContext, ApplicationResult<AnonymousItemServiceRequest>),
    ) -> Self::Future {
        let inner = self.inner.clone();
        Box::pin(async move {
            let (mut cx, req) = req;
            match req {
                Ok(AnonymousItemServiceRequest::GetUser(r)) => {
                    let ret = inner.get_user(r.req, r.shuffle).await;
                    match ret {
                        Ok(r) => {
                            cx.identifier.message_type = TMessageType::Reply;
                            Ok(Some((cx, Ok(AnonymousItemServiceResponse::GetUser(r)))))
                        }
                        Err(e) => {
                            cx.identifier.message_type = TMessageType::Exception;
                            Ok(Some((cx, Err(e))))
                        }
                    }
                }
                Err(e) => {
                    log::error!("unexpected client error: {}", e);
                    Err(new_application_error(
                        ApplicationErrorKind::Unknown,
                        "unexpected client error",
                    ))
                }
            }
        })
    }
}
```

## 0x02.2 使用 Server 结构包装生成结构，提供 serve() 方法
之后再定义 Server 来包装 Service 并包装治理功能（治理功能由 tower 中间件提供）：

本层负责循环接收 transport，之后对每个 transport spawn 一个循环来接收数据、处理并在需要响应时发回响应。
```rust
#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("IO error")]
    IO(#[from] io::Error),
}

const DEFAULT_BUFFER: usize = 1e3 as usize; // FIXME
const DEFAULT_CONCURRENCY_LIMIT: usize = 1e3 as usize; // FIXME

pub struct Server<S, Addr, Req, Resp> {
    concurrency_limit: Option<usize>,
    buffer: Option<usize>,
    inner: S,
    _marker: PhantomData<fn(Addr, Req, Resp)>,
}

impl<S, Addr, Req, Resp> Server<S, Addr, Req, Resp>
where
    S: Service<
        (MsgContext, ApplicationResult<Req>),
        Response = Option<(MsgContext, ApplicationResult<Resp>)>,
        Error = crate::Error,
    > + Send
    + 'static,
    S::Future: Send,
{
    pub fn new(inner: S) -> Self {
        Self {
            concurrency_limit: None,
            buffer: None,
            inner,
            _marker: PhantomData,
        }
    }
}

impl<S, Addr, Req, Resp> Server<S, Addr, Req, Resp>
where
    Addr: Listenable,
    S: Service<
        (MsgContext, ApplicationResult<Req>),
        Response = Option<(MsgContext, ApplicationResult<Resp>)>,
        Error = crate::Error,
    > + Send
    + 'static,
    S::Future: Send,
    Req: Send + Message + 'static,
    Resp: Send + Message + 'static,
{
    pub async fn serve(self, addr: Addr) -> Result<(), ServerError> {
        let listen_stream = addr.bind().await?;
        let make_codec = DefaultMakeCodec::<Resp, Req>::new();
        let mut incoming = Incoming::new(listen_stream, make_codec);

        let buffer = self.buffer.unwrap_or(DEFAULT_BUFFER);
        let concurrency_limit = self.concurrency_limit.unwrap_or(DEFAULT_CONCURRENCY_LIMIT);
        let service = ServiceBuilder::new()
            .buffer(buffer)
            .concurrency_limit(concurrency_limit)
            .service(self.inner);

        loop {
            match incoming.try_next().await? {
                Some(mut ts) => {
                    let mut service = service.clone();
                    tokio::spawn(async move {
                        loop {
                            match ts.try_next().await {
                                Ok(Some(req)) => {
                                    let ready_service = match service.ready().await {
                                        Ok(svc) => svc,
                                        Err(e) => {
                                            log::error!("service not ready error: {:?}", e);
                                            return;
                                        }
                                    };

                                    let mut cx = req.0.clone();
                                    match ready_service.call(req).await {
                                        Ok(Some((cx, resp))) => {
                                            if let Err(e) = ts.send((cx, resp)).await {
                                                log::error!("send reply back error: {}", e);
                                                return;
                                            }
                                        }
                                        Ok(None) => {
                                            // oneway does not need response
                                        }
                                        Err(e) => {
                                            // if oneway, we just return
                                            if cx.identifier.message_type == TMessageType::OneWay {
                                                return;
                                            }
                                            // if not oneway, we must send the exception back
                                            cx.identifier.message_type = TMessageType::Exception;
                                            let app_error = ApplicationError::new(
                                                ApplicationErrorKind::Unknown,
                                                e.to_string(),
                                            );
                                            if let Err(e) = ts.send((cx, Err(app_error))).await {
                                                log::error!("send error back error: {}", e);
                                                return;
                                            }
                                        }
                                    }
                                }
                                Ok(None) => {
                                    return;
                                }
                                Err(e) => {
                                    // receive message error
                                    log::error!("error receiving message {}", e);
                                    return;
                                }
                            }
                        }
                    });
                }
                None => return Ok(()),
            }
        }
    }
}
```
不过目前 Server 还不能接收自定义的中间件，后续会通过 Builder 模式对外暴露相应接口。

## 0x03 客户端逻辑
主要代码在 `client.rs` 和 `lib.rs` 中。

客户端逻辑和服务端有两个不同之处：
1. Server 侧是用户实现 Handler，最后调用 serve 方法；而 Client 侧是用户直接调用生成方法，所以最外层一定是生成结构。
2. Server 侧链路是 接收->处理->发送；Client 侧链路是 发送->接收。

所以在服务端，我们使用通用 Server 实现包装用户 Handler；而在客户端，我们要用生成 Client 包装通用 Client(因为要实现 IDL 定义的方法)，通用 Client 包装 TransportClient。

![](../asserts/images/client.png)

## 0x03.0 Transport Client 实现（传输层抽象）

负责接收 `(MsgContext, ApplicationResult<Req>)` 并返回 `Option<(MsgContext, ApplicationResult<Resp>)>`（为什么有 Option？因为可能是 Oneway，不需要响应）。

同时它会维护 seq_id，在发送请求时做自增。

为什么这个 seq_id 不需要加锁或者 Atomic？因为这个 TransportClient 是在 Tower Buffer 层之后的，不是被并发调用的。

那么会不会有性能问题？答案是几乎不会。因为这些 Service 只是在拼装 Future，真正 Future 执行是并发的，只有拼装过程是串行的。
```rust
pub(crate) struct TransportClient<MCN, MCC> {
    make_transport: FramedMakeTransport<MCC, MCN>,
    seq_id: i32,
}

impl<MCN, MCC> TransportClient<MCN, MCC> {
    #[allow(unused)]
    pub fn new(make_connection: MCN, make_codec: MCC) -> Self {
        Self {
            make_transport: FramedMakeTransport::new(make_connection, make_codec),
            seq_id: 1,
        }
    }
}

/// Call with (MsgContext, ApplicationResult<Req>) returns
/// Option<(MsgContext, ApplicationResult<Resp>)>.
impl<MCN, MCC, Req, Resp> Service<(MsgContext, ApplicationResult<Req>)>
for TransportClient<MCN, MCC>
    where
        MCN: Service<SocketOrUnix> + Clone + Send + 'static,
        MCN::Response: AsyncRead + AsyncWrite + Unpin + Send,
        MCN::Error: std::error::Error + Send + Sync + 'static,
        MCN::Future: Send,
        Req: Send + 'static,
        Resp: Send,
        MCC: MakeCodec<
            EncodeItem = (MsgContext, ApplicationResult<Req>),
            DecodeItem = (MsgContext, ApplicationResult<Resp>),
            Error = crate::Error,
        >,
        MCC::Codec: Send + 'static,
        crate::Error: From<<MCN as tower::Service<SocketOrUnix>>::Error>,
{
    // If oneway, the Response is None
    type Response = Option<(MsgContext, ApplicationResult<Resp>)>;
    type Error = crate::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.make_transport.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, (mut cx, req): (MsgContext, ApplicationResult<Req>)) -> Self::Future {
        // TODO: use service discovery
        let target = cx
            .target
            .clone()
            .expect("unable to retrieve target from context");
        let transport_fut = self.make_transport.call(target);

        self.seq_id += 1;
        cx.identifier.sequence_number = self.seq_id;
        let oneway = cx.identifier.message_type == TMessageType::OneWay;
        Box::pin(async move {
            let mut transport = transport_fut.await?;
            transport.send((cx, req)).await?;
            if oneway {
                return Ok(None);
            }
            transport.try_next().await.map_err(Into::into)
        })
    }
}
```

## 0x03.1 通用 Client 实现

包装 Transport Client，提供 call 和 oneway 方法。在这一步会初始化 MsgContext。
```rust
#[derive(Clone)]
pub struct Client<S> {
    pub(crate) inner: S,
    target: SocketOrUnix,
}

impl<S> Client<S> {
    /// Call with method and Req and returns Result<Resp>
    pub async fn call<Req, Resp>(&mut self, method: &'static str, req: Req) -> crate::Result<Resp>
        where
            S: Service<
                (MsgContext, ApplicationResult<Req>),
                Response = Option<(MsgContext, ApplicationResult<Resp>)>,
                Error = tower::BoxError,
            >,
    {
        let context = MsgContext {
            identifier: TMessageIdentifier {
                name: method.to_string(),
                message_type: TMessageType::Call,

                sequence_number: 0,
            },
            target: Some(self.target.clone()),
        };
        let req = (context, Ok(req));
        // Option<(MsgContext, ApplicationResult<Resp>)>
        let resp = self.inner.ready().await?.call(req).await?;
        resp.expect("returning resp is expected")
            .1
            .map_err(Into::into)
    }

    pub async fn oneway<Req, Resp>(&mut self, method: &'static str, req: Req) -> crate::Result<()>
        where
            S: Service<
                (MsgContext, ApplicationResult<Req>),
                Response = Option<(MsgContext, ApplicationResult<Resp>)>,
                Error = crate::Error,
            >,
    {
        let context = MsgContext {
            identifier: TMessageIdentifier {
                name: method.to_string(),
                message_type: TMessageType::OneWay,

                sequence_number: 0,
            },
            target: Some(self.target.clone()),
        };
        let req = (context, Ok(req));
        self.inner.ready().await?.call(req).await?;
        Ok(())
    }
}
```

通用 Client 通过 ClientBuilder 生成：
```rust
pub struct ClientBuilder<MCC> {
    target: SocketOrUnix,
    make_codec: MCC,
}

impl<E, D> ClientBuilder<DefaultMakeCodec<E, D>> {
    pub fn new(target: SocketOrUnix) -> Self {
        Self {
            target,
            make_codec: DefaultMakeCodec::new(),
        }
    }
}

impl<MCC> ClientBuilder<MCC> {
    pub fn make_codec(self, make_codec: MCC) -> Self {
        Self {
            target: self.target,
            make_codec,
        }
    }
}

const DEFAULT_BUFFER: usize = usize::MAX >> 3;

impl<MCC, Req, Resp> ClientBuilder<MCC>
where
    Req: Send + 'static,
    Resp: Send,
    MCC: MakeCodec<
            EncodeItem = (MsgContext, ApplicationResult<Req>),
            DecodeItem = (MsgContext, ApplicationResult<Resp>),
            Error = crate::Error,
        > + Send
        + 'static,
    MCC::Codec: Send + 'static,
{
    pub fn build(
        self,
    ) -> Client<
        impl Service<
            (MsgContext, ApplicationResult<Req>),
            Response = Option<(MsgContext, ApplicationResult<Resp>)>,
            Error = tower::BoxError,
        >,
    > {
        let make_connection = DefaultMakeConnection;
        let make_codec = self.make_codec;
        let transport_client = TransportClient::new(make_connection, make_codec);
        let inner = BufferLayer::new(DEFAULT_BUFFER).layer(transport_client);
        Client {
            inner,
            target: self.target,
        }
    }
}
```

## 0x03.2 生成的 ItemServiceClient
基于上一小节的通用 Client 实现用户 IDL 定义的调用方法。

```rust
pub struct ItemServiceClient<S> {
    inner_client: Client<S>,
}

impl<S> ItemServiceClient<S> {
    pub fn new(inner: Client<S>) -> Self {
        Self {
            inner_client: inner,
        }
    }
}

impl<S> ItemServiceClient<S>
where
    S: Service<
        (MsgContext, ApplicationResult<AnonymousItemServiceRequest>),
        Response = Option<(MsgContext, ApplicationResult<AnonymousItemServiceResponse>)>,
        Error = tower::BoxError,
    >,
{
    pub async fn get_user(
        &mut self,
        req: GetUserRequest,
        shuffle: bool,
    ) -> Result<AnonymousItemServiceGetUserResult> {
        let anonymous_request =
            AnonymousItemServiceRequest::GetUser(AnonymousItemServiceGetUserArgs { req, shuffle });
        let resp = self.inner_client.call("GetUser", anonymous_request).await?;

        #[allow(irrefutable_let_patterns)]
        if let AnonymousItemServiceResponse::GetUser(r) = resp {
            return Ok(r);
        }
        Err(new_application_error(
            ApplicationErrorKind::Unknown,
            "unable to get response",
        ))
    }
}
```

## 0x04 新增 client 和 server 的 example
首先创建 examples 文件夹，分别创建 `client.rs` 和 `server.rs`。之后在 `Cargo.toml` 中为 example 添加配置：
```toml
[[example]]
name = "client"
crate-type = ["bin"]

[[example]]
name = "server"
crate-type = ["bin"]
```

在 `examples/server.rs` 中写样例实现：
```rust
use mini_lust_chap5::{ItemService, GetUserRequest, AnonymousItemServiceGetUserResult, ApplicationResult, GetUserResponse, User, ItemServiceServer, Server};
use std::net::SocketAddr;

struct Svc;

#[async_trait::async_trait]
impl ItemService for Svc {
    async fn get_user(
        &self,
        req: GetUserRequest,
        shuffle: bool,
    ) -> ApplicationResult<AnonymousItemServiceGetUserResult> {
        log::info!("receive a get_user request: req = {:?}, shuffle = {:?}", req, shuffle);

        let mut resp = GetUserResponse::default();
        resp.users.push(User {
            user_id: req.user_id,
            user_name: req.user_name,
            is_male: shuffle,
            extra: None,
        });
        Ok(AnonymousItemServiceGetUserResult::Success(resp))
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let server = Server::new(ItemServiceServer::new(Svc));
    let addr = "127.0.0.1:12345".parse::<SocketAddr>().unwrap();

    log::info!("Will serve on 127.0.0.1:12345");
    let _ = server.serve(addr).await;
}
```

Client 部分(`examples/client.rs`)：
```rust
use mini_lust_chap5::{GetUserRequest, SocketOrUnix, ClientBuilder, ItemServiceClient};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let target = SocketOrUnix::Socket("127.0.0.1:12345".parse().unwrap());
    let mut client = ItemServiceClientBuilder::new(target).build();

    let resp = client
        .get_user(
            GetUserRequest {
                user_id: 1,
                user_name: "ihciah".to_string(),
                is_male: false,
            },
            true,
        )
        .await
        .unwrap();
    log::info!("{:?}", resp);
}
```

## 0x05 一点优化
1. 为一些小函数和 Message 的实现标记了 inline，为了尽可能使这些函数内联来提升效率。
2. 在 Binary Protocol 实现中新增了长度检查，在遇到异常包时能够返回 ProtocolError 而不是 panic。
   由于绝大多数情况下数据是正确的，我们的长度检查是成功，为了告诉编译器去优化特定的分支我们可以使用 [likely](https://doc.rust-lang.org/std/intrinsics/fn.likely.html) 。
   不过这个是不稳定特性，所以我们定义了一个叫 unstable 的 feature，当该 feature 开启时我们会应用这个优化。
   
   注意：这个 feature 开启后只能用 nightly 版本编译。
