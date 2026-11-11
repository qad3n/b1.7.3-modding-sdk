// builder steps that store one value in one spec field, the field name is why this is a macro
macro_rules! setters
{
    ($($(#[$meta:meta])* $name:ident($kind:ty) => $field:ident;)*) =>
    {
        $(
            $(#[$meta])*
            pub fn $name(mut self, value: $kind) -> Self
            {
                self.m_spec.$field = Some(value.into());
                self
            }
        )*
    };
}
