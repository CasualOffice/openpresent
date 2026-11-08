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
        texts.update((f"ms-extensions/{p.name}", p.read_bytes().decode("utf8"))
                     for p in (validate.SCHEMAS / "ms-extensions").glob("*.xsd"))
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


class Locations(unittest.TestCase):
    WHERE = {"urn:a": "../ooxml/a.xsd", "urn:b": "ms-b.xsd"}

    def test_an_import_is_located_by_its_namespace_once(self):
        text = ('<xsd:import namespace="urn:a" schemaLocation="oart1.xsd"/>'
                '<xsd:import namespace="urn:a" schemaLocation="oart2.xsd"/>'
                '<xsd:import namespace="urn:b"/>')
        self.assertEqual(validate.locate_imports(text, self.WHERE),
                         '<xsd:import namespace="urn:a" schemaLocation="../ooxml/a.xsd"/>'
                         '<xsd:import namespace="urn:b" schemaLocation="ms-b.xsd"/>')

    def test_an_import_of_a_namespace_not_vendored_is_left(self):
        text = '<xsd:import namespace="urn:c" schemaLocation="c.xsd"/>'
        self.assertEqual(validate.locate_imports(text, self.WHERE), text)

    def test_microsofts_chart_schemas_compile(self):
        for schema in ("ms-extensions/ms-odrawxml-22.xsd", "ms-extensions/ms-odrawxml-15.xsd"):
            with tempfile.TemporaryDirectory() as tmp:
                work = pathlib.Path(tmp)
                part = work / "p1.xml"
                part.write_text("<a/>", encoding="utf8")
                status, _ = validate.run_xmllint(schema, [part], work)[str(part)]
                self.assertEqual(status, "invalid", schema)


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


class CommandLine(unittest.TestCase):
    def test_file_names_are_split_to_fit_the_command_line(self):
        """One xmllint over every part of a schema passed Windows' 32,767-character
        command line on a large deck (WinError 206)."""
        names = [f"part-{i:05}.xml" for i in range(5000)]
        runs = list(validate.batches(names))
        self.assertGreater(len(runs), 1)
        self.assertEqual([n for run in runs for n in run], names)
        for run in runs:
            self.assertLessEqual(sum(len(n) + 1 for n in run), validate.COMMAND_LINE_BUDGET)

    def test_a_name_over_the_budget_gets_a_run_of_its_own(self):
        self.assertEqual(list(validate.batches(["a" * 10, "b", "c"], budget=4)),
                         [["a" * 10], ["b", "c"]])

    def test_every_file_of_a_split_run_gets_a_verdict(self):
        with tempfile.TemporaryDirectory() as tmp:
            work = pathlib.Path(tmp)
            parts = []
            for i in range(3):
                part = work / f"p{i}.xml"
                part.write_text("<a/>", encoding="utf8")
                parts.append(part)
            # One file per xmllint process, so every result comes from a separate run.
            original = validate.batches
            validate.batches = lambda paths: original(paths, budget=1)
            try:
                results = validate.run_xmllint("ms-extensions/ms-odrawxml-15.xsd", parts, work)
            finally:
                validate.batches = original
        self.assertEqual(sorted(results), sorted(str(p) for p in parts))
        self.assertTrue(all(status == "invalid" for status, _ in results.values()))


if __name__ == "__main__":
    unittest.main()
