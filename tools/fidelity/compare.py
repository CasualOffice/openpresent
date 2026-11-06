"""Compare two rendered pages: a perceptual score, per-region difference, and word positions.

Standard library only: PNG decode and encode are here, so nothing needs a venv. It is
fast enough for slides at 96 dpi, around two seconds a page in CPython.

A single whole-page number would hide a missing bullet (docs/51), so every page also
gets a grid of regions. Two measures, since each is blind to what the other sees:

  SSIM on luminance, 8 by 8 windows    structure: edges and text, not a colour swap
  CIE76 delta-E between pixels          colour: a scheme colour resolved wrongly
"""
import pathlib
import struct
import zlib
from collections import Counter
from operator import mul

WINDOW = 8
GRID = (16, 9)
DELTA_E_CHANGED = 5.0      # Roughly "clearly different at a glance" (about 2.3 is just noticeable).
NOTICE_REGION_SSIM = 0.97
NOTICE_REGION_CHANGED = 0.01
TEXT_MOVE_TOLERANCE_PT = 1.0
MAX_PIXELS = 40_000_000
MAX_INFLATED = 400_000_000
C1, C2 = (0.01 * 255) ** 2, (0.03 * 255) ** 2


class ImageError(Exception):
    pass


# ---------------------------------------------------------------- PNG

def read_png(path):
    """(width, height, RGB bytes). Alpha is composited on white. 8 and 16 bit, palette and grey included."""
    data = pathlib.Path(path).read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ImageError(f"{path}: not a PNG")
    pos, idat, palette, trans, head = 8, [], None, None, None
    while pos + 8 <= len(data):
        length, tag = struct.unpack(">I4s", data[pos:pos + 8])
        chunk = data[pos + 8:pos + 8 + length]
        pos += 12 + length
        if tag == b"IHDR":
            head = struct.unpack(">IIBBBBB", chunk)
        elif tag == b"PLTE":
            palette = chunk
        elif tag == b"tRNS":
            trans = chunk
        elif tag == b"IDAT":
            idat.append(chunk)
        elif tag == b"IEND":
            break
    if head is None:
        raise ImageError(f"{path}: no IHDR")
    width, height, depth, color, _, _, interlace = head
    if width * height > MAX_PIXELS or not width or not height:
        raise ImageError(f"{path}: {width} by {height} is over the limit")
    if interlace:
        raise ImageError(f"{path}: interlaced PNG; export without interlacing")
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}.get(color)
    if channels is None or depth not in (8, 16) and not (color == 3 and depth == 8):
        raise ImageError(f"{path}: colour type {color} at depth {depth} is not supported")
    stride = width * channels * depth // 8
    d = zlib.decompressobj()
    raw = d.decompress(b"".join(idat), MAX_INFLATED)
    if d.unconsumed_tail:
        raise ImageError(f"{path}: inflates past the limit")
    if len(raw) < (stride + 1) * height:
        raise ImageError(f"{path}: truncated")
    rows = unfilter(raw, stride, height, channels * depth // 8)
    out = bytearray()
    for row in rows:
        if depth == 16:
            row = row[0::2]
        if color == 2:
            out += row
        elif color == 6:
            out += over_white(row)
        elif color == 0:
            out += bytes(v for g in row for v in (g, g, g))
        elif color == 4:
            out += bytes(v for g, a in zip(row[0::2], row[1::2]) for v in (((g * a + 255 * (255 - a)) // 255),) * 3)
        elif color == 3:
            if palette is None:
                raise ImageError(f"{path}: no palette")
            alpha = trans or b""
            for idx in row:
                r, g, b = palette[idx * 3:idx * 3 + 3]
                a = alpha[idx] if idx < len(alpha) else 255
                out += bytes(((r * a + 255 * (255 - a)) // 255, (g * a + 255 * (255 - a)) // 255, (b * a + 255 * (255 - a)) // 255))
    return width, height, bytes(out)


def over_white(row):
    if all(a == 255 for a in row[3::4]):
        out = bytearray(len(row) // 4 * 3)
        out[0::3], out[1::3], out[2::3] = row[0::4], row[1::4], row[2::4]
        return out
    out = bytearray()
    for r, g, b, a in zip(row[0::4], row[1::4], row[2::4], row[3::4]):
        out += bytes(((r * a + 255 * (255 - a)) // 255, (g * a + 255 * (255 - a)) // 255, (b * a + 255 * (255 - a)) // 255))
    return out


def unfilter(raw, stride, height, bpp):
    rows = []
    prev = bytes(stride)
    pos = 0
    for _ in range(height):
        kind = raw[pos]
        cur = bytearray(raw[pos + 1:pos + 1 + stride])
        pos += stride + 1
        if kind == 1:
            for i in range(bpp, stride):
                cur[i] = (cur[i] + cur[i - bpp]) & 255
        elif kind == 2:
            cur = bytearray((a + b) & 255 for a, b in zip(cur, prev))
        elif kind == 3:
            for i in range(stride):
                left = cur[i - bpp] if i >= bpp else 0
                cur[i] = (cur[i] + ((left + prev[i]) >> 1)) & 255
        elif kind == 4:
            for i in range(stride):
                a = cur[i - bpp] if i >= bpp else 0
                b = prev[i]
                c = prev[i - bpp] if i >= bpp else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pred = a if pa <= pb and pa <= pc else b if pb <= pc else c
                cur[i] = (cur[i] + pred) & 255
        elif kind != 0:
            raise ImageError(f"PNG filter {kind}")
        rows.append(bytes(cur))
        prev = cur
    return rows


def write_png(path, width, height, rgb):
    rows = b"".join(b"\0" + rgb[y * width * 3:(y + 1) * width * 3] for y in range(height))

    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data))

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
                + chunk(b"IDAT", zlib.compress(rows, 6)) + chunk(b"IEND", b""))


def resample(width, height, rgb, new_w, new_h):
    """Area average, so a reference exported at another size can still be compared."""
    if (width, height) == (new_w, new_h):
        return rgb
    out = bytearray(new_w * new_h * 3)
    xs = [(x * width // new_w, max(x * width // new_w + 1, (x + 1) * width // new_w)) for x in range(new_w)]
    for ny in range(new_h):
        y0, y1 = ny * height // new_h, max(ny * height // new_h + 1, (ny + 1) * height // new_h)
        for nx, (x0, x1) in enumerate(xs):
            r = g = b = n = 0
            for y in range(y0, y1):
                base = y * width * 3
                for x in range(x0, x1):
                    r += rgb[base + x * 3]
                    g += rgb[base + x * 3 + 1]
                    b += rgb[base + x * 3 + 2]
                    n += 1
            i = (ny * new_w + nx) * 3
            out[i], out[i + 1], out[i + 2] = r // n, g // n, b // n
    return bytes(out)


# ---------------------------------------------------------------- pixels

def luminance(width, height, rgb):
    """Rows of 8-bit luma (BT.709 weights on encoded values, which is what SSIM is usually given)."""
    r, g, b = rgb[0::3], rgb[1::3], rgb[2::3]
    y = bytes((54 * a + 183 * c + 19 * d) >> 8 for a, c, d in zip(r, g, b))
    return [y[i * width:(i + 1) * width] for i in range(height)]


_LAB = {}


def lab(r, g, b):
    key = r << 16 | g << 8 | b
    hit = _LAB.get(key)
    if hit is not None:
        return hit

    def linear(v):
        v /= 255
        return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4

    lr, lg, lb = linear(r), linear(g), linear(b)
    x = (0.4124 * lr + 0.3576 * lg + 0.1805 * lb) / 0.95047
    y = 0.2126 * lr + 0.7152 * lg + 0.0722 * lb
    z = (0.0193 * lr + 0.1192 * lg + 0.9505 * lb) / 1.08883

    def f(t):
        return t ** (1 / 3) if t > 0.008856 else 7.787 * t + 16 / 116

    fx, fy, fz = f(x), f(y), f(z)
    value = (116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz))
    if len(_LAB) < 2_000_000:
        _LAB[key] = value
    return value


def compare_images(a, b, grid=GRID, diff_path=None):
    """Compare (w, h, rgb) with (w, h, rgb) of the same size. Returns scores and flagged regions."""
    width, height, ra = a
    if (width, height) != b[:2]:
        raise ImageError("sizes differ; resample first")
    rb = b[2]
    cols, rows = grid
    cell_w, cell_h = width / cols, height / rows
    ssim_sum = [[0.0] * cols for _ in range(rows)]
    ssim_n = [[0] * cols for _ in range(rows)]
    changed = [[0] * cols for _ in range(rows)]
    pixels = [[0] * cols for _ in range(rows)]
    for y in range(height):
        cy = min(int(y / cell_h), rows - 1)
        for cx in range(cols):
            x0, x1 = int(cx * cell_w), int((cx + 1) * cell_w) if cx < cols - 1 else width
            pixels[cy][cx] += x1 - x0

    identical = ra == rb
    total_ssim, windows, changed_total, delta_sum = 0.0, 0, 0, 0.0
    mask = None
    if identical:
        for row in ssim_sum:
            row[:] = [1.0] * cols
        for row in ssim_n:
            row[:] = [1] * cols
        total_ssim, windows = 1.0, 1
    else:
        la, lb = luminance(width, height, ra), luminance(width, height, rb)
        for y in range(0, height - WINDOW + 1, WINDOW):
            cy = min(int((y + WINDOW / 2) / cell_h), rows - 1)
            rows_a, rows_b = la[y:y + WINDOW], lb[y:y + WINDOW]
            same_rows = rows_a == rows_b
            for x in range(0, width - WINDOW + 1, WINDOW):
                cx = min(int((x + WINDOW / 2) / cell_w), cols - 1)
                if same_rows:
                    value = 1.0
                else:
                    wa = b"".join(r[x:x + WINDOW] for r in rows_a)
                    wb = b"".join(r[x:x + WINDOW] for r in rows_b)
                    value = 1.0 if wa == wb else ssim_window(wa, wb)
                total_ssim += value
                windows += 1
                ssim_sum[cy][cx] += value
                ssim_n[cy][cx] += 1
        mask = bytearray(width * height)
        stride = width * 3
        for y in range(height):
            ca, cb = ra[y * stride:(y + 1) * stride], rb[y * stride:(y + 1) * stride]
            if ca == cb:
                continue
            cy = min(int(y / cell_h), rows - 1)
            for x in range(width):
                i = x * 3
                pa, pb = ca[i:i + 3], cb[i:i + 3]
                if pa == pb:
                    continue
                la_, aa, ba = lab(*pa)
                lb_, ab, bb = lab(*pb)
                delta = ((la_ - lb_) ** 2 + (aa - ab) ** 2 + (ba - bb) ** 2) ** 0.5
                delta_sum += delta
                if delta > DELTA_E_CHANGED:
                    changed_total += 1
                    changed[cy][min(int(x / cell_w), cols - 1)] += 1
                    mask[y * width + x] = 1

    regions = []
    for cy in range(rows):
        for cx in range(cols):
            score = ssim_sum[cy][cx] / ssim_n[cy][cx] if ssim_n[cy][cx] else 1.0
            fraction = changed[cy][cx] / pixels[cy][cx] if pixels[cy][cx] else 0.0
            if score < NOTICE_REGION_SSIM or fraction > NOTICE_REGION_CHANGED:
                regions.append({"col": cx, "row": cy, "ssim": round(score, 4), "changed": round(fraction, 4),
                                "box_fraction": [round(cx / cols, 4), round(cy / rows, 4), round((cx + 1) / cols, 4), round((cy + 1) / rows, 4)]})
    regions.sort(key=lambda r: (r["ssim"], -r["changed"]))
    all_cells = [ssim_sum[cy][cx] / ssim_n[cy][cx] for cy in range(rows) for cx in range(cols) if ssim_n[cy][cx]]
    fractions = [changed[cy][cx] / pixels[cy][cx] for cy in range(rows) for cx in range(cols) if pixels[cy][cx]]
    result = {
        "identical": identical,
        "ssim": round(total_ssim / max(windows, 1), 5),
        "worst_region_ssim": round(min(all_cells), 4),
        "worst_region_changed": round(max(fractions), 4),
        "changed_pixels": round(changed_total / (width * height), 6),
        "mean_delta_e": round(delta_sum / (width * height), 4),
        "regions_flagged": regions,
        "grid": [cols, rows],
        "size": [width, height],
    }
    if diff_path and not identical:
        write_diff(diff_path, width, height, rb, mask, regions, cols, rows)
    return result


def ssim_window(a, b):
    n = len(a)
    sa, sb = sum(a), sum(b)
    va = sum(map(mul, a, a))
    vb = sum(map(mul, b, b))
    cov = sum(map(mul, a, b))
    ma, mb = sa / n, sb / n
    var_a, var_b = va / n - ma * ma, vb / n - mb * mb
    c = cov / n - ma * mb
    return ((2 * ma * mb + C1) * (2 * c + C2)) / ((ma * ma + mb * mb + C1) * (var_a + var_b + C2))


def write_diff(path, width, height, rgb, mask, regions, cols, rows):
    """The second image faded, changed pixels red, flagged regions outlined in blue."""
    out = bytearray(len(rgb))
    for i in range(0, len(rgb), 3):
        out[i] = 128 + rgb[i] // 2
        out[i + 1] = 128 + rgb[i + 1] // 2
        out[i + 2] = 128 + rgb[i + 2] // 2
    if mask:
        for p, hit in enumerate(mask):
            if hit:
                out[p * 3:p * 3 + 3] = b"\xd0\x10\x10"
    for r in regions:
        x0, x1 = int(r["col"] * width / cols), min(int((r["col"] + 1) * width / cols), width - 1)
        y0, y1 = int(r["row"] * height / rows), min(int((r["row"] + 1) * height / rows), height - 1)
        for x in range(x0, x1 + 1):
            for y in (y0, y1):
                out[(y * width + x) * 3:(y * width + x) * 3 + 3] = b"\x10\x40\xd0"
        for y in range(y0, y1 + 1):
            for x in (x0, x1):
                out[(y * width + x) * 3:(y * width + x) * 3 + 3] = b"\x10\x40\xd0"
    write_png(path, width, height, bytes(out))


def compare_files(path_a, path_b, diff_path=None):
    a, b = read_png(path_a), read_png(path_b)
    note = None
    if a[:2] != b[:2]:
        note = f"resampled the second from {b[0]}x{b[1]} to {a[0]}x{a[1]}"
        b = (a[0], a[1], resample(*b, a[0], a[1]))
    result = compare_images(a, b, diff_path=diff_path)
    if note:
        result["note"] = note
    return result


# ---------------------------------------------------------------- text

def compare_words(page_a, page_b, tolerance=TEXT_MOVE_TOLERANCE_PT):
    """Words from the two pages, matched by text and then by nearest position, in the first page's points."""
    sx = page_a["width"] / page_b["width"] if page_b["width"] else 1.0
    sy = page_a["height"] / page_b["height"] if page_b["height"] else 1.0
    by_text = {}
    for w in page_a["words"]:
        by_text.setdefault(w["text"], ([], []))[0].append(w)
    for w in page_b["words"]:
        by_text.setdefault(w["text"], ([], []))[1].append(w)
    missing, extra, moved = [], [], []
    largest, matched = 0.0, 0
    for text, (left, right) in sorted(by_text.items()):
        pairs = []
        for i, wa in enumerate(left):
            for j, wb in enumerate(right):
                dx = wa["x0"] - wb["x0"] * sx
                dy = wa["y0"] - wb["y0"] * sy
                pairs.append(((dx * dx + dy * dy) ** 0.5, i, j))
        pairs.sort()
        used_a, used_b = set(), set()
        for dist, i, j in pairs:
            if i in used_a or j in used_b:
                continue
            used_a.add(i)
            used_b.add(j)
            matched += 1
            largest = max(largest, dist)
            if dist > tolerance:
                moved.append({"text": text, "by_pt": round(dist, 2), "from": [round(left[i]["x0"], 1), round(left[i]["y0"], 1)],
                              "to": [round(right[j]["x0"] * sx, 1), round(right[j]["y0"] * sy, 1)]})
        missing += [{"text": text, "at": [round(left[i]["x0"], 1), round(left[i]["y0"], 1)]} for i in range(len(left)) if i not in used_a]
        extra += [{"text": text, "at": [round(right[j]["x0"] * sx, 1), round(right[j]["y0"] * sy, 1)]} for j in range(len(right)) if j not in used_b]
    moved.sort(key=lambda m: -m["by_pt"])
    return {"words_a": len(page_a["words"]), "words_b": len(page_b["words"]), "matched": matched,
            "missing": missing, "extra": extra, "moved": moved, "largest_move_pt": round(largest, 2)}


def word_totals(results):
    total = Counter()
    for r in results:
        total["missing"] += len(r["missing"])
        total["extra"] += len(r["extra"])
        total["moved"] += len(r["moved"])
    return dict(total)
