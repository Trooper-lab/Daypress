//! # Ports Layer (Hexagonal Interfaces)
//!
//! * [`input`]: Driving use-case traits (`ContentUseCase`, `MigrationUseCase`, `SettingsUseCase`).
//! * [`output`]: Driven repository traits (`PostRepository`, `TaxonomyRepository`, `OptionRepository`, `CommentRepository`, `MigrationRepository`).

pub mod input;
pub mod output;

pub use input::*;
pub use output::*;
