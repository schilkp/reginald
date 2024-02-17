import os

import reginald.builtin_generators.md.doc as builtin_md_doc
from reginald.__main__ import generate
from reginald.cli import CLI

SNAPSHOT_DIR = os.path.dirname(__file__)


def test_md_docs_8b(snapshot):
    generator = builtin_md_doc.Generator()
    cli = CLI(input_file=os.path.join(SNAPSHOT_DIR, "input_8b.yaml"),
              output_file="md_docs_8b.md",
              generator_args=[],
              verify=False)
    output = "\n".join(generate(cli, generator))
    assert output == snapshot


def test_md_docs_9b(snapshot):
    generator = builtin_md_doc.Generator()
    cli = CLI(input_file=os.path.join(SNAPSHOT_DIR, "input_9b.yaml"),
              output_file="md_docs_9b.md",
              generator_args=[],
              verify=False)
    output = "\n".join(generate(cli, generator))
    assert output == snapshot


def test_md_docs_32b(snapshot):
    generator = builtin_md_doc.Generator()
    cli = CLI(input_file=os.path.join(SNAPSHOT_DIR, "input_32b.yaml"),
              output_file="md_docs_32b.md",
              generator_args=[],
              verify=False)
    output = "\n".join(generate(cli, generator))
    assert output == snapshot


def test_md_docs_64b(snapshot):
    generator = builtin_md_doc.Generator()
    cli = CLI(input_file=os.path.join(SNAPSHOT_DIR, "input_64b.yaml"),
              output_file="md_docs_64b.md",
              generator_args=[],
              verify=False)
    output = "\n".join(generate(cli, generator))
    assert output == snapshot
