// TODO: re-export macro from macros sub-crate

pub use macros::TypeEnum;

/// Helper trait to unpack values from a type that used the TypeEnum derive
pub trait TypeEnum<T> {
    /// If the enum is holding a value of type T, return a reference to it.
    fn value(&self) -> Option<&T>;
    /// If the enum is holding a value of type T, return a mutable reference to it.
    fn value_mut(&mut self) -> Option<&mut T>;
    /// If the enum is holding a value of type T, unwrap the enum and return the value.
    /// If not, return the enum unmodified
    fn into_value(self) -> Result<T, Self>
    where
        Self: Sized;
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, PartialEq, TypeEnum)]
    enum ExampleDerive {
        Number(i64),
        String(String),
        Tuple(u8, u8),
    }

    #[test]
    fn test_from_conversions() {
        let string: ExampleDerive = "foo".to_string().into();
        let number: ExampleDerive = 32i64.into();
        let tuple: ExampleDerive = (0u8, 1u8).into();

        // Just check that conversions work
        assert!(matches!(string, ExampleDerive::String(_)));
        assert!(matches!(number, ExampleDerive::Number(_)));
        assert!(matches!(tuple, ExampleDerive::Tuple(_, _)));
    }

    #[test]
    fn test_value_method() {
        let string_enum: ExampleDerive = "foo".to_string().into();
        let number_enum: ExampleDerive = 42i64.into();
        let tuple_enum: ExampleDerive = (10u8, 20u8).into();

        // Test successful extractions
        assert_eq!(string_enum.value(), Some(&"foo".to_string()));
        assert_eq!(number_enum.value(), Some(&42i64));
        // Note: tuple variants can't return references, so this returns None
        let tuple_ref: Option<&(u8, u8)> = tuple_enum.value();
        assert_eq!(tuple_ref, None);

        // Test failed extractions (wrong type)
        let wrong_i64: Option<&i64> = string_enum.value();
        assert_eq!(wrong_i64, None);
        let wrong_string: Option<&String> = number_enum.value();
        assert_eq!(wrong_string, None);
        let wrong_string_tuple: Option<&String> = tuple_enum.value();
        assert_eq!(wrong_string_tuple, None);
    }

    #[test]
    fn test_value_mut_method() {
        let mut string_enum: ExampleDerive = "foo".to_string().into();
        let mut number_enum: ExampleDerive = 42i64.into();
        let mut tuple_enum: ExampleDerive = (10u8, 20u8).into();

        // Test successful mutable extractions and modifications
        if let Some(s) = <ExampleDerive as TypeEnum<String>>::value_mut(&mut string_enum) {
            s.push_str("bar");
        }
        assert_eq!(string_enum.value(), Some(&"foobar".to_string()));

        if let Some(n) = <ExampleDerive as TypeEnum<i64>>::value_mut(&mut number_enum) {
            *n += 10;
        }
        assert_eq!(number_enum.value(), Some(&52i64));

        // Note: tuple variants can't return mutable references, so this returns None
        let tuple_mut: Option<&mut (u8, u8)> = tuple_enum.value_mut();
        if let Some(t) = tuple_mut {
            t.0 += 5;
            t.1 += 5;
        }
        let tuple_check: Option<&(u8, u8)> = tuple_enum.value();
        assert_eq!(tuple_check, None);

        // Test failed mutable extractions (wrong type)
        let wrong_i64_mut: Option<&mut i64> = string_enum.value_mut();
        assert_eq!(wrong_i64_mut, None);
        let wrong_string_mut: Option<&mut String> = number_enum.value_mut();
        assert_eq!(wrong_string_mut, None);
        let wrong_string_tuple_mut: Option<&mut String> = tuple_enum.value_mut();
        assert_eq!(wrong_string_tuple_mut, None);
    }

    #[test]
    fn test_into_value_method() {
        let string_enum: ExampleDerive = "hello".to_string().into();
        let number_enum: ExampleDerive = 123i64.into();
        let tuple_enum: ExampleDerive = (5u8, 10u8).into();

        // Test successful unwrapping
        assert_eq!(string_enum.into_value(), Ok("hello".to_string()));
        assert_eq!(number_enum.into_value(), Ok(123i64));
        assert_eq!(tuple_enum.into_value(), Ok((5u8, 10u8)));

        // Test failed unwrapping (wrong type) - should return the original enum
        let wrong_type_enum: ExampleDerive = "test".to_string().into();
        let result: Result<i64, ExampleDerive> = wrong_type_enum.into_value();
        match result {
            Err(ExampleDerive::String(s)) => assert_eq!(s, "test"),
            _ => panic!("Expected Err with original enum"),
        }
    }
}
