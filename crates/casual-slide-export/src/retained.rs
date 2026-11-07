//! A retained object's identity is read-only (`SEC-C07`, Tier 1L). Its markup and its bytes
//! are never written from the model, so a changed identity could only be lost: it is refused.

use casual_slide_model::retained::RetainedObject;

/// Whether a slide's controls are as they were read.
pub(crate) fn controls_unchanged(before: &[RetainedObject], after: &[RetainedObject]) -> bool {
    before == after
}

/// Whether a node's object is as it was read.
pub(crate) fn object_unchanged(
    before: Option<&RetainedObject>,
    after: Option<&RetainedObject>,
) -> bool {
    before == after
}
