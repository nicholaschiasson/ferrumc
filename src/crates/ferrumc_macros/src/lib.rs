#![feature(proc_macro_quote)]
extern crate proc_macro;

use proc_macro::TokenStream;

mod decode;
mod ecs;
mod encode;
mod nbt_decode;
mod packet;
mod profile;
mod utils;

#[proc_macro_derive(NetDecode)]
pub fn decode_derive(input: TokenStream) -> TokenStream {
    decode::derive(input)
}

#[proc_macro_derive(NetEncode, attributes(encode))]
pub fn encode_derive(input: TokenStream) -> TokenStream {
    encode::derive(input)
}

#[proc_macro_attribute]
pub fn packet(args: TokenStream, input: TokenStream) -> TokenStream {
    packet::attribute(args, input)
}

#[proc_macro_attribute]
pub fn profile(args: TokenStream, input: TokenStream) -> TokenStream {
    profile::profile_fn(args, input)
}

#[proc_macro]
pub fn bake_packet_registry(input: TokenStream) -> TokenStream {
    packet::bake(input)
}

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    ecs::derive_component(input)
}
#[proc_macro_derive(Constructor)]
pub fn derive_constructor(input: TokenStream) -> TokenStream {
    ecs::derive_constructor(input)
}

#[proc_macro_derive(AutoGenName)]
pub fn derive_name(input: TokenStream) -> TokenStream {
    utils::derive_name(input)
}

#[proc_macro_derive(Getter)]
pub fn derive_getter(input: TokenStream) -> TokenStream {
    utils::derive_getter(input)
}
