#[macro_use]
pub mod video;

pub mod io;
pub mod pic;
pub mod keyboard;

pub fn configure() {
    pic::MASTER.init(0x20, true);
    pic::SLAVE.init(0x28, false);

}
