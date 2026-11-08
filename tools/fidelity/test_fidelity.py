#!/usr/bin/env python3
"""The fidelity pipeline's safety and its comparison, each seen failing on a seeded case.

    python3 tools/fidelity/test_fidelity.py

The tests that start LibreOffice are skipped when soffice is absent. No test renders
anything from fixtures/hostile: those decks only ever reach the preflight.
"""
import json
import pathlib
import os
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import unittest
import zipfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import compare as cmp  # noqa: E402
import fidelity  # noqa: E402
import render as rnd  # noqa: E402

REPO = pathlib.Path(__file__).resolve().parents[2]
FIXTURES = REPO / "fixtures"
# Off unless asked for, or every run of this file starts LibreOffice on the machine.
HAVE_SOFFICE = bool(shutil.which("soffice")) and os.environ.get("OPENPRESENT_ALLOW_SOFFICE") == "1"
T = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
SLIDE_RELS = "ppt/slides/_rels/slide1.xml.rels"
RELS_HEAD = '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'


def mutate(source, target, part, change):
    """A copy of a deck with one part changed."""
    with zipfile.ZipFile(source) as src, zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as out:
        for name in src.namelist():
            data = src.read(name)
            if name == part:
                data = change(data.decode("utf8")).encode("utf8")
            out.writestr(name, data)


def add_rel(xml, typ, target, mode="External"):
    return xml.replace("</Relationships>", f'<Relationship Id="rId99" Type="{T}/{typ}" Target="{target}" TargetMode="{mode}"/></Relationships>')


class Preflight(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = pathlib.Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def seeded(self, change, part=SLIDE_RELS):
        path = self.dir / "deck.pptx"
        mutate(FIXTURES / "generated/minimal.pptx", path, part, change)
        return path

    def refused(self, path, fragment):
        with self.assertRaises(rnd.Refused) as caught:
            rnd.preflight(path)
        self.assertIn(fragment, str(caught.exception))

    def test_every_hostile_fixture_is_refused_by_path(self):
        decks = list((FIXTURES / "hostile").glob("*.bin"))
        self.assertGreater(len(decks), 30)
        for deck in decks:
            self.refused(deck, "never rendered")

    def test_a_renamed_copy_of_a_hostile_fixture_is_refused_by_checksum(self):
        copy = self.dir / "innocent.pptx"
        shutil.copy(FIXTURES / "hostile/external-image-tracking.bin", copy)
        self.refused(copy, "same bytes")

    def test_render_refuses_before_starting_soffice(self):
        real = shutil.which
        rnd.shutil.which = lambda name: self.fail(f"looked for {name}")
        try:
            with self.assertRaises(rnd.Refused):
                rnd.render(FIXTURES / "hostile/vba-project.bin", self.dir / "out")
        finally:
            rnd.shutil.which = real

    def test_external_references_are_refused(self):
        for typ, target in (("image", "https://tracker.example/p.png"), ("image", "\\\\attacker.example\\s\\a.png"),
                            ("image", "file:///etc/passwd"), ("attachedTemplate", "https://a.example/t.potx")):
            self.refused(self.seeded(lambda x, t=typ, g=target: add_rel(x, t, g)), "")

    def test_an_absolute_target_without_external_mode_is_refused(self):
        self.refused(self.seeded(lambda x: add_rel(x, "image", "http://a.example/p.png", "Internal")), "outside the package")

    def test_a_target_above_the_root_is_refused(self):
        self.refused(self.seeded(lambda x: add_rel(x, "image", "../../../../etc/passwd", "Internal")), "above the package root")

    def test_an_external_hyperlink_is_not_followed_and_is_allowed(self):
        self.assertIn("sha256", rnd.preflight(self.seeded(lambda x: add_rel(x, "hyperlink", "https://example.com/"))))

    def test_doctype_is_refused(self):
        self.refused(self.seeded(lambda x: '<!DOCTYPE p [<!ENTITY a "b">]>' + x[x.index("<p:sld"):], "ppt/slides/slide1.xml"), "DOCTYPE")

    def test_macros_and_ole_are_refused(self):
        path = self.dir / "deck.pptx"
        with zipfile.ZipFile(FIXTURES / "generated/minimal.pptx") as src, zipfile.ZipFile(path, "w") as out:
            for name in src.namelist():
                out.writestr(name, src.read(name))
            out.writestr("ppt/vbaProject.bin", b"x")
        self.refused(path, "active content")
        self.refused(self.seeded(lambda x: add_rel(x, "oleObject", "../embeddings/o.bin", "Internal")), "active content")

    def test_svg_with_a_remote_reference_is_refused(self):
        path = self.dir / "deck.pptx"
        with zipfile.ZipFile(FIXTURES / "generated/minimal.pptx") as src, zipfile.ZipFile(path, "w") as out:
            for name in src.namelist():
                out.writestr(name, src.read(name))
            out.writestr("ppt/media/a.svg", '<svg xmlns:x="http://www.w3.org/1999/xlink"><image x:href="https://a.example/p.png"/></svg>')
        self.refused(path, "refers outside")

    def test_a_bomb_stops_at_the_cap_not_at_the_declared_size(self):
        original = rnd.MAX_TOTAL
        rnd.MAX_TOTAL = 1 << 20
        try:
            path = self.dir / "bomb.pptx"
            with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as out:
                out.writestr("ppt/media/a.bin", b"\0" * (8 << 20))
            self.refused(path, "inflates past")
        finally:
            rnd.MAX_TOTAL = original

    def test_clean_decks_pass(self):
        for name in ("minimal", "image", "preset-shapes", "area-text"):
            self.assertGreater(rnd.preflight(FIXTURES / f"generated/{name}.pptx")["xml_parts"], 3)


def flat(width, height, colour=(255, 255, 255)):
    return width, height, bytes(colour) * (width * height)


def with_box(image, x0, y0, x1, y1, colour):
    width, height, rgb = image
    out = bytearray(rgb)
    for y in range(y0, y1):
        out[(y * width + x0) * 3:(y * width + x1) * 3] = bytes(colour) * (x1 - x0)
    return width, height, bytes(out)


class Images(unittest.TestCase):
    def test_identical_is_one(self):
        a = with_box(flat(320, 180), 10, 10, 50, 50, (0, 0, 0))
        result = cmp.compare_images(a, a)
        self.assertTrue(result["identical"])
        self.assertEqual(result["ssim"], 1.0)
        self.assertEqual(result["regions_flagged"], [])

    def test_a_small_loss_scores_high_overall_but_its_region_is_flagged(self):
        """The reason for regions: a whole-page number barely moves for a missing bullet."""
        a = with_box(flat(1280, 720), 40, 40, 1240, 60, (0, 0, 0))
        b = with_box(a, 640, 360, 656, 376, (20, 20, 20))
        result = cmp.compare_images(a, b)
        self.assertGreater(result["ssim"], 0.995)
        self.assertEqual(len(result["regions_flagged"]), 1)
        region = result["regions_flagged"][0]
        self.assertEqual((region["col"], region["row"]), (8, 4))
        self.assertLess(region["ssim"], 0.97)

    def test_a_colour_swap_of_equal_luminance_is_seen(self):
        """SSIM on luminance alone would call this nearly identical; delta-E must not."""
        a = with_box(flat(320, 180), 0, 0, 320, 180, (200, 40, 40))
        b = with_box(flat(320, 180), 0, 0, 320, 180, (40, 95, 150))
        result = cmp.compare_images(a, b)
        self.assertGreater(result["changed_pixels"], 0.99)

    def test_png_round_trip_and_diff(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp) / "a.png"
            image = with_box(flat(64, 32), 3, 3, 9, 9, (10, 200, 30))
            cmp.write_png(path, *image)
            self.assertEqual(cmp.read_png(path), image)
            diff = pathlib.Path(tmp) / "diff.png"
            cmp.compare_images(image, flat(64, 32), diff_path=diff)
            self.assertEqual(cmp.read_png(diff)[:2], (64, 32))

    def test_png_limits(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp) / "bad.png"
            path.write_bytes(b"not a png")
            with self.assertRaises(cmp.ImageError):
                cmp.read_png(path)

    def test_sizes_differ_means_resampled_and_said_so(self):
        with tempfile.TemporaryDirectory() as tmp:
            a, b = pathlib.Path(tmp) / "a.png", pathlib.Path(tmp) / "b.png"
            cmp.write_png(a, *flat(80, 40))
            cmp.write_png(b, *flat(160, 80))
            result = cmp.compare_files(a, b)
            self.assertIn("resampled", result["note"])
            self.assertTrue(result["identical"])


class Words(unittest.TestCase):
    PAGE = {"width": 960.0, "height": 540.0}

    def page(self, *words):
        return dict(self.PAGE, words=[{"text": t, "x0": x, "y0": y, "x1": x + 10, "y1": y + 10} for t, x, y in words])

    def test_missing_extra_and_moved(self):
        result = cmp.compare_words(self.page(("one", 10, 10), ("two", 50, 10), ("gone", 90, 10)),
                                   self.page(("one", 10, 10), ("two", 55, 10), ("new", 90, 10)))
        self.assertEqual([m["text"] for m in result["missing"]], ["gone"])
        self.assertEqual([m["text"] for m in result["extra"]], ["new"])
        self.assertEqual([m["text"] for m in result["moved"]], ["two"])
        self.assertEqual(result["largest_move_pt"], 5.0)

    def test_repeated_words_pair_by_nearest(self):
        result = cmp.compare_words(self.page(("a", 10, 10), ("a", 100, 10)), self.page(("a", 100.5, 10), ("a", 10.5, 10)))
        self.assertEqual(result["moved"], [])

    def test_a_page_of_another_size_compares_in_proportion(self):
        other = {"width": 480.0, "height": 270.0, "words": [{"text": "x", "x0": 25.0, "y0": 25.0, "x1": 30.0, "y1": 30.0}]}
        result = cmp.compare_words(self.page(("x", 50, 50)), other)
        self.assertEqual(result["largest_move_pt"], 0.0)


class Labels(unittest.TestCase):
    """LibreOffice results are a second opinion, in every word they use."""

    def test_libreoffice_on_either_side_is_never_f6(self):
        lo, pp, os_ = {"producer": "libreoffice"}, {"producer": "powerpoint-win-16.90"}, {"producer": "openslide"}
        self.assertEqual(fidelity.tier_of(lo, lo), rnd.SECOND_OPINION)
        self.assertEqual(fidelity.tier_of(pp, lo), rnd.SECOND_OPINION)
        self.assertEqual(fidelity.tier_of(lo, pp), rnd.SECOND_OPINION)
        self.assertEqual(fidelity.tier_of(os_, {"producer": "keynote-14"}), rnd.SECOND_OPINION)
        self.assertEqual(fidelity.tier_of(pp, os_), "F6-pixels")

    def test_no_threshold_gives_no_result(self):
        result = {"tier": rnd.SECOND_OPINION, "pages": [], "pages_a": 1, "pages_b": 1}
        self.assertEqual(fidelity.assess(result, None), "no threshold: measured only")

    def test_a_second_opinion_within_thresholds_still_says_nothing_about_powerpoint(self):
        result = {"tier": rnd.SECOND_OPINION, "pages": [], "pages_a": 1, "pages_b": 1}
        words = fidelity.assess(result, {"min_ssim": 0, "min_region_ssim": 0, "max_changed_pixels": 1, "max_text_move_pt": 1})
        self.assertIn("says nothing about PowerPoint", words)
        self.assertNotIn("match", words)

    def test_a_png_reference_set_loads_and_says_it_has_no_text(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)
            folder = root / "refs/powerpoint-win-16.90/deck"
            folder.mkdir(parents=True)
            cmp.write_png(folder / "page-001.png", *flat(32, 18))
            (folder / "meta.json").write_text(json.dumps({"producer_version": "16.90"}))
            ref = fidelity.load_reference(root / "refs", "powerpoint-win-16.90", "deck", root / "work")
            meta = fidelity.read_meta(ref)
            self.assertEqual(meta["producer"], "powerpoint-win-16.90")
            self.assertFalse(meta["text_compared"])
            self.assertIsNone(fidelity.load_reference(root / "refs", "powerpoint-win-16.90", "other", root / "work"))


@unittest.skipUnless(HAVE_SOFFICE, "soffice not installed")
class EndToEnd(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = pathlib.Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_a_seeded_loss_is_located(self):
        """One label and one shape removed from 32: the score stays high, and the region and the word are named."""
        source = FIXTURES / "generated/preset-shapes.pptx"
        changed = self.dir / "changed.pptx"

        def remove_star5(xml):
            start = xml.index('name="star5')
            start = xml.rindex("<p:sp>", 0, start)
            end = xml.index("</p:sp>", start) + len("</p:sp>")
            xml = xml[:start] + xml[end:]
            word = xml.index("<a:t>star5</a:t>")
            start = xml.rindex("<p:sp>", 0, word)
            end = xml.index("</p:sp>", word) + len("</p:sp>")
            return xml[:start] + xml[end:]

        mutate(source, changed, "ppt/slides/slide1.xml", remove_star5)
        rnd.render(source, self.dir / "a")
        rnd.render(changed, self.dir / "b")
        result = fidelity.compare_sets(self.dir / "a", self.dir / "b", self.dir / "cmp")
        self.assertEqual(result["tier"], rnd.SECOND_OPINION)
        self.assertIn("SECOND OPINION", result["banner"])
        first = result["pages"][0]
        self.assertEqual([m["text"] for m in first["text"]["missing"]], ["star5"])
        self.assertGreater(len(first["image"]["regions_flagged"]), 0)
        self.assertEqual(result["pages"][1]["image"]["identical"], True)
        self.assertTrue((self.dir / "cmp/diff-001.png").exists())
        (cols, rows) = first["image"]["grid"]
        for region in first["image"]["regions_flagged"]:  # star5 is the first shape of row 3 of 4, at the left
            self.assertLessEqual(region["col"], 2 * cols // 8)
            self.assertGreaterEqual(region["row"], rows // 2 - 1)

    def test_the_denial_stops_a_connection_that_succeeds_without_it(self):
        """The control is the point: the same code connects outside the sandbox and not inside."""
        listener = socket.socket()
        listener.bind(("127.0.0.1", 0))
        listener.listen(4)
        port = listener.getsockname()[1]
        probe = f"import socket; socket.create_connection(('127.0.0.1', {port}), 5).close()"
        prefix, label = rnd.isolation_prefix([str(self.dir)])
        self.assertIsNotNone(prefix, "no network denial available here")
        try:
            self.assertEqual(subprocess.run([sys.executable, "-I", "-c", probe], capture_output=True, timeout=30).returncode, 0)
            denied = subprocess.run(prefix + [sys.executable, "-I", "-c", probe], capture_output=True, timeout=30)
        finally:
            listener.close()
        self.assertNotEqual(denied.returncode, 0, label)
        self.assertRegex(denied.stderr.decode(), "Operation not permitted|Network is unreachable|PermissionError")

    @unittest.skipUnless(sys.platform == "darwin", "the write limit is the macOS profile's")
    def test_writes_outside_temp_are_denied(self):
        target = REPO / "tools/fidelity/.sandbox-probe"
        prefix, _ = rnd.isolation_prefix([str(self.dir)])
        code = f"open({str(target)!r}, 'w').write('x')"
        run = subprocess.run(prefix + [sys.executable, "-I", "-c", code], capture_output=True, timeout=30)
        self.assertNotEqual(run.returncode, 0)
        self.assertFalse(target.exists())

    def test_a_linked_image_past_the_preflight_makes_no_connection(self):
        """Past the preflight on purpose: a deck of ours with an image on a loopback listener. Note what this does
        not show: LibreOffice 26.8 made no connection for this deck without the sandbox either (tried by hand,
        2026-11-07), so this guards a future change of behaviour, and the test above is what proves the denial."""
        listener = socket.socket()
        listener.bind(("127.0.0.1", 0))
        listener.listen(4)
        listener.settimeout(0.2)
        port = listener.getsockname()[1]
        connections = []
        stop = threading.Event()

        def accept():
            while not stop.is_set():
                try:
                    connections.append(listener.accept()[0])
                except OSError:
                    pass

        thread = threading.Thread(target=accept)
        thread.start()
        deck = self.dir / "fetches.pptx"
        url = f"http://127.0.0.1:{port}/p.png"

        def picture(xml):
            pic = ('<p:pic><p:nvPicPr><p:cNvPr id="9" name="p"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:link="rId99"/>'
                   '<a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm>'
                   '<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic>')
            return xml.replace("</p:spTree>", pic + "</p:spTree>")

        with zipfile.ZipFile(FIXTURES / "generated/minimal.pptx") as src, zipfile.ZipFile(deck, "w") as out:
            for name in src.namelist():
                data = src.read(name).decode("utf8")
                if name == SLIDE_RELS:
                    data = add_rel(data, "image", url)
                elif name == "ppt/slides/slide1.xml":
                    data = picture(data)
                out.writestr(name, data)
        with self.assertRaises(rnd.Refused):
            rnd.preflight(deck)
        try:
            pdf, isolation = rnd.convert_to_pdf(deck, self.dir / "out")
        finally:
            stop.set()
            thread.join()
            listener.close()
        for c in connections:
            c.close()
        self.assertTrue(pdf.exists())
        self.assertIn("network denied" if sys.platform == "darwin" else "network namespace", isolation)
        self.assertEqual(connections, [])


try:
    import numpy  # noqa: F401
    import PIL  # noqa: F401
    HAVE_SCORE = True
except ImportError:
    HAVE_SCORE = False


@unittest.skipUnless(HAVE_SCORE, "score.py needs numpy and Pillow (CI's gallery job has them)")
class Score(unittest.TestCase):
    def test_a_reference_one_pixel_wider_is_cropped_not_resampled(self):
        # pdftoppm rounds a 13.333 in page up to 1281 px at 96 dpi; resampling that
        # to 1280 blurs every edge, and the blur scored as our difference (FH-05).
        import score
        from PIL import Image, ImageDraw
        with tempfile.TemporaryDirectory() as tmp:
            ours, lo = pathlib.Path(tmp) / "ours.png", pathlib.Path(tmp) / "lo.png"
            for path, width in ((ours, 400), (lo, 401)):
                image = Image.new("RGB", (width, 100), "white")
                draw = ImageDraw.Draw(image)
                for x in range(0, 400, 8):
                    draw.line((x, 0, x, 99), fill="black")
                image.save(path)
            result = score.score_pair(ours, lo)
            self.assertEqual(result["differ"], 0.0)
            self.assertNotIn("resized", result.get("note", ""))


if __name__ == "__main__":
    unittest.main()
