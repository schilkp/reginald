PY := env_var_or_default("PY", "python")
PYENV := "env" / "bin" / "python"

_default:
    just --list

# Auto-format all code.
format:
    {{PYENV}} -m autopep8 --in-place --recursive src/reginald/ test/snapshot

# Run flake8 linting.
lint:
    {{PYENV}} -m flake8 src/reginald/

# Setup development environment.
setup_dev:
    {{PY}} -m venv env
    {{PYENV}} -m pip install -e .[dev]

# Build distribution bundle.
build:
    -rm -rf dist
    {{PYENV}} -m build

# Publish distribution bundle to pypi.
publish:
    {{PYENV}} -m twine upload  dist/*

# Run reginald unit and integration tests.
test +ARGS="-v":
    {{PYENV}} -m pytest {{ARGS}}

# Accept new output snapshots.
update_test_snapshots +ARGS="-v":
    {{PYENV}} -m pytest --snapshot-update {{ARGS}}
