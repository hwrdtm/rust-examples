use serde::ser::SerializeMap;
use serde::{Serialize, Serializer, de::Visitor, de::MapAccess, Deserialize, Deserializer};
use std::fmt;

#[derive(Debug, PartialEq)]
enum Type {
    Alpha,
    Beta
}

#[derive(Debug, PartialEq)]
struct Custom {
    pub first: String,
    pub second: u32,
    pub third: Option<String>,

    typ: Type,
}

impl Custom {
    pub fn new(first: String, second: u32, third: Option<String>) -> Self {
        Custom {
            first,
            second,
            third,
            typ: Type::Alpha,
        }
    }

    pub(self) fn new_with_typ(first: String, second: u32, third: Option<String>, typ: Type) -> Self {
        Custom {
            first,
            second,
            third,
            typ,
        }
    }

    pub fn determine_typ(third: &Option<String>) -> Type {
        match third {
            Some(_) => Type::Beta,
            None => Type::Alpha,
        }
    }
}

impl Serialize for Custom {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_map(Some(3))?;
        seq.serialize_entry("first", &self.first)?;
        seq.serialize_entry("second", &self.second)?;

        if let Some(third) = &self.third {
            seq.serialize_entry("third", third)?;
        }

        seq.end()
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum CustomField { First, Second, Third }

impl<'de> Deserialize<'de> for Custom {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
    {
        //deserializer.deserialize_any(CustomVisitor)
        deserializer.deserialize_map(CustomVisitor)
    }
}

struct CustomVisitor;

impl<'de> Visitor<'de> for CustomVisitor {
    type Value = Custom;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a map with keys 'first' and 'second', and optionally 'third'")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>
    {
        let mut first = None;
        let mut second = None;
        let mut third = None;

        while let Some(key) = map.next_key()? {
            match key {
                CustomField::First => {
                    if first.is_some() {
                        return Err(serde::de::Error::duplicate_field("first"));
                    }
                    first = Some(map.next_value()?);
                },
                CustomField::Second => {
                    if second.is_some() {
                        return Err(serde::de::Error::duplicate_field("second"));
                    }
                    second = Some(map.next_value()?);
                },
                CustomField::Third => {
                    if third.is_some() {
                        return Err(serde::de::Error::duplicate_field("third"));
                    }
                    third = Some(map.next_value()?);
                },
            }
        }

        let first = first.ok_or_else(|| serde::de::Error::missing_field("first"))?;
        let second = second.ok_or_else(|| serde::de::Error::missing_field("second"))?;

        // Determine the type
        let typ = Custom::determine_typ(&third);

        Ok(Custom::new_with_typ(first, second, third, typ))
    }
}


fn main() {
    let stru = Custom::new(
        "Hello".to_string(),
        123,
        Some("World".to_string()),
    );
    println!("Orig {:?}", stru);

    let serialized = serde_json::to_string(&stru).expect("err ser");

    println!("Seri {}", serialized);

    let unse : Custom = serde_json::from_str(&serialized).expect("err unser");
    println!("New {:?}", unse);
}

#[cfg(test)]
mod tests {
    use crate::{Custom, Type};

    struct SerTestCase {
        input: Custom,
        expected: &'static str,
    }

    struct DeserTestCase {
        input: &'static str,
        expected: Custom,
    }

    #[test]
    fn test_ser() {
        let test_cases = get_test_ser_test_cases();

        for test_case in test_cases {
            let serialized = serde_json::to_string(&test_case.input).expect("err ser");
            assert_eq!(serialized, test_case.expected);
        }
    }

    #[test]
    fn test_deser() {
        let test_cases = get_test_deser_test_cases();

        for test_case in test_cases {
            let deserialized: Custom = serde_json::from_str(test_case.input).expect("err deser");
            assert_eq!(deserialized, test_case.expected);
        }
    }

    fn get_test_ser_test_cases() -> Vec<SerTestCase> {
        vec![
            SerTestCase {
                input: Custom::new(
                    "Hello".to_string(),
                    123,
                    None,
                ),
                expected: r#"{"first":"Hello","second":123}"#,
            },
            SerTestCase {
                input: Custom::new(
                    "Hello".to_string(),
                    123,
                    Some("World".to_string()),
                ),
                expected: r#"{"first":"Hello","second":123,"third":"World"}"#,
            },
        ]
    }

    fn get_test_deser_test_cases() -> Vec<DeserTestCase> {
        vec![
            DeserTestCase {
                input: r#"{"first":"Hello","second":123}"#,
                expected: Custom::new_with_typ(
                    "Hello".to_string(),
                    123,
                    None,
                    Type::Alpha,
                ),
            },
            DeserTestCase {
                input: r#"{"first":"Hello","second":123,"third":"World"}"#,
                expected: Custom::new_with_typ(
                    "Hello".to_string(),
                    123,
                    Some("World".to_string()),
                    Type::Beta,
                ),
            },
        ]
    }
}