#!/usr/bin/env python3
"""Each gate seen failing on a seeded gap. Run: python3 tools/spec-audit/test_audit.py"""
import contextlib
import csv
import io
import pathlib
import shutil
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import audit  # noqa: E402
import count  # noqa: E402

REPO = pathlib.Path(__file__).resolve().parents[2]
CHECKLIST = REPO / "docs/spec/checklist.tsv"

RUST = r'''
const OFF: &str = "off";

/// "commentName" in a doc comment is not a reference.
pub fn read_xfrm(tree: &XmlTree, id: ElementId) {
    // "commentOnly" in a comment is not a reference either.
    for child in tree.child_elements(id) {
        match tree.local_name(child) {
            OFF => read_point(tree, child),
            "ext" => {
                let cx = tree.attribute(child, "cx");
                let wrong = tree.attribute(child, "kumimoji");
            }
            _ => {}
        }
    }
}

fn read_point(tree: &XmlTree, id: ElementId) {
    let x = tree.attribute(id, "x");
}

fn read_para(tree: &XmlTree, id: ElementId) {
    match tree.attribute(id, "algn").as_deref() {
        Some("just") => {}
        _ => {}
    }
    let lonely = tree.attribute(id, "rotWithShape");
}

pub fn write_outline(out: &mut String) {
    out.push_str(r#"<a:ln w="9525"><a:round/></a:ln>"#);
}

#[cfg(test)]
mod tests {
    fn only_a_test() {
        let z = tree.attribute(id, "sndAc");
    }
}
'''


def make_crates(folder, importer=RUST, exporter="", model=""):
    for name, text in (("casual-slide-import", importer), ("casual-slide-export", exporter), ("casual-slide-model", model)):
        src = folder / name / "src"
        src.mkdir(parents=True)
        (src / "lib.rs").write_text(text, encoding="utf8")


def seed_checklist(target, marks):
    """The real checklist, with status columns set for {row id: {column: value}}."""
    with open(CHECKLIST, encoding="utf8", newline="") as src, open(target, "w", encoding="utf8", newline="") as out:
        reader = csv.DictReader(src, delimiter="\t", quoting=csv.QUOTE_NONE)
        out.write("\t".join(reader.fieldnames) + "\n")
        seen = set()
        for row in reader:
            if row["id"] in marks:
                row.update(marks[row["id"]])
                seen.add(row["id"])
            out.write("\t".join(row[c] for c in reader.fieldnames) + "\n")
    missing = set(marks) - seen
    if missing:
        raise AssertionError(f"not checklist rows: {missing}")


def run_audit(*args):
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = audit.main(["--root", str(REPO), *map(str, args)])
    return code, out.getvalue()


class Lexer(unittest.TestCase):
    def strings(self, text):
        return [v for k, v, _ in audit.strip_test_items(audit.lex(text)) if k == "str"]

    def test_raw_byte_and_escaped_strings(self):
        self.assertEqual(self.strings(r'''let a = "one"; let b = b"two"; let c = r#"th"ree"#; let d = "fo\"ur";'''),
                         ["one", "two", 'th"ree', 'fo"ur'])

    def test_comments_chars_and_lifetimes_do_not_confuse_it(self):
        text = """// "no"
/* "no" /* "nested" */ */
fn f<'a>(x: &'a str) { let c = '"'; let e = '\\''; let s = "yes"; }"""
        self.assertEqual(self.strings(text), ["yes"])

    def test_cfg_test_items_are_dropped(self):
        self.assertEqual(self.strings('let a = "kept"; #[cfg(test)] mod t { fn f() { "gone"; } } let b = "kept2";'), ["kept", "kept2"])


class CodeGate(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = pathlib.Path(self.tmp.name)
        make_crates(self.dir / "crates")

    def tearDown(self):
        self.tmp.cleanup()

    def audit(self, marks, *extra):
        seed_checklist(self.dir / "checklist.tsv", marks)
        return run_audit("--crates", self.dir / "crates", "--checklist", self.dir / "checklist.tsv", "--check",
                         "--gaps", self.dir / "gaps.tsv", *extra)

    def gaps(self):
        with open(self.dir / "gaps.tsv", encoding="utf8", newline="") as f:
            return {r["id"]: r for r in csv.DictReader(f, delimiter="\t", quoting=csv.QUOTE_NONE)}

    def test_a_marked_row_with_a_reference_passes(self):
        code, out = self.audit({"a:CT_Point2D/@x": {"I": "y"}})
        self.assertEqual(code, 0, out[-800:])
        self.assertEqual(self.gaps()["a:CT_Point2D/@x"]["import"], "context")

    def test_seeded_gap_is_reported_and_fails(self):
        code, out = self.audit({"a:CT_Point2D/@y": {"I": "y"}})
        self.assertEqual(code, 1)
        self.assertIn("FAIL a:CT_Point2D/@y: I=y, no reference in casual-slide-import", out)

    def test_a_name_under_the_wrong_owner_is_not_coverage(self):
        # `kumimoji` is read inside the arm for <ext>. The schema puts it on run properties.
        code, out = self.audit({"a:CT_TextCharacterProperties/@kumimoji": {"I": "y"}})
        self.assertEqual(code, 1)
        self.assertIn("referenced only under other owners", out)
        self.assertEqual(self.gaps()["a:CT_TextCharacterProperties/@kumimoji"]["import"], "elsewhere")

    def test_precision_is_stated_not_invented(self):
        self.audit({})
        gaps = self.gaps()
        # Owner through a helper's call site, through a named constant.
        self.assertEqual(gaps["a:CT_Point2D/@x"]["import"], "context")
        self.assertEqual(gaps["a:CT_PositiveSize2D/@cx"]["import"], "context")
        # read_para's own name is no element, and nothing calls it: an owner cannot be determined.
        self.assertEqual(gaps["a:CT_BlipFillProperties/@rotWithShape"]["import"], "name")
        # An enum value inside the match on its attribute.
        self.assertEqual(gaps["a:ST_TextAlignType/=just"]["import"], "context")
        # Markup in a literal says exactly what owns what.
        self.assertEqual(gaps["a:CT_LineProperties/@w"]["import"], "context")
        self.assertEqual(gaps["a:EG_LineJoinProperties/round"]["import"], "context")
        # Comments and test modules are not references.
        self.assertEqual(gaps["p:CT_SlideTransition/sndAc"]["import"], "none")

    def test_strict_refuses_name_precision(self):
        marks = {"a:CT_BlipFillProperties/@rotWithShape": {"I": "y"}}
        self.assertEqual(self.audit(marks)[0], 0)
        code, out = self.audit(marks, "--strict")
        self.assertEqual(code, 1)
        self.assertIn("only `name` precision", out)

    def test_each_crate_answers_for_its_own_column(self):
        code, out = self.audit({"a:CT_Point2D/@x": {"E": "y"}})
        self.assertEqual(code, 1)
        self.assertIn("E=y, no reference in casual-slide-export", out)

    def test_wildcards_need_an_annotation(self):
        row = "a:CT_OfficeArtExtension/xsd:any"
        self.assertEqual(self.audit({row: {"I": "y"}})[0], 1)
        shutil.rmtree(self.dir / "crates")
        make_crates(self.dir / "crates", importer=f"// spec-audit: {row}\npub fn keep() {{}}\n")
        _, out = self.audit({row: {"I": "y"}})
        self.assertEqual(self.gaps()[row]["import"], "annotated")
        self.assertNotIn(f"FAIL {row}: I=y, no reference", out)


class FixtureGate(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = pathlib.Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def schema(self):
        return audit.Schema(REPO)

    def deck(self, relative, parts):
        path = self.dir / "fixtures" / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(path, "w") as z:
            for name, data in parts.items():
                z.writestr(name, data)
        return path

    def test_a_y_row_no_fixture_exercises_fails(self):
        make_crates(self.dir / "crates", importer='fn read_ext(t: T) { t.attribute(i, "cx"); }')
        (self.dir / "fixtures/generated").mkdir(parents=True)
        seed_checklist(self.dir / "c.tsv", {"a:CT_PositiveSize2D/@cx": {"I": "y"}})
        code, out = run_audit("--crates", self.dir / "crates", "--checklist", self.dir / "c.tsv",
                              "--fixtures", self.dir / "fixtures", "--check")
        self.assertEqual(code, 1)
        self.assertIn("FAIL a:CT_PositiveSize2D/@cx: I=y, no fixture exercises it", out)

    def test_coverage_is_by_owner_not_by_bare_name(self):
        schema = self.schema()
        slide = ('<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" '
                 'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:cSld><p:spTree>'
                 '<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>'
                 '<p:sp><p:nvSpPr><p:cNvPr id="2" name="s"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>'
                 '<a:xfrm><a:off x="1" y="2"/><a:ext cx="3" cy="4"/></a:xfrm>'
                 '<a:prstGeom prst="star5"><a:avLst/></a:prstGeom></p:spPr></p:sp></p:spTree></p:cSld></p:sld>')
        path = self.deck("generated/one.pptx", {"ppt/slides/slide1.xml": slide})
        cov = audit.FixtureCoverage(schema)
        cov.scan_deck(path, "generated/one.pptx")
        self.assertGreater(cov.hits["a:CT_Point2D/@x"], 0)
        self.assertGreater(cov.hits["a:ST_ShapeType/=star5"], 0)
        self.assertEqual(cov.hits["a:ST_ShapeType/=star4"], 0)
        self.assertEqual(cov.hits["a:CT_Point3D/@x"], 0)  # `x` elsewhere must not count here
        self.assertEqual(cov.refused, [])

    def test_doctype_is_refused_before_any_tree(self):
        path = self.deck("hostile/x.bin", {"a.xml": '<?xml version="1.0"?><!DOCTYPE r [<!ENTITY a "aaaaaaaaaa">]><r>&a;</r>'})
        cov = audit.FixtureCoverage(self.schema())
        cov.scan_deck(path, "hostile/x.bin", counting=False)
        self.assertEqual(len(cov.refused), 1)
        self.assertIn("DOCTYPE", cov.refused[0])

    def test_utf16_doctype_is_refused_too(self):
        body = '<?xml version="1.0" encoding="UTF-16"?><!DOCTYPE r SYSTEM "file:///etc/passwd"><r/>'.encode("utf-16")
        path = self.deck("hostile/y.bin", {"a.xml": body})
        cov = audit.FixtureCoverage(self.schema())
        cov.scan_deck(path, "hostile/y.bin", counting=False)
        self.assertIn("DOCTYPE", cov.refused[0])

    def test_depth_is_bounded_without_recursion(self):
        path = self.deck("hostile/z.bin", {"a.xml": "<a>" * 5000 + "</a>" * 5000})
        cov = audit.FixtureCoverage(self.schema())
        cov.scan_deck(path, "hostile/z.bin", counting=False)
        self.assertIn("nesting deeper", cov.refused[0])

    def test_hostile_decks_do_not_earn_coverage(self):
        slide = '<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"/>'
        self.deck("hostile/h.bin", {"s.xml": slide})
        cov = audit.scan_fixtures(self.dir / "fixtures", self.schema())
        self.assertEqual(cov.hits["p:<sld>"], 0)
        self.assertEqual(cov.parts, 1)

    def test_the_real_corpus_has_every_preset(self):
        schema = self.schema()
        cov = audit.scan_fixtures(REPO / "fixtures", schema)
        values = schema.simples[("http://schemas.openxmlformats.org/drawingml/2006/main", "ST_ShapeType")]["values"]
        self.assertEqual(len(values), 187)
        self.assertEqual([v for v, row in values.items() if not cov.hits[row]], [])


class CountGate(unittest.TestCase):
    """SPEC-03: the independent count, seen failing."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.tmp.name)
        shutil.copytree(REPO / "schemas", self.root / "schemas")
        (self.root / "docs/spec").mkdir(parents=True)
        for name in ("checklist.tsv", "README.md"):
            shutil.copy(REPO / "docs/spec" / name, self.root / "docs/spec" / name)

    def tearDown(self):
        self.tmp.cleanup()

    def problems(self):
        return count.compare(self.root, self.root / "docs/spec/checklist.tsv")[2]

    def edit_schema(self, old, new):
        path = self.root / "schemas/ooxml/pml.xsd"
        text = path.read_text(encoding="utf8")
        self.assertIn(old, text)
        path.write_text(text.replace(old, new, 1), encoding="utf8")

    def test_the_real_checklist_agrees(self):
        self.assertEqual(self.problems(), [])

    def test_a_dropped_row_is_a_mismatch(self):
        path = self.root / "docs/spec/checklist.tsv"
        lines = path.read_text(encoding="utf8").splitlines(keepends=True)
        kept = [ln for ln in lines if "\tvalue\t" not in ln or not ln.startswith("p:ST_TransitionSideDirectionType/=l\t")]
        self.assertEqual(len(kept), len(lines) - 1)
        path.write_text("".join(kept), encoding="utf8")
        self.assertEqual(self.problems(), ["schemas/ooxml/pml.xsd [p] value: schema 254, checklist 253"])

    def test_an_added_attribute_is_a_mismatch(self):
        self.edit_schema('<xsd:complexType name="CT_SideDirectionTransition">',
                         '<xsd:complexType name="CT_SideDirectionTransition"><xsd:attribute name="seeded" type="xsd:string"/>')
        self.assertTrue(any("attribute: schema" in p for p in self.problems()), self.problems())

    def test_a_declaration_the_generator_would_not_reach_is_named(self):
        self.edit_schema('<xsd:complexType name="CT_SideDirectionTransition">',
                         '<xsd:complexType name="CT_Seeded"><xsd:sequence><xsd:element name="outer"><xsd:complexType><xsd:sequence>'
                         '<xsd:element name="inner" type="xsd:string"/></xsd:sequence></xsd:complexType></xsd:element></xsd:sequence></xsd:complexType>'
                         '<xsd:complexType name="CT_SideDirectionTransition">')
        self.assertTrue(any("nested-element" in p for p in self.problems()), self.problems())

    def test_a_schema_file_nobody_lists_fails(self):
        shutil.copy(self.root / "schemas/ooxml/pml.xsd", self.root / "schemas/ooxml/stray.xsd")
        self.assertTrue(any("stray.xsd: on disk" in p for p in self.problems()), self.problems())


if __name__ == "__main__":
    unittest.main()
