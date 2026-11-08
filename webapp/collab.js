// The browser half of collaboration: a socket, and the protocol over it.
//
// Copied from OpenCalc's `webapp/collab.js` at `b23c67c` (webapp/README.md) and
// changed only where slides need it (`COL-04`):
// - The engine runs in a worker here, so every engine call is asynchronous;
//   messages are handled one at a time, in arrival order, through `serial`.
// - A join carries the deck as `seed` (base64) for a room nobody is in yet, and
//   every welcome hands back the room's seed and its log, which the engine opens
//   and replays, so all copies start from the same reading of the same bytes.
// - No resume (`COL-05`): a dropped connection joins again from the room's seed
//   and log, and an edit that was never acknowledged is reported as lost, as
//   OpenCalc's first join path does.
// - No redirect between nodes, no recalculation budget: there is one server and no
//   formula.
//
// Everything under this file is Rust — the transform, the client session, the
// wire format, the server. This carries messages between them, and owns exactly
// what the engine deliberately does not: the socket, the reconnect, the backoff.

/// How long to wait before the first reconnect, and the ceiling it doubles to.
const RETRY_FLOOR_MS = 250;
const RETRY_CEILING_MS = 15_000;

/// How often to send an application heartbeat: presence expires on silence, and a
/// participant reading rather than typing is still present.
const HEARTBEAT_MS = 10_000;

/// How often to ask the server whether this connection is still alive, and how
/// long to wait for the answer: the only way to notice a half-open socket.
const PING_MS = 15_000;
const PONG_DEADLINE_MS = 10_000;

/// How long local edits accumulate before being sent. Not zero: a submission per
/// keystroke is a submission per keystroke for everyone else to apply.
const FLUSH_MS = 120;

/// Bytes to base64, in slices, since a deck is too large for one `fromCharCode`.
export function toBase64(bytes) {
  let s = "";
  for (let i = 0; i < bytes.length; i += 0x8000) {
    s += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
  }
  return btoa(s);
}

function fromBase64(text) {
  const s = atob(text);
  const out = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) out[i] = s.charCodeAt(i);
  return out;
}

/// Join a collaborative session.
///
/// `engine` is `{ call(fn, ...args), load(bytes) }` over the worker. `seed` is a
/// function giving the deck's bytes, asked for only when this participant may be
/// the room's first. `onStatus`, `onDocument` and `onPresence` are how the host
/// learns what happened; none is optional, because a caller that ignores a status
/// is a caller that shows a stale document forever.
export function collaborate({ url, room, name, seed, engine, onStatus, onDocument, onPresence }) {
  let socket = null;
  let closed = false;
  let retryMs = RETRY_FLOOR_MS;
  let heartbeat = null;
  let flusher = null;
  let pinger = null;
  let joined = false;
  let everJoined = false;
  let generation = 0;
  let nonce = 0;
  let awaiting = null;
  let flushing = false;
  let queue = Promise.resolve();

  const status = (state, detail) => onStatus({ state, detail });
  const serial = (fn) => { queue = queue.then(fn, fn); return queue; };

  async function open() {
    if (closed) return;
    const mine = ++generation;
    const current = () => mine === generation;
    const target = new URL(url);
    target.searchParams.set("room", room);
    socket = new WebSocket(target);

    socket.onopen = async () => {
      status("connecting");
      const protocol = await engine.call("protocol_version");
      // The deck goes along in case the room is new (or ended while this one was
      // away); the server ignores it otherwise.
      const bytes = await seed();
      send({ type: "join", protocol, name, seed: bytes ? toBase64(bytes) : null });
    };

    socket.onmessage = (event) => {
      let message;
      try {
        message = JSON.parse(event.data);
      } catch {
        return;
      }
      serial(async () => {
        if (!current()) return;
        try {
          await receive(message);
        } catch (err) {
          // An arrival that does not apply: this copy no longer agrees, so it joins
          // again from the room's seed and log.
          status("desynced", String(err && err.message ? err.message : err));
          rejoin();
        }
      });
    };

    socket.onclose = () => {
      if (!current()) return;
      stopTimers();
      joined = false;
      if (closed) return;
      const wait = Math.random() * retryMs;
      retryMs = Math.min(retryMs * 2, RETRY_CEILING_MS);
      status("reconnecting", Math.round(wait));
      setTimeout(open, wait);
    };

    socket.onerror = () => {};
  }

  async function receive(message) {
    switch (message.type) {
      case "welcome": {
        if (message.protocol !== (await engine.call("protocol_version"))) {
          status("stopped", "protocolVersion");
          close();
          break;
        }
        const losing = everJoined && (await engine.call("collab_unacknowledged"));
        // Every participant, the first included, starts from the room's own bytes.
        await engine.load(fromBase64(message.snapshot));
        await engine.call("collab_replay", JSON.stringify(message.ops));
        await engine.call("collab_begin", message.client, message.revision);
        joined = true;
        everJoined = true;
        retryMs = RETRY_FLOOR_MS;
        if (losing) status("lost", "unsentEdits");
        onDocument({ reason: "joined", client: message.client, colour: message.colour, revision: message.revision });
        if (!losing) status("live");
        startTimers();
        break;
      }
      case "pong":
        if (awaiting?.nonce === message.nonce) awaiting = null;
        break;
      case "ack":
        await engine.call("collab_acknowledge", message.through, message.revision);
        break;
      case "apply": {
        const touched = await engine.call("collab_receive", JSON.stringify(message.ops), message.revision);
        onDocument({ reason: "remote", revision: message.revision, touched });
        break;
      }
      case "presence":
        onPresence({ kind: "here", ...message });
        break;
      case "departed":
        onPresence({ kind: "gone", client: message.client });
        break;
      case "refused":
        status("refused", message.reason);
        break;
      case "stopped":
        status("stopped", message.reason);
        close();
        break;
      default:
        break;
    }
  }

  function startTimers() {
    stopTimers();
    heartbeat = setInterval(() => send({ type: "heartbeat" }), HEARTBEAT_MS);
    flusher = setInterval(flush, FLUSH_MS);
    pinger = setInterval(ping, PING_MS);
  }

  function stopTimers() {
    clearInterval(heartbeat);
    clearInterval(flusher);
    clearInterval(pinger);
    heartbeat = flusher = pinger = null;
    awaiting = null;
  }

  function ping() {
    if (socket?.readyState !== WebSocket.OPEN) return;
    if (awaiting && Date.now() - awaiting.sentAt > PONG_DEADLINE_MS) {
      status("stale");
      awaiting = null;
      socket.close();
      return;
    }
    if (awaiting) return;
    nonce += 1;
    awaiting = { nonce, sentAt: Date.now() };
    send({ type: "ping", nonce });
  }

  /// Send what has been edited since the last flush. Ordered with arrivals, so an
  /// arrival is never rebased over half a flush.
  function flush() {
    if (!joined || flushing || socket?.readyState !== WebSocket.OPEN) return;
    flushing = true;
    serial(async () => {
      try {
        const submission = await engine.call("collab_flush");
        if (submission && socket?.readyState === WebSocket.OPEN) socket.send(submission);
      } catch (err) {
        status("refused", String(err && err.message ? err.message : err));
      } finally {
        flushing = false;
      }
    });
  }

  function send(message) {
    if (socket?.readyState === WebSocket.OPEN) socket.send(JSON.stringify(message));
  }

  /// Say where this participant is: the slide, the selected shapes, the caret.
  async function present(slide, shapes, caret) {
    if (!joined || socket?.readyState !== WebSocket.OPEN) return;
    const c = caret || { node: 0, offset: 0, anchor: 0 };
    const text = await engine.call("collab_presence", slide, shapes, c.node, c.offset, c.anchor);
    if (socket?.readyState === WebSocket.OPEN) socket.send(text);
  }

  function close() {
    closed = true;
    stopTimers();
    send({ type: "leave" });
    engine.call("collab_end").catch(() => {});
    socket?.close();
    socket = null;
  }

  function rejoin() {
    if (closed) return;
    joined = false;
    stopTimers();
    socket?.close();
  }

  open();
  return { present, close, flush };
}
