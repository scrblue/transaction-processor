use serde::{
    de::{Error, Visitor},
    Deserializer, Serializer,
};

/// Serialize an i64 value such that its four least significant values are behind a decimal point
pub fn serialize<S: Serializer>(value: &i64, serializer: S) -> Result<S::Ok, S::Error> {
    let mut value = *value;

    let one = value % 10;
    value /= 10;
    let two = value % 10;
    value /= 10;
    let three = value % 10;
    value /= 10;
    let four = value % 10;
    value /= 10;

    let mut string = String::new();
    string.push_str(&value.to_string());
    string.push('.');
    string.push_str(&four.to_string());
    string.push_str(&three.to_string());
    string.push_str(&two.to_string());
    string.push_str(&one.to_string());

    serializer.serialize_str(&string)
}

/// Deserialize a floating point number into an i64 where its four least significant decimal digits
/// are considered behind a decimal point
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<i64>, D::Error> {
    struct FixedPoint;
    impl<'de> Visitor<'de> for FixedPoint {
        type Value = Option<i64>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                f,
                "decimal value with at most four place values behind the decimal or nothing"
            )
        }

        fn visit_none<E: Error>(self) -> Result<Option<i64>, E> {
            Ok(None)
        }

        fn visit_some<D: Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Option<i64>, D::Error> {
            deserializer.deserialize_f32(FixedPoint)
        }

        fn visit_f32<E: Error>(self, value: f32) -> Result<Option<i64>, E> {
            let mut fp_num = value * 10_000f32;
            fp_num = fp_num.round();

            Ok(Some(fp_num as i64))
        }
    }

    deserializer.deserialize_option(FixedPoint)
}
