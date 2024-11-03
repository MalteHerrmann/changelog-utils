// This is solved using the Marco implementation to be able to dynamically add options to the `ReleaseType` enum
// while also generating the corresponding function `ReleaseType::all`, which returns all options.
//
// TODO: check if this can be done less complicated
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
