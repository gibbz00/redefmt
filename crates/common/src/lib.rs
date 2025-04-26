//! redefmt commons.

#![no_std]
// TEMP:
#![allow(missing_docs)]

mod pointer_width;
pub(crate) use pointer_width::PointerWidth;

mod frontmatter;
pub use frontmatter::FrontMatter;
