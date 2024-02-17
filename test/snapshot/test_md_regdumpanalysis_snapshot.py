import os

import reginald.builtin_generators.md.regdumpanalysis as builtin_md_regdumpanalysis
from reginald.__main__ import generate
from reginald.cli import CLI

SNAPSHOT_DIR = os.path.dirname(__file__)


def test_md_regdumpanalysis_8b(snapshot):
    generator = builtin_md_regdumpanalysis.Generator()
    cli = CLI(input_file=os.path.join(SNAPSHOT_DIR, "input_8b.yaml"),
              output_file="md_regdumpanalysis_8b.md",
              generator_args=[os.path.join(SNAPSHOT_DIR, "md_regdumpanalysis_8b.yaml")],
              verify=False)
    output = "\n".join(generate(cli, generator))
    assert output == snapshot


def test_md_regdumpanalysis_9b(snapshot):
    generator = builtin_md_regdumpanalysis.Generator()
    cli = CLI(input_file=os.path.join(SNAPSHOT_DIR, "input_9b.yaml"),
              output_file="md_regdumpanalysis_9b.md",
              generator_args=[os.path.join(SNAPSHOT_DIR, "md_regdumpanalysis_9b.yaml")],
              verify=False)
    output = "\n".join(generate(cli, generator))
    assert output == snapshot


def test_md_regdumpanalysis_32b(snapshot):
    generator = builtin_md_regdumpanalysis.Generator()
    cli = CLI(input_file=os.path.join(SNAPSHOT_DIR, "input_32b.yaml"),
              output_file="md_regdumpanalysis_32b.md",
              generator_args=[os.path.join(SNAPSHOT_DIR, "md_regdumpanalysis_32b.yaml")],
              verify=False)
    output = "\n".join(generate(cli, generator))
    assert output == snapshot


def test_md_regdumpanalysis_64b(snapshot):
    generator = builtin_md_regdumpanalysis.Generator()
    cli = CLI(input_file=os.path.join(SNAPSHOT_DIR, "input_64b.yaml"),
              output_file="md_regdumpanalysis_64b.md",
              generator_args=[os.path.join(SNAPSHOT_DIR, "md_regdumpanalysis_64b.yaml")],
              verify=False)
    output = "\n".join(generate(cli, generator))
    assert output == snapshot
