import sys

from reginald.args import parse_args
from reginald.datamodel import RegisterMap
from reginald.error import ReginaldException
from reginald.validator import validate_RegisterMap


def main():

    try:

        # Parse command line args:
        cli = parse_args()

        # Open, parse, and validate input file:
        r = RegisterMap.from_yaml_file(cli.input_file)
        validate_RegisterMap(r)

        # Generate output using selected generator:
        output = cli.generator.generate(r, cli.generator_args)

        # Print to stdout:
        print(output)

    except ReginaldException as e:
        print(e, file=sys.stderr)
        exit(-1)

    exit(0)


if __name__ == '__main__':
    main()
