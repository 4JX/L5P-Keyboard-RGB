use crate::{bail, bytes_codec::BytesCodec, ResultType};
use bytes::{BufMut, Bytes, BytesMut};
use futures::{SinkExt, StreamExt};
use protobuf::Message;
use sodiumoxide::crypto::secretbox::{self, Key, Nonce};
use std::{
    io::{self, Error, ErrorKind},
    net::SocketAddr,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{lookup_host, TcpListener, TcpSocket, ToSocketAddrs},
};
use tokio_socks::{tcp::Socks5Stream, IntoTargetAddr, ToProxyAddrs};
use tokio_util::codec::Framed;

pub trait TcpStreamTrait: AsyncRead + AsyncWrite + Unpin {}
pub struct DynTcpStream(Box<dyn TcpStreamTrait + Send + Sync>);

pub struct FramedStream(
    Framed<DynTcpStream, BytesCodec>,
    SocketAddr,
    Option<(Key, u64, u64)>,
    u64,
);

impl Deref for FramedStream {
    type Target = Framed<DynTcpStream, BytesCodec>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FramedStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for DynTcpStream {
    type Target = Box<dyn TcpStreamTrait + Send + Sync>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DynTcpStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn new_socket(addr: std::net::SocketAddr, reuse: bool) -> Result<TcpSocket, std::io::Error> {
    let socket = match addr {
        std::net::SocketAddr::V4(..) => TcpSocket::new_v4()?,
        std::net::SocketAddr::V6(..) => TcpSocket::new_v6()?,
    };
    if reuse {
        // windows has no reuse_port, but it's reuse_address
        // almost equals to unix's reuse_port + reuse_address,
        // though may introduce nondeterministic behavior
        #[cfg(unix)]
        socket.set_reuseport(true)?;
        socket.set_reuseaddr(true)?;
    }
    socket.bind(addr)?;
    Ok(socket)
}

impl FramedStream {
    pub async fn new<T1: ToSocketAddrs, T2: ToSocketAddrs>(
        remote_addr: T1,
        local_addr: T2,
        ms_timeout: u64,
    ) -> ResultType<Self> {
        for local_addr in lookup_host(&local_addr).await? {
            for remote_addr in lookup_host(&remote_addr).await? {
                let stream = super::timeout(
                    ms_timeout,
                    new_socket(local_addr, true)?.connect(remote_addr),
                )
                .await??;
                stream.set_nodelay(true).ok();
                let addr = stream.local_addr()?;
                return Ok(Self(
                    Framed::new(DynTcpStream(Box::new(stream)), BytesCodec::new()),
                    addr,
                    None,
                    0,
                ));
            }
        }
        bail!("could not resolve to any address");
    }

    pub async fn connect<'a, 't, P, T1, T2>(
        proxy: P,
        target: T1,
        local: T2,
        username: &'a str,
        password: &'a str,
        ms_timeout: u64,
    ) -> ResultType<Self>
    where
        P: ToProxyAddrs,
        T1: IntoTargetAddr<'t>,
        T2: ToSocketAddrs,
    {
        if let Some(local) = lookup_host(&local).await?.next() {
            if let Some(proxy) = proxy.to_proxy_addrs().next().await {
                let stream =
                    super::timeout(ms_timeout, new_socket(local, true)?.connect(proxy?)).await??;
                stream.set_nodelay(true).ok();
                let stream = if username.trim().is_empty() {
                    super::timeout(
                        ms_timeout,
                        Socks5Stream::connect_with_socket(stream, target),
                    )
                    .await??
                } else {
                    super::timeout(
                        ms_timeout,
                        Socks5Stream::connect_with_password_and_socket(
                            stream, target, username, password,
                        ),
                    )
                    .await??
                };
                let addr = stream.local_addr()?;
                return Ok(Self(
                    Framed::new(DynTcpStream(Box::new(stream)), BytesCodec::new()),
                    addr,
                    None,
                    0,
                ));
            };
        };
        bail!("could not resolve to any address");
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.1
    }

    pub fn set_send_timeout(&mut self, ms: u64) {
        self.3 = ms;
    }

    pub fn from(stream: impl TcpStreamTrait + Send + Sync + 'static, addr: SocketAddr) -> Self {
        Self(
            Framed::new(DynTcpStream(Box::new(stream)), BytesCodec::new()),
            addr,
            None,
            0,
        )
    }

    pub fn set_raw(&mut self) {
        self.0.codec_mut().set_raw();
        self.2 = None;
    }

    pub fn is_secured(&self) -> bool {
        self.2.is_some()
    }

    #[inline]
    pub async fn send(&mut self, msg: &impl Message) -> ResultType<()> {
        self.send_raw(msg.write_to_bytes()?).await
    }

    #[inline]
    pub async fn send_raw(&mut self, msg: Vec<u8>) -> ResultType<()> {
        let mut msg = msg;
        if let Some(key) = self.2.as_mut() {
            key.1 += 1;
            let nonce = Self::get_nonce(key.1);
            msg = secretbox::seal(&msg, &nonce, &key.0);
        }
        self.send_bytes(bytes::Bytes::from(msg)).await?;
        Ok(())
    }

    #[inline]
    pub async fn send_bytes(&mut self, bytes: Bytes) -> ResultType<()> {
        if self.3 > 0 {
            super::timeout(self.3, self.0.send(bytes)).await??;
        } else {
            self.0.send(bytes).await?;
        }
        Ok(())
    }

    #[inline]
    pub async fn next(&mut self) -> Option<Result<BytesMut, Error>> {
        let mut res = self.0.next().await;
        if let Some(key) = self.2.as_mut() {
            if let Some(Ok(bytes)) = res.as_mut() {
                key.2 += 1;
                let nonce = Self::get_nonce(key.2);
                match secretbox::open(&bytes, &nonce, &key.0) {
                    Ok(res) => {
                        bytes.clear();
                        bytes.put_slice(&res);
                    }
                    Err(()) => {
                        return Some(Err(Error::new(ErrorKind::Other, "decryption error")));
                    }
                }
            }
        }
        res
    }

    #[inline]
    pub async fn next_timeout(&mut self, ms: u64) -> Option<Result<BytesMut, Error>> {
        if let Ok(res) = super::timeout(ms, self.next()).await {
            res
        } else {
            None
        }
    }

    pub fn set_key(&mut self, key: Key) {
        self.2 = Some((key, 0, 0));
    }

    fn get_nonce(seqnum: u64) -> Nonce {
        let mut nonce = Nonce([0u8; secretbox::NONCEBYTES]);
        nonce.0[..std::mem::size_of_val(&seqnum)].copy_from_slice(&seqnum.to_le_bytes());
        nonce
    }
}

const DEFAULT_BACKLOG: u32 = 128;

#[allow(clippy::never_loop)]
pub async fn new_listener<T: ToSocketAddrs>(addr: T, reuse: bool) -> ResultType<TcpListener> {
    if !reuse {
        Ok(TcpListener::bind(addr).await?)
    } else {
        for addr in lookup_host(&addr).await? {
            let socket = new_socket(addr, true)?;
            return Ok(socket.listen(DEFAULT_BACKLOG)?);
        }
        bail!("could not resolve to any address");
    }
}

impl Unpin for DynTcpStream {}

impl AsyncRead for DynTcpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        AsyncRead::poll_read(Pin::new(&mut self.0), cx, buf)
    }
}

impl AsyncWrite for DynTcpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        AsyncWrite::poll_write(Pin::new(&mut self.0), cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.0), cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        AsyncWrite::poll_shutdown(Pin::new(&mut self.0), cx)
    }
}

impl<R: AsyncRead + AsyncWrite + Unpin> TcpStreamTrait for R {}
