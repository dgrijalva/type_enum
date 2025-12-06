// TODO: re-export macro from macros sub-crate

/// Helper trait to unpack values from a type that used the TypeEnum derive
pub trait TypeEnum {
    /// If the enum is holding a value of type T, return a reference to it.
    fn value<T>(&self) -> Option<&T>;
    /// If the enum is holding a value of type T, return a mutable reference to it.
    fn value_mut<T>(&mut self) -> Option<&mut T>;
    /// If the enum is holding a value of type T, unwrap the enum and return the value.
    /// If not, return the enum unmodified
    fn into_value<T>(self) -> Result<T, Self>
    where
        Self: Sized;
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(TypeEnum)]
    enum ExampleDerive {
        Number(i64),
        String(String),
        Tuple(u8, u8),
    }

    #[test]
    fn test_example() {
        let string: ExampleDerive = "foo".to_string().into();
        let number: ExampleDerive = 32i64.into();
        let tuple: ExampleDerive = (0, 1).into();
    }
}
