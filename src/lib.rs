// Re-export derive macro from `macros` sub-crate
pub use macros::TypeEnum;

/// Trait for extracting immutable references from enum variants
///
/// For single field variants like `Variant(String)`, implement `Value<'a, &'a String>`
/// For multi-field variants like `Variant(u8, u8)`, implement `Value<'a, (&'a u8, &'a u8)>`
pub trait Value<'a, T> {
    /// If the enum is holding a value of the matching type, return a reference to it.
    fn value(&'a self) -> Option<T>;
}

/// Trait for extracting mutable references from enum variants
///
/// For single field variants like `Variant(String)`, implement `ValueMut<'a, &'a mut String>`
/// For multi-field variants like `Variant(u8, u8)`, implement `ValueMut<'a, (&'a mut u8, &'a mut u8)>`
pub trait ValueMut<'a, T> {
    /// If the enum is holding a value of the matching type, return a mutable reference to it.
    fn value_mut(&'a mut self) -> Option<T>;
}

/// Trait for extracting owned values by consuming the enum
///
/// For single field variants like `Variant(String)`, implement `IntoValue<String>`
/// For multi-field variants like `Variant(u8, u8)`, implement `IntoValue<(u8, u8)>`
pub trait IntoValue<T> {
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

        // Test successful extractions with new trait design
        let string_ref: Option<&String> = string_enum.value();
        assert_eq!(string_ref, Some(&"foo".to_string()));

        let number_ref: Option<&i64> = number_enum.value();
        assert_eq!(number_ref, Some(&42i64));

        // Test tuple-of-references - this now works with lifetime parameter!
        let tuple_refs: Option<(&u8, &u8)> = tuple_enum.value();
        assert_eq!(tuple_refs, Some((&10u8, &20u8)));

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
        let s_opt: Option<&mut String> = string_enum.value_mut();
        if let Some(s) = s_opt {
            s.push_str("bar");
        }
        let string_check: Option<&String> = string_enum.value();
        assert_eq!(string_check, Some(&"foobar".to_string()));

        let n_opt: Option<&mut i64> = number_enum.value_mut();
        if let Some(n) = n_opt {
            *n += 10;
        }
        let number_check: Option<&i64> = number_enum.value();
        assert_eq!(number_check, Some(&52i64));

        // Test tuple-of-mutable-references - this now works with lifetime parameter!
        let tuple_opt: Option<(&mut u8, &mut u8)> = tuple_enum.value_mut();
        if let Some((a, b)) = tuple_opt {
            *a += 5;
            *b += 5;
        }
        // Check the values were modified
        let tuple_refs: Option<(&u8, &u8)> = tuple_enum.value();
        assert_eq!(tuple_refs, Some((&15u8, &25u8)));

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

        // Test successful unwrapping with new trait design
        let string_result: Result<String, ExampleDerive> = string_enum.into_value();
        assert_eq!(string_result, Ok("hello".to_string()));

        let number_result: Result<i64, ExampleDerive> = number_enum.into_value();
        assert_eq!(number_result, Ok(123i64));

        let tuple_result: Result<(u8, u8), ExampleDerive> = tuple_enum.into_value();
        assert_eq!(tuple_result, Ok((5u8, 10u8)));

        // Test failed unwrapping (wrong type) - should return the original enum
        let wrong_type_enum: ExampleDerive = "test".to_string().into();
        let result: Result<i64, ExampleDerive> = wrong_type_enum.into_value();
        match result {
            Err(ExampleDerive::String(s)) => assert_eq!(s, "test"),
            _ => panic!("Expected Err with original enum"),
        }
    }

    #[test]
    fn test_clean_syntax_with_inference() {
        let text_enum: ExampleDerive = "clean".to_string().into();
        let number_enum: ExampleDerive = 99i64.into();
        let tuple_enum: ExampleDerive = (7u8, 8u8).into();

        // Test that type inference works in assert_eq! context - the compiler
        // can infer the type from the expected value
        assert_eq!(text_enum.value(), Some(&"clean".to_string()));
        assert_eq!(number_enum.value(), Some(&99i64));
        assert_eq!(tuple_enum.value(), Some((&7u8, &8u8)));

        // Test that inference works for into_value when the return type is constrained
        let consumed_text: ExampleDerive = "consume".to_string().into();
        let consumed_number: ExampleDerive = 42i64.into();
        let consumed_tuple: ExampleDerive = (1u8, 2u8).into();

        assert_eq!(consumed_text.into_value(), Ok("consume".to_string()));
        assert_eq!(consumed_number.into_value(), Ok(42i64));
        assert_eq!(consumed_tuple.into_value(), Ok((1u8, 2u8)));

        // When context doesn't provide enough info, explicit annotations are needed:
        let explicit_string: Option<&String> = text_enum.value();
        let explicit_tuple: Option<(&u8, &u8)> = tuple_enum.value();
        assert_eq!(explicit_string, Some(&"clean".to_string()));
        assert_eq!(explicit_tuple, Some((&7u8, &8u8)));
    }

    #[test]
    fn test_fn_overloading() {
        #[derive(TypeEnum)]
        enum FooInputTypes {
            String(String),
            Num(u16),
        }

        fn foo(data: impl Into<FooInputTypes>) -> String {
            match data.into() {
                FooInputTypes::String(val) => format!("You gave me a string: {val}"),
                FooInputTypes::Num(val) => format!("You gave me a number: {val}"),
            }
        }

        assert_eq!(&foo("foo".to_string()), "You gave me a string: foo");
        assert_eq!(&foo(42u16), "You gave me a number: 42");
    }
}
