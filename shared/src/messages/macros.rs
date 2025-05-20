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
            impl From<$variant_type> for $name {
                fn from(value: $variant_type) -> $name {
                    $name::$variant(value)
                }
            }
        )*)*
    }
}

pub(crate) use impl_into_message;
