_default:
    just --list

format:
    autopep8 --in-place --recursive src/reginald/

lint:
    flake8 src/reginald/
