# Parameter schema work plan

Scope: every point-type table in Chapter 3 of [D301180X012, October 2022](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).
This is an implementation inventory, not a reproduction of the vendor tables.
See [roadmap](OUTSTANDING_WORK.md) and [service plan](SPEC_WORK_PLAN.md).

## Common schema infrastructure

The provider boundary is an early deliverable in [EXT-3](EXTENSIBILITY.md).
Users can supply their own schemas before the complete built-in catalog exists.
Provider extensibility and a particular built-in PT row have separate completion
criteria; caller extensions do not imply verified firmware coverage for the crate.

Research leads and source qualifications are recorded in
[DEC-10 findings](reference/AMBIGUITIES.md#dec-10--concrete-schema-validation-lead).
PT-177/0 has a published driver-width correction to verify; PT-91 has distinct
layout and enhanced-security fields. These leads do not close catalog rows.

- [ ] Define a schema identity including device family, firmware applicability, point type and parameter number.
- [ ] Model width/type, name, units where documented, access (including conditional access), ranges, defaults, update ownership, enumerations, bit fields and reserved fields.
- [ ] Represent version-introduced/changed/deprecated parameters without selecting a firmware version from opcode 6 revision alone.
- [ ] Separate known metadata from unspecified metadata; do not invent ranges, defaults or units.
- [ ] Model logical versus physical location rules from sections 3.1-3.2; validate against opcode 50 discovery.
- [ ] Support caller-defined/vendor schemas and opcode 100 classifications without claiming that classifications disclose parameter widths.
- [ ] Define unknown-parameter behavior and explicit overrides for firmware not yet catalogued.
- [ ] Build schema-aware reads and writes over 167/180 and 166/181; preserve raw access.
- [ ] Define batch splitting by request AND response budgets; expose partial progress and avoid claiming atomicity across exchanges.
- [ ] Generate or maintain catalog artifacts reproducibly; review redistribution/licensing before copying vendor text.
- [ ] Add catalog consistency checks for duplicate keys, invalid widths, overlapping applicability and missing source references.

## Completion contract for EACH row

A row closes only when all parameters in its referenced table are accounted for,
including reserved and conditional entries. Keep a per-parameter manifest beside
the eventual catalog, with these fields:

`point type | parameter | manual section | firmware/device applicability | type/width | access | constraints/bit meanings | implementation | literal fixture | integration evidence | hardware evidence | unresolved issue`

For each row:

1. Inventory all parameter numbers and variants; record omissions and contradictions explicitly.
2. Implement metadata and useful typed access, retaining unknown enumeration/bit values.
3. Verify independent literals, boundary values, access rules and location mapping.
4. Exercise representative reads and applicable writes over TCP; cross-check all catalog metadata against the source.
5. Record physical evidence separately, with model/firmware; catalogue completion does not establish hardware compatibility.

Status: all rows below are **open**. Existing caller-supplied `DataType` reads and
a handful of literal values do not constitute a built-in schema for any row.
“Hardware: none” applies to every row. Introduce separate implementation/test/hardware
statuses as work lands; never replace them with one ambiguous “supported” flag.

| ID | Point type | Area | Manual section | Catalog status |
| --- | --- | --- | --- | --- |
| PT-82 | 82 | Virtual discrete outputs | 3.4.1 | Open |
| PT-84 | 84 | Extended HART | 3.4.2 | Open |
| PT-85 | 85 | HART | 3.4.3 | Open |
| PT-91 | 91 | System variables | 3.4.4 | Open |
| PT-92 | 92 | Logon | 3.4.5 | Open |
| PT-95 | 95 | Communication ports | 3.4.6 | Open |
| PT-96 | 96 | FST | 3.4.7 | Open |
| PT-97 | 97 | FST register tags | 3.4.8 | Open |
| PT-98 | 98 | Soft points | 3.4.9 | Open |
| PT-99 | 99 | Configurable opcode tables | 3.4.10 | Open |
| PT-100 | 100 | Power control | 3.4.11 | Open |
| PT-101 | 101 | Discrete inputs | 3.4.12 | Open |
| PT-102 | 102 | Discrete outputs | 3.4.13 | Open |
| PT-103 | 103 | Analog inputs | 3.4.14 | Open |
| PT-104 | 104 | Analog outputs | 3.4.15 | Open |
| PT-105 | 105 | Pulse inputs | 3.4.16 | Open |
| PT-106 | 106 | RTD | 3.4.17 | Open |
| PT-107 | 107 | Thermocouple | 3.4.18 | Open |
| PT-108 | 108 | Multi-variable sensor | 3.4.19 | Open |
| PT-109 | 109 | System analog inputs | 3.4.20 | Open |
| PT-110 | 110 | PID | 3.4.21 | Open |
| PT-111 | 111 | Sampler/odorizer | 3.4.22 | Open |
| PT-112 | 112 | Station | 3.4.23 | Open |
| PT-113 | 113 | Orifice meter configuration | 3.4.24 | Open |
| PT-114 | 114 | Orifice meter values | 3.4.25 | Open |
| PT-115 | 115 | Turbine meter configuration | 3.4.26 | Open |
| PT-116 | 116 | Turbine meter values | 3.4.27 | Open |
| PT-117 | 117 | Modbus configuration | 3.4.28 | Open |
| PT-118 | 118 | Modbus register/TLP mapping | 3.4.29 | Open |
| PT-119 | 119 | Modbus log/history tables | 3.4.30 | Open |
| PT-120 | 120 | Modbus master modem | 3.4.31 | Open |
| PT-121 | 121 | Modbus master tables | 3.4.32 | Open |
| PT-122 | 122 | DS800 | 3.4.33 | Open |
| PT-123 | 123 | Security groups | 3.4.34 | Open |
| PT-124 | 124 | History segments | 3.4.35 | Open |
| PT-125 | 125 | History segment 0 points | 3.4.36 | Open |
| PT-126 | 126 | History segment 1 points | 3.4.37 | Open |
| PT-127 | 127 | History segment 2 points | 3.4.38 | Open |
| PT-128 | 128 | History segment 3 points | 3.4.39 | Open |
| PT-129 | 129 | History segment 4 points | 3.4.40 | Open |
| PT-130 | 130 | History segment 5 points | 3.4.41 | Open |
| PT-131 | 131 | History segment 6 points | 3.4.42 | Open |
| PT-132 | 132 | History segment 7 points | 3.4.43 | Open |
| PT-133 | 133 | History segment 8 points | 3.4.44 | Open |
| PT-134 | 134 | History segment 9 points | 3.4.45 | Open |
| PT-135 | 135 | History segment 10 points | 3.4.46 | Open |
| PT-136 | 136 | Clock | 3.4.47 | Open |
| PT-137 | 137 | Internet configuration | 3.4.48 | Open |
| PT-138 | 138 | User C++ host | 3.4.49 | Open |
| PT-139 | 139 | Smart I/O modules | 3.4.50 | Open |
| PT-140 | 140 | AC I/O | 3.4.51 | Open |
| PT-141 | 141 | Advanced pulse module | 3.4.52 | Open |
| PT-142 | 142 | History segment 11 points | 3.4.53 | Open |
| PT-143 | 143 | History segment 12 points | 3.4.54 | Open |
| PT-144 | 144 | Transaction history configuration | 3.4.55 | Open |
| PT-145 | 145 | Transaction history points | 3.4.56 | Open |
| PT-172 | 172 | RTU network discovery | 3.4.57 | Open |
| PT-173 | 173 | Network commissioned list | 3.4.58 | Open |
| PT-174 | 174 | Network export | 3.4.59 | Open |
| PT-175 | 175 | Network import | 3.4.60 | Open |
| PT-176 | 176 | IEC62591 live list | 3.4.61 | Open |
| PT-177 | 177 | IEC62591 commissioned list | 3.4.62 | Open |

## Suggested catalog delivery batches

These batches determine delivery order; every row remains in scope.

- Foundation: PT-91, 92, 95, 99, 136, 139; supports security, discovery and common services.
- Basic I/O: PT-82, 100-107, 109, 140, 141; then PT-84, 85, 108.
- Control and metering: PT-110-116.
- History: PT-124-135, 142-145.
- Application and communication: PT-96-98, 117-123, 137-138.
- Network: PT-172-177.

Completion of all batches includes the documented parameter interfaces for HART,
Modbus, DS800, user programs and wireless networks. It does not implement those
other protocols themselves or undocumented ROC services.
