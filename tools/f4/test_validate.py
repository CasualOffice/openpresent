#!/usr/bin/env python3
"""F4's own failure paths, which a deck cannot reach. Run by crates/casual-slide-ooxml/tests/corpus.rs.

    python3 tools/f4/test_validate.py
"""
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import validate  # noqa: E402


class Allowances(unittest.TestCase):
    def row(self, count="1", old="<a/>", new="<b/>"):
        return {"id": "T", "file": "x.xsd", "count": count, "old": old, "new": new}

    def assert_stops(self, texts, rows):
        with self.assertRaises(SystemExit) as stop:
            validate.apply_allowances(texts, rows)
        self.assertEqual(stop.exception.code, 2)

    def test_every_recorded_allowance_applies_to_the_vendored_schemas(self):
        texts = {p.name: p.read_bytes().decode("utf8") for p in (validate.SCHEMAS / "ooxml").glob("*.xsd")}
        before = dict(texts)
        validate.apply_allowances(texts, validate.allowances())
        self.assertNotEqual(texts, before)

    def test_every_allowance_has_evidence_and_a_reason(self):
        rows = validate.allowances()
        self.assertTrue(rows)
        for row in rows:
            self.assertTrue(row["evidence"].strip() and row["reason"].strip(), row["id"])

    def test_an_edit_that_matches_too_few_times_stops_the_tool(self):
        self.assert_stops({"x.xsd": "<a/>"}, [self.row(count="2")])
        self.assert_stops({"x.xsd": "<c/>"}, [self.row()])

    def test_an_edit_that_matches_too_often_stops_the_tool(self):
        self.assert_stops({"x.xsd": "<a/><a/>"}, [self.row()])

    def test_an_edit_for_a_missing_file_stops_the_tool(self):
        self.assert_stops({}, [self.row()])

    def test_a_newline_in_an_edit_is_a_crlf(self):
        texts = validate.apply_allowances({"x.xsd": "<a/>\r\n<c/>"}, [self.row(old="<a/>\\n<c/>", new="<b/>")])
        self.assertEqual(texts["x.xsd"], "<b/>")


class Records(unittest.TestCase):
    def test_a_row_of_the_wrong_shape_stops_the_tool(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp) / "t.tsv"
            path.write_text("# a comment\none\ttwo\n", encoding="utf8")
            with self.assertRaises(SystemExit) as stop:
                validate.tsv_rows(path, ("a", "b", "c"))
            self.assertEqual(stop.exception.code, 2)

    def test_a_row_covers_only_its_deck_part_and_message(self):
        rows = [{"deck": "sdk-*.pptx", "part": "ppt/*", "message": "union type"}]
        self.assertTrue(validate.explained_by(rows, "sdk-a.pptx", "ppt/a.xml", "x union type y"))
        self.assertFalse(validate.explained_by(rows, "other.pptx", "ppt/a.xml", "x union type y"))
        self.assertFalse(validate.explained_by(rows, "sdk-a.pptx", "docProps/a.xml", "x union type y"))
        self.assertFalse(validate.explained_by(rows, "sdk-a.pptx", "ppt/a.xml", "not expected"))


class SchemaErrors(unittest.TestCase):
    def test_a_schema_that_will_not_compile_is_reported_for_every_file(self):
        """It reached the caller keyed by bare name, and the tool died with a KeyError."""
        with tempfile.TemporaryDirectory() as tmp:
            work = pathlib.Path(tmp)
            (work / "broken.xsd").write_text("<not-a-schema/>", encoding="utf8")
            part = work / "p1.xml"
            part.write_text("<a/>", encoding="utf8")
            results = validate.run_xmllint(str(work / "broken.xsd"), [part], work)
        self.assertEqual(list(results), [str(part)])
        self.assertEqual(results[str(part)][0], "schema-error")


if __name__ == "__main__":
    unittest.main()
