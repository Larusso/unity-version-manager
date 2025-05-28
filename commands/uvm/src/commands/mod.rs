pub mod detect;
pub mod launch;
pub mod list;
pub mod install;
pub mod uninstall;
pub mod versions;
pub mod external;
mod error;

pub type Result<T> = std::result::Result<T, error::CommandError>;

pub trait Command {
    fn execute(&self) -> Result<()>;
}