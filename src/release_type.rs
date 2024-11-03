macro_rules! release_type {
    ($($name:ident),*) => {
        #[derive(Debug, Clone)]
        pub enum ReleaseType {
            $($name),*
        }

        impl ReleaseType {
            pub fn all() -> Vec<ReleaseType> {
                vec![$(ReleaseType::$name),*]
            }

            pub fn as_str(&self) -> &'static str {
                match self {
                    $(ReleaseType::$name => stringify!($name),)*
                }
            }
        }
    };
}

// Define the ReleaseType enum using the macro
release_type!(Major, Minor, Patch, RcMajor, RcMinor, RcPatch);

#[cfg(test)]
mod tests {
    use super::ReleaseType;

    #[test]
    fn test_all() {
        assert_eq!(ReleaseType::all().len(), 6);
    }
}
