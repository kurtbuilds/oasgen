use oasgen::{oasgen, OaSchema, Server};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

#[derive(Deserialize, Serialize, OaSchema)]
pub struct Addresses {
    pub ip: IpAddr,
    pub ip6: Ipv6Addr,
    pub ip4: Ipv4Addr,
    pub sock: SocketAddr,
}

#[derive(Serialize, OaSchema)]
pub struct AddressesResponse {
    pub ok: bool,
}

#[derive(OaSchema, Serialize, Deserialize)]
pub struct Foo {
    #[oasgen(inline)]
    addresses: Addresses,
}

#[oasgen]
async fn addresses(_body: Addresses) -> AddressesResponse {
    AddressesResponse { ok: false }
}

fn main() {
    use pretty_assertions::assert_eq;

    let _ = Server::none().get("/hello", addresses);

    let schema = Foo::schema();
    let spec = serde_yaml::to_string(&schema).unwrap();
    assert_eq!(spec, include_str!("07-ipaddr.yaml"));
}
