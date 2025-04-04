/// Macro to auto implement message into enum variant
macro_rules! impl_into_message{
    {
        #[derive($($attr:tt,)*)]
        $vis:vis enum $name:ident {
            $($variant:ident$(($variant_type:ty))*,)*
        }
    } => {
        #[derive($($attr,)*)]
        $vis enum $name {
            $($variant$(($variant_type))*,)*
        }

        $($(
            impl Into<$name> for $variant_type {
                fn into(self) -> $name {
                    $name::$variant(self)
                }
            }
        )*)*
    }
}

pub(crate) use impl_into_message;
