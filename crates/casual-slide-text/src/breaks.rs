//! Where a line may end: UAX #14 break opportunities, from `unicode-linebreak`.
//!
//! New here: OpenCalc does not wrap. East Asian rules beyond UAX #14 (kinsoku,
//! hanging punctuation) and Thai, which needs a dictionary, are `T2D5-01` and
//! `TXT-07`.

/// A place a line may end, as a byte offset: the line holds the text before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Break {
    /// Byte offset into the paragraph's text, on a character boundary.
    pub at: usize,
    /// The line must end here: a line break, or the end of the text.
    pub mandatory: bool,
}

/// Every break opportunity in `text`, in order. The last is always the end of
/// the text, mandatory, so a caller can rely on one being there.
pub(crate) fn opportunities(text: &str) -> Vec<Break> {
    let mut out: Vec<Break> = unicode_linebreak::linebreaks(text)
        .map(|(at, opportunity)| Break {
            at,
            mandatory: opportunity == unicode_linebreak::BreakOpportunity::Mandatory,
        })
        .collect();
    if out.last().is_none_or(|b| b.at != text.len()) {
        out.push(Break {
            at: text.len(),
            mandatory: true,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spaces_offer_a_break_after_them_and_the_end_is_mandatory() {
        let breaks = opportunities("one two");
        assert_eq!(
            breaks,
            vec![
                Break {
                    at: 4,
                    mandatory: false
                },
                Break {
                    at: 7,
                    mandatory: true
                },
            ]
        );
    }

    #[test]
    fn a_line_feed_is_a_mandatory_break_after_it() {
        let breaks = opportunities("a\nb");
        assert!(breaks.contains(&Break {
            at: 2,
            mandatory: true
        }));
    }

    #[test]
    fn empty_text_still_has_its_end() {
        assert_eq!(
            opportunities(""),
            vec![Break {
                at: 0,
                mandatory: true
            }]
        );
    }
}
