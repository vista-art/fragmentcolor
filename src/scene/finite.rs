//! `check_finite`, the one check the scene types run on the numbers they take.

/// A number that isn't finite, such as NaN or an infinity, with the label of its call and item.
/// Each scene error with a `NotFinite` variant converts from it, so `?` lifts it there.
#[derive(Debug)]
pub(crate) struct NotFinite {
    pub(crate) label: String,
    pub(crate) value: f32,
}

/// Refuses a number of `values` that isn't finite, naming `call`, then `item`,
/// then the item's index when `values` holds more than one number.
pub(crate) fn check_finite(
    call: &str,
    item: Option<&str>,
    values: &[f32],
) -> Result<(), NotFinite> {
    let found = values
        .iter()
        .enumerate()
        .find(|(_, value)| !value.is_finite());
    let Some((index, &value)) = found else {
        return Ok(());
    };
    let mut label = call.to_string();
    if let Some(item) = item {
        label = format!("{label}: {item}");
    }
    if values.len() > 1 {
        label = format!("{label}: item {index}");
    }
    Err(NotFinite { label, value })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: the label names the call, the item and, for a list, the index of the first
    // number that isn't finite, and a list of finite numbers passes.
    #[test]
    fn names_the_call_the_item_and_the_index() {
        let labels = [
            check_finite("Light::point", Some("position"), &[0.0, f32::NAN, 0.0]),
            check_finite("Light::set_intensity", Some("value"), &[f32::INFINITY]),
            check_finite("Scene::ambient", None, &[0.0, 0.0, f32::NEG_INFINITY]),
        ]
        .map(|checked| {
            checked
                .err()
                .map(|error| (error.label, error.value.to_string()))
        });

        assert_eq!(
            labels,
            [
                Some(("Light::point: position: item 1".into(), "NaN".into())),
                Some(("Light::set_intensity: value".into(), "inf".into())),
                Some(("Scene::ambient: item 2".into(), "-inf".into())),
            ]
        );
        assert!(check_finite("Model::scale", None, &[f32::MAX, f32::MIN, 0.0]).is_ok());
    }
}
