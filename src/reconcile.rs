//! Reconciliation engine.
//!
//! Compares the desired managed-block content (from the rio source repo) with
//! the current file state, detects drift, and orchestrates the interactive
//! promote / ignore / delete workflow (v0.6).
