# Input System

OpenQASM programs can declare classical inputs and outputs. These are supplied at runtime from the command line.

## Declaring Inputs

In your `.qasm` file, use the `input` keyword:

```openqasm
input int[32] n;
input float[64] theta;
input bit[4] data;
input bool flag;
input int[2][2] matrix;   // 2×2 array
```

Supported types: `bit`, `int`, `uint`, `float`, `angle`, `bool`, `duration`, `stretch`, `complex`, and arrays of any of these.

Outputs are declared with `output`:

```openqasm
output bit[4] result;
```

## Running a Program

Basic usage:

```bash
quantumvm program.qasm
```

### Supplying Input Values

**Individual values** — `--input <key>=<value>` (repeatable):

```bash
quantumvm program.qasm --input n=5 --input theta=3.14 --input flag=true
```

Value formats:
- **Integers**: `42`, `0`, `-7`
- **Floats**: `3.14`, `-0.5`, `1e-4`
- **Booleans**: `true`, `false`
- **Bits**: `0` or `1`
- **Arrays**: `[1,2,3]` or nested `[[1,2],[3,4]]`

**JSON file** — `--inputs <file.json>`:

```bash
quantumvm program.qasm --inputs values.json
```

The JSON file must be a flat object. Values can be numbers, booleans, strings, or arrays:

```json
{
  "n": 5,
  "theta": 3.14,
  "flag": true,
  "matrix": [[1, 2], [3, 4]]
}
```

You can combine both sources. `--input` takes precedence over `--inputs` — if a key exists in both, the CLI value wins.

## What Happens at Runtime

1. QuantumVM scans the program for all `input` declarations.
2. For each declared input, it looks up the value — first in `--input` arguments, then in the JSON file.
3. The value is parsed and validated against the declared type (e.g. `int[32]` expects an integer, `float[64]` accepts `42` and converts it to `42.0`).
4. If a declared input has no supplied value, execution fails with a **missing required input** error.
5. If a value can't be parsed or doesn't match the type, execution fails with a descriptive error (including whether the value came from CLI or JSON).
6. If you supply a value for a variable that isn't declared as `input`, a warning is printed but execution continues — it's ignored.

## Outputs

After a successful run, output values are printed to stdout:

```
result = 1010
```

## Examples

```bash
# Just run with no inputs
quantumvm hello.qasm

# Supply inputs individually
quantumvm bell.qasm --input theta=1.57 --input qubits=2

# Use a JSON file
quantumvm grover.qasm --inputs grover_params.json

# Mix CLI and JSON (CLI overrides JSON for 'n')
quantumvm program.qasm --inputs defaults.json --input n=100
```
