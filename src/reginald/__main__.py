import sys

from reginald.cli import parse_args
from reginald.error import ReginaldException
from reginald.input.convert_yaml import YAMLConverter
from reginald.input.parse_yaml import YAML_RegisterMap
from reginald.input.validate_map import MapValidator


def main():

    try:

        # Parse command line args:
        cli, generator = parse_args()

        # Open, parse, and validate input file:
        r = YAML_RegisterMap.from_yaml_file(cli.input_file)
        r = YAMLConverter(r).convert()
        MapValidator(r).validate()

        # Generate output using selected generator:
        output = generator.generate(r, cli.input_file, cli.output_file, cli.generator_args)
        
        # Write to file:
        with open(cli.output_file, 'w') as outfile:
            outfile.write("\n".join(output) + "\n")

    except ReginaldException as e:
        print(e, file=sys.stderr)
        exit(-1)

    exit(0)


if __name__ == '__main__':
    main()
