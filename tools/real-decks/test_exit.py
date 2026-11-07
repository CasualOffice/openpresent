#!/usr/bin/env python3
"""`P1X-01`: the exit tools report honestly. Each test plants a failure and watches the tool
say so. Uses fake probes, small stand-ins that crash, hang or answer on demand, so a tool's
own judgement is what is tested, and no cargo build, deck download or validator is needed.

    python3 -I tools/real-decks/test_exit.py
"""

import importlib.util
import json
import pathlib
import stat
import subprocess
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(HERE))
import census  # noqa: E402
import exitlib  # noqa: E402
import features  # noqa: E402

spec = importlib.util.spec_from_file_location("phase1_exit", ROOT / "tools/phase1-exit.py")
phase1_exit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(phase1_exit)

# Built, so that the repository gate does not read it as a citation of a real row.
PLANTED_ROW = "T1E" + "-99"
OPEN = '{"deck": "x", "outcome": "open", "slides": 1, "unedited_same": true}'


def fake_probe(folder, body):
    """An executable taking the probe's arguments as `sys.argv`, running `body`."""
    path = pathlib.Path(folder) / "probe"
    path.write_text(f"#!{sys.executable} -I\nimport os, signal, sys, time\nargs = sys.argv[1:]\n{body}\n", encoding="utf8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)
    return str(path)


def run_tool(name, *args):
    return subprocess.run([sys.executable, "-I", str(HERE / name), *args], capture_output=True, text=True, check=False)


def fixtures_with(folder, names=("a.bin", "b.bin"), listed=()):
    root = pathlib.Path(folder) / "fixtures"
    (root / "hostile").mkdir(parents=True)
    for name in names:
        (root / "hostile" / name).write_bytes(b"not a deck")
    rows = "".join(f"hostile/{n}\t0\topen\n" for n in (*names, *listed))
    (root / "manifest.tsv").write_text("path\tsha256\texpect\n" + rows, encoding="utf8")
    return root


class HostileRun(unittest.TestCase):
    def run_with(self, body, *extra):
        with tempfile.TemporaryDirectory() as tmp:
            fixtures = fixtures_with(tmp, *extra)
            report = pathlib.Path(tmp) / "r.json"
            done = run_tool("hostile_run.py", "--probe", fake_probe(tmp, body), "--fixtures", str(fixtures),
                            "--seconds", "2", "--json", str(report))
            return done, json.loads(report.read_text()) if report.exists() else None

    def test_a_clean_run_passes(self):
        done, report = self.run_with(f"print('{OPEN}')")
        self.assertEqual(done.returncode, 0, done.stdout)
        self.assertEqual(report["hostile_opened"], 2)

    def test_a_refusal_is_counted_and_is_not_a_crash(self):
        done, report = self.run_with("print('{\"deck\": \"x\", \"outcome\": \"refused\", \"stage\": \"open\", \"code\": \"CS-XML-0004\"}')")
        self.assertEqual(done.returncode, 0, done.stdout)
        self.assertEqual(report["hostile_refused"], 2)

    def test_a_panic_exit_fails_and_names_the_deck(self):
        done, report = self.run_with("sys.exit(101)")
        self.assertEqual(done.returncode, 1)
        self.assertIn("crash: exit 101", report["failures"][0]["why"])
        self.assertIn("a.bin", report["failures"][0]["deck"])

    def test_a_signal_fails(self):
        done, report = self.run_with("os.kill(os.getpid(), signal.SIGSEGV)")
        self.assertEqual(done.returncode, 1)
        self.assertIn("signal", report["failures"][0]["why"])

    def test_a_hang_is_killed_and_fails(self):
        done, report = self.run_with("time.sleep(60)")
        self.assertEqual(done.returncode, 1)
        self.assertIn("hang", report["failures"][0]["why"])

    def test_output_that_is_no_answer_fails(self):
        done, _ = self.run_with("print('fine, honest')")
        self.assertEqual(done.returncode, 1)

    def test_a_fixture_missing_from_disk_fails_rather_than_shrinking_the_run(self):
        done, report = self.run_with(f"print('{OPEN}')", ("a.bin",), ("gone.bin",))
        self.assertEqual(done.returncode, 1)
        self.assertIn("gone.bin", report["failures"][0]["deck"])

    def test_memory_over_the_bound_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            done = run_tool("hostile_run.py", "--probe", fake_probe(tmp, f"data = bytearray(64 << 20)\nprint('{OPEN}')"),
                            "--fixtures", str(fixtures_with(tmp)), "--memory-mb", "16")
        self.assertEqual(done.returncode, 1)
        self.assertIn("MiB", done.stdout)


BUDGET_OK = """
stage = args[1]
line = {"open": '"slides": 500', "open-all": '"slides": 500', "save": '"unedited_same": true', "edit": '"changed_parts": 1'}[stage]
print('{"deck": "x", "outcome": "ok", "stage": "' + stage + '", "ms": 5, ' + line + '}')
"""


class Budget(unittest.TestCase):
    def run_with(self, body, *extra):
        with tempfile.TemporaryDirectory() as tmp:
            report = pathlib.Path(tmp) / "r.json"
            done = run_tool("budget.py", "--probe", fake_probe(tmp, body), "--repeat", "1", "--json", str(report), *extra)
            return done, json.loads(report.read_text())

    def test_within_budget_passes(self):
        done, report = self.run_with(BUDGET_OK)
        self.assertEqual(done.returncode, 0, done.stdout)
        self.assertEqual(len(report["stages"]), 4)

    def test_over_a_time_budget_fails(self):
        done, report = self.run_with(BUDGET_OK.replace('"ms": 5', '"ms": 999999'))
        self.assertEqual(done.returncode, 1)
        self.assertEqual(len(report["failures"]), 4)

    def test_over_a_memory_budget_fails(self):
        done, report = self.run_with("data = bytearray(64 << 20)\n" + BUDGET_OK, "--scale", "0.05")
        self.assertEqual(done.returncode, 1)
        self.assertFalse(report["stages"]["open"]["within"])

    def test_a_changed_unedited_save_fails_within_time(self):
        done, report = self.run_with(BUDGET_OK.replace('"unedited_same": true', '"unedited_same": false'))
        self.assertEqual(done.returncode, 1)
        self.assertIn("did not return the input", " ".join(report["failures"]))

    def test_reading_fewer_slides_than_the_deck_fails(self):
        done, _ = self.run_with(BUDGET_OK.replace('"slides": 500', '"slides": 3'))
        self.assertEqual(done.returncode, 1)

    def test_the_deck_is_the_same_bytes_every_time(self):
        import budget
        with tempfile.TemporaryDirectory() as tmp:
            a, b = pathlib.Path(tmp) / "a.pptx", pathlib.Path(tmp) / "b.pptx"
            budget.make_deck(a, 12)
            budget.make_deck(b, 12)
            self.assertEqual(a.read_bytes(), b.read_bytes())
            self.assertEqual(features.scan(str(a))["slides"], 12)


class Damage(unittest.TestCase):
    def run_with(self, body):
        with tempfile.TemporaryDirectory() as tmp:
            report = pathlib.Path(tmp) / "r.json"
            base = ROOT / "fixtures/generated/three-slides.pptx"
            done = run_tool("damage.py", "--probe", fake_probe(tmp, body), "--bases", str(base), "--no-validate",
                            "--seconds", "5", "--json", str(report))
            return done, json.loads(report.read_text())

    def test_every_deck_opening_passes(self):
        done, report = self.run_with("print('{\"deck\": \"x\", \"outcome\": \"edited\", \"changed_parts\": 1}')")
        self.assertEqual(done.returncode, 0, done.stdout)
        self.assertGreater(report["cases"], 40)

    def test_a_refusal_docs_29_allows_passes(self):
        done, _ = self.run_with("print('{\"deck\": \"x\", \"outcome\": \"refused\", \"stage\": \"open\", \"code\": \"CS-PKG-0005\"}')")
        self.assertEqual(done.returncode, 0, done.stdout)

    def test_a_refusal_docs_29_does_not_allow_fails_with_its_code(self):
        done, report = self.run_with("print('{\"deck\": \"x\", \"outcome\": \"refused\", \"stage\": \"open\", \"code\": \"CS-XML-0004\"}')")
        self.assertEqual(done.returncode, 1)
        self.assertEqual(list(report["codes"]), ["CS-XML-0004"])

    def test_a_crash_fails(self):
        done, _ = self.run_with("sys.exit(101)")
        self.assertEqual(done.returncode, 1)

    def test_the_same_damage_every_run(self):
        import random
        import damage
        base = ROOT / "fixtures/generated/three-slides.pptx"
        first = [(k, d) for k, d in damage.damaged(base, random.Random(1))]
        second = [(k, d) for k, d in damage.damaged(base, random.Random(1))]
        self.assertEqual(first, second)
        self.assertTrue(all(d != base.read_bytes() for _, d in first), "every case really is damaged")


class Census(unittest.TestCase):
    def test_a_target_out_of_reach_fails_and_is_named(self):
        with tempfile.TemporaryDirectory() as tmp:
            targets = pathlib.Path(tmp) / "t.tsv"
            targets.write_text("axis\tvalue\tmin\tscope\twhy\nproducer\tKeynote\t9999\tcorpus\tplanted\n", encoding="utf8")
            done = run_tool("census.py", "--targets", str(targets))
        self.assertEqual(done.returncode, 1)
        self.assertIn("UNDER producer Keynote", done.stdout)

    def test_a_target_within_reach_passes(self):
        with tempfile.TemporaryDirectory() as tmp:
            targets = pathlib.Path(tmp) / "t.tsv"
            targets.write_text("axis\tvalue\tmin\tscope\twhy\ntotal\tdecks\t1\tcorpus\tplanted\n", encoding="utf8")
            done = run_tool("census.py", "--targets", str(targets))
        self.assertEqual(done.returncode, 0, done.stdout)

    def test_a_local_deck_without_provenance_is_named_and_not_counted(self):
        with tempfile.TemporaryDirectory() as tmp:
            local = pathlib.Path(tmp) / "local"
            local.mkdir()
            import budget
            budget.make_deck(local / "stranger.pptx", 2)
            targets = pathlib.Path(tmp) / "t.tsv"
            targets.write_text("axis\tvalue\tmin\tscope\twhy\ntotal\tdecks\t1\tcorpus\tplanted\n", encoding="utf8")
            done = run_tool("census.py", "--targets", str(targets), "--local", str(local))
        self.assertIn("NOT COUNTED stranger.pptx", done.stdout)

    def test_every_committed_real_deck_has_a_provenance_row(self):
        decks = [p for p in (ROOT / "fixtures/real").glob("*.pptx")]
        table, _ = census.provenance(ROOT)
        self.assertTrue(all(p.name in table for p in decks), "every committed deck has a row")

    def test_producers_are_read_from_marks_not_only_app_xml(self):
        self.assertEqual(features.producer_of("", 'name="Google Shape;87;p13"')[0], "Google Slides")
        self.assertEqual(features.producer_of("Microsoft Office PowerPoint", 'name="Google Shape;87;p13"')[0], "PowerPoint Windows")
        self.assertEqual(features.producer_of("Microsoft Macintosh PowerPoint", "Title Text Body Level One")[0], "Keynote")
        self.assertEqual(features.producer_of("Microsoft Macintosh PowerPoint", "")[0], "PowerPoint Mac")
        self.assertEqual(features.producer_of("Microsoft Office PowerPoint", "")[0], "PowerPoint Windows")
        self.assertEqual(features.producer_of("", "")[0], "Unknown")

    def test_scripts_are_counted_only_over_a_floor(self):
        self.assertEqual(features.scripts_of("abc"), {})
        self.assertEqual(list(features.scripts_of("ก" * 30 + "a" * 5)), ["Thai"])

    def test_strict_form_tells_converted_from_saved(self):
        converted = {"ppt/theme/theme1.xml": '<a:tint val="50000"/>'}
        saved = {"ppt/theme/theme1.xml": '<a:tint val="50%"/>'}
        self.assertEqual(features.strict_form(converted), "numeric")
        self.assertEqual(features.strict_form(saved), "percent")


class Exit(unittest.TestCase):
    ROADMAP = "**Phase 1 closes when:**\n- [ ] All tiers closed.\n- [ ] Owner sign-off.\n\n## Phase 2\n"

    def tracker(self, tmp, status):
        ids = [f"T-{t}" for t in phase1_exit.TIERS if t not in phase1_exit.STAND_INS] + ["T1N-01", "REC-01"]
        rows = "".join(f"| {i} | x | {status} | P0 | x | x |\n" for i in ids)
        path = pathlib.Path(tmp) / "tracker.md"
        path.write_text(rows, encoding="utf8")
        return path

    def test_todays_tracker_says_tiers_are_not_met(self):
        met, evidence = phase1_exit.tiers(ROOT / "docs/14-EXECUTION-TRACKER.md")
        self.assertFalse(met, evidence)
        self.assertIn("open:", evidence)

    def test_all_done_says_met(self):
        with tempfile.TemporaryDirectory() as tmp:
            met, evidence = phase1_exit.tiers(self.tracker(tmp, "Done"))
        self.assertTrue(met, evidence)

    def test_one_open_tier_says_not_met_and_names_it(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = self.tracker(tmp, "Done")
            path.write_text(path.read_text().replace("| T-1J | x | Done", "| T-1J | x | In progress"), encoding="utf8")
            met, evidence = phase1_exit.tiers(path)
        self.assertFalse(met)
        self.assertIn("1J", evidence)

    def test_a_done_tier_with_an_open_leftover_is_not_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = self.tracker(tmp, "Done")
            with path.open("a", encoding="utf8") as handle:
                handle.write(f"| {PLANTED_ROW} | x | Open | P1 | x | x |\n")
            met, evidence = phase1_exit.tiers(path)
        self.assertFalse(met)
        self.assertIn(PLANTED_ROW, evidence)

    def test_a_missing_tier_row_is_not_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = self.tracker(tmp, "Done")
            path.write_text(path.read_text().replace("| T-1K | x | Done | P0 | x | x |\n", ""), encoding="utf8")
            met, evidence = phase1_exit.tiers(path)
        self.assertFalse(met)
        self.assertIn("1K (no row)", evidence)

    def run_exit(self, roadmap, tracker, *extra):
        return subprocess.run([sys.executable, "-I", str(ROOT / "tools/phase1-exit.py"), "--no-gates",
                               "--roadmap", str(roadmap), "--tracker", str(tracker), *extra],
                              capture_output=True, text=True, check=False)

    def test_the_owner_sign_off_is_only_the_box(self):
        with tempfile.TemporaryDirectory() as tmp:
            roadmap = pathlib.Path(tmp) / "road.md"
            roadmap.write_text(self.ROADMAP, encoding="utf8")
            done = self.run_exit(roadmap, self.tracker(tmp, "Done"))
            self.assertIn("NOT MET Owner sign-off", done.stdout)
            self.assertIn("MET     All tiers closed", done.stdout)
            self.assertEqual(done.returncode, 1)
            roadmap.write_text(self.ROADMAP.replace("- [ ] Owner", "- [x] Owner"), encoding="utf8")
            done = self.run_exit(roadmap, self.tracker(tmp, "Done"))
            self.assertIn("MET     Owner sign-off", done.stdout)

    def test_gates_not_run_keep_the_phase_from_closing(self):
        with tempfile.TemporaryDirectory() as tmp:
            roadmap = pathlib.Path(tmp) / "road.md"
            roadmap.write_text(self.ROADMAP.replace("[ ]", "[x]"), encoding="utf8")
            done = self.run_exit(roadmap, self.tracker(tmp, "Done"))
        self.assertIn("NOT MET Gates", done.stdout)
        self.assertEqual(done.returncode, 1)

    def test_an_item_this_tool_does_not_know_is_not_met(self):
        with tempfile.TemporaryDirectory() as tmp:
            roadmap = pathlib.Path(tmp) / "road.md"
            roadmap.write_text(self.ROADMAP.replace("- [ ] Owner", "- [x] Moon landing.\n- [ ] Owner"), encoding="utf8")
            done = self.run_exit(roadmap, self.tracker(tmp, "Done"))
        self.assertIn("NOT MET Moon landing", done.stdout)
        self.assertIn("no check", done.stdout)

    def test_a_roadmap_without_the_list_stops_the_run(self):
        with tempfile.TemporaryDirectory() as tmp:
            roadmap = pathlib.Path(tmp) / "road.md"
            roadmap.write_text("nothing\n", encoding="utf8")
            done = self.run_exit(roadmap, self.tracker(tmp, "Done"))
        self.assertEqual(done.returncode, 2)

    def test_the_real_closing_list_has_a_check_for_every_item(self):
        items = phase1_exit.closing_list(ROOT / "docs/06-ROADMAP-AND-DELIVERY.md")
        self.assertGreaterEqual(len(items), 6)
        import re
        for text, _ in items:
            self.assertTrue(any(re.search(p, text) for p, _ in phase1_exit.CHECKS), text)

    def test_libreoffice_is_not_met_until_it_covers_every_export(self):
        self.assertFalse(phase1_exit.libreoffice(None, {"a"})[0])
        with tempfile.TemporaryDirectory() as tmp:
            evidence = pathlib.Path(tmp) / "lo.tsv"
            evidence.write_text("a\tok\tx\nb\tfailed\ty\n", encoding="utf8")
            self.assertTrue(phase1_exit.libreoffice(evidence, {"a"})[0])
            self.assertFalse(phase1_exit.libreoffice(evidence, {"a", "b"})[0])
            self.assertFalse(phase1_exit.libreoffice(evidence, set())[0])

    def test_libreoffice_tool_refuses_without_permission(self):
        done = subprocess.run([sys.executable, "-I", str(HERE / "libreoffice_reopen.py"), "x", "y"],
                              capture_output=True, text=True, check=False,
                              env={"PATH": "/usr/bin:/bin"})
        self.assertEqual(done.returncode, 2)
        self.assertIn("refusing", done.stderr)

    def test_a_bounded_run_reports_time_and_memory(self):
        result = exitlib.bounded([sys.executable, "-c", "print(1)"], 30)
        self.assertEqual(result["status"], "exit")
        self.assertGreater(result["peak_bytes"], 1 << 20)


if __name__ == "__main__":
    unittest.main()
