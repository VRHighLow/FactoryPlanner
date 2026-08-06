# ERA 1 — Core Recipe Fixes v1.1
## Apply on top of RECIPE_DATABASE 001–500 explicit recipes

Global: remove Heat Energy; fill missing schema fields; fix banned IDs.

---

### R013 — Ferrite Smelting (FIXED)
```
recipe_id: era1_recipe_ferrite_smelting
name: Ferrite Smelting
category: metallurgy
machine: era1_machine_thermal_smelter_mk1
inputs:
  - { id: era1_material_ferrite_powder, amount: 10 }
  - { id: era1_material_carbon_powder, amount: 2 }
outputs:
  - { id: era1_material_ferrite_plate, amount: 8 }
waste_outputs:
  - { id: era1_waste_carbon_residue, amount: 1 }
processing_time: 12
power_consumption: { thermal: 400 }
purity_effect: -2
grade_effect: industrial
technology_unlock: era1_tech_basic_metallurgy
description: Smelts ferrite powder into structural plates.
```

### R031 — Graphite Production (FIXED)
```
recipe_id: era1_recipe_graphite
name: Graphite Production
category: carbon
machine: era1_machine_carbon_furnace_mk1
inputs:
  - { id: era1_material_carbon_powder, amount: 10 }
outputs:
  - { id: era1_material_graphite, amount: 8 }
waste_outputs:
  - { id: era1_waste_carbon_residue, amount: 1 }
processing_time: 15
power_consumption: { thermal: 500 }
purity_effect: +3
grade_effect: industrial
technology_unlock: era1_tech_carbon_processing
description: Thermally converts carbon powder into graphite.
```

### R042 — Glass Production (FIXED)
```
recipe_id: era1_recipe_glass
name: Glass Production
category: silicate
machine: era1_machine_ceramic_furnace_mk1
inputs:
  - { id: era1_material_silicon_sand, amount: 5 }
outputs:
  - { id: era1_material_glass, amount: 5 }
waste_outputs: []
processing_time: 12
power_consumption: { thermal: 350 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_ceramic_engineering
description: Melts silicon sand into industrial glass.
```

### R043 — Ceramic Compound (FIXED inputs)
```
recipe_id: era1_recipe_ceramic_compound
name: Ceramic Compound
category: silicate
machine: era1_machine_ceramic_furnace_mk1
inputs:
  - { id: era1_material_silicon_powder, amount: 10 }
  - { id: era1_material_mineral_binder, amount: 2 }
outputs:
  - { id: era1_material_ceramic, amount: 8 }
waste_outputs:
  - { id: era1_waste_stone_dust, amount: 1 }
processing_time: 18
power_consumption: { thermal: 450 }
purity_effect: +2
grade_effect: industrial
technology_unlock: era1_tech_ceramic_engineering
description: Forms ceramic compound from silicon powder and mineral binder.
```

### R060 — Atmospheric Condensation (FIXED)
```
recipe_id: era1_recipe_atmospheric_condensation
name: Atmospheric Condensation
category: water
machine: era1_machine_atmospheric_condenser_mk1
inputs:
  - { id: era1_gas_atmospheric_mix, amount: 50 }
outputs:
  - { id: era1_fluid_condensed_water, amount: 20 }
waste_outputs: []
processing_time: 20
power_consumption: { electrical: 500 }
purity_effect: 0
grade_effect: none
technology_unlock: era1_tech_fluid_engineering
description: Condenses atmospheric mix into industrial water.
```

### R061 — Water Purification (FIXED)
```
recipe_id: era1_recipe_water_purification
name: Water Purification
category: water
machine: era1_machine_water_purifier_mk1
inputs:
  - { id: era1_fluid_condensed_water, amount: 20 }
  - { id: era1_component_chemical_filter, amount: 1 }
outputs:
  - { id: era1_fluid_purified_water, amount: 15 }
waste_outputs:
  - { id: era1_waste_mineral_slurry, amount: 5 }
processing_time: 15
power_consumption: { electrical: 200 }
purity_effect: +15
grade_effect: none
technology_unlock: era1_tech_fluid_engineering
description: Filters condensed water into purified industrial water.
```

### R062 — Ultra Pure Water (FIXED)
```
recipe_id: era1_recipe_ultra_pure_water
name: Ultra Pure Water
category: water
machine: era1_machine_precision_water_processor_mk1
inputs:
  - { id: era1_fluid_purified_water, amount: 20 }
  - { id: era1_component_filter_cartridge, amount: 1 }
outputs:
  - { id: era1_fluid_ultra_pure_water, amount: 15 }
waste_outputs:
  - { id: era1_waste_chemical_residue, amount: 2 }
processing_time: 25
power_consumption: { electrical: 350 }
purity_effect: +25
grade_effect: none
technology_unlock: era1_tech_precision_chemistry
description: Produces electronics-grade ultra pure water.
```

### R081 — Mechanical Shaft (FIXED lubricant)
```
recipe_id: era1_recipe_mechanical_shaft
name: Mechanical Shaft
category: mechanical
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_material_ferrite_plate, amount: 3 }
  - { id: era1_fluid_lubricant, amount: 1 }
outputs:
  - { id: era1_component_mechanical_shaft, amount: 2 }
waste_outputs: []
processing_time: 8
power_consumption: { electrical: 80 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_industrial_automation
description: Machines ferrite into lubricated mechanical shafts.
```

### R083 — Bearing (FIXED)
```
recipe_id: era1_recipe_bearing
name: Bearing
category: mechanical
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_material_ferrite_plate, amount: 2 }
  - { id: era1_fluid_lubricant, amount: 1 }
outputs:
  - { id: era1_component_bearing, amount: 2 }
waste_outputs: []
processing_time: 8
power_consumption: { electrical: 80 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_industrial_automation
description: Assembles lubricated bearings.
```

### R085 — Servo Motor (FIXED circuit)
```
recipe_id: era1_recipe_servo_motor
name: Servo Motor
category: mechanical
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_component_gear, amount: 2 }
  - { id: era1_material_conductive_wire, amount: 5 }
  - { id: era1_component_control_module, amount: 1 }
outputs:
  - { id: era1_component_servo_motor, amount: 1 }
waste_outputs: []
processing_time: 12
power_consumption: { electrical: 120 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_industrial_automation
description: Assembles a servo motor from gear, wire, and control module.
```

### R116 — Gas Separation (FIXED atmosphere ID)
```
recipe_id: era1_recipe_gas_separation
name: Gas Separation
category: chemistry
machine: era1_machine_atmospheric_separator_mk1
inputs:
  - { id: era1_gas_atmospheric_mix, amount: 100 }
outputs:
  - { id: era1_gas_nitrogen, amount: 30 }
  - { id: era1_gas_oxygen, amount: 20 }
  - { id: era1_gas_carbon_dioxide, amount: 10 }
waste_outputs: []
processing_time: 20
power_consumption: { electrical: 400 }
purity_effect: 0
grade_effect: none
technology_unlock: era1_tech_fluid_engineering
description: Separates atmospheric mix into industrial gases.
```

### R118 — Acid Solution (clarify water)
```
recipe_id: era1_recipe_acid_solution
name: Acid Solution
category: chemistry
machine: era1_machine_chemical_reactor_mk1
inputs:
  - { id: era1_fluid_purified_water, amount: 10 }
  - { id: era1_fluid_chemical_feedstock, amount: 5 }
outputs:
  - { id: era1_fluid_acid_solution, amount: 10 }
waste_outputs:
  - { id: era1_waste_chemical_residue, amount: 1 }
processing_time: 12
power_consumption: { electrical: 250 }
purity_effect: 0
grade_effect: none
technology_unlock: era1_tech_chemical_manufacturing
description: Synthesizes acid solution for purification and chemistry.
```

### R322–R325 — Science Data (FIXED specific inputs)
```
recipe_id: era1_recipe_engineering_data
name: Engineering Data
category: science
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_component_precision_mechanical_assembly, amount: 2 }
  - { id: era1_component_gear, amount: 2 }
  - { id: era1_component_bearing, amount: 1 }
  - { id: era1_component_data_storage_module, amount: 2 }
outputs:
  - { id: era1_science_engineering_data, amount: 1 }
waste_outputs: []
processing_time: 30
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: none
technology_unlock: era1_tech_research_infrastructure
description: Validates mechanical engineering samples into research data.
```

```
recipe_id: era1_recipe_chemical_data
name: Chemical Data
category: science
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_material_polymer_resin, amount: 5 }
  - { id: era1_fluid_catalyst_solution, amount: 2 }
  - { id: era1_fluid_acid_solution, amount: 2 }
  - { id: era1_component_data_storage_module, amount: 2 }
outputs:
  - { id: era1_science_chemical_data, amount: 1 }
waste_outputs:
  - { id: era1_waste_chemical_residue, amount: 1 }
processing_time: 30
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: none
technology_unlock: era1_tech_research_infrastructure
description: Validates chemical samples into research data.
```

```
recipe_id: era1_recipe_computational_data
name: Computational Data
category: science
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_component_logic_board, amount: 3 }
  - { id: era1_component_basic_circuit, amount: 5 }
  - { id: era1_component_data_storage_module, amount: 2 }
outputs:
  - { id: era1_science_computational_data, amount: 1 }
waste_outputs: []
processing_time: 30
power_consumption: { electrical: 250 }
purity_effect: 0
grade_effect: none
technology_unlock: era1_tech_research_infrastructure
description: Validates electronic systems into computational research data.
```

```
recipe_id: era1_recipe_defense_data
name: Defense Data
category: science
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_military_military_control_unit, amount: 1 }
  - { id: era1_military_weapon_housing, amount: 1 }
  - { id: era1_military_targeting_module, amount: 1 }
  - { id: era1_component_data_storage_module, amount: 2 }
outputs:
  - { id: era1_science_defense_data, amount: 1 }
waste_outputs: []
processing_time: 35
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: none
technology_unlock: era1_tech_defense_research
description: Validates military systems into defense research data.
```

---

## Missing chain recipes (from patch plan)

### Advanced Ceramic
```
recipe_id: era1_recipe_advanced_ceramic
name: Advanced Ceramic
category: ceramics
machine: era1_machine_ceramic_furnace_mk2
inputs:
  - { id: era1_material_ceramic, amount: 10 }
  - { id: era1_material_mineral_binder, amount: 5 }
  - { id: era1_fluid_ultra_pure_water, amount: 2 }
outputs:
  - { id: era1_material_advanced_ceramic, amount: 10 }
waste_outputs:
  - { id: era1_waste_stone_dust, amount: 1 }
processing_time: 30
power_consumption: { thermal: 800 }
purity_effect: +5
grade_effect: industrial_plus
technology_unlock: era1_tech_advanced_ceramics
description: Fires ceramic compound into advanced high-temp ceramic.
```

### Silicon Powder
```
recipe_id: era1_recipe_silicon_powder
name: Grind Silicon Powder
category: silicate
machine: era1_machine_industrial_grinder_mk1
inputs:
  - { id: era1_material_refined_silicon, amount: 10 }
outputs:
  - { id: era1_material_silicon_powder, amount: 10 }
waste_outputs:
  - { id: era1_waste_stone_dust, amount: 1 }
processing_time: 8
power_consumption: { mechanical: 150 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_ceramic_engineering
description: Grinds refined silicon into powder.
```

### Mineral Compound / Binder
```
recipe_id: era1_recipe_mineral_compound
name: Mineral Compound
category: silicate
machine: era1_machine_chemical_processor_mk1
inputs:
  - { id: era1_material_mineral_dust, amount: 10 }
  - { id: era1_fluid_alkaline_solution, amount: 2 }
outputs:
  - { id: era1_material_mineral_compound, amount: 10 }
waste_outputs:
  - { id: era1_waste_mineral_slurry, amount: 1 }
processing_time: 10
power_consumption: { electrical: 120 }
purity_effect: +4
grade_effect: none
technology_unlock: era1_tech_chemical_manufacturing
description: Treats mineral dust into mineral compound.
```

```
recipe_id: era1_recipe_mineral_binder
name: Mineral Binder
category: silicate
machine: era1_machine_chemical_processor_mk1
inputs:
  - { id: era1_material_mineral_compound, amount: 10 }
  - { id: era1_fluid_chemical_additive, amount: 2 }
outputs:
  - { id: era1_material_mineral_binder, amount: 8 }
waste_outputs: []
processing_time: 12
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_chemical_manufacturing
description: Processes mineral compound into ceramic binder.
```

### Reinforced Glass
```
recipe_id: era1_recipe_reinforced_glass
name: Reinforced Glass
category: silicate
machine: era1_machine_ceramic_furnace_mk1
inputs:
  - { id: era1_material_glass, amount: 5 }
  - { id: era1_material_polymer_resin, amount: 2 }
outputs:
  - { id: era1_material_reinforced_glass, amount: 5 }
waste_outputs: []
processing_time: 15
power_consumption: { thermal: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_polymer_science
description: Laminates glass with polymer resin.
```

### Chemical Filter / Cartridge
```
recipe_id: era1_recipe_chemical_filter
name: Chemical Filter
category: chemistry
machine: era1_machine_component_fabricator_mk1
inputs:
  - { id: era1_material_synthetic_fiber, amount: 3 }
  - { id: era1_material_advanced_ceramic, amount: 2 }
outputs:
  - { id: era1_component_chemical_filter, amount: 5 }
waste_outputs: []
processing_time: 10
power_consumption: { electrical: 100 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Fabricates chemical filter media.
```

```
recipe_id: era1_recipe_filter_cartridge
name: Filter Cartridge
category: chemistry
machine: era1_machine_component_fabricator_mk1
inputs:
  - { id: era1_component_chemical_filter, amount: 2 }
  - { id: era1_component_machine_housing, amount: 1 }
outputs:
  - { id: era1_component_filter_cartridge, amount: 2 }
waste_outputs: []
processing_time: 8
power_consumption: { electrical: 80 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_precision_chemistry
description: Seals filters into precision cartridges.
```

### Chemical Additive / Electrolyte
```
recipe_id: era1_recipe_chemical_additive
name: Chemical Additive
category: chemistry
machine: era1_machine_chemical_reactor_mk1
inputs:
  - { id: era1_fluid_chemical_feedstock, amount: 10 }
  - { id: era1_fluid_catalyst_solution, amount: 2 }
outputs:
  - { id: era1_fluid_chemical_additive, amount: 10 }
waste_outputs:
  - { id: era1_waste_chemical_residue, amount: 1 }
processing_time: 12
power_consumption: { electrical: 200 }
purity_effect: +2
grade_effect: none
technology_unlock: era1_tech_chemical_manufacturing
description: Synthesizes specialty chemical additive.
```

```
recipe_id: era1_recipe_electrolyte
name: Battery Electrolyte
category: chemistry
machine: era1_machine_chemical_reactor_mk1
inputs:
  - { id: era1_fluid_acid_solution, amount: 10 }
  - { id: era1_material_conductive_trace, amount: 5 }
outputs:
  - { id: era1_fluid_electrolyte, amount: 10 }
waste_outputs:
  - { id: era1_waste_chemical_residue, amount: 1 }
processing_time: 15
power_consumption: { electrical: 220 }
purity_effect: +3
grade_effect: none
technology_unlock: era1_tech_power_systems
description: Produces battery electrolyte from acid and conductive traces.
```

### Heat Exchanger Plate
```
recipe_id: era1_recipe_heat_exchanger_plate
name: Heat Exchanger Plate
category: mechanical
machine: era1_machine_precision_fabricator_mk1
inputs:
  - { id: era1_material_hardened_steel, amount: 5 }
  - { id: era1_component_reinforced_pipe, amount: 3 }
  - { id: era1_material_industrial_coating, amount: 2 }
outputs:
  - { id: era1_component_heat_exchanger_plate, amount: 2 }
waste_outputs: []
processing_time: 20
power_consumption: { electrical: 180 }
purity_effect: 0
grade_effect: precision
technology_unlock: era1_tech_advanced_automation
description: Fabricates heat exchanger plates for cooling assemblies.
```

### Atmospheric Intake
```
recipe_id: era1_recipe_atmospheric_intake
name: Atmospheric Intake
category: water
machine: era1_machine_atmospheric_intake_mk1
inputs: []
outputs:
  - { id: era1_gas_atmospheric_mix, amount: 50 }
waste_outputs: []
processing_time: 10
power_consumption: { electrical: 100 }
purity_effect: 0
grade_effect: none
technology_unlock: era1_tech_fluid_engineering
description: Compresses ambient atmosphere into processable atmospheric mix.
```

### Machine part kits
```
recipe_id: era1_recipe_electronics_printer_parts
name: Electronics Printer Parts
category: machine_parts
machine: era1_machine_precision_component_fabricator_mk1
inputs:
  - { id: era1_component_precision_housing, amount: 2 }
  - { id: era1_component_basic_circuit, amount: 5 }
  - { id: era1_material_conductive_foil, amount: 5 }
outputs:
  - { id: era1_component_electronics_printer_parts, amount: 1 }
waste_outputs: []
processing_time: 20
power_consumption: { electrical: 150 }
purity_effect: 0
grade_effect: precision
technology_unlock: era1_tech_electronics
description: Builds parts kit for electronics printers.
```

```
recipe_id: era1_recipe_assembler_parts
name: Assembler Parts
category: machine_parts
machine: era1_machine_component_fabricator_mk1
inputs:
  - { id: era1_component_gear, amount: 4 }
  - { id: era1_component_servo_motor, amount: 2 }
  - { id: era1_component_machine_housing, amount: 2 }
outputs:
  - { id: era1_component_assembler_parts, amount: 1 }
waste_outputs: []
processing_time: 18
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_industrial_automation
description: Builds modular assembler construction kit.
```

```
recipe_id: era1_recipe_chemical_storage_parts
name: Chemical Storage Parts
category: machine_parts
machine: era1_machine_component_assembler_mk1
inputs:
  - { id: era1_component_reinforced_pipe, amount: 5 }
  - { id: era1_component_pressure_chamber, amount: 1 }
outputs:
  - { id: era1_component_chemical_storage_parts, amount: 1 }
waste_outputs: []
processing_time: 15
power_consumption: { electrical: 120 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Prefabricates chemical tank construction parts.
```
