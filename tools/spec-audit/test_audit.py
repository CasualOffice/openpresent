#!/usr/bin/env python3
"""Each gate seen failing on a seeded gap. Run: python3 tools/spec-audit/test_audit.py"""
import contextlib
import csv
import importlib.util
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

# Scanning the real corpus takes most of a minute and the answer cannot change in one run.
_scan_fixtures = audit.scan_fixtures
_scanned = {}


def _cached_scan(folder, schema):
    if pathlib.Path(folder).resolve() != (REPO / "fixtures").resolve():
        return _scan_fixtures(folder, schema)
    if folder not in _scanned:
        _scanned[folder] = _scan_fixtures(folder, schema)
    return _scanned[folder]


audit.scan_fixtures = _cached_scan

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


# What the seeded crates' tests name, so a mark's only gap is the one a test seeds.
NAMED_BY_TESTS = 'fn names() { let _ = "x y cx cy off algn just w round rotWithShape kumimoji"; }'


def make_crates(folder, importer=RUST, exporter="", model="", tests=NAMED_BY_TESTS):
    for name, text in (("casual-slide-import", importer), ("casual-slide-export", exporter), ("casual-slide-model", model)):
        src = folder / name / "src"
        src.mkdir(parents=True)
        (src / "lib.rs").write_text(text, encoding="utf8")
        (folder / name / "tests").mkdir()
        (folder / name / "tests/names.rs").write_text(tests, encoding="utf8")


def seed_checklist(target, marks, keep=False):
    """The real checklist, with status columns set for {row id: {column: value}}. Its own
    marks rest on the real crates, so they are cleared unless `keep` says otherwise."""
    with open(CHECKLIST, encoding="utf8", newline="") as src, open(target, "w", encoding="utf8", newline="") as out:
        reader = csv.DictReader(src, delimiter="\t", quoting=csv.QUOTE_NONE)
        out.write("\t".join(reader.fieldnames) + "\n")
        seen = set()
        for row in reader:
            if not keep:
                row.update({c: "." for c in "MIE"})
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
        # A seeded checklist or seeded crates answer for what is seeded, not for a tier's blanks.
        words = list(map(str, args))
        seeded = {"--crates", "--checklist"} & set(words) and "--gate" not in words
        code = audit.main(["--root", str(REPO), *words, *(["--gate", ""] if seeded else [])])
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

    def test_name_precision_is_not_enough_for_a_mark(self):
        code, out = self.audit({"a:CT_BlipFillProperties/@rotWithShape": {"I": "y"}})
        self.assertEqual(code, 1)
        self.assertIn("only `name` precision", out)

    def test_a_mark_needs_a_test_that_names_it(self):
        shutil.rmtree(self.dir / "crates")
        make_crates(self.dir / "crates", tests='fn other() { let _ = "y"; /* x */ }')
        code, out = self.audit({"a:CT_Point2D/@x": {"I": "y"}})
        self.assertEqual(code, 1)
        self.assertIn("I=y, no test in casual-slide-import names it", out)

    def test_a_test_module_in_the_source_counts_as_a_test(self):
        shutil.rmtree(self.dir / "crates")
        make_crates(self.dir / "crates", importer=RUST + '#[cfg(test)] mod more { fn t() { let _ = "x"; } }', tests="")
        code, out = self.audit({"a:CT_Point2D/@x": {"I": "y"}})
        self.assertEqual(code, 0, out[-800:])

    def test_a_type_needs_everything_it_declares(self):
        code, out = self.audit({"a:CT_Point2D": {"I": "y"}})
        self.assertEqual(code, 1)  # `y` is not read in the seeded source
        self.assertIn("a:CT_Point2D: I=y, not all it declares is done, such as a:CT_Point2D/@y", out)
        shutil.rmtree(self.dir / "crates")
        make_crates(self.dir / "crates", importer=RUST.replace('tree.attribute(id, "x");', 'tree.attribute(id, "x"); tree.attribute(id, "y");'))
        code, out = self.audit({"a:CT_Point2D": {"I": "y"}})
        self.assertEqual(code, 0, out[-800:])

    def test_a_group_row_answers_for_the_group_it_includes(self):
        code, out = self.audit({"a:CT_Color/EG_ColorChoice": {"I": "y"}})
        self.assertEqual(code, 1)
        self.assertIn("not all it includes is done", out)

    def test_an_extension_list_never_earns_a_mark(self):
        shutil.rmtree(self.dir / "crates")
        reads = 'pub fn read_cSld(t: &T, c: C) { match t.local_name(c) { "extLst" => {} _ => {} } }'
        make_crates(self.dir / "crates", importer=reads, tests='fn t() { let _ = "extLst"; }')
        code, out = self.audit({"p:CT_CommonSlideData/extLst": {"I": "y"}})
        self.assertEqual(code, 1)
        self.assertIn("retained, not read", out)

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


class ModelGate(unittest.TestCase):
    """M rests on a declaration, not on an identifier that happens to normalise to the name."""

    MODEL = """
token_enum!(
    /// `a:ST_BlackWhiteMode`. How a shape renders.
    BlackWhiteMode {
        Clr = "clr",
        Auto = "auto",
    }
);
pub struct Kinsoku {
    /// The language.
    pub lang: Option<String>,
    pub(crate) private_thing: u8,
}
pub struct Other {
    pub invalid_st_chars: String,
}
fn f() { let invalid_end_chars = 1; let vert_bar_state = 2; }
"""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = pathlib.Path(self.tmp.name)
        make_crates(self.dir / "crates", importer="", model=self.MODEL,
                    tests='fn t() { let _ = "lang invalStChars invalEndChars vertBarState clr auto gray"; }')

    def tearDown(self):
        self.tmp.cleanup()

    def audit(self, marks):
        seed_checklist(self.dir / "c.tsv", marks)
        return run_audit("--crates", self.dir / "crates", "--checklist", self.dir / "c.tsv", "--check")

    def test_a_field_of_the_owners_struct_earns_it(self):
        code, out = self.audit({"p:CT_Kinsoku/@lang": {"M": "y"}})
        self.assertEqual(code, 0, out[-800:])

    def test_a_field_of_another_struct_does_not(self):
        code, out = self.audit({"p:CT_Kinsoku/@invalStChars": {"M": "y"}})
        self.assertEqual(code, 1)
        self.assertIn("p:CT_Kinsoku/@invalStChars: M=y, no declaration", out)

    def test_a_local_variable_does_not(self):
        code, out = self.audit({"p:CT_NormalViewProperties/@vertBarState": {"M": "y"}})
        self.assertEqual(code, 1)
        self.assertIn("M=y, no declaration", out)

    def test_an_enum_value_must_be_in_the_enum_for_its_type(self):
        self.assertEqual(self.audit({"a:ST_BlackWhiteMode/=auto": {"M": "y"}})[0], 0)
        code, out = self.audit({"a:ST_BlackWhiteMode/=gray": {"M": "y"}})
        self.assertEqual(code, 1)
        self.assertIn("a:ST_BlackWhiteMode/=gray: M=y, no declaration", out)

    def test_a_doc_comment_declares_only_for_the_owner_it_names(self):
        model = audit.ModelDeclarations.__new__(audit.ModelDeclarations)
        model.enums, model.structs, model.documented = {}, {}, set()
        model.noted = set()
        model.read_docs(
            "/// `a:CT_Thing`.\npub struct Thing {\n    /// `@wide` and `child/@deep`.\n    pub a: u8,\n}\n"
            "pub enum Choice {\n    /// `glow`.\n    Glow {\n        /// `@rad`.\n        radius: u8,\n    },\n}\n"
        )
        row = lambda kind, name, owner: {"kind": kind, "name": name, "owner": owner}
        self.assertTrue(model.declares(row("attribute", "@wide", "CT_Thing")))
        self.assertTrue(model.declares(row("attribute", "@deep", "CT_Child"), ("child",)))
        self.assertIsNone(model.declares(row("attribute", "@wide", "CT_Other")))
        self.assertIsNone(model.declares(row("attribute", "@rad", "CT_Other")))
        self.assertTrue(model.declares(row("attribute", "@rad", "CT_GlowEffect"), ("glow",)))
        self.assertIsNone(model.declares(row("attribute", "@rad", "CT_BlurEffect"), ("blur",)))

    def test_an_owner_note_places_a_helper_and_only_that_helper(self):
        scan = audit.Scan({"bevelT"})
        with tempfile.TemporaryDirectory() as d:
            path = pathlib.Path(d) / "x.rs"
            path.write_text('// spec-audit: owner=bevelT\nfn bevel(e: &E) {\n    e.lex("w", 1);\n}\nfn other(e: &E) {\n    e.lex("h", 1);\n}\n')
            scan.scan_file(path, "x.rs", {})
        refs = {r.value: r.ctx for r in scan.refs}
        self.assertIn("bevelT", refs["w"])
        self.assertNotIn("bevelT", refs["h"])

    def test_the_builder_names_its_element_for_the_chain(self):
        scan = audit.Scan({"glow"})
        with tempfile.TemporaryDirectory() as d:
            path = pathlib.Path(d) / "x.rs"
            path.write_text('fn make() -> Node {\n    Node::a("glow").lex("rad", 1);\n    Node::a("blur").lex("grow", 1)\n}\n')
            scan.scan_file(path, "x.rs", {})
        refs = {r.value: r.ctx for r in scan.refs}
        self.assertIn("glow", refs["rad"])
        self.assertNotIn("glow", refs["grow"])

    def test_a_value_needs_its_enum_and_an_attribute_placed_in_that_crate(self):
        # In the enum, but no attribute of its type is read with a known owner in this tree.
        code, out = self.audit({"a:ST_BlackWhiteMode/=auto": {"I": "y"}})
        self.assertIn("a:ST_BlackWhiteMode/=auto", out if code else "a:ST_BlackWhiteMode/=auto")
        code, out = self.audit({"a:ST_BlackWhiteMode/=gray": {"I": "y"}})
        self.assertEqual(code, 1)
        self.assertIn("a:ST_BlackWhiteMode/=gray: I=y", out)

    def test_a_simple_type_needs_all_its_values(self):
        code, out = self.audit({"a:ST_BlackWhiteMode": {"M": "y"}})
        self.assertEqual(code, 1)
        self.assertIn("not all its values are done", out)


class Marks(unittest.TestCase):
    """STAT-01: what --propose earns, what --apply writes, and false marks on the real checklist."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = pathlib.Path(self.tmp.name)
        self.copy = self.dir / "checklist.tsv"

    def tearDown(self):
        self.tmp.cleanup()

    def propose(self, crates, *extra):
        seed_checklist(self.copy, {})
        out = io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
            code = audit.main(["--root", str(REPO), "--crates", str(crates), "--checklist", str(self.copy),
                               "--propose", "1A,1B,1C,1D", *extra])
        self.assertEqual(code, 0)
        return dict(line.split("\t") for line in out.getvalue().splitlines())

    def test_a_proposal_needs_context_a_test_and_a_fixture(self):
        make_crates(self.dir / "crates")
        marks = self.propose(self.dir / "crates")
        self.assertIn("I=y", marks.get("a:CT_Point2D/@x"))        # all three
        self.assertNotIn("I=y", marks.get("a:CT_Point2D/@y", ""))  # not read
        self.assertNotIn("a:CT_Point3D/@x", marks)                # another owner, and tier 1E
        self.assertNotIn("I=y", marks.get("a:CT_Point2D", ""))    # only half of what it declares

    def test_a_proposal_stays_inside_the_tiers_asked_for(self):
        make_crates(self.dir / "crates")
        marks = self.propose(self.dir / "crates")
        self.assertNotIn("a:CT_LineProperties/@w", marks)         # LINE is Tier 1E, and has context

    def test_an_unknown_tier_is_refused(self):
        with contextlib.redirect_stderr(io.StringIO()), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(audit.main(["--root", str(REPO), "--propose", "9Z"]), 2)

    def test_apply_writes_marks_that_pass_the_gate_and_nothing_else(self):
        make_crates(self.dir / "crates")
        marks = self.propose(self.dir / "crates", "--apply")
        self.assertTrue(marks)
        after = self.copy.read_text(encoding="utf8").splitlines()
        for line in after[1:]:
            cols = line.split("\t")
            self.assertEqual(len(cols), 14)
            if cols[0] in marks and "I=y" in marks[cols[0]]:
                self.assertEqual(cols[10], "y")
        code, out = run_audit("--crates", self.dir / "crates", "--checklist", self.copy, "--check")
        self.assertEqual(code, 0, out[-800:])

    def test_write_marks_keeps_the_rest_of_the_file_byte_for_byte(self):
        path = self.dir / "t.tsv"
        text = "id\tM\tI\tE\trest\nq\t.\t.\t.\tx y\nr\t.\tr\t.\tz\n"
        path.write_text(text, encoding="utf8")
        audit.write_marks(path, {"q": {"I": "y"}})
        self.assertEqual(path.read_text(encoding="utf8"), text.replace("q\t.\t.", "q\t.\ty"))

    def false_mark(self, marks):
        seed_checklist(self.copy, marks, keep=True)
        return run_audit("--checklist", self.copy, "--check")

    def test_the_checklist_as_committed_passes(self):
        code, out = run_audit("--check")
        self.assertEqual(code, 0, out[-1500:])

    def test_the_committed_marks_are_real_and_inside_their_tiers(self):
        rows = audit.read_checklist(CHECKLIST)
        marked = {r["area"] for r in rows if any(r[c] == "y" for c in "MIE")}
        for tier, areas in audit.TIERS.items():
            self.assertTrue(marked & set(areas), f"tier {tier} has no marks")

    def test_a_false_mark_on_the_real_checklist_fails_check(self):
        cases = {
            # Nothing writes it: it stays in the tree.
            "x:thememl/2012/main:CT_ThemeFamily/@vid": ("E", "casual-slide-export"),
            # Nothing in the crates reads charts yet.
            "c:CT_NumFmt/@formatCode": ("I", "no reference in casual-slide-import"),
            # In code and in a fixture, and no model struct field holds it under that name.
            "x:drawing/2010/main:CT_PictureEffectBlur/@radius": ("M", "M=y, no declaration"),
            # Half of the type is not done.
            "p:CT_Presentation": ("I", "not all it declares is done"),
        }
        for rid, (column, expected) in cases.items():
            code, out = self.false_mark({rid: {column: "y"}})
            self.assertEqual(code, 1, rid)
            self.assertIn(f"FAIL {rid}: {column}=y", out)
            self.assertIn(expected, out, rid)

    def test_a_blank_with_no_reason_fails_in_a_gated_tier(self):
        rows = audit.read_checklist(CHECKLIST)
        rid = next(r["id"] for r in rows if r["area"] == "VIEW" and r["kind"] == "attribute" and r["I"] == "y")
        seed_checklist(self.copy, {rid: {"I": "."}}, keep=True)
        code, out = run_audit("--checklist", self.copy, "--check", "--gate", "1B")
        self.assertEqual(code, 1)
        self.assertIn(f"FAIL {rid}: I is blank, with no reason in docs/spec/open-rows.tsv", out)

    def test_a_reason_on_a_marked_row_fails(self):
        rid, column = next(iter(audit.read_open_rows(REPO / audit.OPEN_ROWS)[0]))
        seed_checklist(self.copy, {rid: {column: "r"}}, keep=True)
        code, out = run_audit("--checklist", self.copy, "--check", "--gate", "1B")
        self.assertEqual(code, 1)
        self.assertIn(f"{rid} {column} has a reason and a mark r", out)

    def test_not_applicable_needs_something_the_tool_can_see(self):
        rows = audit.read_checklist(CHECKLIST)
        rid = next(r["id"] for r in rows if r["area"] == "VIEW" and r["kind"] == "attribute")
        seed_checklist(self.copy, {rid: {"E": "n/a"}}, keep=True)
        code, out = run_audit("--checklist", self.copy, "--check")
        self.assertEqual(code, 1)
        self.assertIn(f"FAIL {rid}: E=n/a, and the tool sees nothing that says so", out)

    def test_retained_needs_a_fixture(self):
        rows = {r["id"]: r for r in audit.read_checklist(CHECKLIST)}
        rid = "p:CT_SlideTransition/@spd"
        self.assertIn(rid, rows)
        (self.dir / "fixtures").mkdir(exist_ok=True)
        seed_checklist(self.copy, {rid: {"E": "r"}}, keep=True)
        code, out = run_audit("--checklist", self.copy, "--fixtures", self.dir / "fixtures", "--check")
        self.assertEqual(code, 1)
        self.assertIn(f"FAIL {rid}: E=r, no fixture exercises it", out)

    def test_retained_in_the_model_forbids_done_in_the_columns(self):
        rows = audit.read_checklist(CHECKLIST)
        row = next(r for r in rows if r["area"] == "VIEW" and r["kind"] == "attribute" and r["I"] == "y" and r["M"] == "y")
        seed_checklist(self.copy, {row["id"]: {"M": "r"}}, keep=True)
        code, out = run_audit("--checklist", self.copy, "--check")
        self.assertEqual(code, 1)
        self.assertIn(f"FAIL {row['id']}: M=r, so nothing reads or writes it", out)

    def test_a_mark_with_code_and_no_fixture_fails_check(self):
        rows = audit.read_checklist(CHECKLIST)
        self.assertTrue(rows)
        code, out = self.false_mark({"p:CT_Presentation/@firstSlideNum": {"I": "y"}})
        self.assertEqual(code, 0, out[-800:])  # earned
        make_crates(self.dir / "crates")
        (self.dir / "fixtures/generated").mkdir(parents=True)
        seed_checklist(self.copy, {"a:CT_Point2D/@x": {"I": "y"}})
        code, out = run_audit("--crates", self.dir / "crates", "--checklist", self.copy,
                              "--fixtures", self.dir / "fixtures", "--check")
        self.assertEqual(code, 1)
        self.assertIn("a:CT_Point2D/@x: I=y, no fixture exercises it", out)


class MarkupCompatibility(unittest.TestCase):
    """SPEC-08: rows no schema gives, kept by the generator and held to the same bar."""

    NAMES = ("mc:<AlternateContent>", "mc:AlternateContent/Choice", "mc:AlternateContent/Fallback", "mc:Choice/@Requires",
             "mc:@Ignorable", "mc:@ProcessContent", "mc:@MustUnderstand")

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = pathlib.Path(self.tmp.name)
        self.copy = self.dir / "checklist.tsv"

    def tearDown(self):
        self.tmp.cleanup()

    def generator(self):
        spec = importlib.util.spec_from_file_location("spec_generate", REPO / "tools/spec-checklist/generate.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def test_the_rows_named_by_the_tracker_are_in_the_checklist(self):
        rows = {r["id"]: r for r in audit.read_checklist(CHECKLIST)}
        for name in self.NAMES:
            self.assertEqual(rows[name]["area"], "MCE", name)
        self.assertIn("BEH-18", (REPO / "docs/spec/MCE.md").read_text(encoding="utf8"))

    def test_the_generator_keeps_them_and_their_status(self):
        generator = self.generator()
        rows, sources, skipped = generator.collect()
        self.assertTrue(set(self.NAMES) <= {r["id"] for r in rows})
        files = generator.render(rows, sources, skipped, {"mc:@Ignorable": {"I": "y"}})
        line = next(ln for ln in files["checklist.tsv"].splitlines() if ln.startswith("mc:@Ignorable\t"))
        self.assertEqual(line.split("\t")[10], "y")
        self.assertIn("mc:", files["MCE.md"] + "mc:")
        self.assertIn("BEH-18", files["MCE.md"])

    def test_a_row_of_the_wrong_shape_stops_generation(self):
        generator = self.generator()
        path = self.dir / "t.tsv"
        path.write_text("one\ttwo\n", encoding="utf8")
        with self.assertRaises(SystemExit):
            generator.hand_written_rows(path)

    def test_the_count_and_the_schema_model_do_not_call_them_strays(self):
        self.assertEqual(count.compare(REPO, CHECKLIST)[2], [])

    def test_a_false_mark_on_one_fails_check(self):
        seed_checklist(self.copy, {"mc:@Ignorable": {"E": "y"}}, keep=True)
        code, out = run_audit("--checklist", self.copy, "--check")
        self.assertEqual(code, 1)
        self.assertIn("FAIL mc:@Ignorable: E=y, only I applies", out)

    def test_hostile_decks_do_not_earn_a_fixture(self):
        (self.dir / "fixtures/hostile").mkdir(parents=True)
        with zipfile.ZipFile(self.dir / "fixtures/hostile/h.bin", "w") as z:
            z.writestr("a.xml", '<r xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" mc:Ignorable=""/>')
        cov = audit.scan_fixtures(self.dir / "fixtures", audit.Schema(REPO))
        self.assertEqual(cov.hits["mc:@Ignorable"], 0)


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
