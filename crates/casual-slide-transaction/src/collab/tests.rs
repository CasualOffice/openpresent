//! TP1 and convergence over the real model (`docs/24` §Text OT; `VS-09`).

use casual_slide_model::presentation::{Coordinate, Point, PositiveSize};
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode, Transform};
use casual_slide_model::slide::{CommonSlideData, SlideKind, SlidePart, SlideParts};
use casual_slide_model::text::{
    CharacterProperties, Paragraph, TextBody, TextItem, TextRun, UnderlineType,
};

use super::wire::{Codec, Id, SlideEngine, WireAttribute, WireOp, WireTransform};
use super::{ClientId, ClientSession, Commit, Engine, Replica, ServerSession, Side};
use crate::{Operation, apply};

const ROOT: ObjectId = ObjectId { part: 0, node: 1 };
const A: ObjectId = ObjectId { part: 0, node: 2 };
const B: ObjectId = ObjectId { part: 0, node: 3 };

fn run(text: &str, bold: Option<bool>) -> TextItem {
    TextItem::Run(TextRun {
        properties: bold.map(|b| {
            Box::new(CharacterProperties {
                bold: Some(b),
                ..Default::default()
            })
        }),
        text: Some(text.to_owned()),
    })
}

fn shape(id: ObjectId) -> ShapeNode {
    let mut shape = ShapeNode::new(id, NodeKind::Shape);
    shape.text_body = Some(Box::new(TextBody {
        paragraphs: vec![
            Paragraph {
                items: vec![run("Hello ", None), run("world", Some(true))],
                ..Default::default()
            },
            Paragraph {
                items: vec![
                    run("Sec", None),
                    TextItem::Break(Default::default()),
                    run("ond", None),
                ],
                ..Default::default()
            },
        ],
        ..Default::default()
    }));
    shape
}

fn deck() -> SlideParts {
    let mut tree = ShapeNode::new(ROOT, NodeKind::Group);
    tree.children.push(shape(A));
    tree.children.push(shape(B));
    let mut part = SlidePart::new(ObjectId::part(0), SlideKind::Slide);
    part.common = Some(CommonSlideData {
        name: None,
        background: None,
        shape_tree: Some(tree),
        controls: Vec::new(),
    });
    SlideParts {
        parts: vec![Some(part)],
        ..Default::default()
    }
}

/// The test's codec: run properties cross as an index into a table every copy in
/// the test shares, and a node as nothing but its number. The host's codec writes and
/// reads real XML (`casual-slide-wasm`'s `collab.rs`).
struct Table;

thread_local! {
    static RUNS: std::cell::RefCell<Vec<CharacterProperties>> = const { std::cell::RefCell::new(Vec::new()) };
}

impl Codec for Table {
    fn node_xml(&self, _: &ShapeNode) -> Option<String> {
        Some(String::new())
    }
    fn run_xml(&self, p: &CharacterProperties) -> Option<String> {
        RUNS.with(|r| {
            let mut r = r.borrow_mut();
            r.push(p.clone());
            Some((r.len() - 1).to_string())
        })
    }
    fn read_node(&self, _xml: &str, id: ObjectId) -> Option<ShapeNode> {
        Some(ShapeNode::new(id, NodeKind::Shape))
    }
    fn read_run(&self, xml: &str) -> Option<CharacterProperties> {
        let i: usize = xml.parse().ok()?;
        RUNS.with(|r| r.borrow().get(i).cloned())
    }
}

struct Copy(SlideParts);

impl Replica<SlideEngine> for Copy {
    type Error = ();
    fn apply_remote(&mut self, op: &WireOp) -> Result<(), ()> {
        let op = op.operation(&Table).map_err(|_| ())?;
        apply(&mut self.0, op).map(|_| ()).map_err(|_| ())
    }
    fn write_snapshot(&self) -> Result<Vec<u8>, ()> {
        Ok(Vec::new())
    }
    fn read_snapshot(&mut self, _: &[u8]) -> Result<(), ()> {
        Ok(())
    }
}

/// What a reader sees: per shape in z-order, its id, transform, paragraphs, and each
/// character with its bold, italic and underline. Run boundaries are not compared,
/// since two orders may cut the same text into runs differently and draw it alike.
fn seen(parts: &SlideParts) -> String {
    seen_as(parts, true)
}

/// The text alone, paragraphs and line breaks included.
fn text_seen(parts: &SlideParts) -> String {
    seen_as(parts, false)
}

fn seen_as(parts: &SlideParts, formats: bool) -> String {
    let mut out = String::new();
    let root = parts.get(0).and_then(|p| p.shape_tree()).unwrap();
    for node in &root.children {
        out += &format!("[{} {:?}]", node.id, node.transform);
        if let Some(body) = node.text_body.as_deref() {
            for p in &body.paragraphs {
                out.push('¶');
                for item in &p.items {
                    let props = super::super::properties_of(item)
                        .cloned()
                        .unwrap_or_default();
                    let flags = if formats {
                        format!(
                            "{:?}{:?}{:?}{:?}",
                            props.bold, props.italic, props.underline, props.size
                        )
                    } else {
                        String::new()
                    };
                    match item {
                        TextItem::Run(r) => {
                            for c in r.text.as_deref().unwrap_or("").chars() {
                                out += &format!("{c}{flags}|");
                            }
                        }
                        TextItem::Break(_) => out += &format!("⏎{flags}|"),
                        _ => {}
                    }
                }
            }
        }
    }
    out
}

/// A fixed-seed generator: tests are deterministic (CLAUDE.md).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % (n.max(1) as u64)) as usize
    }
}

fn length(parts: &SlideParts, id: ObjectId) -> usize {
    parts
        .get(0)
        .and_then(|p| p.node(id))
        .and_then(|n| n.text_body.as_deref())
        .map_or(0, crate::body_length)
}

/// A random shareable edit of `parts`, in its wire form, against that state.
fn random_op(rng: &mut Lcg, parts: &SlideParts, fresh: u32, breaks: bool) -> WireOp {
    let object = if rng.below(2) == 0 { A } else { B };
    let len = length(parts, object);
    let attribute = match rng.below(4) {
        0 => WireAttribute::Bold(Some(rng.below(2) == 0)),
        1 => WireAttribute::Italic(Some(true)),
        2 => WireAttribute::Size(Some(1200 + 100 * rng.below(3) as i32)),
        _ => WireAttribute::Underline(Some("sng".to_owned())),
    };
    let op = match rng.below(9) {
        0..=2 => Operation::InsertText {
            object,
            at: rng.below(len + 1),
            text: if breaks {
                ["x", "yz", "\n", "a\u{b}b", "Q\nR"][rng.below(5)]
            } else {
                ["x", "yz", "abc"][rng.below(3)]
            }
            .to_owned(),
        },
        3 | 4 => {
            let s = rng.below(len + 1);
            Operation::DeleteRange {
                object,
                start: s,
                end: s + rng.below(len + 1 - s),
            }
        }
        5 | 6 => {
            let s = rng.below(len + 1);
            Operation::FormatRange {
                object,
                start: s,
                end: s + rng.below(len + 1 - s),
                attribute: match attribute {
                    WireAttribute::Bold(v) => crate::Attribute::Bold(v),
                    WireAttribute::Italic(v) => crate::Attribute::Italic(v),
                    WireAttribute::Underline(_) => {
                        crate::Attribute::Underline(Some(UnderlineType::Sng))
                    }
                    WireAttribute::Size(v) => crate::Attribute::Size(v),
                    _ => unreachable!(),
                },
            }
        }
        7 => Operation::SetTransform {
            node: object,
            transform: Some(Transform {
                offset: Some(Point {
                    x: Coordinate(rng.below(1000) as i64),
                    y: Coordinate(0),
                }),
                extent: Some(PositiveSize { cx: 10, cy: 10 }),
                ..Default::default()
            }),
        },
        _ => {
            let root = parts.get(0).and_then(|p| p.shape_tree()).unwrap();
            let id = ObjectId {
                part: 0,
                node: fresh,
            };
            return WireOp::InsertNode {
                parent: ROOT.into(),
                index: rng.below(root.children.len() + 1),
                node: id.into(),
                xml: String::new(),
                media: None,
            };
        }
    };
    let mut scratch = parts.clone();
    let inverse = apply(&mut scratch, op.clone()).unwrap();
    // Typed text is sent as the runs it became, as a shared history sends it.
    let sent = crate::sent_form(&scratch, op);
    WireOp::of(&sent, &inverse, &Table).unwrap()
}

fn applied(parts: &SlideParts, ops: &[&WireOp]) -> SlideParts {
    let mut copy = Copy(parts.clone());
    for op in ops {
        if !SlideEngine::is_noop(op) {
            copy.apply_remote(op).unwrap();
        }
    }
    copy.0
}

#[test]
fn every_pair_of_concurrent_edits_converges_whichever_order_it_is_applied_in() {
    tp1(false, seen);
}

/// Text typed with a paragraph or line break in it converges, format and all.
#[test]
fn concurrent_edits_with_breaks_converge_with_their_formatting() {
    tp1(true, seen);
}

fn tp1(breaks: bool, seen: fn(&SlideParts) -> String) {
    // TP1: a then b' leaves what b then a' leaves.
    let mut rng = Lcg(9);
    for case in 0..4000 {
        // Half the cases start from a state a third edit made, with the two edits
        // rebased over it, so transforms of what transform made (batches, split
        // ranges) are checked too.
        let origin = deck();
        let first = random_op(&mut rng, &origin, 99, breaks);
        let a = random_op(&mut rng, &origin, 100, breaks);
        let b = random_op(&mut rng, &origin, 101, breaks);
        let (start, a, b) = if case % 2 == 0 {
            (origin, a, b)
        } else {
            (
                applied(&origin, &[&first]),
                SlideEngine::transform(&a, &first, Side::Later),
                SlideEngine::transform(&b, &first, Side::Later),
            )
        };
        let b_after_a = SlideEngine::transform(&b, &a, Side::Later);
        let a_after_b = SlideEngine::transform(&a, &b, Side::Earlier);
        let left = applied(&start, &[&a, &b_after_a]);
        let right = applied(&start, &[&b, &a_after_b]);
        assert_eq!(
            seen(&left),
            seen(&right),
            "case {case}: a = {a:?}, b = {b:?}, b' = {b_after_a:?}, a' = {a_after_b:?}"
        );
    }
}

#[test]
fn a_delete_beats_a_concurrent_move_of_the_same_shape() {
    let start = deck();
    let mv = WireOp::SetTransform {
        node: A.into(),
        transform: Some(WireTransform {
            x: Some(5),
            y: Some(5),
            ..Default::default()
        }),
    };
    let del = WireOp::DeleteNode {
        node: A.into(),
        parent: ROOT.into(),
        index: 0,
    };
    assert_eq!(SlideEngine::transform(&mv, &del, Side::Later), WireOp::Noop);
    let left = applied(
        &start,
        &[&del, &SlideEngine::transform(&mv, &del, Side::Later)],
    );
    let right = applied(
        &start,
        &[&mv, &SlideEngine::transform(&del, &mv, Side::Earlier)],
    );
    assert_eq!(seen(&left), seen(&right));
}

#[test]
fn of_two_moves_of_one_shape_the_one_ordered_later_stands() {
    let mv = |x| WireOp::SetTransform {
        node: A.into(),
        transform: Some(WireTransform {
            x: Some(x),
            y: Some(0),
            ..Default::default()
        }),
    };
    assert_eq!(SlideEngine::transform(&mv(1), &mv(2), Side::Later), mv(1));
    assert_eq!(
        SlideEngine::transform(&mv(1), &mv(2), Side::Earlier),
        WireOp::Noop
    );
}

/// Two clients and a server, with every edit made concurrently and messages delivered
/// in a random interleaving: every copy ends with the server's text, shapes and order.
#[test]
fn two_clients_editing_at_once_through_the_server_end_with_the_same_deck() {
    converge(false, text_seen);
}

/// The same, comparing every character's bold, italic, underline and size too: typed
/// text carries the run properties it took where it was typed (`COL-01`).
#[test]
fn two_clients_editing_at_once_end_with_the_same_formatting() {
    converge(false, seen);
}

/// As above with breaks typed, formatting compared.
#[test]
fn two_clients_typing_breaks_at_once_end_with_the_same_formatting() {
    converge(true, seen);
}

/// What the server sent a client: operations and the revision they reach, or an
/// acknowledgement of a chunk.
type Delivery = (Vec<WireOp>, u64, bool, u64);

fn converge(breaks: bool, seen: fn(&SlideParts) -> String) {
    for seed in 0..60u64 {
        let mut rng = Lcg(seed);
        let mut server = ServerSession::<SlideEngine>::new();
        let mut copies = [Copy(deck()), Copy(deck())];
        let mut sessions = [
            ClientSession::<SlideEngine>::new(ClientId(1), 0),
            ClientSession::<SlideEngine>::new(ClientId(2), 0),
        ];
        // What the server sent each client, not yet delivered.
        let mut inbox: [Vec<Delivery>; 2] = [Vec::new(), Vec::new()];
        // What each client sent, not yet delivered to the server.
        let mut outbox: [Vec<super::Submission<SlideEngine>>; 2] = [Vec::new(), Vec::new()];
        let mut ordered: Vec<WireOp> = Vec::new();
        let mut fresh = 1000u32;
        for _ in 0..80 {
            let who = rng.below(2);
            match rng.below(4) {
                0 | 1 => {
                    fresh += 1;
                    let op = random_op(
                        &mut rng,
                        &copies[who].0,
                        fresh + who as u32 * 100_000,
                        breaks,
                    );
                    copies[who].apply_remote(&op).unwrap();
                    sessions[who].record(op);
                }
                2 => {
                    if let Some(s) = sessions[who].flush() {
                        outbox[who].push(s);
                    }
                }
                _ => {
                    // Deliver one thing: to the server, or to a client.
                    if rng.below(2) == 0 && !outbox[who].is_empty() {
                        let s = outbox[who].remove(0);
                        if let Commit::Applied { ops, revision } = server.commit(&s).unwrap() {
                            ordered.extend(ops.iter().cloned());
                            let other = 1 - who;
                            inbox[other].push((ops, revision, false, 0));
                            inbox[who].push((Vec::new(), revision, true, s.seq));
                        }
                    } else if !inbox[who].is_empty() {
                        let (ops, revision, ack, seq) = inbox[who].remove(0);
                        if ack {
                            sessions[who].acknowledge(seq, revision);
                        } else {
                            for op in &ops {
                                sessions[who]
                                    .receive(&mut copies[who], op, revision)
                                    .unwrap();
                            }
                        }
                    }
                }
            }
        }
        // Drain: flush everything, deliver everything.
        loop {
            let mut moved = false;
            for who in 0..2 {
                if let Some(s) = sessions[who].flush() {
                    outbox[who].push(s);
                }
                for s in outbox[who].drain(..).collect::<Vec<_>>() {
                    moved = true;
                    if let Commit::Applied { ops, revision } = server.commit(&s).unwrap() {
                        ordered.extend(ops.iter().cloned());
                        inbox[1 - who].push((ops, revision, false, 0));
                        inbox[who].push((Vec::new(), revision, true, s.seq));
                    }
                }
            }
            for who in 0..2 {
                for (ops, revision, ack, seq) in inbox[who].drain(..).collect::<Vec<_>>() {
                    moved = true;
                    if ack {
                        sessions[who].acknowledge(seq, revision);
                    } else {
                        for op in &ops {
                            sessions[who]
                                .receive(&mut copies[who], op, revision)
                                .unwrap();
                        }
                    }
                }
            }
            if !moved {
                break;
            }
        }
        let truth = applied(&deck(), &ordered.iter().collect::<Vec<_>>());
        assert_eq!(seen(&copies[0].0), seen(&truth), "seed {seed}: client 1");
        assert_eq!(seen(&copies[1].0), seen(&truth), "seed {seed}: client 2");
        assert!(!sessions[0].has_unacknowledged() && !sessions[1].has_unacknowledged());
    }
}

#[test]
fn a_wire_operation_survives_json_both_ways() {
    let op = WireOp::Batch {
        ops: vec![
            WireOp::InsertText {
                object: Id { part: 1, node: 4 },
                at: 3,
                text: "hé\n".to_owned(),
                run: "<a:rPr b=\"1\"/>".to_owned(),
                formats: vec![WireAttribute::Size(Some(2400))],
            },
            WireOp::FormatRange {
                object: Id { part: 1, node: 4 },
                start: 0,
                end: 2,
                attribute: WireAttribute::Underline(Some("dbl".to_owned())),
            },
        ],
    };
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(serde_json::from_str::<WireOp>(&json).unwrap(), op);
}

#[test]
fn a_value_outside_the_format_is_refused_not_applied() {
    let bad = WireOp::SetTransform {
        node: A.into(),
        transform: Some(WireTransform {
            x: Some(i64::MAX),
            y: Some(0),
            ..Default::default()
        }),
    };
    assert_eq!(
        bad.operation(&Table).unwrap_err(),
        super::wire::WireError::BadValue
    );
    let underline = WireOp::FormatRange {
        object: A.into(),
        start: 0,
        end: 1,
        attribute: WireAttribute::Underline(Some("<script>".to_owned())),
    };
    assert!(underline.operation(&Table).is_err());
    let size = WireOp::FormatRange {
        object: A.into(),
        start: 0,
        end: 1,
        attribute: WireAttribute::Size(Some(i32::MAX)),
    };
    assert!(size.operation(&Table).is_err());
}

#[test]
fn a_shared_history_refuses_what_it_cannot_send_and_journals_what_it_applies() {
    let mut parts = deck();
    let mut history = crate::History::new();
    history.share(Some(super::wire::shareable));
    let refused = history.apply(
        &mut parts,
        Operation::SetSlideShow {
            ordinal: 0,
            show: Some(false),
        },
    );
    assert_eq!(refused, Err(crate::TxnError::NotShared));
    history
        .apply(
            &mut parts,
            Operation::InsertText {
                object: A,
                at: 0,
                text: "x".to_owned(),
            },
        )
        .unwrap();
    let journal = history.take_journal();
    assert_eq!(journal.len(), 1);
    // Typing is sent as the run it became, with that run's properties.
    assert!(matches!(
        &journal[0].0,
        Operation::InsertRun { at: 0, text, .. } if text == "x"
    ));
    // Undo of typing is a delete, which travels: applied and journaled.
    let before = parts.clone();
    assert!(history.undo(&mut parts).unwrap().is_some());
    assert_ne!(parts, before);
    let journal = history.take_journal();
    assert_eq!(
        journal[0].0,
        Operation::DeleteRange {
            object: A,
            start: 0,
            end: 1
        }
    );
    assert_eq!(text_seen(&parts), text_seen(&deck()));
}

/// One copy of the deck in a shared session, editing through its own history.
struct Peer {
    parts: SlideParts,
    history: crate::History,
    session: ClientSession<SlideEngine>,
}

impl Peer {
    fn new(client: u64) -> Self {
        let mut history = crate::History::new();
        history.share(Some(super::wire::shareable));
        Self {
            parts: deck(),
            history,
            session: ClientSession::new(ClientId(client), 0),
        }
    }

    fn drain(&mut self) {
        for (op, inverse) in self.history.take_journal() {
            self.session
                .record(WireOp::of(&op, &inverse, &Table).unwrap());
        }
    }

    fn edit(&mut self, op: Operation) {
        self.history.apply(&mut self.parts, op).unwrap();
        self.drain();
    }

    fn undo(&mut self) {
        self.history.undo(&mut self.parts).unwrap();
        self.drain();
    }

    fn submit(&mut self, server: &mut ServerSession<SlideEngine>) -> (Vec<WireOp>, u64) {
        let s = self.session.flush().unwrap();
        match server.commit(&s).unwrap() {
            Commit::Applied { ops, revision } => {
                self.session.acknowledge(s.seq, revision);
                (ops, revision)
            }
            Commit::Duplicate { .. } => panic!("duplicate"),
        }
    }

    fn receive(&mut self, ops: &[WireOp], revision: u64) {
        let mut copy = Copy(core::mem::take(&mut self.parts));
        for op in ops {
            let applied = self.session.receive(&mut copy, op, revision).unwrap();
            super::rebase_history(&mut self.history, &copy.0, &applied);
        }
        self.parts = copy.0;
    }
}

fn first_paragraph(parts: &SlideParts) -> String {
    text_seen(parts)
        .split('¶')
        .nth(1)
        .unwrap()
        .split('|')
        .collect::<String>()
}

/// Ada types, Grace types before Ada's text, then Ada undoes: Ada's typing goes,
/// Grace's stays, on both copies (`docs/24` §Undo, OpenCalc's docs/69).
#[test]
fn undo_of_typing_in_a_shared_session_takes_back_only_my_own_over_others_edits() {
    let mut server = ServerSession::<SlideEngine>::new();
    let mut ada = Peer::new(1);
    let mut grace = Peer::new(2);
    ada.edit(Operation::InsertText {
        object: A,
        at: 6,
        text: "dear ".to_owned(),
    });
    let (ops, rev) = ada.submit(&mut server);
    grace.receive(&ops, rev);
    grace.edit(Operation::InsertText {
        object: A,
        at: 0,
        text: "Oh, ".to_owned(),
    });
    let (ops, rev) = grace.submit(&mut server);
    ada.receive(&ops, rev);
    assert_eq!(first_paragraph(&ada.parts), "Oh, Hello dear world");
    ada.undo();
    let (ops, rev) = ada.submit(&mut server);
    grace.receive(&ops, rev);
    assert_eq!(first_paragraph(&ada.parts), "Oh, Hello world");
    assert_eq!(seen(&ada.parts), seen(&grace.parts));
    // Redo puts it back where it now belongs.
    ada.history.redo(&mut ada.parts).unwrap();
    ada.drain();
    let (ops, rev) = ada.submit(&mut server);
    grace.receive(&ops, rev);
    assert_eq!(first_paragraph(&grace.parts), "Oh, Hello dear world");
    assert_eq!(seen(&ada.parts), seen(&grace.parts));
}

/// Undo of a delete types the deleted runs back with their own formats, and of a
/// format puts each character's old value back, on both copies.
#[test]
fn undo_of_a_delete_and_a_format_in_a_shared_session_restores_runs_on_both_copies() {
    let mut server = ServerSession::<SlideEngine>::new();
    let mut ada = Peer::new(1);
    let mut grace = Peer::new(2);
    // "Hello " plain, "world" bold: delete across both, then unbold what is left.
    ada.edit(Operation::DeleteRange {
        object: A,
        start: 3,
        end: 8,
    });
    ada.edit(Operation::FormatRange {
        object: A,
        start: 0,
        end: 6,
        attribute: crate::Attribute::Bold(Some(false)),
    });
    let (ops, rev) = ada.submit(&mut server);
    grace.receive(&ops, rev);
    grace.edit(Operation::InsertText {
        object: B,
        at: 0,
        text: "!".to_owned(),
    });
    let (ops, rev) = grace.submit(&mut server);
    ada.receive(&ops, rev);
    ada.undo();
    ada.undo();
    let (ops, rev) = ada.submit(&mut server);
    grace.receive(&ops, rev);
    assert_eq!(seen(&ada.parts), seen(&grace.parts));
    let mut expected = deck();
    apply(
        &mut expected,
        Operation::InsertText {
            object: B,
            at: 0,
            text: "!".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(seen(&ada.parts), seen(&expected));
}
