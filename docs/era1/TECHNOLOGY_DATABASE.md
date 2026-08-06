# ERA 1 — TECHNOLOGY DATABASE v1.1
## Planetary Recovery Era (reconciled with recipe/machine IDs)

Source: User Tech DB v1.0 (T001–T027) + bridge techs for recipe-referenced IDs.

### Science types
- `engineering_data` → `era1_science_engineering_data`
- `chemical_data` → `era1_science_chemical_data`
- `computational_data` → `era1_science_computational_data`
- `defense_data` → `era1_science_defense_data`

### Progression spine
```
Survival (T001)
  → Extraction (T002) → Crushing (T003) → Refinement (T004)
  → Structure (T005) → Metallurgy (T006)
  → Fluids/Water (T011) → Chem (T008) / Hydrocarbon (T009) / Atmosphere (T010)
  → Polymers (T012) → Ceramics (T007)
  → Automation (T013–T015) → Electronics (T016–T017)
  → Power (T018–T019) → Defense (T020–T022) → Laser (T021b)
  → Optimization (T023) → Advanced Mfg (T024)
  → Nexus (T025–T027) → ERA 2
```

---

### T001 — Planetary Recovery Protocol
```
tech_id: era1_tech_basic_recovery
name: Planetary Recovery Protocol
tier: 0
era: 1
description: Bootstrap systems for reclaiming abandoned planetary infrastructure.
purpose: Starting technology. Enables emergency survival industry.
science_cost:
  {}
science_types: []
prerequisites:
  []
unlocks:
  - era1_machine_ferrite_drill_mk1 (limited starter)
  - Hand crafting / emergency fabricator recipes
  - era1_logistics_conveyor_segment (basic)
  - era1_machine_storage_container
  - era1_machine_compact_generator / solar starter
  - Alias: era1_tech_recovery_protocol
research_time: 0s
```
### T002 — Basic Extraction Systems
```
tech_id: era1_tech_basic_extraction
name: Basic Extraction Systems
tier: 1
era: 1
description: Automated drills for the four solid planetary deposits.
purpose: Unlock full mining suite.
science_cost:
  engineering_data: 20
science_types: [engineering_data]
prerequisites:
  - era1_tech_basic_recovery
unlocks:
  - era1_machine_ferrite_drill_mk1
  - era1_machine_conductive_drill_mk1
  - era1_machine_carbon_drill_mk1
  - era1_machine_silicate_drill_mk1
  - Recipes: extract_ferrite/conductive/carbon/silicate
research_time: 60s
```
### T003 — Material Crushing
```
tech_id: era1_tech_material_processing
name: Material Crushing
tier: 1
era: 1
description: Primary size reduction for ores and minerals.
purpose: Crushing and grinding.
science_cost:
  engineering_data: 40
science_types: [engineering_data]
prerequisites:
  - era1_tech_basic_extraction
unlocks:
  - era1_machine_crusher_mk1
  - era1_machine_industrial_grinder_mk1
  - Recipes: crushed ferrite/conductive, carbon powder, silicon sand, mineral dust
research_time: 90s
```
### T004 — Ore Refinement
```
tech_id: era1_tech_basic_metallurgy
name: Ore Refinement
tier: 1
era: 1
description: Purification and smelting into structural plates.
purpose: First refined metals.
science_cost:
  engineering_data: 50
  chemical_data: 20
science_types: [engineering_data, chemical_data]
prerequisites:
  - era1_tech_material_processing
unlocks:
  - era1_machine_ore_purifier_mk1
  - era1_machine_thermal_smelter_mk1
  - era1_machine_material_processor_mk1
  - Recipes: purified ferrite, ferrite/conductive plate, ferrite powder
research_time: 120s
```
### T005 — Structural Engineering
```
tech_id: era1_tech_structural_engineering
name: Structural Engineering
tier: 1
era: 1
description: Basic structural and machine framing components.
purpose: Frames, housings, pipes.
science_cost:
  engineering_data: 75
science_types: [engineering_data]
prerequisites:
  - era1_tech_basic_metallurgy
unlocks:
  - Recipes: structural_frame, machine_housing, pipe_segment, mechanical_shaft, gear, bearing
  - era1_machine_assembler_mk1 (limited early recipes)
research_time: 120s
```
### T006 — Industrial Metallurgy
```
tech_id: era1_tech_advanced_metallurgy
name: Industrial Metallurgy
tier: 2
era: 1
description: Steels, alloys, advanced furnaces and presses.
purpose: Heavy metal industry.
science_cost:
  engineering_data: 100
  chemical_data: 50
science_types: [engineering_data, chemical_data]
prerequisites:
  - era1_tech_basic_metallurgy
unlocks:
  - era1_machine_alloy_furnace_mk1
  - era1_machine_heat_treatment_furnace_mk1
  - era1_machine_advanced_purifier_mk1
  - era1_machine_reduction_furnace_mk1
  - era1_machine_hydraulic_press_mk1
  - era1_machine_heavy_assembler_mk1
  - Recipes: steel_composite, hardened_steel, alloy_blank, reinforced_ferrite, metal powder
research_time: 180s
```
### T007 — Advanced Ceramic Processing
```
tech_id: era1_tech_advanced_ceramics
name: Advanced Ceramic Processing
tier: 2
era: 1
description: High-temperature ceramics and furnace linings.
purpose: Ceramic / glass advanced path.
science_cost:
  chemical_data: 100
  engineering_data: 50
science_types: [chemical_data, engineering_data]
prerequisites:
  - era1_tech_ceramic_engineering
unlocks:
  - era1_machine_ceramic_furnace_mk2
  - Recipes: advanced_ceramic, heat_resistant_ceramic, reactor_lining, ceramic_composite
research_time: 180s
```
### T007b — Ceramic Engineering
```
tech_id: era1_tech_ceramic_engineering
name: Ceramic Engineering
tier: 1
era: 1
description: Basic glass and ceramic compound production.
purpose: Prerequisite ceramic tier (inserted to satisfy recipe refs).
science_cost:
  engineering_data: 40
  chemical_data: 20
science_types: [engineering_data, chemical_data]
prerequisites:
  - era1_tech_material_processing
unlocks:
  - era1_machine_ceramic_furnace_mk1
  - Recipes: glass, ceramic_compound, silicon_powder, reinforced_glass (with polymer)
research_time: 90s
```
### T008 — Industrial Chemistry
```
tech_id: era1_tech_chemical_manufacturing
name: Industrial Chemistry
tier: 2
era: 1
description: Reactors, acids, catalysts, solvents, additives.
purpose: Core chemistry unlock.
science_cost:
  chemical_data: 150
science_types: [chemical_data]
prerequisites:
  - era1_tech_fluid_engineering
unlocks:
  - era1_machine_chemical_reactor_mk1
  - era1_machine_chemical_reactor_mk2
  - era1_machine_chemical_processor_mk1
  - era1_machine_fluid_processor_mk1
  - era1_machine_coating_unit_mk1
  - Recipes: acid, alkaline, catalyst, solvent, additive, electrolyte paths
research_time: 200s
```
### T009 — Hydrocarbon Refining
```
tech_id: era1_tech_hydrocarbon_refining
name: Hydrocarbon Refining
tier: 2
era: 1
description: Pump and fractionate planetary hydrocarbons.
purpose: Oil → feedstock/polymers fuels.
science_cost:
  chemical_data: 150
  engineering_data: 50
science_types: [chemical_data, engineering_data]
prerequisites:
  - era1_tech_fluid_engineering
unlocks:
  - era1_machine_hydrocarbon_pump_mk1
  - era1_machine_distillation_tower_mk1
  - Recipes: fractions, chemical_feedstock, lubricant, fuel_oil, bitumen
research_time: 200s
```
### T010 — Atmospheric Processing
```
tech_id: era1_tech_atmospheric_processing
name: Atmospheric Processing
tier: 2
era: 1
description: Intake and separate planetary atmosphere.
purpose: Industrial gases.
science_cost:
  chemical_data: 100
science_types: [chemical_data]
prerequisites:
  - era1_tech_fluid_engineering
unlocks:
  - era1_machine_atmospheric_intake_mk1
  - era1_machine_atmospheric_separator_mk1
  - era1_machine_electrochemical_separator_mk1
  - Recipes: atmospheric_mix, N2/O2/CO2, H2/O2 electrolysis
research_time: 160s
```
### T011 — Water Recovery / Fluid Engineering
```
tech_id: era1_tech_fluid_engineering
name: Water Recovery / Fluid Engineering
tier: 2
era: 1
description: Industrial water synthesis and fluid logistics foundation.
purpose: Water + pipes + pumps + basic fluid machines.
science_cost:
  chemical_data: 150
science_types: [chemical_data]
prerequisites:
  - era1_tech_basic_metallurgy
unlocks:
  - era1_machine_atmospheric_condenser_mk1
  - era1_machine_water_purifier_mk1
  - era1_machine_fluid_pump_mk1
  - era1_machine_storage_tank
  - era1_machine_pipe_junction
  - era1_machine_chemical_filter_machine
  - era1_machine_boiler_mk1
  - era1_machine_gas_compressor
  - Recipes: condensed/purified water, chemical_filter, filter_cartridge
  - Note: merges user T011 Water Recovery with fluid engineering spine
research_time: 180s
```
### T011b — Precision Chemistry
```
tech_id: era1_tech_precision_chemistry
name: Precision Chemistry
tier: 3
era: 1
description: Ultra pure water and high-spec chemical finishing.
purpose: Electronics-grade fluids.
science_cost:
  chemical_data: 200
  computational_data: 50
science_types: [chemical_data, computational_data]
prerequisites:
  - era1_tech_fluid_engineering
  - era1_tech_chemical_manufacturing
unlocks:
  - era1_machine_precision_water_processor_mk1
  - Recipes: ultra_pure_water, optical_silicon prep
research_time: 220s
```
### T012 — Polymer Engineering
```
tech_id: era1_tech_polymer_science
name: Polymer Engineering
tier: 3
era: 1
description: Resins, rubbers, fibers, advanced polymers.
purpose: Polymer industry.
science_cost:
  chemical_data: 250
science_types: [chemical_data]
prerequisites:
  - era1_tech_hydrocarbon_refining
  - era1_tech_chemical_manufacturing
unlocks:
  - era1_machine_polymer_reactor_mk1
  - era1_machine_polymer_reactor_mk2
  - era1_machine_fiber_processor_mk1
  - era1_machine_composite_processor_mk1
  - Recipes: polymer_resin, synthetic_rubber, fiber, advanced_polymer, composites
research_time: 240s
```
### T013 — Mechanical Automation
```
tech_id: era1_tech_industrial_automation
name: Mechanical Automation
tier: 3
era: 1
description: Assemblers, fabricators, servos, motors.
purpose: Factory self-building begins.
science_cost:
  engineering_data: 250
science_types: [engineering_data]
prerequisites:
  - era1_tech_structural_engineering
unlocks:
  - era1_machine_assembler_mk1
  - era1_machine_component_fabricator_mk1
  - era1_machine_component_assembler_mk1
  - era1_machine_motor_assembly_mk1
  - era1_machine_machine_fabricator_mk1
  - Recipes: servo_motor, industrial_motor, assembler_parts, machine crafting tier1
research_time: 240s
```
### T014 — Conveyor Automation
```
tech_id: era1_tech_conveyor_automation
name: Conveyor Automation
tier: 3
era: 1
description: Belts, splitters, mergers, inserter upgrades.
purpose: Logistics throughput.
science_cost:
  engineering_data: 200
  computational_data: 50
science_types: [engineering_data, computational_data]
prerequisites:
  - era1_tech_industrial_automation
unlocks:
  - Recipes: fast_conveyor, splitter, merger, filter/fast inserters, underground conveyor
  - era1_machine_conveyor_fabricator
  - era1_machine_sorting_machine
research_time: 200s
```
### T015 — Robotic Systems
```
tech_id: era1_tech_robotics
name: Robotic Systems
tier: 3
era: 1
description: Drones, robot cores, pads, roboports.
purpose: Robotics foundation.
science_cost:
  engineering_data: 300
  computational_data: 150
science_types: [engineering_data, computational_data]
prerequisites:
  - era1_tech_industrial_automation
  - era1_tech_electronics
unlocks:
  - era1_machine_robotics_factory_mk1
  - era1_machine_robotics_component_printer_mk1
  - era1_machine_transport_hub
  - Recipes: drone_chassis, autonomous_controller, robot_core, logistic/construction drones
research_time: 300s
```
### T016 — Electronics Manufacturing
```
tech_id: era1_tech_electronics
name: Electronics Manufacturing
tier: 3
era: 1
description: Circuits, sensors, basic electronic modules.
purpose: Electronics foundation.
science_cost:
  computational_data: 250
science_types: [computational_data]
prerequisites:
  - era1_tech_basic_metallurgy
  - era1_tech_ceramic_engineering
unlocks:
  - era1_machine_electronics_printer_mk1
  - era1_machine_circuit_printer_mk1
  - era1_machine_electronics_assembler_mk1
  - era1_machine_precision_roller_mk1
  - era1_machine_component_processor_mk1
  - era1_machine_precision_component_fabricator_mk1
  - Recipes: substrate, basic_circuit, sensor, control_module, conductive_foil/wire
research_time: 240s
```
### T017 — Semiconductor Engineering
```
tech_id: era1_tech_advanced_electronics
name: Semiconductor Engineering
tier: 4
era: 1
description: Logic boards, processors, high-density circuits.
purpose: Advanced electronics.
science_cost:
  computational_data: 400
  chemical_data: 200
science_types: [computational_data, chemical_data]
prerequisites:
  - era1_tech_electronics
  - era1_tech_precision_chemistry
unlocks:
  - era1_machine_electronics_printer_mk2
  - era1_machine_electronics_printer_mk3
  - era1_machine_semiconductor_processor
  - Recipes: logic_board, processor_core, wafers, high_density_circuit, data_storage_module
research_time: 360s
```
### T018 — Energy Storage
```
tech_id: era1_tech_power_systems
name: Energy Storage
tier: 3
era: 1
description: Cells, batteries, power components.
purpose: Energy storage industry.
science_cost:
  engineering_data: 250
  chemical_data: 150
science_types: [engineering_data, chemical_data]
prerequisites:
  - era1_tech_chemical_manufacturing
  - era1_tech_electronics
unlocks:
  - era1_machine_battery_processor_mk1
  - era1_machine_battery_assembler_mk1
  - era1_machine_power_component_factory_mk1
  - era1_machine_solar_panel_mk1
  - Recipes: energy_cell, battery_pack, capacitor, electrolyte
research_time: 240s
```
### T019 — Power Distribution
```
tech_id: era1_tech_power_distribution
name: Power Distribution
tier: 3
era: 1
description: Relays, transformers, grid control.
purpose: Power grid.
science_cost:
  engineering_data: 200
  computational_data: 100
science_types: [engineering_data, computational_data]
prerequisites:
  - era1_tech_power_systems
unlocks:
  - Recipes: power_relay, transformer_core, grid_controller, HV cable, switchgear, load_balancer
research_time: 200s
```
### T020 — Defensive Engineering
```
tech_id: era1_tech_defense_industry
name: Defensive Engineering
tier: 3
era: 1
description: Walls, ballistic turrets, ammunition.
purpose: Basic defense.
science_cost:
  defense_data: 200
  engineering_data: 150
science_types: [defense_data, engineering_data]
prerequisites:
  - era1_tech_industrial_automation
  - era1_tech_advanced_metallurgy
unlocks:
  - era1_machine_military_fabricator_mk1
  - era1_machine_ammunition_factory_mk1
  - era1_machine_armor_processor_mk1
  - era1_machine_defense_fabricator
  - era1_machine_military_assembly_bay_mk1
  - Recipes: walls, ballistic_turret, standard_ammunition, armor_plate
research_time: 260s
```
### T021 — Advanced Defense Systems
```
tech_id: era1_tech_defense_research
name: Advanced Defense Systems
tier: 4
era: 1
description: Missiles, radar, defense networks, artillery, CIWS.
purpose: Advanced defense.
science_cost:
  defense_data: 500
  computational_data: 200
science_types: [defense_data, computational_data]
prerequisites:
  - era1_tech_defense_industry
  - era1_tech_advanced_electronics
unlocks:
  - era1_machine_missile_factory_mk1
  - era1_machine_defense_assembly_machine
  - era1_machine_turret_factory
  - era1_machine_ammo_logistics_factory
  - Recipes: missile_turret, radar, defense_network, artillery, targeting_computer
research_time: 400s
```
### T021b — Laser Defense
```
tech_id: era1_tech_laser_defense
name: Laser Defense
tier: 4
era: 1
description: First energy weapons — optical silicon and laser turrets.
purpose: Late Era 1 energy defense.
science_cost:
  defense_data: 400
  computational_data: 300
  chemical_data: 200
science_types: [defense_data, computational_data, chemical_data]
prerequisites:
  - era1_tech_defense_research
  - era1_tech_precision_chemistry
unlocks:
  - Recipes: optical_silicon, laser_lens, laser_emitter, laser_turret, laser_capacitor
research_time: 420s
```
### T022 — Military Automation
```
tech_id: era1_tech_military_automation
name: Military Automation
tier: 4
era: 1
description: Repair drones, ammo logistics, automated defense control.
purpose: Defense automation.
science_cost:
  defense_data: 400
  engineering_data: 300
science_types: [defense_data, engineering_data]
prerequisites:
  - era1_tech_defense_industry
  - era1_tech_robotics
unlocks:
  - era1_machine_repair_system_factory
  - Recipes: repair stations/towers, ammo hub, auto_defense_controller, fortress_core
research_time: 360s
```
### T023 — Factory Optimization
```
tech_id: era1_tech_systems_science
name: Factory Optimization
tier: 4
era: 1
description: Monitoring, quality, optimization modules, smart factory.
purpose: Factory optimization.
science_cost:
  computational_data: 500
  engineering_data: 300
science_types: [computational_data, engineering_data]
prerequisites:
  - era1_tech_advanced_automation
unlocks:
  - era1_machine_research_fabricator
  - era1_machine_logistics_controller
  - Recipes: production_monitoring, quality_control, optimization_modules, smart_factory_node, AI scheduler
research_time: 400s
```
### T023b — Advanced Automation
```
tech_id: era1_tech_advanced_automation
name: Advanced Automation
tier: 3
era: 1
description: Smart inserters, assemblers Mk2/3, controllers.
purpose: Bridge automation → optimization.
science_cost:
  engineering_data: 300
  computational_data: 200
science_types: [engineering_data, computational_data]
prerequisites:
  - era1_tech_industrial_automation
  - era1_tech_electronics
unlocks:
  - era1_machine_assembler_mk2
  - era1_machine_assembler_mk3
  - Recipes: smart/high-speed inserters, recipe locks, priority controllers
research_time: 280s
```
### T024 — Advanced Manufacturing
```
tech_id: era1_tech_precision_manufacturing
name: Advanced Manufacturing
tier: 4
era: 1
description: Heavy/precision fabricators and precision alloys.
purpose: Advanced manufacturing.
science_cost:
  engineering_data: 700
  computational_data: 400
science_types: [engineering_data, computational_data]
prerequisites:
  - era1_tech_advanced_metallurgy
  - era1_tech_advanced_automation
unlocks:
  - era1_machine_heavy_fabricator
  - era1_machine_precision_fabricator_mk1
  - era1_machine_precision_alloy_furnace_mk1
  - era1_machine_calibration_station
  - Recipes: precision_alloy, precision_housing, heat_exchanger_plate
research_time: 480s
```
### T024b — Material Science
```
tech_id: era1_tech_material_science
name: Material Science
tier: 4
era: 1
description: Purity/grade research samples and analyzers.
purpose: Signature purity/grade optimization science.
science_cost:
  engineering_data: 300
  chemical_data: 300
  computational_data: 200
science_types: [engineering_data, chemical_data, computational_data]
prerequisites:
  - era1_tech_advanced_metallurgy
  - era1_tech_research_infrastructure
unlocks:
  - Recipes: material/chemical/mechanical science samples, purity_analyzer, grade_assessor, purity/grade sorters, purity modules
research_time: 360s
```
### T024c — Waste Recovery
```
tech_id: era1_tech_waste_recovery
name: Waste Recovery
tier: 2
era: 1
description: Recovery and recycling loops.
purpose: Waste → resources.
science_cost:
  engineering_data: 80
  chemical_data: 80
science_types: [engineering_data, chemical_data]
prerequisites:
  - era1_tech_material_processing
unlocks:
  - era1_machine_recovery_plant_mk1
  - era1_machine_recycling_plant_mk1
  - era1_machine_waste_extractor_mk1
  - era1_machine_waste_treatment_plant
  - Recipes: tailings recovery, carbon/polymer recycle, waste neutralizer
research_time: 150s
```
### T024d — Carbon Processing
```
tech_id: era1_tech_carbon_processing
name: Carbon Processing
tier: 2
era: 1
description: Graphite, activated carbon, carbon chemistry.
purpose: Carbon industry.
science_cost:
  chemical_data: 80
  engineering_data: 40
science_types: [chemical_data, engineering_data]
prerequisites:
  - era1_tech_material_processing
unlocks:
  - era1_machine_carbon_furnace_mk1
  - Recipes: graphite, activated_carbon, carbon_slurry/gas paths
research_time: 140s
```
### T024e — Research Infrastructure
```
tech_id: era1_tech_research_infrastructure
name: Research Infrastructure
tier: 3
era: 1
description: Laboratories and science data production.
purpose: Enable the four science branches.
science_cost:
  engineering_data: 150
  computational_data: 100
science_types: [engineering_data, computational_data]
prerequisites:
  - era1_tech_electronics
  - era1_tech_industrial_automation
unlocks:
  - era1_machine_research_laboratory
  - era1_machine_laboratory_module
  - era1_machine_research_analyzer
  - Recipes: engineering/chemical/computational/defense data, lab modules, analyzers
research_time: 200s
```
### T025 — Nexus Construction
```
tech_id: era1_tech_nexus_construction
name: Nexus Construction
tier: 5
era: 1
description: Unlocks Planetary Fabrication Nexus component crafting.
purpose: Begin Nexus build.
science_cost:
  engineering_data: 1000
  computational_data: 800
  chemical_data: 500
science_types: [engineering_data, computational_data, chemical_data]
prerequisites:
  - era1_tech_precision_manufacturing
  - era1_tech_advanced_electronics
  - era1_tech_systems_science
unlocks:
  - era1_machine_construction_site
  - era1_machine_heavy_fabricator (nexus recipes)
  - Recipes: nexus_foundation_frame, nexus scaffolds/anchors/permits
research_time: 900s
```
### T026 — Planetary Integration
```
tech_id: era1_tech_planetary_integration
name: Planetary Integration
tier: 5
era: 1
description: Power, manufacturing, and compute cores for the Nexus.
purpose: Nexus major modules.
science_cost:
  engineering_data: 1500
  computational_data: 1200
  defense_data: 800
science_types: [engineering_data, computational_data, defense_data]
prerequisites:
  - era1_tech_nexus_construction
  - era1_tech_power_distribution
  - era1_tech_defense_research
unlocks:
  - era1_machine_energy_facility
  - era1_machine_advanced_electronics_printer
  - era1_machine_tech_nexus_interface
  - Recipes: nexus_power_core, computational_core, manufacturing_module, resource_interface, security_grid
research_time: 1200s
```
### T027 — Era Transition Protocol
```
tech_id: era1_tech_era_transition
name: Era Transition Protocol
tier: 5
era: 1
description: Final Era 1 technology. Activates Planetary Fabrication Nexus and opens Era 2.
purpose: Begins Era 2.
science_cost:
  engineering_data: 2500
  chemical_data: 1500
  computational_data: 1500
  defense_data: 1000
science_types: [engineering_data, chemical_data, computational_data, defense_data]
prerequisites:
  - era1_tech_planetary_integration
  - era1_tech_laser_defense
  - era1_tech_material_science
unlocks:
  - Recipes: planetary_fabrication_nexus commissioning, era_transition_beacon, era2_access_key
  - Gameplay: Era 2 Industrial Expansion unlocked
research_time: 1800s
```

# ID MAP — User names → canonical tech_id

| User Tech | Canonical tech_id |
|---|---|
| Planetary Recovery Protocol | `era1_tech_basic_recovery` |
| Basic Extraction Systems | `era1_tech_basic_extraction` |
| Material Crushing | `era1_tech_material_processing` |
| Ore Refinement | `era1_tech_basic_metallurgy` |
| Structural Engineering | `era1_tech_structural_engineering` |
| Industrial Metallurgy | `era1_tech_advanced_metallurgy` |
| Advanced Ceramic Processing | `era1_tech_advanced_ceramics` |
| Industrial Chemistry | `era1_tech_chemical_manufacturing` |
| Hydrocarbon Refining | `era1_tech_hydrocarbon_refining` |
| Atmospheric Processing | `era1_tech_atmospheric_processing` |
| Water Recovery | `era1_tech_fluid_engineering` |
| Polymer Engineering | `era1_tech_polymer_science` |
| Mechanical Automation | `era1_tech_industrial_automation` |
| Conveyor Automation | `era1_tech_conveyor_automation` |
| Robotic Systems | `era1_tech_robotics` |
| Electronics Manufacturing | `era1_tech_electronics` |
| Semiconductor Engineering | `era1_tech_advanced_electronics` |
| Energy Storage | `era1_tech_power_systems` |
| Power Distribution | `era1_tech_power_distribution` |
| Defensive Engineering | `era1_tech_defense_industry` |
| Advanced Defense Systems | `era1_tech_defense_research` |
| Military Automation | `era1_tech_military_automation` |
| Factory Optimization | `era1_tech_systems_science` |
| Advanced Manufacturing | `era1_tech_precision_manufacturing` |
| Nexus Construction | `era1_tech_nexus_construction` |
| Planetary Integration | `era1_tech_planetary_integration` |
| Era Transition Protocol | `era1_tech_era_transition` |

### Bridge techs added (needed by recipes, not in original 27 list as separate nodes)
- `era1_tech_ceramic_engineering`
- `era1_tech_precision_chemistry`
- `era1_tech_advanced_automation`
- `era1_tech_laser_defense`
- `era1_tech_material_science`
- `era1_tech_waste_recovery`
- `era1_tech_carbon_processing`
- `era1_tech_research_infrastructure`

**Total tech nodes defined: 27 user + 8 bridges = 35**

All 27 recipe-pending IDs are covered by canonical or bridge entries.
