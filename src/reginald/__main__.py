import os
import sys
from typing import List

from reginald.cli import CLI, parse_args
from reginald.error import ReginaldException
from reginald.generator import OutputGenerator
from reginald.input.convert_yaml import YAMLConverter
from reginald.input.parse_yaml import YAML_RegisterMap
from reginald.input.validate_map import MapValidator


def main():
    # Parse command line args:
    cli, generator = parse_args()

    # Generate reginald output:
    output = generate(cli, generator)

    if cli.verify:
        verify_file(output, cli)
    else:
        # Write to file:
        with open(cli.output_file, 'w') as outfile:
            outfile.write("\n".join(output) + "\n")


def generate(cli: CLI, generator: OutputGenerator) -> List[str]:
    # Open, parse, and validate input file:
    r = YAML_RegisterMap.from_yaml_file(cli.input_file)
    r = YAMLConverter(r).convert()
    MapValidator(r).validate()

    # Generate output using selected generator:
    return generator.generate(r, cli.input_file, cli.output_file, cli.generator_args)


def verify_file(output: List[str], cli: CLI) -> int:
    try:
        filename = os.path.basename(cli.output_file)

        # Open existing file:
        with open(cli.output_file, 'r') as f:
            file_lines = f.readlines()

        # Verify:
        for line_idx, (file_line, output_line) in enumerate(zip(file_lines, output)):
            file_line = file_line.rstrip("\r\n")
            output_line = output_line.rstrip("\r\n")
            if file_line != output_line:
                msg = f"Error. Existing file '{filename}' differs at line {line_idx+1}:\n"
                msg += f"Expected: '{output_line}'"
                msg += f"Found:    '{file_line}'"
                raise ReginaldException(msg)

        if len(file_lines) != len(output):
            msg = f"Error: Existing file '{filename}' has unexpected number of lines."
            msg += f"Expected {len(output)}, is {len(file_lines)}"
            raise ReginaldException(msg)

        return 0

    except FileNotFoundError:
        raise ReginaldException(f"Failed to open file '{cli.output_file}'.")


if __name__ == '__main__':
    try:
        main()
        exit(0)
    except ReginaldException as e:
        print(e, file=sys.stderr)
        exit(1)
