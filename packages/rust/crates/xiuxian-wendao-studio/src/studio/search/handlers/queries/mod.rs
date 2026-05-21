//! Coordinates the Studio search handlers queries branch and keeps its child modules behind one documented reasoning-tree boundary.

mod ast;
mod attachment;
mod global;
mod reference;
mod symbol;

pub use self::ast::AstSearchQuery;
pub use self::attachment::AttachmentSearchQuery;
#[cfg(test)]
pub use self::global::SearchQuery;
pub use self::reference::ReferenceSearchQuery;
pub use self::symbol::SymbolSearchQuery;
