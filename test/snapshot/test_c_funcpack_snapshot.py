import os

import reginald.builtin_generators.c.funcpack as builtin_c_funcpack
from reginald.__main__ import generate
from reginald.cli import CLI

SNAPSHOT_DIR = os.path.dirname(__file__)


def test_c_funcpack_8b(snapshot):
    generator = builtin_c_funcpack.Generator()
    cli = CLI(input_file=os.path.join(SNAPSHOT_DIR, "input_8b.yaml"),
              output_file="c_funcpack_snapshot_8b.h",
              generator_args=[],
              verify=False)
    output = "\n".join(generate(cli, generator))
    assert output == snapshot


def test_c_funcpack_9b(snapshot):
    generator = builtin_c_funcpack.Generator()
    cli = CLI(input_file=os.path.join(SNAPSHOT_DIR, "input_9b.yaml"),
              output_file="c_funcpack_snapshot_9b.h",
              generator_args=[],
              verify=False)
    output = "\n".join(generate(cli, generator))
    assert output == snapshot


def test_c_funcpack_32b(snapshot):
    generator = builtin_c_funcpack.Generator()
    cli = CLI(input_file=os.path.join(SNAPSHOT_DIR, "input_32b.yaml"),
              output_file="c_funcpack_snapshot_32b.h",
              generator_args=[],
              verify=False)
    output = "\n".join(generate(cli, generator))
    assert output == snapshot

def test_c_funcpack_64b(snapshot):
    generator = builtin_c_funcpack.Generator()
    cli = CLI(input_file=os.path.join(SNAPSHOT_DIR, "input_64b.yaml"),
              output_file="c_funcpack_snapshot_64b.h",
              generator_args=[],
              verify=False)
    output = "\n".join(generate(cli, generator))
    assert output == snapshot

