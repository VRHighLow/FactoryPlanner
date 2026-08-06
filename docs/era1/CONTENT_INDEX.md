# ERA 1 — Content Bible Index (post gap-fill)

**Version:** 1.1  
**Date:** 2026-08-04  
**Status:** Recipe/item gaps filled — ready for Machine Database + Tech Tree from you

---

## What you sent (locked)

| Document | Status |
|---|---|
| Master design + 6 phases | ✔ |
| Industrial families + dependency graph | ✔ |
| No free water (all fluids industrial) | ✔ |
| Purity / Grade / Stability signature | ✔ |
| Item DB v1.0 (foundational solids) | ✔ |
| Fluid/Gas/Waste DB v1.0 | ✔ |
| Recipes 001–500 overview | ✔ |
| Recipe Patch Plan v1.0 | ✔ applied |

---

## What was generated (this pass)

| File | Contents |
|---|---|
| [ITEM_DATABASE_PATCH.md](ITEM_DATABASE_PATCH.md) | Renames + ~95 critical new item definitions |
| [ITEM_DATABASE_SUPPLEMENT.md](ITEM_DATABASE_SUPPLEMENT.md) | Auto-stubs for every remaining item ID introduced by gap recipes |
| [RECIPE_CORE_FIXES.md](RECIPE_CORE_FIXES.md) | Heat Energy removal, science input fixes, missing chain recipes |
| [RECIPE_GAPS_108-200.md](RECIPE_GAPS_108-200.md) | R108–200 fully specified |
| [RECIPE_GAPS_214-350.md](RECIPE_GAPS_214-350.md) | R214–350 fully specified |
| [RECIPE_GAPS_361-500.md](RECIPE_GAPS_361-500.md) | R361–500 fully specified |
| [MACHINE_IDS_PENDING.md](MACHINE_IDS_PENDING.md) | Machine IDs referenced (for your Machine DB) |
| [TECH_IDS_PENDING.md](TECH_IDS_PENDING.md) | Tech IDs referenced (for your Tech Tree) |

---

## Gap-fill summary

### Fixed globally
- Power is never an item (`energy_input` + kW)
- Single lubricant: `era1_fluid_lubricant`
- Circuits: `basic_circuit` / `control_module` only
- Atmosphere: `era1_gas_atmospheric_mix`
- Propellant: `era1_fluid_ballistic_propellant`
- Science inputs are specific items
- Advanced Ceramic + filter/electrolyte/heat exchanger chains added

### Placeholder ranges → full recipes
All former “includes: …” blocks now have complete schema:

`recipe_id, name, category, machine, inputs, outputs, processing_time, power_consumption, purity_effect, grade_effect, waste_outputs, technology_unlock, description`

Approx **250+** newly written/fixed recipe definitions across core fixes + gap packs (on top of your original ~280 explicit recipes).

### Vein properties (engine contract)
```
purity    → quality / refining depth / min gates
richness  → ore per tile / amount density
yield     → extraction speed (richness% style)
stability → interruptions, wear, waste spikes
```

---

## Machine Database

| File | Status |
|---|---|
| [MACHINE_DATABASE.md](MACHINE_DATABASE.md) | ✔ v1.1 — 75 canonical + 21 appendix (full schema) |

## Technology Database

| File | Status |
|---|---|
| [TECHNOLOGY_DATABASE.md](TECHNOLOGY_DATABASE.md) | ✔ v1.1 — 27 user techs + 8 bridges (35 nodes) |

## Era 1 design checklist

| Pillar | Status |
|---|---|
| Master items + patch/supplement | ✔ |
| Fluids / gases / waste | ✔ |
| Recipes 001–500 + gap fill | ✔ |
| Machines M001–075 + appendix | ✔ |
| Technologies T001–027 + bridges | ✔ |
| Enemy system | ✗ remaining |
| Military / logistics / purity balance numbers | ✗ remaining |
| Starting progression scripting | ✗ remaining |
| Implementation JSON/RON schemas | ✗ remaining | 

---

## Implementation order (recommended)

1. Freeze data-pack schema (`era1_*.toml` / ron / json)  
2. Ingest item DBs (v1 + patch + supplement)  
3. Ingest all recipe files  
4. Wire machines from your Machine DB  
5. Wire tech unlocks  
6. Implement purity/grade/stability sim  

Send machines whenever ready.
