import os
import sys
from typing import List

from reginald.cli import CLI, parse_args
from reginald.error import ReginaldException
from reginald.input.convert_yaml import YAMLConverter
from reginald.input.parse_yaml import YAML_RegisterMap
from reginald.input.validate_map import MapValidator


def main() -> int:
    try:
        # Parse command line args:
        cli, generator = parse_args()

        # Open, parse, and validate input file:
        r = YAML_RegisterMap.from_yaml_file(cli.input_file)
        r = YAMLConverter(r).convert()
        MapValidator(r).validate()

        # Generate output using selected generator:
        output = generator.generate(r, cli.input_file, cli.output_file, cli.generator_args)

        if cli.verify:
            verify_file(output, cli)
        else:
            # Write to file:
            with open(cli.output_file, 'w') as outfile:
                outfile.write("\n".join(output) + "\n")

    except ReginaldException as e:
        print(e, file=sys.stderr)
        return 1

    return 0


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
                print(f"Error. Existing file '{filename}' differs at line {line_idx+1}:")
                print(f"Expected: '{output_line}'")
                print(f"Found:    '{file_line}'")
                return 1

        if len(file_lines) != len(output):
            print(f"Error: Existing file '{filename}' has unexpected number of lines.")
            print(f"Expected {len(output)}, is {len(file_lines)}")
            return 1

        return 0

    except FileNotFoundError:
        print(f"Failed to open file '{cli.output_file}'.")
        return 1


if __name__ == '__main__':
    exit(main())
