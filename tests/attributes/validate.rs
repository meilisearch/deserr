use std::convert::Infallible;

use deserr::{
    deserialize, serde_json::JsonError, DeserializeError, Deserr, ErrorKind,
    ValuePointerRef,
};
use insta::{assert_debug_snapshot, assert_display_snapshot};
use serde_json::json;

#[test]
fn validate() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    #[deserr(validate = validate_range -> __Deserr_E)]
    struct Range {
        start: usize,
        end: usize,
    }

    fn validate_range<E: DeserializeError>(
        range: Range,
        location: ValuePointerRef,
    ) -> Result<Range, E> {
        if range.end < range.start {
            Err(deserr::take_result_content(E::error::<Infallible>(
                None,
                ErrorKind::Unexpected {
                    msg: format!(
                        "`end` (`{}`) should be greater than `start` (`{}`)",
                        range.end, range.start
                    ),
                },
                location,
            )))
        } else {
            Ok(range)
        }
    }

    let data = deserialize::<Range, _, JsonError>(json!({ "start": 2, "end": 6 })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Range {
        start: 2,
        end: 6,
    }
    "###);

    let data = deserialize::<Range, _, JsonError>(json!({ "start": 6, "end": 2 })).unwrap_err();

    assert_display_snapshot!(data, @"`end` (`2`) should be greater than `start` (`6`) at ``.");
}
