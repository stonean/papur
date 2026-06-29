//! Role resolution (spec 002).
//!
//! Resolves a parsed [`RoleRef`]'s namespace against a [`RoleRegistry`]. Spec
//! 002 owns the resolution *algorithm*; the registry's population is downstream
//! work — local definitions come from same-document layers and the global set
//! arrives with theming (spec 004) and the CSS layer (spec 005). Until those
//! land the global set is empty, so a forced `g.foo` resolves only against an
//! injected registry; tests drive `resolve` with one directly.
//!
//! [`Namespace`] and [`RoleRef`] are parsed in the [`attr`](crate::attr) module
//! and re-exported here for callers working at the resolution layer.

pub use crate::attr::{Namespace, RoleRef};

use crate::block::ParseMode;
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::span::Span;

/// The scope a role resolved in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Defined in the document's local scope.
    Local,
    /// Defined in the project-wide global scope.
    Global,
}

/// The outcome of resolving a [`RoleRef`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Matched a definition in the named scope.
    Resolved(Scope),
    /// An `Auto` role matched nowhere — the class is emitted verbatim; not an
    /// error (plain-CSS classes with no registered role are valid).
    Unresolved,
    /// A forced prefix (`g.`/`l.`) could not be satisfied in its scope. In
    /// strict mode this is accompanied by a `P023` diagnostic; in lenient mode
    /// the caller emits the class unresolved and records a warning.
    ForcedMiss,
}

/// The set of defined roles a resolution runs against. 002 owns the algorithm;
/// implementors supply the data (local from same-document layers, global from
/// theming / the CSS layer).
pub trait RoleRegistry {
    /// Whether `name` is defined in local scope.
    fn has_local(&self, name: &str) -> bool;
    /// Whether `name` is defined in global scope.
    fn has_global(&self, name: &str) -> bool;
}

/// Resolve one role reference against `registry`.
///
/// - `Auto`: local-first, then global; an unresolved `Auto` is **not** an error.
/// - `Global` / `Local`: forced; a miss yields [`Resolution::ForcedMiss`] plus a
///   `P023` diagnostic in strict mode (none in lenient mode).
///
/// `span` locates the role token in the source so the diagnostic can point at
/// it.
pub fn resolve(
    role: &RoleRef,
    registry: &dyn RoleRegistry,
    mode: ParseMode,
    span: Span,
) -> (Resolution, Option<Diagnostic>) {
    match role.namespace {
        Namespace::Auto => {
            if registry.has_local(&role.name) {
                (Resolution::Resolved(Scope::Local), None)
            } else if registry.has_global(&role.name) {
                (Resolution::Resolved(Scope::Global), None)
            } else {
                (Resolution::Unresolved, None)
            }
        }
        Namespace::Local => {
            if registry.has_local(&role.name) {
                (Resolution::Resolved(Scope::Local), None)
            } else {
                (Resolution::ForcedMiss, forced_miss_diag(role, mode, span))
            }
        }
        Namespace::Global => {
            if registry.has_global(&role.name) {
                (Resolution::Resolved(Scope::Global), None)
            } else {
                (Resolution::ForcedMiss, forced_miss_diag(role, mode, span))
            }
        }
    }
}

/// Build the `P023` diagnostic for a forced-prefix miss (strict mode only).
fn forced_miss_diag(role: &RoleRef, mode: ParseMode, span: Span) -> Option<Diagnostic> {
    if mode != ParseMode::Strict {
        return None;
    }
    let prefix = match role.namespace {
        Namespace::Global => "g",
        Namespace::Local => "l",
        Namespace::Auto => "",
    };
    Some(Diagnostic::new(
        DiagnosticCode::UnresolvedForcedRole,
        format!(
            "forced role `{prefix}.{}` resolved to no definition",
            role.name
        ),
        span,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Reg {
        local: &'static [&'static str],
        global: &'static [&'static str],
    }

    impl RoleRegistry for Reg {
        fn has_local(&self, name: &str) -> bool {
            self.local.contains(&name)
        }
        fn has_global(&self, name: &str) -> bool {
            self.global.contains(&name)
        }
    }

    fn span() -> Span {
        Span {
            start_line: 1,
            start_col: 1,
            start_byte: 0,
            end_byte: 0,
        }
    }

    fn role(namespace: Namespace, name: &str) -> RoleRef {
        RoleRef {
            namespace,
            name: name.into(),
        }
    }

    /// `card` is local-only; `btn` is global-only.
    fn reg() -> Reg {
        Reg {
            local: &["card"],
            global: &["btn"],
        }
    }

    #[test]
    fn auto_prefers_local() {
        let both = Reg {
            local: &["x"],
            global: &["x"],
        };
        let (res, diag) = resolve(
            &role(Namespace::Auto, "x"),
            &both,
            ParseMode::Strict,
            span(),
        );
        assert_eq!(res, Resolution::Resolved(Scope::Local));
        assert!(diag.is_none());
    }

    #[test]
    fn auto_falls_through_to_global() {
        let (res, _) = resolve(
            &role(Namespace::Auto, "btn"),
            &reg(),
            ParseMode::Strict,
            span(),
        );
        assert_eq!(res, Resolution::Resolved(Scope::Global));
    }

    #[test]
    fn auto_unresolved_is_not_an_error() {
        let (res, diag) = resolve(
            &role(Namespace::Auto, "nope"),
            &reg(),
            ParseMode::Strict,
            span(),
        );
        assert_eq!(res, Resolution::Unresolved);
        assert!(diag.is_none());
    }

    #[test]
    fn forced_global_hit() {
        let (res, diag) = resolve(
            &role(Namespace::Global, "btn"),
            &reg(),
            ParseMode::Strict,
            span(),
        );
        assert_eq!(res, Resolution::Resolved(Scope::Global));
        assert!(diag.is_none());
    }

    #[test]
    fn forced_local_hit() {
        let (res, _) = resolve(
            &role(Namespace::Local, "card"),
            &reg(),
            ParseMode::Strict,
            span(),
        );
        assert_eq!(res, Resolution::Resolved(Scope::Local));
    }

    #[test]
    fn forced_global_miss_strict_is_p023() {
        // `card` is local-only, so forcing global misses.
        let (res, diag) = resolve(
            &role(Namespace::Global, "card"),
            &reg(),
            ParseMode::Strict,
            span(),
        );
        assert_eq!(res, Resolution::ForcedMiss);
        assert_eq!(diag.unwrap().code.code(), "PAPUR-P023");
    }

    #[test]
    fn forced_local_miss_lenient_has_no_diag() {
        // `btn` is global-only, so forcing local misses; lenient stays quiet.
        let (res, diag) = resolve(
            &role(Namespace::Local, "btn"),
            &reg(),
            ParseMode::Lenient,
            span(),
        );
        assert_eq!(res, Resolution::ForcedMiss);
        assert!(diag.is_none());
    }
}
