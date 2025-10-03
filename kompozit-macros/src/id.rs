use crate::NAMESPACE;

pub struct EntityId<N: AsRef<str>> {
    pub name: N,
    pub span: proc_macro2::Span,
}

impl<N: AsRef<str>> EntityId<N> {
    /// global type ident
    pub fn gt(&self) -> proc_macro2::Ident {
        quote::format_ident!(
            "{NAMESPACE}{}",
            heck::ToPascalCase::to_pascal_case(self.name.as_ref()),
            span = self.span
        )
    }

    /// local type ident
    pub fn lt(&self) -> proc_macro2::Ident {
        quote::format_ident!(
            "{}",
            heck::ToPascalCase::to_pascal_case(self.name.as_ref()),
            span = self.span
        )
    }

    /// global value ident
    pub fn gv(&self) -> proc_macro2::Ident {
        quote::format_ident!(
            "{NAMESPACE}{}",
            heck::ToSnakeCase::to_snake_case(self.name.as_ref()),
            span = self.span
        )
    }

    /// local value ident
    pub fn lv(&self) -> proc_macro2::Ident {
        quote::format_ident!(
            "{}",
            heck::ToSnakeCase::to_snake_case(self.name.as_ref()),
            span = self.span
        )
    }
}
