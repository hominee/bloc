#[macro_use]
extern crate lazy_static;

pub mod block;
pub mod blockchain;
pub mod constant;
pub mod secp256k1;
pub mod transaction;
pub mod web;

fn main() {
    web::app::run_app();
}
