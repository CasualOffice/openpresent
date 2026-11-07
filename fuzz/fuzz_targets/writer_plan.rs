#![no_main]
//! Random save plans over decks. Bytes starting `PK` are also tried as a deck of their own,
//! otherwise each picks a committed fixture. The writer never panics. **A plan it accepts
//! writes a package that opens, whose retained parts are unchanged** and whose saved-again
//! form is a fixed point; a plan it refuses leaves the session able to save its input.

use casual_slide_export::writer::{check_zip, Op, SavePlan, SlideRef, WriterLimits};
use casual_slide_import::properties::ImportLimits;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use libfuzzer_sys::fuzz_target;

const DECKS: [&[u8]; 6] = [
    include_bytes!("../../fixtures/generated/three-slides.pptx"),
    include_bytes!("../../fixtures/generated/rr-sections.pptx"),
    include_bytes!("../../fixtures/generated/rr-notes.pptx"),
    include_bytes!("../../fixtures/generated/rr-shared-media.pptx"),
    include_bytes!("../../fixtures/generated/rr-rich-clone.pptx"),
    include_bytes!("../../fixtures/generated/rr-two-masters.pptx"),
];
const SECTION: &str = "{11111111-AAAA-4BBB-8CCC-000000000001}";

fn slide(byte: u8) -> SlideRef {
    match byte % 4 {
        0 => SlideRef::Id(256 + u64::from(byte >> 2)),
        _ => SlideRef::Index(usize::from(byte >> 2) % 8),
    }
}

fn op(kind: u8, a: u8, b: u8) -> Op {
    match kind % 18 {
        0 => Op::AddSlide { position: Some(usize::from(a) % 8), layout: None },
        1 => Op::AddSlide { position: None, layout: None },
        2 => Op::DuplicateSlide { slide: slide(a) },
        3 => Op::DeleteSlide { slide: slide(a) },
        4 => Op::MoveSlide { slide: slide(a), position: usize::from(b) % 8 },
        5 => Op::AddNotes { slide: slide(a), text: format!("n{b}") },
        6 => Op::RemoveNotes { slide: slide(a) },
        7 => Op::AddPicture {
            slide: slide(a),
            bytes: vec![b; 8],
            extension: "png".to_owned(),
            content_type: "image/png".to_owned(),
        },
        8 => Op::RemovePicture { slide: slide(a), shape_id: u32::from(b) },
        9 => Op::ReplaceMedia {
            slide: slide(a),
            rel_id: format!("rId{}", b % 6),
            bytes: vec![b, a],
            extension: "png".to_owned(),
            content_type: "image/png".to_owned(),
        },
        10 => Op::AddLayout {
            master: "ppt/slideMasters/slideMaster1.xml".to_owned(),
            copy_of: format!("ppt/slideLayouts/slideLayout{}.xml", b % 3 + 1),
        },
        11 => Op::RemoveLayout { layout: format!("ppt/slideLayouts/slideLayout{}.xml", b % 3 + 1) },
        12 => Op::AddMaster { copy_of: format!("ppt/slideMasters/slideMaster{}.xml", b % 2 + 1) },
        13 => Op::RemoveMaster { master: format!("ppt/slideMasters/slideMaster{}.xml", b % 2 + 1) },
        14 => Op::AddSection {
            name: format!("S{b}"),
            id: format!("{{{a:08X}-AAAA-4BBB-8CCC-{b:012X}}}"),
            at: (a % 2 == 0).then(|| slide(b)),
        },
        15 => Op::RemoveSection { id: SECTION.to_owned() },
        16 => Op::MoveSlideToSection {
            slide: slide(a),
            section: SECTION.to_owned(),
            index: usize::from(b) % 4,
        },
        _ => Op::CopyShape { slide: slide(a), shape_id: u32::from(b), creation_id: None },
    }
}

fn plan(data: &[u8]) -> SavePlan<'static> {
    let limits = WriterLimits { max_clone_parts: 200, ..WriterLimits::default() };
    let mut plan = SavePlan::new().with_limits(ImportLimits::default(), limits);
    for ops in data.chunks_exact(3).take(16) {
        plan = plan.op(op(ops[0], ops[1], ops[2]));
    }
    plan
}

fuzz_target!(|data: &[u8]| {
    let xml = OoxmlLimits::default();
    let (source, ops): (Vec<u8>, &[u8]) = if data.starts_with(b"PK") {
        (data.to_vec(), b"\x01\x00\x00\x02\x00\x00\x03\x00\x00")
    } else {
        let Some((first, rest)) = data.split_first() else {
            return;
        };
        (DECKS[usize::from(*first) % DECKS.len()].to_vec(), rest)
    };
    let Ok(mut package) = PresentationPackage::open(source.clone(), xml) else {
        return;
    };
    match plan(ops).save(&mut package) {
        Ok(outcome) => {
            let changed = outcome.changed_parts.iter().cloned().collect();
            if data.starts_with(b"PK") {
                // A hostile input may be one this check cannot vouch for; it must not panic.
                let _ = check_zip(&source, &outcome.bytes, &changed);
                let _ = PresentationPackage::open(outcome.bytes, xml);
                return;
            }
            check_zip(&source, &outcome.bytes, &changed).expect("retained parts are unchanged");
            let mut again =
                PresentationPackage::open(outcome.bytes.clone(), xml).expect("the result opens");
            let second = SavePlan::new().save(&mut again).expect("an empty plan saves");
            assert_eq!(second.bytes, outcome.bytes, "an empty plan is a fixed point");
            // The result is a deck the writer can go on working with.
            let _ = SavePlan::new()
                .op(Op::AddSlide { position: None, layout: None })
                .save(&mut again)
                .map(|o| PresentationPackage::open(o.bytes, xml).expect("the second result opens"));
        }
        Err(err) => {
            assert_eq!(package.save_unedited(), source, "a refused plan leaves the input");
            // On a clean deck, an operation may be refused for what it asks, but what it
            // produced breaking a rule, or failing after writing, is a writer bug.
            if !data.starts_with(b"PK") {
                assert!(
                    !matches!(err.code(), "CS-SAV-0012" | "CS-SAV-0020"),
                    "{err}"
                );
            }
        }
    }
});
