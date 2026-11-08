//! Two clients co-editing one deck through a running server (`VS-09`): each edits
//! the same text box and moves the same shape at once, every edit reaches the other,
//! and both end with the same deck; a third who joins late catches up to it.
//!
//! The server is started on 127.0.0.1 with an ephemeral port, as CI's `collab` job
//! runs it; the clients speak the real protocol over real sockets, and each holds a
//! real model replica that applies what arrives.

use std::time::Duration;

use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode};
use casual_slide_model::slide::{CommonSlideData, SlideKind, SlidePart, SlideParts};
use casual_slide_model::text::{Paragraph, TextBody, TextItem, TextRun};
use casual_slide_transaction::collab::wire::{Id, SlideEngine, WireOp, WireTransform};
use casual_slide_transaction::collab::{
    ClientId, ClientMessage, ClientSession, Presence, Replica, ServerMessage,
};
use casual_slide_transaction::{Operation, apply};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

const ROOT: ObjectId = ObjectId { part: 0, node: 1 };
const BOX: ObjectId = ObjectId { part: 0, node: 2 };

fn deck() -> SlideParts {
    let mut shape = ShapeNode::new(BOX, NodeKind::Shape);
    shape.text_body = Some(Box::new(TextBody {
        paragraphs: vec![Paragraph {
            items: vec![TextItem::Run(TextRun {
                properties: None,
                text: Some("Hello world".to_owned()),
            })],
            ..Default::default()
        }],
        ..Default::default()
    }));
    let mut tree = ShapeNode::new(ROOT, NodeKind::Group);
    tree.children.push(shape);
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

fn text(parts: &SlideParts) -> String {
    let body = parts
        .get(0)
        .unwrap()
        .node(BOX)
        .unwrap()
        .text_body
        .as_deref()
        .unwrap();
    body.paragraphs
        .iter()
        .map(|p| {
            p.items
                .iter()
                .map(|i| match i {
                    TextItem::Run(r) => r.text.clone().unwrap_or_default(),
                    _ => String::new(),
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

struct Copy(SlideParts);

impl Replica<SlideEngine> for Copy {
    type Error = String;
    fn apply_remote(&mut self, op: &WireOp) -> Result<(), String> {
        let op = op
            .operation(&|_, id| Some(ShapeNode::new(id, NodeKind::Shape)))
            .map_err(|e| e.to_string())?;
        apply(&mut self.0, op)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    fn write_snapshot(&self) -> Result<Vec<u8>, String> {
        Ok(b"deck".to_vec())
    }
    fn read_snapshot(&mut self, _: &[u8]) -> Result<(), String> {
        self.0 = deck();
        Ok(())
    }
}

struct Client {
    socket: Socket,
    copy: Copy,
    session: ClientSession<SlideEngine>,
    seen_presence: Vec<(ClientId, Presence)>,
}

async fn send(socket: &mut Socket, message: &ClientMessage<SlideEngine>) {
    socket
        .send(Message::Text(serde_json::to_string(message).unwrap()))
        .await
        .unwrap();
}

async fn join(url: &str, name: &str) -> Client {
    let (mut socket, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    send(
        &mut socket,
        &ClientMessage::Join {
            protocol: 1,
            name: name.to_owned(),
            seed: Some("ZGVjaw==".to_owned()),
        },
    )
    .await;
    let mut copy = Copy(SlideParts::default());
    loop {
        let Some(Ok(Message::Text(t))) = socket.next().await else {
            panic!("no welcome")
        };
        if let ServerMessage::<SlideEngine>::Welcome {
            client,
            snapshot,
            ops,
            revision,
            ..
        } = serde_json::from_str(t.as_str()).unwrap()
        {
            assert_eq!(snapshot, "ZGVjaw==", "the seed comes back as it was given");
            copy.read_snapshot(b"deck").unwrap();
            for op in &ops {
                copy.apply_remote(op).unwrap();
            }
            return Client {
                socket,
                copy,
                session: ClientSession::new(client, revision),
                seen_presence: Vec::new(),
            };
        }
    }
}

impl Client {
    /// Make an edit locally and note it for sending, as the editor does.
    fn edit(&mut self, op: Operation) {
        let inverse = apply(&mut self.copy.0, op.clone()).unwrap();
        let wire = WireOp::of(&op, &inverse, &|_| None, &|_, _| false).unwrap();
        self.session.record(wire);
    }

    async fn flush(&mut self) {
        if let Some(s) = self.session.flush() {
            send(&mut self.socket, &ClientMessage::Submit(s)).await;
        }
    }

    /// Read what has arrived for a moment, applying it.
    async fn pump(&mut self) {
        while let Ok(Some(Ok(Message::Text(t)))) =
            tokio::time::timeout(Duration::from_millis(150), self.socket.next()).await
        {
            match serde_json::from_str::<ServerMessage<SlideEngine>>(t.as_str()).unwrap() {
                ServerMessage::Ack { through, revision } => {
                    self.session.acknowledge(through, revision)
                }
                ServerMessage::Apply { ops, revision } => {
                    for op in &ops {
                        self.session.receive(&mut self.copy, op, revision).unwrap();
                    }
                }
                ServerMessage::Presence {
                    client, presence, ..
                } => self.seen_presence.push((client, presence)),
                _ => {}
            }
            self.flush().await;
        }
    }
}

#[tokio::test]
async fn two_clients_see_each_others_edits_and_cursors_and_converge() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(casual_slide_collab_server::serve(
        listener,
        casual_slide_collab_server::Limits::default(),
    ));
    let url = format!("ws://127.0.0.1:{port}/collab?room=test-room-0001");

    let mut ada = join(&url, "Ada").await;
    let mut bo = join(&url, "Bo").await;
    assert_ne!(ada.session.id(), bo.session.id());

    // Both type into the same box at once, and both move it.
    ada.edit(Operation::InsertText {
        object: BOX,
        at: 5,
        text: ", dear".to_owned(),
    });
    bo.edit(Operation::InsertText {
        object: BOX,
        at: 11,
        text: "!".to_owned(),
    });
    bo.edit(Operation::DeleteRange {
        object: BOX,
        start: 0,
        end: 1,
    });
    let moved = |x: i64| {
        WireOp::SetTransform {
            node: Id::from(BOX),
            transform: Some(WireTransform {
                x: Some(x),
                y: Some(0),
                cx: Some(100),
                cy: Some(100),
                ..Default::default()
            }),
        }
        .operation(&|_, _| None)
        .unwrap()
    };
    ada.edit(moved(10));
    bo.edit(moved(20));
    // Presence: Bo's caret.
    send(
        &mut bo.socket,
        &ClientMessage::Presence(Presence {
            slide: 0,
            shapes: vec![2],
            caret: Some(casual_slide_transaction::collab::Caret {
                node: 2,
                offset: 3,
                anchor: 3,
            }),
        }),
    )
    .await;
    ada.flush().await;
    bo.flush().await;
    for _ in 0..6 {
        ada.pump().await;
        bo.pump().await;
    }
    assert!(!ada.session.has_unacknowledged() && !bo.session.has_unacknowledged());
    assert_eq!(text(&ada.copy.0), "ello, dear world!");
    assert_eq!(ada.copy.0, bo.copy.0, "both copies are the same deck");
    assert!(
        ada.seen_presence
            .iter()
            .any(|(c, p)| *c == bo.session.id() && p.caret.is_some_and(|c| c.offset == 3)),
        "Ada sees Bo's caret"
    );

    // A third joins late and catches up from the seed and the log.
    let cy = join(&url, "Cy").await;
    assert_eq!(cy.copy.0, ada.copy.0, "a late joiner reaches the same deck");
}

#[tokio::test]
async fn a_malformed_message_is_refused_and_the_connection_stays() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(casual_slide_collab_server::serve(
        listener,
        casual_slide_collab_server::Limits::default(),
    ));
    let url = format!("ws://127.0.0.1:{port}/collab?room=test-room-0002");
    let mut ada = join(&url, "Ada").await;
    ada.socket
        .send(Message::Text(
            "{\"type\":\"submit\",\"ops\":[{\"op\":\"nope\"}]}".into(),
        ))
        .await
        .unwrap();
    let Some(Ok(Message::Text(t))) = ada.socket.next().await else {
        panic!("no answer")
    };
    assert!(t.as_str().contains("CS-COL-0004"), "{t}");
    send(&mut ada.socket, &ClientMessage::Ping { nonce: 7 }).await;
    let Some(Ok(Message::Text(t))) = ada.socket.next().await else {
        panic!("no pong")
    };
    assert!(t.as_str().contains("pong"), "{t}");
}
