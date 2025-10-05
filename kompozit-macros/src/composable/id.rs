use super::NAMESPACE;

pub(super) struct Id<N: AsRef<str>> {
    pub name: N,
    pub span: proc_macro2::Span,
}

impl<N: AsRef<str>> Id<N> {
    /// global type ident
    pub(super) fn gt(&self) -> proc_macro2::Ident {
        quote::format_ident!(
            "{NAMESPACE}{}",
            heck::ToPascalCase::to_pascal_case(self.name.as_ref()),
            span = self.span
        )
    }

    /// local type ident
    pub(super) fn lt(&self) -> proc_macro2::Ident {
        quote::format_ident!(
            "{}",
            heck::ToPascalCase::to_pascal_case(self.name.as_ref()),
            span = self.span
        )
    }

    /// global value ident
    pub(super) fn gv(&self) -> proc_macro2::Ident {
        quote::format_ident!(
            "{NAMESPACE}{}",
            heck::ToSnakeCase::to_snake_case(self.name.as_ref()),
            span = self.span
        )
    }

    /// local value ident
    pub(super) fn lv(&self) -> proc_macro2::Ident {
        quote::format_ident!(
            "{}",
            heck::ToSnakeCase::to_snake_case(self.name.as_ref()),
            span = self.span
        )
    }
}
