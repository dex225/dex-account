use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

#[derive(Clone, Copy)]
pub struct ClientIp(pub std::net::IpAddr);

pub async fn client_ip_middleware(
    State(_state): State<()>,
    mut req: Request,
    next: Next,
) -> Response {
    let ip = extract_ip_from_request(&req);
    req.extensions_mut().insert(ClientIp(ip));
    next.run(req).await
}

fn extract_ip_from_request(req: &Request) -> std::net::IpAddr {
    if let Some(addr) = req.extensions().get::<std::net::SocketAddr>() {
        return addr.ip();
    }

    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip_str) = forwarded_str.split(',').next() {
                if let Ok(ip) = ip_str.trim().parse() {
                    return ip;
                }
            }
        }
    }

    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse() {
                return ip;
            }
        }
    }

    std::net::IpAddr::from([127, 0, 0, 1])
}
