# ERA 1 — Master Item Database Patch v1.1
## Gap fill + renames from RECIPE_PATCH_PLAN v1.0

**Status:** Complete for implementation handoff  
**Rule:** Power is never an item. Use `energy_input` on recipes/machines.

---

## Removals / Renames

| Remove / Old | Replace With |
|---|---|
| Heat Energy (any form) | Machine `power_consumption` + `energy_input` |
| `era1_fluid_carbon_lubricant` / Carbon Lubricant | `era1_fluid_lubricant` |
| Control Circuit | `era1_component_basic_circuit` (simple) or `era1_component_control_module` (complex) |
| Atmospheric Gas | `era1_gas_atmospheric_mix` |
| Generic Propellant | `era1_fluid_ballistic_propellant` |
| Generic "Mechanical Components" | Specific IDs (see science recipes) |
| Generic "Chemical Products" | Specific chemical IDs |
| Generic "Military Components" | Specific military IDs |

---

## New Items

### Mineral processing

```
id: era1_material_mineral_dust
name: Mineral Dust
era: 1
family: silicate
category: processed_mineral
stack_size: 200
purity_supported: true
grade_supported: false
produced_by: [crusher_mk1]
used_in: [mineral_compound, ceramic_additives, waste_recovery]
description: Fine mineral particulate from ore crushing. Feedstock for ceramics and chemical treatment.
```

```
id: era1_material_mineral_compound
name: Mineral Compound
era: 1
family: silicate
category: processed_mineral
stack_size: 200
purity_supported: true
grade_supported: false
produced_by: [chemical_processor_mk1]
used_in: [alkaline_solution, mineral_binder, ceramic_compound]
description: Chemically treated mineral dust used as ceramic and alkaline feedstock.
```

```
id: era1_material_mineral_binder
name: Mineral Binder
era: 1
family: silicate
category: processed_mineral
stack_size: 200
purity_supported: false
grade_supported: true
produced_by: [chemical_processor_mk1]
used_in: [ceramic_compound, advanced_ceramic, construction]
description: Binding agent for ceramics and construction composites.
```

### Silicon / glass

```
id: era1_material_silicon_powder
name: Silicon Powder
era: 1
family: silicate
category: refined_material
stack_size: 200
purity_supported: true
grade_supported: true
produced_by: [industrial_grinder_mk1]
used_in: [electronics, semiconductors, ceramic_slurry]
description: Ground refined silicon for electronics and precision ceramics.
```

```
id: era1_material_reinforced_glass
name: Reinforced Glass
era: 1
family: silicate
category: refined_material
stack_size: 100
purity_supported: true
grade_supported: true
produced_by: [ceramic_furnace_mk1]
used_in: [pressure_chamber, sensors, observation]
description: Polymer-laminated glass for pressure systems and advanced machines.
```

### Chemical / filters

```
id: era1_component_chemical_filter
name: Chemical Filter
era: 1
family: chemistry
category: component
stack_size: 100
purity_supported: false
grade_supported: true
produced_by: [component_fabricator_mk1]
used_in: [water_purifier, gas_processing]
description: Consumable filter media for water and gas purification.
```

```
id: era1_component_filter_cartridge
name: Filter Cartridge
era: 1
family: chemistry
category: component
stack_size: 50
purity_supported: false
grade_supported: true
produced_by: [component_fabricator_mk1]
used_in: [precision_water_processor, ultra_pure_water]
description: Sealed filter cartridge for precision purification machines.
```

```
id: era1_fluid_chemical_additive
name: Chemical Additive
era: 1
family: chemistry
category: fluid
state: liquid
stack_size: 0
storage: fluid_tank
purity_supported: true
hazard_level: 2
produced_by: [chemical_reactor_mk1]
used_in: [coolant, polymers, coatings]
description: Specialty additive for coolants, polymers, and surface treatments.
```

```
id: era1_fluid_electrolyte
name: Electrolyte
era: 1
family: chemistry
category: fluid
state: liquid
storage: fluid_tank
purity_supported: true
hazard_level: 3
produced_by: [chemical_reactor_mk1]
used_in: [energy_cell, battery_pack]
description: Conductive battery electrolyte fluid.
```

```
id: era1_gas_atmospheric_mix
name: Atmospheric Mix
era: 1
family: atmosphere
category: gas
state: gas
storage: gas_tank
purity_supported: false
hazard_level: 0
produced_by: [atmospheric_intake_mk1]
used_in: [atmospheric_separator, atmospheric_condenser]
description: Compressed planetary atmosphere intake for separation and water synthesis.
```

### Metallurgy recovery / advanced

```
id: era1_material_ferrite_dust
name: Ferrite Dust
era: 1
family: ferrite
category: recovered_material
stack_size: 200
purity_supported: true
grade_supported: false
produced_by: [recovery_plant_mk1]
used_in: [ferrite_powder, metallurgy]
description: Recovered ferrite particulate from metallic tailings.
```

```
id: era1_material_conductive_trace
name: Conductive Trace
era: 1
family: conductive
category: recovered_material
stack_size: 200
purity_supported: true
grade_supported: false
produced_by: [recovery_plant_mk1]
used_in: [electrolyte, electronics_recycling]
description: Trace conductive metals recovered from refining waste.
```

```
id: era1_material_reduced_ferrite
name: Reduced Ferrite
era: 1
family: ferrite
category: refined_material
stack_size: 200
purity_supported: true
grade_supported: true
produced_by: [reduction_furnace_mk1]
used_in: [ferrite_powder, steel_paths]
description: Chemically reduced ferrite ready for high-efficiency forming.
```

```
id: era1_material_ferrite_concentrate
name: Ferrite Concentrate
era: 1
family: ferrite
category: refined_material
stack_size: 200
purity_supported: true
grade_supported: true
produced_by: [advanced_purifier_mk1]
used_in: [smelting, powder]
description: High-efficiency purified ferrite feedstock.
```

```
id: era1_material_precision_alloy
name: Precision Alloy
era: 1
family: metallurgy
category: refined_material
stack_size: 100
purity_supported: true
grade_supported: true
produced_by: [precision_alloy_furnace_mk1]
used_in: [motors, precision_machines, weapons]
description: High-grade alloy for precision mechanical and military parts.
```

```
id: era1_material_metal_powder
name: Metal Powder
era: 1
family: metallurgy
category: refined_material
stack_size: 200
purity_supported: true
grade_supported: false
produced_by: [industrial_grinder_mk1]
used_in: [compaction, ammunition]
description: Ground steel composite powder for pressing and munitions.
```

```
id: era1_material_reinforced_metal_block
name: Reinforced Metal Block
era: 1
family: metallurgy
category: refined_material
stack_size: 50
purity_supported: true
grade_supported: true
produced_by: [hydraulic_press_mk1]
used_in: [heavy_frames, nexus]
description: Polymer-compacted metal block for heavy structures.
```

```
id: era1_material_structural_panel
name: Structural Panel
era: 1
family: metallurgy
category: component
stack_size: 100
purity_supported: false
grade_supported: true
produced_by: [fabricator_mk1]
used_in: [chemical_tank, nexus_interface]
description: Reinforced structural panel for tanks and large machines.
```

```
id: era1_component_heavy_structural_frame
name: Heavy Structural Frame
era: 1
family: mechanical
category: component
stack_size: 50
purity_supported: false
grade_supported: true
produced_by: [heavy_assembler_mk1]
used_in: [crushers, smelters, nexus]
description: Oversized load-bearing frame for heavy industry.
```

```
id: era1_material_heat_resistant_ceramic
name: Heat Resistant Ceramic
era: 1
family: ceramics
category: refined_material
stack_size: 100
purity_supported: true
grade_supported: true
produced_by: [ceramic_furnace_mk2]
used_in: [smelters, furnaces]
description: High-temperature ceramic for furnace linings.
```

```
id: era1_material_ceramic_composite
name: Ceramic Composite
era: 1
family: ceramics
category: refined_material
stack_size: 100
purity_supported: true
grade_supported: true
produced_by: [composite_processor_mk1]
used_in: [armor, electrochemical_separator]
description: Carbon-fiber reinforced ceramic composite.
```

```
id: era1_material_industrial_coating
name: Industrial Coating
era: 1
family: chemistry
category: material
stack_size: 100
purity_supported: false
grade_supported: true
produced_by: [chemical_coating_unit_mk1]
used_in: [protected_plates, housings]
description: Protective surface coating precursor.
```

```
id: era1_material_protected_ferrite_plate
name: Protected Ferrite Plate
era: 1
family: ferrite
category: refined_material
stack_size: 200
purity_supported: true
grade_supported: true
produced_by: [coating_unit_mk1]
used_in: [outdoor_structures, chemical_equipment]
description: Corrosion-protected ferrite plate.
```

```
id: era1_component_reinforced_pipe
name: Reinforced Pipe
era: 1
family: mechanical
category: component
stack_size: 100
purity_supported: false
grade_supported: true
produced_by: [component_assembler_mk1]
used_in: [valves, tanks, nexus]
description: Hardened pipe for high-pressure fluid systems.
```

```
id: era1_component_industrial_valve
name: Industrial Valve
era: 1
family: mechanical
category: component
stack_size: 50
purity_supported: false
grade_supported: true
produced_by: [component_assembler_mk1]
used_in: [reactors, tanks, logistics]
description: Actuated industrial valve assembly.
```

```
id: era1_component_pressure_chamber
name: Pressure Chamber
era: 1
family: mechanical
category: component
stack_size: 20
purity_supported: false
grade_supported: true
produced_by: [machine_fabricator_mk1]
used_in: [purifiers, reactors, distillation]
description: Sealed pressure vessel for chemical and purification machines.
```

```
id: era1_component_chemical_storage_tank
name: Chemical Storage Tank
era: 1
family: logistics
category: building_part
stack_size: 10
purity_supported: false
grade_supported: true
produced_by: [machine_fabricator_mk1]
used_in: [fluid_storage, chem_plants]
description: Assembled chemical storage tank unit.
```

### Polymers / fibers

```
id: era1_material_advanced_polymer
name: Advanced Polymer
era: 1
family: polymer
category: material
stack_size: 100
purity_supported: true
grade_supported: true
produced_by: [polymer_reactor_mk2]
used_in: [missiles, insulation, housings]
description: Reinforced polymer for advanced manufacturing.
```

```
id: era1_material_synthetic_fiber
name: Synthetic Fiber
era: 1
family: polymer
category: material
stack_size: 200
purity_supported: false
grade_supported: true
produced_by: [fiber_processor_mk1]
used_in: [chemical_filter, composites]
description: Spun polymer-carbon fiber for filters and composites.
```

### Electronics / machine parts

```
id: era1_component_data_storage_module
name: Data Storage Module
era: 1
family: electronics
category: component
stack_size: 50
purity_supported: false
grade_supported: true
produced_by: [electronics_printer_mk2]
used_in: [science_data, research]
description: Solid-state module for packing research data.
```

```
id: era1_component_electronics_printer_parts
name: Electronics Printer Parts
era: 1
family: electronics
category: machine_part
stack_size: 20
purity_supported: false
grade_supported: true
produced_by: [precision_component_fabricator_mk1]
used_in: [circuit_printer_mk1]
description: Precision parts kit for electronics printers.
```

```
id: era1_component_assembler_parts
name: Assembler Parts
era: 1
family: mechanical
category: machine_part
stack_size: 20
purity_supported: false
grade_supported: true
produced_by: [component_fabricator_mk1]
used_in: [assembler_mk1, component_fabricator]
description: Modular parts kit for assembler construction.
```

```
id: era1_component_chemical_storage_parts
name: Chemical Storage Parts
era: 1
family: logistics
category: machine_part
stack_size: 20
purity_supported: false
grade_supported: true
produced_by: [component_assembler_mk1]
used_in: [fluid_storage_tank]
description: Prefab parts for chemical fluid tanks.
```

```
id: era1_component_heat_exchanger_plate
name: Heat Exchanger Plate
era: 1
family: mechanical
category: component
stack_size: 50
purity_supported: false
grade_supported: true
produced_by: [precision_fabricator_mk1]
used_in: [cooling_assembly]
description: High-surface plate for thermal exchange assemblies.
```

### Automation / robotics (from recipe expansions)

```
id: era1_component_precision_mechanical_assembly
name: Precision Mechanical Assembly
era: 1
family: mechanical
category: component
stack_size: 20
purity_supported: false
grade_supported: true
produced_by: [component_fabricator_mk1]
used_in: [industrial_motor, science_engineering]
description: Calibrated gear-bearing-servo assembly.
```

```
id: era1_component_precision_housing
name: Precision Housing
era: 1
family: mechanical
category: component
stack_size: 20
purity_supported: false
grade_supported: true
produced_by: [component_fabricator_mk1]
used_in: [electronics_printer, robotics]
description: Tight-tolerance machine housing.
```

```
id: era1_component_hydraulic_system
name: Hydraulic System
era: 1
family: mechanical
category: component
stack_size: 20
purity_supported: false
grade_supported: true
produced_by: [component_fabricator_mk1]
used_in: [actuator, heavy_machines]
description: Pressurized hydraulic subsystem.
```

```
id: era1_component_pneumatic_system
name: Pneumatic System
era: 1
family: mechanical
category: component
stack_size: 20
purity_supported: false
grade_supported: true
produced_by: [component_fabricator_mk1]
used_in: [automation, presses]
description: Compressed-air actuation subsystem.
```

```
id: era1_component_cooling_assembly
name: Cooling Assembly
era: 1
family: power
category: component
stack_size: 20
purity_supported: false
grade_supported: true
produced_by: [component_fabricator_mk1]
used_in: [condensers, high_power_machines]
description: Integrated coolant loop assembly.
```

```
id: era1_component_industrial_motor
name: Industrial Motor
era: 1
family: mechanical
category: component
stack_size: 20
purity_supported: false
grade_supported: true
produced_by: [motor_assembly_mk1]
used_in: [crushers, conveyors, inserters]
description: Standard industrial drive motor.
```

```
id: era1_component_heavy_motor
name: Heavy Motor
era: 1
family: mechanical
category: component
stack_size: 10
purity_supported: false
grade_supported: true
produced_by: [motor_assembly_mk1]
used_in: [heavy_machines, nexus]
description: High-torque motor for heavy industry.
```

```
id: era1_component_robotic_joint
name: Robotic Joint
era: 1
family: robotics
category: component
stack_size: 20
purity_supported: false
grade_supported: true
produced_by: [robotics_component_printer_mk1]
used_in: [drone_chassis, repair_drones]
description: Sensor-guided actuator joint.
```

```
id: era1_component_robotic_frame
name: Robotic Frame
era: 1
family: robotics
category: component
stack_size: 10
purity_supported: false
grade_supported: true
produced_by: [robotics_component_printer_mk1]
used_in: [drone_chassis]
description: Lightweight composite robot chassis frame.
```

```
id: era1_component_drone_chassis
name: Drone Chassis
era: 1
family: robotics
category: component
stack_size: 10
purity_supported: false
grade_supported: true
produced_by: [robotics_factory_mk1]
used_in: [industrial_robot_core, repair_drones]
description: Assembled drone body ready for controller integration.
```

```
id: era1_component_autonomous_controller
name: Autonomous Controller
era: 1
family: electronics
category: component
stack_size: 20
purity_supported: false
grade_supported: true
produced_by: [electronics_printer_mk2]
used_in: [robot_core, logistic_controller]
description: Onboard autonomy processor for drones and logistics.
```

```
id: era1_component_industrial_robot_core
name: Industrial Robot Core
era: 1
family: robotics
category: component
stack_size: 5
purity_supported: false
grade_supported: true
produced_by: [robotics_factory_mk1]
used_in: [repair_stations, advanced_automation]
description: Complete industrial robot control core.
```

### Military expansions

```
id: era1_fluid_ballistic_propellant
name: Ballistic Propellant
era: 1
family: military
category: fluid
state: liquid
storage: fluid_tank
hazard_level: 4
purity_supported: false
produced_by: [chemical_reactor_mk2]
used_in: [standard_ammunition, guided_missile]
description: Stabilized propellant for Era 1 ballistic munitions.
```

```
id: era1_military_standard_ammunition
name: Standard Ammunition
era: 1
family: military
category: ammunition
stack_size: 200
grade_supported: true
produced_by: [ammunition_factory_mk1]
used_in: [ballistic_turret]
description: Standard ballistic rounds for Era 1 turrets.
```

```
id: era1_military_reinforced_armor_plate
name: Reinforced Armor Plate
era: 1
family: military
category: material
stack_size: 100
grade_supported: true
produced_by: [armor_processor_mk1]
used_in: [armor_composite, walls]
description: Ceramic-backed armor plate.
```

```
id: era1_military_armor_composite
name: Armor Composite
era: 1
family: military
category: material
stack_size: 50
grade_supported: true
produced_by: [composite_processor_mk2]
used_in: [heavy_walls, vehicles]
description: Layered armor composite for heavy defense.
```

```
id: era1_military_military_control_unit
name: Military Control Unit
era: 1
family: military
category: component
stack_size: 20
grade_supported: true
produced_by: [electronics_printer_mk2]
used_in: [heavy_turrets, defense_network, science_defense]
description: Hardened fire-control computer.
```

```
id: era1_military_defensive_wall_segment
name: Defensive Wall Segment
era: 1
family: military
category: structure
stack_size: 50
produced_by: [assembler_mk2]
used_in: [reinforced_wall]
description: Placeable Era 1 defensive wall segment.
```

```
id: era1_military_reinforced_wall
name: Reinforced Wall
era: 1
family: military
category: structure
stack_size: 50
produced_by: [military_fabricator_mk1]
used_in: [defense_perimeter]
description: Hardened wall segment.
```

```
id: era1_military_ballistic_turret_frame
name: Ballistic Turret Frame
era: 1
family: military
category: structure_part
stack_size: 10
produced_by: [military_fabricator_mk1]
used_in: [ballistic_turret]
description: Structural frame for ballistic turrets.
```

```
id: era1_military_ballistic_turret
name: Ballistic Turret
era: 1
family: military
category: defense
stack_size: 5
produced_by: [military_assembly_bay_mk1]
used_in: [defense]
description: Automated ballistic defense turret.
```

```
id: era1_military_heavy_ballistic_turret
name: Heavy Ballistic Turret
era: 1
family: military
category: defense
stack_size: 5
produced_by: [military_assembly_bay_mk1]
used_in: [defense]
description: Upgraded high-caliber ballistic turret.
```

```
id: era1_military_missile_launcher_frame
name: Missile Launcher Frame
era: 1
family: military
category: structure_part
stack_size: 5
produced_by: [military_fabricator_mk1]
used_in: [missile_turret]
description: Frame for guided missile launchers.
```

```
id: era1_military_guided_missile
name: Guided Missile
era: 1
family: military
category: ammunition
stack_size: 20
produced_by: [missile_factory_mk1]
used_in: [missile_turret]
description: Era 1 guided missile munition.
```

```
id: era1_military_missile_turret
name: Missile Turret
era: 1
family: military
category: defense
stack_size: 5
produced_by: [military_assembly_bay_mk1]
used_in: [defense]
description: Guided missile defense platform.
```

```
id: era1_military_radar_tower
name: Radar Tower
era: 1
family: military
category: defense
stack_size: 5
produced_by: [defense_assembly_machine]
used_in: [defense_network]
description: Sensor tower revealing threats.
```

```
id: era1_military_defense_control_network
name: Defense Control Network
era: 1
family: military
category: defense
stack_size: 5
produced_by: [electronics_printer_mk3]
used_in: [coordinated_defense]
description: Network node linking turrets and radar.
```

### Nexus parts

```
id: era1_nexus_foundation_frame
name: Nexus Foundation Frame
era: 1
family: nexus
category: nexus_part
stack_size: 1
produced_by: [heavy_fabricator]
used_in: [planetary_fabrication_nexus]
description: Massive foundation for the Planetary Fabrication Nexus.
```

```
id: era1_nexus_power_core
name: Nexus Power Core
era: 1
family: nexus
category: nexus_part
stack_size: 1
produced_by: [energy_facility]
used_in: [planetary_fabrication_nexus]
description: Central energy core for the Nexus.
```

```
id: era1_nexus_computational_core
name: Nexus Computational Core
era: 1
family: nexus
category: nexus_part
stack_size: 1
produced_by: [advanced_electronics_printer]
used_in: [planetary_fabrication_nexus]
description: Planetary-scale compute core.
```

```
id: era1_nexus_manufacturing_module
name: Nexus Manufacturing Module
era: 1
family: nexus
category: nexus_part
stack_size: 1
produced_by: [heavy_fabricator]
used_in: [planetary_fabrication_nexus]
description: High-throughput manufacturing module.
```

```
id: era1_nexus_resource_interface
name: Nexus Resource Interface
era: 1
family: nexus
category: nexus_part
stack_size: 1
produced_by: [heavy_fabricator]
used_in: [planetary_fabrication_nexus]
description: Resource intake/output interface for the Nexus.
```

```
id: era1_nexus_planetary_fabrication_nexus
name: Planetary Fabrication Nexus
era: 1
family: nexus
category: mega_structure
stack_size: 1
produced_by: [construction_site]
used_in: [era2_unlock]
description: Era 1 completion structure. Unlocks Era 2 industrial expansion.
```

### Logistics / automation products

```
id: era1_logistics_underground_conveyor
name: Underground Conveyor
era: 1
family: logistics
category: logistics
stack_size: 50
produced_by: [assembler_mk1]
used_in: [factory_logistics]
description: Subsurface conveyor segment pair.
```

```
id: era1_logistics_splitter
name: Conveyor Splitter
era: 1
family: logistics
category: logistics
stack_size: 50
produced_by: [assembler_mk1]
used_in: [factory_logistics]
description: Balanced belt splitter.
```

```
id: era1_logistics_merger
name: Conveyor Merger
era: 1
family: logistics
category: logistics
stack_size: 50
produced_by: [assembler_mk1]
used_in: [factory_logistics]
description: Belt merger junction.
```

```
id: era1_logistics_filter_inserter
name: Filter Inserter
era: 1
family: logistics
category: logistics
stack_size: 50
produced_by: [assembler_mk1]
used_in: [factory_logistics]
description: Item-filtering inserter.
```

```
id: era1_logistics_fast_inserter
name: Fast Inserter
era: 1
family: logistics
category: logistics
stack_size: 50
produced_by: [assembler_mk1]
used_in: [factory_logistics]
description: High-speed inserter.
```

```
id: era1_logistics_smart_inserter
name: Smart Inserter
era: 1
family: logistics
category: logistics
stack_size: 50
produced_by: [assembler_mk2]
used_in: [factory_logistics]
description: Sensor-guided inserter.
```

```
id: era1_logistics_high_speed_inserter
name: High-Speed Inserter
era: 1
family: logistics
category: logistics
stack_size: 50
produced_by: [assembler_mk2]
used_in: [factory_logistics]
description: Top-tier Era 1 inserter.
```

```
id: era1_logistics_fast_conveyor_segment
name: Fast Conveyor Segment
era: 1
family: logistics
category: logistics
stack_size: 200
produced_by: [assembler_mk2]
used_in: [factory_logistics]
description: Mk2 belt segment.
```

```
id: era1_logistics_smart_storage_unit
name: Smart Storage Unit
era: 1
family: logistics
category: logistics
stack_size: 20
produced_by: [assembler_mk2]
used_in: [factory_logistics]
description: Controllable storage with logistics interface.
```

```
id: era1_logistics_logistic_controller
name: Logistic Controller
era: 1
family: logistics
category: component
stack_size: 20
produced_by: [electronics_printer_mk3]
used_in: [transport_node, smart_factory]
description: Network controller for automated logistics.
```

```
id: era1_logistics_automated_transport_node
name: Automated Transport Node
era: 1
family: logistics
category: logistics
stack_size: 10
produced_by: [assembler_mk3]
used_in: [factory_logistics]
description: Autonomous transport routing node.
```

```
id: era1_component_optimization_module
name: Factory Optimization Module
era: 1
family: systems
category: module
stack_size: 20
produced_by: [research_fabricator]
used_in: [machine_upgrades]
description: Module improving machine efficiency when installed.
```

```
id: era1_component_production_monitoring_system
name: Production Monitoring System
era: 1
family: systems
category: module
stack_size: 10
produced_by: [electronics_assembler_mk1]
used_in: [smart_factory]
description: Sensor suite for throughput monitoring.
```

```
id: era1_component_quality_control_unit
name: Quality Control Unit
era: 1
family: systems
category: module
stack_size: 10
produced_by: [electronics_assembler_mk1]
used_in: [grade_lines, purity_lines]
description: Inline quality/purity inspection unit.
```

```
id: era1_building_research_laboratory
name: Research Laboratory
era: 1
family: science
category: building
stack_size: 5
produced_by: [assembler_mk2]
used_in: [science_data]
description: Laboratory that converts industrial samples into research data.
```

---

## Counts

| Patch additions | ~95 item IDs |
| Removals/consolidations | 6 rules |
| Ready for machine DB | Yes |
