# ERA 1 — Recipe Gap Fill A: R108–200
## Fully specified placeholder replacements


  ### era1_recipe_heavy_machine_casing — Heavy Machine Casing
  ```
  recipe_id: era1_recipe_heavy_machine_casing
  name: Heavy Machine Casing
  category: metallurgy
  machine: era1_machine_heavy_assembler_mk1
  inputs:
    - { id: era1_material_hardened_steel, amount: 8 }
- { id: era1_component_machine_housing, amount: 2 }
  outputs:
    - { id: era1_component_heavy_machine_casing, amount: 1 }
  waste_outputs:
    - { id: era1_waste_metallic_tailings, amount: 1 }
  processing_time: 25
  power_consumption: { electrical: 300 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_advanced_metallurgy
  description: Heavy casing for large industrial machines.
  ```

  ### era1_recipe_heat_exchanger_plates_bulk — Heat Exchanger Plate Batch
  ```
  recipe_id: era1_recipe_heat_exchanger_plates_bulk
  name: Heat Exchanger Plate Batch
  category: metallurgy
  machine: era1_machine_precision_fabricator_mk1
  inputs:
    - { id: era1_material_hardened_steel, amount: 10 }
- { id: era1_component_reinforced_pipe, amount: 6 }
- { id: era1_material_industrial_coating, amount: 4 }
  outputs:
    - { id: era1_component_heat_exchanger_plate, amount: 6 }
  waste_outputs:
    []
  processing_time: 30
  power_consumption: { electrical: 220 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_advanced_metallurgy
  description: Batch production of heat exchanger plates.
  ```

  ### era1_recipe_reactor_lining — Reactor Lining
  ```
  recipe_id: era1_recipe_reactor_lining
  name: Reactor Lining
  category: metallurgy
  machine: era1_machine_ceramic_furnace_mk2
  inputs:
    - { id: era1_material_heat_resistant_ceramic, amount: 10 }
- { id: era1_material_advanced_ceramic, amount: 5 }
  outputs:
    - { id: era1_material_reactor_lining, amount: 5 }
  waste_outputs:
    - { id: era1_waste_stone_dust, amount: 1 }
  processing_time: 35
  power_consumption: { thermal: 700 }
  purity_effect: 3
  grade_effect: industrial
  technology_unlock: era1_tech_advanced_ceramics
  description: Forms chemical-resistant reactor lining tiles.
  ```

  ### era1_recipe_industrial_frame — Industrial Frame
  ```
  recipe_id: era1_recipe_industrial_frame
  name: Industrial Frame
  category: metallurgy
  machine: era1_machine_heavy_assembler_mk1
  inputs:
    - { id: era1_component_heavy_structural_frame, amount: 2 }
- { id: era1_material_steel_composite, amount: 10 }
  outputs:
    - { id: era1_component_industrial_frame, amount: 1 }
  waste_outputs:
    []
  processing_time: 28
  power_consumption: { electrical: 280 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_advanced_metallurgy
  description: Assembles industrial-grade frames for mid-tier machines.
  ```

  ### era1_recipe_precision_housing_alt — Precision Housing (Alloy)
  ```
  recipe_id: era1_recipe_precision_housing_alt
  name: Precision Housing (Alloy)
  category: metallurgy
  machine: era1_machine_precision_fabricator_mk1
  inputs:
    - { id: era1_material_precision_alloy, amount: 5 }
- { id: era1_component_machine_housing, amount: 2 }
- { id: era1_material_industrial_coating, amount: 2 }
  outputs:
    - { id: era1_component_precision_housing, amount: 2 }
  waste_outputs:
    []
  processing_time: 22
  power_consumption: { electrical: 200 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_precision_manufacturing
  description: Precision housing from alloy stock.
  ```

  ### era1_recipe_turbine_blade — Turbine Blade
  ```
  recipe_id: era1_recipe_turbine_blade
  name: Turbine Blade
  category: metallurgy
  machine: era1_machine_precision_alloy_furnace_mk1
  inputs:
    - { id: era1_material_heat_treated_alloy, amount: 5 }
- { id: era1_material_carbon_composite, amount: 2 }
  outputs:
    - { id: era1_component_turbine_blade, amount: 2 }
  waste_outputs:
    []
  processing_time: 40
  power_consumption: { thermal: 500 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_advanced_metallurgy
  description: Forges turbine blades for generators and compressors.
  ```

  ### era1_recipe_magnetic_component — Magnetic Component
  ```
  recipe_id: era1_recipe_magnetic_component
  name: Magnetic Component
  category: metallurgy
  machine: era1_machine_component_fabricator_mk1
  inputs:
    - { id: era1_material_ferrite_powder, amount: 10 }
- { id: era1_material_conductive_wire, amount: 10 }
- { id: era1_material_graphite, amount: 2 }
  outputs:
    - { id: era1_component_magnetic_component, amount: 4 }
  waste_outputs:
    []
  processing_time: 18
  power_consumption: { electrical: 160 }
  purity_effect: 2
  grade_effect: industrial
  technology_unlock: era1_tech_electronics
  description: Forms magnetic cores for motors and sensors.
  ```

  ### era1_recipe_high_pressure_fitting — High Pressure Fitting
  ```
  recipe_id: era1_recipe_high_pressure_fitting
  name: High Pressure Fitting
  category: metallurgy
  machine: era1_machine_component_assembler_mk1
  inputs:
    - { id: era1_component_reinforced_pipe, amount: 4 }
- { id: era1_material_hardened_steel, amount: 4 }
  outputs:
    - { id: era1_component_high_pressure_fitting, amount: 4 }
  waste_outputs:
    []
  processing_time: 14
  power_consumption: { electrical: 140 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_fluid_engineering
  description: Produces high-pressure pipe fittings.
  ```

  ### era1_recipe_thermal_coolant — Thermal Coolant
  ```
  recipe_id: era1_recipe_thermal_coolant
  name: Thermal Coolant
  category: chemistry
  machine: era1_machine_fluid_processor_mk1
  inputs:
    - { id: era1_fluid_coolant, amount: 10 }
- { id: era1_fluid_chemical_additive, amount: 5 }
  outputs:
    - { id: era1_fluid_thermal_coolant, amount: 12 }
  waste_outputs:
    []
  processing_time: 12
  power_consumption: { electrical: 180 }
  purity_effect: 0
  grade_effect: none
  technology_unlock: era1_tech_chemical_manufacturing
  description: Upgrades coolant for high-heat machines.
  ```

  ### era1_recipe_battery_electrolyte_alt — Battery Electrolyte (Feedstock Path)
  ```
  recipe_id: era1_recipe_battery_electrolyte_alt
  name: Battery Electrolyte (Feedstock Path)
  category: chemistry
  machine: era1_machine_chemical_reactor_mk1
  inputs:
    - { id: era1_fluid_acid_solution, amount: 8 }
- { id: era1_fluid_chemical_feedstock, amount: 4 }
- { id: era1_material_conductive_trace, amount: 3 }
  outputs:
    - { id: era1_fluid_electrolyte, amount: 10 }
  waste_outputs:
    - { id: era1_waste_chemical_residue, amount: 1 }
  processing_time: 14
  power_consumption: { electrical: 210 }
  purity_effect: 2
  grade_effect: none
  technology_unlock: era1_tech_power_systems
  description: Alternate electrolyte synthesis path.
  ```

  ### era1_recipe_electronic_cleaning_fluid — Electronic Cleaning Fluid
  ```
  recipe_id: era1_recipe_electronic_cleaning_fluid
  name: Electronic Cleaning Fluid
  category: chemistry
  machine: era1_machine_chemical_reactor_mk1
  inputs:
    - { id: era1_fluid_industrial_solvent, amount: 10 }
- { id: era1_fluid_ultra_pure_water, amount: 5 }
  outputs:
    - { id: era1_fluid_electronic_cleaning_fluid, amount: 12 }
  waste_outputs:
    []
  processing_time: 10
  power_consumption: { electrical: 160 }
  purity_effect: 5
  grade_effect: none
  technology_unlock: era1_tech_electronics
  description: Produces cleaning fluid for circuit fabrication.
  ```

  ### era1_recipe_polymer_sheet — Polymer Sheet
  ```
  recipe_id: era1_recipe_polymer_sheet
  name: Polymer Sheet
  category: chemistry
  machine: era1_machine_polymer_reactor_mk1
  inputs:
    - { id: era1_fluid_polymer_mix, amount: 15 }
- { id: era1_material_polymer_resin, amount: 5 }
  outputs:
    - { id: era1_material_polymer_sheet, amount: 10 }
  waste_outputs:
    - { id: era1_waste_polymer_scrap, amount: 1 }
  processing_time: 16
  power_consumption: { electrical: 200 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_polymer_science
  description: Forms polymer sheets for housings and insulation.
  ```

  ### era1_recipe_insulation_material — Insulation Material
  ```
  recipe_id: era1_recipe_insulation_material
  name: Insulation Material
  category: chemistry
  machine: era1_machine_composite_processor_mk1
  inputs:
    - { id: era1_material_polymer_sheet, amount: 5 }
- { id: era1_material_ceramic, amount: 5 }
  outputs:
    - { id: era1_material_insulation, amount: 8 }
  waste_outputs:
    []
  processing_time: 18
  power_consumption: { electrical: 180 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_polymer_science
  description: Creates electrical/thermal insulation material.
  ```

  ### era1_recipe_gas_purification_cartridge — Gas Purification Cartridge
  ```
  recipe_id: era1_recipe_gas_purification_cartridge
  name: Gas Purification Cartridge
  category: chemistry
  machine: era1_machine_component_fabricator_mk1
  inputs:
    - { id: era1_component_chemical_filter, amount: 3 }
- { id: era1_component_filter_cartridge, amount: 1 }
- { id: era1_material_activated_carbon, amount: 2 }
  outputs:
    - { id: era1_component_gas_purification_cartridge, amount: 2 }
  waste_outputs:
    []
  processing_time: 12
  power_consumption: { electrical: 100 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_fluid_engineering
  description: Cartridge for gas scrubbing systems.
  ```

  ### era1_recipe_industrial_solvent_bulk — Industrial Solvent
  ```
  recipe_id: era1_recipe_industrial_solvent_bulk
  name: Industrial Solvent
  category: chemistry
  machine: era1_machine_chemical_reactor_mk1
  inputs:
    - { id: era1_fluid_medium_hydrocarbon, amount: 15 }
- { id: era1_fluid_catalyst_solution, amount: 3 }
  outputs:
    - { id: era1_fluid_industrial_solvent, amount: 15 }
  waste_outputs:
    - { id: era1_waste_chemical_residue, amount: 1 }
  processing_time: 14
  power_consumption: { electrical: 220 }
  purity_effect: 0
  grade_effect: none
  technology_unlock: era1_tech_chemical_manufacturing
  description: Distills industrial solvent from medium hydrocarbons.
  ```

  ### era1_recipe_waste_neutralizer — Waste Neutralizer
  ```
  recipe_id: era1_recipe_waste_neutralizer
  name: Waste Neutralizer
  category: chemistry
  machine: era1_machine_chemical_reactor_mk1
  inputs:
    - { id: era1_fluid_alkaline_solution, amount: 10 }
- { id: era1_fluid_acid_solution, amount: 5 }
- { id: era1_fluid_chemical_additive, amount: 2 }
  outputs:
    - { id: era1_fluid_waste_neutralizer, amount: 12 }
  waste_outputs:
    []
  processing_time: 15
  power_consumption: { electrical: 200 }
  purity_effect: 0
  grade_effect: none
  technology_unlock: era1_tech_waste_recovery
  description: Produces compound for neutralizing chemical waste.
  ```

  ### era1_recipe_activated_carbon — Activated Carbon
  ```
  recipe_id: era1_recipe_activated_carbon
  name: Activated Carbon
  category: chemistry
  machine: era1_machine_carbon_furnace_mk1
  inputs:
    - { id: era1_material_carbon_powder, amount: 15 }
- { id: era1_fluid_steam, amount: 5 }
  outputs:
    - { id: era1_material_activated_carbon, amount: 10 }
  waste_outputs:
    - { id: era1_waste_carbon_residue, amount: 2 }
  processing_time: 20
  power_consumption: { thermal: 400 }
  purity_effect: 4
  grade_effect: industrial
  technology_unlock: era1_tech_carbon_processing
  description: Activates carbon for filtration media.
  ```

### era1_recipe_steam — Process Steam
```
recipe_id: era1_recipe_steam
name: Process Steam
category: chemistry
machine: era1_machine_boiler_mk1
inputs:
  - { id: era1_fluid_purified_water, amount: 20 }
outputs:
  - { id: era1_fluid_steam, amount: 20 }
waste_outputs:
  []
processing_time: 8
power_consumption: { thermal: 300 }
purity_effect: 0
grade_effect: none
technology_unlock: era1_tech_fluid_engineering
description: Boils purified water into process steam.
```

  ### era1_recipe_oxidizer — Industrial Oxidizer
  ```
  recipe_id: era1_recipe_oxidizer
  name: Industrial Oxidizer
  category: chemistry
  machine: era1_machine_chemical_reactor_mk1
  inputs:
    - { id: era1_gas_oxygen, amount: 20 }
- { id: era1_fluid_catalyst_solution, amount: 2 }
  outputs:
    - { id: era1_fluid_oxidizer, amount: 10 }
  waste_outputs:
    []
  processing_time: 12
  power_consumption: { electrical: 180 }
  purity_effect: 0
  grade_effect: none
  technology_unlock: era1_tech_chemical_manufacturing
  description: Stabilizes oxygen into usable oxidizer fluid.
  ```

  ### era1_recipe_reducing_agent — Reducing Agent
  ```
  recipe_id: era1_recipe_reducing_agent
  name: Reducing Agent
  category: chemistry
  machine: era1_machine_chemical_processor_mk1
  inputs:
    - { id: era1_material_carbon_powder, amount: 10 }
- { id: era1_gas_hydrogen, amount: 10 }
  outputs:
    - { id: era1_material_reducing_agent, amount: 10 }
  waste_outputs:
    []
  processing_time: 14
  power_consumption: { electrical: 200 }
  purity_effect: 2
  grade_effect: industrial
  technology_unlock: era1_tech_basic_metallurgy
  description: Produces reducing agent for metal reduction furnaces.
  ```

  ### era1_recipe_polymer_foam — Polymer Foam
  ```
  recipe_id: era1_recipe_polymer_foam
  name: Polymer Foam
  category: chemistry
  machine: era1_machine_polymer_reactor_mk1
  inputs:
    - { id: era1_fluid_polymer_mix, amount: 10 }
- { id: era1_gas_nitrogen, amount: 5 }
  outputs:
    - { id: era1_material_polymer_foam, amount: 8 }
  waste_outputs:
    - { id: era1_waste_polymer_scrap, amount: 1 }
  processing_time: 12
  power_consumption: { electrical: 160 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_polymer_science
  description: Foams polymer mix into insulation foam.
  ```

  ### era1_recipe_flexible_polymer — Flexible Polymer
  ```
  recipe_id: era1_recipe_flexible_polymer
  name: Flexible Polymer
  category: chemistry
  machine: era1_machine_polymer_reactor_mk2
  inputs:
    - { id: era1_material_polymer_resin, amount: 10 }
- { id: era1_material_synthetic_rubber, amount: 5 }
  outputs:
    - { id: era1_material_flexible_polymer, amount: 10 }
  waste_outputs:
    []
  processing_time: 16
  power_consumption: { electrical: 220 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_polymer_science
  description: Blends resin and rubber into flexible polymer.
  ```

  ### era1_recipe_rigid_polymer — Rigid Polymer
  ```
  recipe_id: era1_recipe_rigid_polymer
  name: Rigid Polymer
  category: chemistry
  machine: era1_machine_polymer_reactor_mk2
  inputs:
    - { id: era1_material_polymer_resin, amount: 10 }
- { id: era1_material_carbon_fiber, amount: 3 }
  outputs:
    - { id: era1_material_rigid_polymer, amount: 10 }
  waste_outputs:
    []
  processing_time: 16
  power_consumption: { electrical: 240 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_polymer_science
  description: Forms rigid polymer stock for housings.
  ```

  ### era1_recipe_composite_binder — Composite Binder
  ```
  recipe_id: era1_recipe_composite_binder
  name: Composite Binder
  category: chemistry
  machine: era1_machine_chemical_reactor_mk1
  inputs:
    - { id: era1_material_polymer_resin, amount: 8 }
- { id: era1_fluid_catalyst_solution, amount: 3 }
  outputs:
    - { id: era1_material_composite_binder, amount: 8 }
  waste_outputs:
    []
  processing_time: 12
  power_consumption: { electrical: 180 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_polymer_science
  description: Creates binder for composite layups.
  ```

  ### era1_recipe_fuel_oil — Fuel Oil
  ```
  recipe_id: era1_recipe_fuel_oil
  name: Fuel Oil
  category: chemistry
  machine: era1_machine_distillation_tower_mk1
  inputs:
    - { id: era1_fluid_heavy_hydrocarbon, amount: 40 }
  outputs:
    - { id: era1_fluid_fuel_oil, amount: 30 }
- { id: era1_fluid_industrial_solvent, amount: 5 }
  waste_outputs:
    - { id: era1_waste_chemical_residue, amount: 3 }
  processing_time: 25
  power_consumption: { thermal: 350 }
  purity_effect: 0
  grade_effect: none
  technology_unlock: era1_tech_hydrocarbon_refining
  description: Refines heavy fraction into fuel oil.
  ```

  ### era1_recipe_synthetic_fuel — Synthetic Fuel
  ```
  recipe_id: era1_recipe_synthetic_fuel
  name: Synthetic Fuel
  category: chemistry
  machine: era1_machine_chemical_reactor_mk2
  inputs:
    - { id: era1_fluid_light_hydrocarbon, amount: 20 }
- { id: era1_gas_hydrogen, amount: 10 }
  outputs:
    - { id: era1_fluid_synthetic_fuel, amount: 20 }
  waste_outputs:
    []
  processing_time: 20
  power_consumption: { electrical: 300 }
  purity_effect: 0
  grade_effect: none
  technology_unlock: era1_tech_hydrocarbon_refining
  description: Hydrogenates light hydrocarbons into synthetic fuel.
  ```

  ### era1_recipe_heavy_resin — Heavy Resin
  ```
  recipe_id: era1_recipe_heavy_resin
  name: Heavy Resin
  category: chemistry
  machine: era1_machine_polymer_reactor_mk1
  inputs:
    - { id: era1_fluid_heavy_hydrocarbon, amount: 20 }
- { id: era1_fluid_catalyst_solution, amount: 4 }
  outputs:
    - { id: era1_material_heavy_resin, amount: 12 }
  waste_outputs:
    - { id: era1_waste_polymer_scrap, amount: 2 }
  processing_time: 18
  power_consumption: { electrical: 240 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_polymer_science
  description: Produces heavy resin from heavy hydrocarbons.
  ```

  ### era1_recipe_bitumen — Bitumen
  ```
  recipe_id: era1_recipe_bitumen
  name: Bitumen
  category: chemistry
  machine: era1_machine_distillation_tower_mk1
  inputs:
    - { id: era1_fluid_raw_hydrocarbon, amount: 50 }
  outputs:
    - { id: era1_material_bitumen, amount: 15 }
- { id: era1_fluid_heavy_hydrocarbon, amount: 10 }
  waste_outputs:
    - { id: era1_waste_carbon_residue, amount: 5 }
  processing_time: 30
  power_consumption: { thermal: 400 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_hydrocarbon_refining
  description: Recovers bitumen and heavy fraction from residue-rich feedstock.
  ```

  ### era1_recipe_optical_sensor — Optical Sensor
  ```
  recipe_id: era1_recipe_optical_sensor
  name: Optical Sensor
  category: electronics
  machine: era1_machine_electronics_assembler_mk1
  inputs:
    - { id: era1_component_sensor, amount: 2 }
- { id: era1_material_glass, amount: 2 }
- { id: era1_component_basic_circuit, amount: 2 }
  outputs:
    - { id: era1_component_optical_sensor, amount: 2 }
  waste_outputs:
    []
  processing_time: 14
  power_consumption: { electrical: 140 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_electronics
  description: Assembles optical sensors.
  ```

  ### era1_recipe_processor_core — Processor Core
  ```
  recipe_id: era1_recipe_processor_core
  name: Processor Core
  category: electronics
  machine: era1_machine_electronics_printer_mk2
  inputs:
    - { id: era1_component_logic_board, amount: 2 }
- { id: era1_material_silicon_powder, amount: 5 }
- { id: era1_fluid_electronic_cleaning_fluid, amount: 2 }
  outputs:
    - { id: era1_component_processor_core, amount: 1 }
  waste_outputs:
    []
  processing_time: 25
  power_consumption: { electrical: 280 }
  purity_effect: 3
  grade_effect: precision
  technology_unlock: era1_tech_advanced_electronics
  description: Prints a compact processor core.
  ```

  ### era1_recipe_communication_module — Communication Module
  ```
  recipe_id: era1_recipe_communication_module
  name: Communication Module
  category: electronics
  machine: era1_machine_electronics_assembler_mk1
  inputs:
    - { id: era1_component_basic_circuit, amount: 4 }
- { id: era1_component_signal_amplifier, amount: 1 }
- { id: era1_material_conductive_foil, amount: 4 }
  outputs:
    - { id: era1_component_communication_module, amount: 1 }
  waste_outputs:
    []
  processing_time: 16
  power_consumption: { electrical: 160 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_electronics
  description: Builds short-range communication modules.
  ```

  ### era1_recipe_signal_amplifier — Signal Amplifier
  ```
  recipe_id: era1_recipe_signal_amplifier
  name: Signal Amplifier
  category: electronics
  machine: era1_machine_electronics_assembler_mk1
  inputs:
    - { id: era1_component_basic_circuit, amount: 3 }
- { id: era1_component_capacitor, amount: 2 }
- { id: era1_material_conductive_wire, amount: 5 }
  outputs:
    - { id: era1_component_signal_amplifier, amount: 2 }
  waste_outputs:
    []
  processing_time: 12
  power_consumption: { electrical: 130 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_electronics
  description: Amplifies control and sensor signals.
  ```

  ### era1_recipe_machine_controller — Machine Controller
  ```
  recipe_id: era1_recipe_machine_controller
  name: Machine Controller
  category: electronics
  machine: era1_machine_electronics_printer_mk2
  inputs:
    - { id: era1_component_control_module, amount: 2 }
- { id: era1_component_processor_core, amount: 1 }
  outputs:
    - { id: era1_component_machine_controller, amount: 1 }
  waste_outputs:
    []
  processing_time: 20
  power_consumption: { electrical: 200 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_advanced_automation
  description: Controller unit for complex machines.
  ```

  ### era1_recipe_robotics_controller — Robotics Controller
  ```
  recipe_id: era1_recipe_robotics_controller
  name: Robotics Controller
  category: electronics
  machine: era1_machine_electronics_printer_mk2
  inputs:
    - { id: era1_component_autonomous_controller, amount: 1 }
- { id: era1_component_machine_controller, amount: 1 }
  outputs:
    - { id: era1_component_robotics_controller, amount: 1 }
  waste_outputs:
    []
  processing_time: 22
  power_consumption: { electrical: 220 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_robotics
  description: Specialized controller for robotic systems.
  ```

  ### era1_recipe_navigation_module — Navigation Module
  ```
  recipe_id: era1_recipe_navigation_module
  name: Navigation Module
  category: electronics
  machine: era1_machine_electronics_assembler_mk1
  inputs:
    - { id: era1_component_optical_sensor, amount: 2 }
- { id: era1_component_logic_board, amount: 1 }
  outputs:
    - { id: era1_component_navigation_module, amount: 1 }
  waste_outputs:
    []
  processing_time: 18
  power_consumption: { electrical: 180 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_robotics
  description: Navigation package for drones.
  ```

  ### era1_recipe_research_processor — Research Processor
  ```
  recipe_id: era1_recipe_research_processor
  name: Research Processor
  category: electronics
  machine: era1_machine_electronics_printer_mk3
  inputs:
    - { id: era1_component_processor_core, amount: 2 }
- { id: era1_component_data_storage_module, amount: 4 }
  outputs:
    - { id: era1_component_research_processor, amount: 1 }
  waste_outputs:
    []
  processing_time: 30
  power_consumption: { electrical: 300 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_research_infrastructure
  description: High-throughput processor for laboratories.
  ```

  ### era1_recipe_memory_wafer — Memory Wafer
  ```
  recipe_id: era1_recipe_memory_wafer
  name: Memory Wafer
  category: electronics
  machine: era1_machine_electronics_printer_mk2
  inputs:
    - { id: era1_material_refined_silicon, amount: 5 }
- { id: era1_material_conductive_foil, amount: 5 }
- { id: era1_fluid_ultra_pure_water, amount: 2 }
  outputs:
    - { id: era1_component_memory_wafer, amount: 4 }
  waste_outputs:
    - { id: era1_waste_chemical_residue, amount: 1 }
  processing_time: 20
  power_consumption: { electrical: 240 }
  purity_effect: 5
  grade_effect: precision
  technology_unlock: era1_tech_advanced_electronics
  description: Prints memory wafers.
  ```

  ### era1_recipe_logic_wafer — Logic Wafer
  ```
  recipe_id: era1_recipe_logic_wafer
  name: Logic Wafer
  category: electronics
  machine: era1_machine_electronics_printer_mk2
  inputs:
    - { id: era1_material_refined_silicon, amount: 5 }
- { id: era1_material_silicon_powder, amount: 5 }
- { id: era1_fluid_electronic_cleaning_fluid, amount: 2 }
  outputs:
    - { id: era1_component_logic_wafer, amount: 4 }
  waste_outputs:
    []
  processing_time: 22
  power_consumption: { electrical: 260 }
  purity_effect: 5
  grade_effect: precision
  technology_unlock: era1_tech_advanced_electronics
  description: Prints logic wafers for boards.
  ```

  ### era1_recipe_sensor_wafer — Sensor Wafer
  ```
  recipe_id: era1_recipe_sensor_wafer
  name: Sensor Wafer
  category: electronics
  machine: era1_machine_electronics_printer_mk1
  inputs:
    - { id: era1_material_refined_silicon, amount: 4 }
- { id: era1_material_glass, amount: 2 }
  outputs:
    - { id: era1_component_sensor_wafer, amount: 4 }
  waste_outputs:
    []
  processing_time: 15
  power_consumption: { electrical: 180 }
  purity_effect: 2
  grade_effect: industrial
  technology_unlock: era1_tech_electronics
  description: Produces sensor wafers.
  ```

  ### era1_recipe_circuit_trace_bundle — Circuit Trace Bundle
  ```
  recipe_id: era1_recipe_circuit_trace_bundle
  name: Circuit Trace Bundle
  category: electronics
  machine: era1_machine_component_processor_mk1
  inputs:
    - { id: era1_material_conductive_foil, amount: 10 }
- { id: era1_material_conductive_wire, amount: 10 }
  outputs:
    - { id: era1_component_circuit_trace_bundle, amount: 10 }
  waste_outputs:
    []
  processing_time: 10
  power_consumption: { electrical: 100 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_electronics
  description: Bundles conductive traces for circuit printing.
  ```

  ### era1_recipe_power_board — Power Board
  ```
  recipe_id: era1_recipe_power_board
  name: Power Board
  category: electronics
  machine: era1_machine_electronics_assembler_mk1
  inputs:
    - { id: era1_component_power_regulator, amount: 2 }
- { id: era1_component_basic_circuit, amount: 4 }
- { id: era1_component_capacitor, amount: 2 }
  outputs:
    - { id: era1_component_power_board, amount: 2 }
  waste_outputs:
    []
  processing_time: 14
  power_consumption: { electrical: 150 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Assembles power distribution boards.
  ```

  ### era1_recipe_interface_module — Interface Module
  ```
  recipe_id: era1_recipe_interface_module
  name: Interface Module
  category: electronics
  machine: era1_machine_electronics_assembler_mk1
  inputs:
    - { id: era1_component_basic_circuit, amount: 3 }
- { id: era1_component_communication_module, amount: 1 }
  outputs:
    - { id: era1_component_interface_module, amount: 2 }
  waste_outputs:
    []
  processing_time: 12
  power_consumption: { electrical: 130 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_electronics
  description: Human/machine interface modules.
  ```

  ### era1_recipe_calibration_chip — Calibration Chip
  ```
  recipe_id: era1_recipe_calibration_chip
  name: Calibration Chip
  category: electronics
  machine: era1_machine_electronics_printer_mk2
  inputs:
    - { id: era1_component_logic_wafer, amount: 2 }
- { id: era1_component_sensor_wafer, amount: 1 }
  outputs:
    - { id: era1_component_calibration_chip, amount: 4 }
  waste_outputs:
    []
  processing_time: 16
  power_consumption: { electrical: 170 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_precision_manufacturing
  description: Chips used to calibrate precision machines.
  ```

  ### era1_recipe_firmware_module — Firmware Module
  ```
  recipe_id: era1_recipe_firmware_module
  name: Firmware Module
  category: electronics
  machine: era1_machine_electronics_printer_mk2
  inputs:
    - { id: era1_component_memory_wafer, amount: 2 }
- { id: era1_component_basic_circuit, amount: 2 }
  outputs:
    - { id: era1_component_firmware_module, amount: 2 }
  waste_outputs:
    []
  processing_time: 14
  power_consumption: { electrical: 160 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_advanced_electronics
  description: Stores machine firmware images.
  ```

  ### era1_recipe_sensor_array — Sensor Array
  ```
  recipe_id: era1_recipe_sensor_array
  name: Sensor Array
  category: electronics
  machine: era1_machine_electronics_assembler_mk1
  inputs:
    - { id: era1_component_sensor, amount: 4 }
- { id: era1_component_optical_sensor, amount: 2 }
- { id: era1_component_signal_amplifier, amount: 1 }
  outputs:
    - { id: era1_component_sensor_array, amount: 1 }
  waste_outputs:
    []
  processing_time: 18
  power_consumption: { electrical: 180 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_electronics
  description: Multi-sensor array package.
  ```

  ### era1_recipe_control_bus — Control Bus
  ```
  recipe_id: era1_recipe_control_bus
  name: Control Bus
  category: electronics
  machine: era1_machine_electronics_assembler_mk1
  inputs:
    - { id: era1_component_circuit_trace_bundle, amount: 5 }
- { id: era1_component_basic_circuit, amount: 2 }
  outputs:
    - { id: era1_component_control_bus, amount: 2 }
  waste_outputs:
    []
  processing_time: 10
  power_consumption: { electrical: 110 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_electronics
  description: Backplane bus for machine controllers.
  ```

  ### era1_recipe_diagnostic_module — Diagnostic Module
  ```
  recipe_id: era1_recipe_diagnostic_module
  name: Diagnostic Module
  category: electronics
  machine: era1_machine_electronics_assembler_mk1
  inputs:
    - { id: era1_component_sensor_array, amount: 1 }
- { id: era1_component_logic_board, amount: 1 }
  outputs:
    - { id: era1_component_diagnostic_module, amount: 1 }
  waste_outputs:
    []
  processing_time: 16
  power_consumption: { electrical: 150 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_advanced_automation
  description: Diagnostics package for factory machines.
  ```

  ### era1_recipe_relay_board — Relay Board
  ```
  recipe_id: era1_recipe_relay_board
  name: Relay Board
  category: electronics
  machine: era1_machine_power_component_factory_mk1
  inputs:
    - { id: era1_power_relay, amount: 2 }
- { id: era1_component_basic_circuit, amount: 3 }
  outputs:
    - { id: era1_component_relay_board, amount: 2 }
  waste_outputs:
    []
  processing_time: 12
  power_consumption: { electrical: 140 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Board mounting multiple power relays.
  ```

  ### era1_recipe_high_density_circuit — High Density Circuit
  ```
  recipe_id: era1_recipe_high_density_circuit
  name: High Density Circuit
  category: electronics
  machine: era1_machine_electronics_printer_mk3
  inputs:
    - { id: era1_component_logic_wafer, amount: 4 }
- { id: era1_component_circuit_trace_bundle, amount: 4 }
- { id: era1_fluid_ultra_pure_water, amount: 3 }
  outputs:
    - { id: era1_component_high_density_circuit, amount: 2 }
  waste_outputs:
    - { id: era1_waste_chemical_residue, amount: 1 }
  processing_time: 28
  power_consumption: { electrical: 320 }
  purity_effect: 4
  grade_effect: precision
  technology_unlock: era1_tech_advanced_electronics
  description: Dense multilayer circuit for late Era 1.
  ```

  ### era1_recipe_ai_assistant_module — Industrial AI Assistant Module
  ```
  recipe_id: era1_recipe_ai_assistant_module
  name: Industrial AI Assistant Module
  category: electronics
  machine: era1_machine_electronics_printer_mk3
  inputs:
    - { id: era1_component_processor_core, amount: 2 }
- { id: era1_component_research_processor, amount: 1 }
- { id: era1_science_computational_data, amount: 1 }
  outputs:
    - { id: era1_component_industrial_ai_assistant, amount: 1 }
  waste_outputs:
    []
  processing_time: 40
  power_consumption: { electrical: 400 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_systems_science
  description: Limited industrial AI assist module (not true AI core).
  ```

  ### era1_recipe_clock_crystal — Clock Crystal
  ```
  recipe_id: era1_recipe_clock_crystal
  name: Clock Crystal
  category: electronics
  machine: era1_machine_precision_fabricator_mk1
  inputs:
    - { id: era1_material_refined_silicon, amount: 3 }
- { id: era1_material_advanced_ceramic, amount: 1 }
  outputs:
    - { id: era1_component_clock_crystal, amount: 6 }
  waste_outputs:
    []
  processing_time: 12
  power_consumption: { electrical: 120 }
  purity_effect: 2
  grade_effect: precision
  technology_unlock: era1_tech_electronics
  description: Timing crystals for digital systems.
  ```

  ### era1_recipe_power_storage_module — Power Storage Module
  ```
  recipe_id: era1_recipe_power_storage_module
  name: Power Storage Module
  category: power
  machine: era1_machine_battery_assembler_mk1
  inputs:
    - { id: era1_power_battery_pack, amount: 2 }
- { id: era1_component_power_board, amount: 1 }
  outputs:
    - { id: era1_power_storage_module, amount: 1 }
  waste_outputs:
    []
  processing_time: 20
  power_consumption: { electrical: 180 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Modular factory power storage.
  ```

  ### era1_recipe_generator_rotor — Generator Rotor
  ```
  recipe_id: era1_recipe_generator_rotor
  name: Generator Rotor
  category: power
  machine: era1_machine_motor_assembly_mk1
  inputs:
    - { id: era1_component_turbine_blade, amount: 4 }
- { id: era1_component_magnetic_component, amount: 4 }
- { id: era1_component_mechanical_shaft, amount: 2 }
  outputs:
    - { id: era1_component_generator_rotor, amount: 1 }
  waste_outputs:
    []
  processing_time: 30
  power_consumption: { electrical: 250 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Assembles generator rotor.
  ```

  ### era1_recipe_generator_stator — Generator Stator
  ```
  recipe_id: era1_recipe_generator_stator
  name: Generator Stator
  category: power
  machine: era1_machine_power_component_factory_mk1
  inputs:
    - { id: era1_component_magnetic_component, amount: 6 }
- { id: era1_material_conductive_wire, amount: 20 }
- { id: era1_component_machine_housing, amount: 1 }
  outputs:
    - { id: era1_component_generator_stator, amount: 1 }
  waste_outputs:
    []
  processing_time: 28
  power_consumption: { electrical: 240 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Assembles generator stator.
  ```

  ### era1_recipe_compact_generator — Compact Generator
  ```
  recipe_id: era1_recipe_compact_generator
  name: Compact Generator
  category: power
  machine: era1_machine_machine_fabricator_mk1
  inputs:
    - { id: era1_component_generator_rotor, amount: 1 }
- { id: era1_component_generator_stator, amount: 1 }
- { id: era1_component_cooling_assembly, amount: 1 }
  outputs:
    - { id: era1_machine_compact_generator, amount: 1 }
  waste_outputs:
    []
  processing_time: 40
  power_consumption: { electrical: 300 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Builds a placeable compact generator.
  ```

  ### era1_recipe_high_voltage_cable — High Voltage Cable
  ```
  recipe_id: era1_recipe_high_voltage_cable
  name: High Voltage Cable
  category: power
  machine: era1_machine_component_processor_mk1
  inputs:
    - { id: era1_material_conductive_wire, amount: 20 }
- { id: era1_material_insulation, amount: 5 }
  outputs:
    - { id: era1_power_high_voltage_cable, amount: 10 }
  waste_outputs:
    []
  processing_time: 12
  power_consumption: { electrical: 120 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Insulated high-voltage cable segments.
  ```

  ### era1_recipe_grid_controller — Grid Controller
  ```
  recipe_id: era1_recipe_grid_controller
  name: Grid Controller
  category: power
  machine: era1_machine_electronics_printer_mk3
  inputs:
    - { id: era1_component_machine_controller, amount: 1 }
- { id: era1_component_power_board, amount: 2 }
- { id: era1_power_relay, amount: 4 }
  outputs:
    - { id: era1_power_grid_controller, amount: 1 }
  waste_outputs:
    []
  processing_time: 25
  power_consumption: { electrical: 260 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_systems_science
  description: Controls local power grid balancing.
  ```

  ### era1_recipe_backup_power_system — Backup Power System
  ```
  recipe_id: era1_recipe_backup_power_system
  name: Backup Power System
  category: power
  machine: era1_machine_battery_assembler_mk1
  inputs:
    - { id: era1_power_storage_module, amount: 2 }
- { id: era1_power_grid_controller, amount: 1 }
  outputs:
    - { id: era1_power_backup_system, amount: 1 }
  waste_outputs:
    []
  processing_time: 30
  power_consumption: { electrical: 200 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Assembles facility backup power package.
  ```

  ### era1_recipe_cooling_loop — Power Cooling Loop
  ```
  recipe_id: era1_recipe_cooling_loop
  name: Power Cooling Loop
  category: power
  machine: era1_machine_component_fabricator_mk1
  inputs:
    - { id: era1_component_cooling_assembly, amount: 1 }
- { id: era1_fluid_thermal_coolant, amount: 10 }
- { id: era1_component_reinforced_pipe, amount: 4 }
  outputs:
    - { id: era1_component_power_cooling_loop, amount: 1 }
  waste_outputs:
    []
  processing_time: 18
  power_consumption: { electrical: 150 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Cooling loop for generators and transformers.
  ```

  ### era1_recipe_transformer_mk2_parts — Advanced Transformer Parts
  ```
  recipe_id: era1_recipe_transformer_mk2_parts
  name: Advanced Transformer Parts
  category: power
  machine: era1_machine_power_component_factory_mk1
  inputs:
    - { id: era1_power_transformer_core, amount: 2 }
- { id: era1_power_high_voltage_cable, amount: 10 }
- { id: era1_material_insulation, amount: 5 }
  outputs:
    - { id: era1_power_advanced_transformer_parts, amount: 1 }
  waste_outputs:
    []
  processing_time: 22
  power_consumption: { electrical: 220 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Parts for higher-capacity transformers.
  ```

  ### era1_recipe_capacitor_bank — Capacitor Bank
  ```
  recipe_id: era1_recipe_capacitor_bank
  name: Capacitor Bank
  category: power
  machine: era1_machine_electronics_assembler_mk1
  inputs:
    - { id: era1_power_capacitor, amount: 10 }
- { id: era1_component_power_board, amount: 1 }
  outputs:
    - { id: era1_power_capacitor_bank, amount: 1 }
  waste_outputs:
    []
  processing_time: 15
  power_consumption: { electrical: 140 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Banks capacitors for surge handling.
  ```

  ### era1_recipe_energy_cell_mk2 — High Density Energy Cell
  ```
  recipe_id: era1_recipe_energy_cell_mk2
  name: High Density Energy Cell
  category: power
  machine: era1_machine_battery_processor_mk1
  inputs:
    - { id: era1_material_graphite, amount: 8 }
- { id: era1_material_conductive_plate, amount: 8 }
- { id: era1_fluid_electrolyte, amount: 5 }
  outputs:
    - { id: era1_power_energy_cell, amount: 8 }
  waste_outputs:
    - { id: era1_waste_chemical_residue, amount: 1 }
  processing_time: 18
  power_consumption: { electrical: 200 }
  purity_effect: 3
  grade_effect: precision
  technology_unlock: era1_tech_power_systems
  description: Optimized energy cell recipe (+output).
  ```

  ### era1_recipe_solar_absorber_plate — Solar Absorber Plate
  ```
  recipe_id: era1_recipe_solar_absorber_plate
  name: Solar Absorber Plate
  category: power
  machine: era1_machine_precision_fabricator_mk1
  inputs:
    - { id: era1_material_conductive_plate, amount: 5 }
- { id: era1_material_glass, amount: 5 }
- { id: era1_material_industrial_coating, amount: 2 }
  outputs:
    - { id: era1_power_solar_absorber_plate, amount: 4 }
  waste_outputs:
    []
  processing_time: 16
  power_consumption: { electrical: 160 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Absorber plates for solar arrays.
  ```

  ### era1_recipe_solar_panel_mk1 — Solar Panel Mk1
  ```
  recipe_id: era1_recipe_solar_panel_mk1
  name: Solar Panel Mk1
  category: power
  machine: era1_machine_machine_fabricator_mk1
  inputs:
    - { id: era1_power_solar_absorber_plate, amount: 4 }
- { id: era1_component_power_regulator, amount: 1 }
- { id: era1_component_structural_frame, amount: 1 }
  outputs:
    - { id: era1_machine_solar_panel_mk1, amount: 1 }
  waste_outputs:
    []
  processing_time: 25
  power_consumption: { electrical: 180 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Placeable solar panel.
  ```

  ### era1_recipe_power_pole_kit — Power Pole Kit
  ```
  recipe_id: era1_recipe_power_pole_kit
  name: Power Pole Kit
  category: power
  machine: era1_machine_assembler_mk1
  inputs:
    - { id: era1_material_ferrite_plate, amount: 4 }
- { id: era1_power_high_voltage_cable, amount: 2 }
- { id: era1_material_insulation, amount: 1 }
  outputs:
    - { id: era1_power_pole_kit, amount: 1 }
  waste_outputs:
    []
  processing_time: 10
  power_consumption: { electrical: 80 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Kit for power distribution poles.
  ```

  ### era1_recipe_switchgear — Switchgear Unit
  ```
  recipe_id: era1_recipe_switchgear
  name: Switchgear Unit
  category: power
  machine: era1_machine_power_component_factory_mk1
  inputs:
    - { id: era1_component_relay_board, amount: 2 }
- { id: era1_power_advanced_transformer_parts, amount: 1 }
  outputs:
    - { id: era1_power_switchgear, amount: 1 }
  waste_outputs:
    []
  processing_time: 24
  power_consumption: { electrical: 240 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Industrial switchgear for substations.
  ```

  ### era1_recipe_grounding_kit — Grounding Kit
  ```
  recipe_id: era1_recipe_grounding_kit
  name: Grounding Kit
  category: power
  machine: era1_machine_assembler_mk1
  inputs:
    - { id: era1_material_conductive_plate, amount: 4 }
- { id: era1_material_ferrite_rod, amount: 2 }
  outputs:
    - { id: era1_power_grounding_kit, amount: 2 }
  waste_outputs:
    []
  processing_time: 8
  power_consumption: { electrical: 60 }
  purity_effect: 0
  grade_effect: industrial
  technology_unlock: era1_tech_power_systems
  description: Grounding kits for electrical safety.
  ```

### era1_recipe_ferrite_rod — Ferrite Rod
```
recipe_id: era1_recipe_ferrite_rod
name: Ferrite Rod
category: power
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_material_ferrite_plate, amount: 2 }
outputs:
  - { id: era1_material_ferrite_rod, amount: 4 }
waste_outputs:
  []
processing_time: 6
power_consumption: { electrical: 50 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_basic_metallurgy
description: Forms ferrite rods for magnets and grounding.
```

  ### era1_recipe_inverter_module — Inverter Module
  ```
  recipe_id: era1_recipe_inverter_module
  name: Inverter Module
  category: power
  machine: era1_machine_electronics_assembler_mk1
  inputs:
    - { id: era1_component_power_board, amount: 2 }
- { id: era1_power_capacitor_bank, amount: 1 }
  outputs:
    - { id: era1_power_inverter_module, amount: 1 }
  waste_outputs:
    []
  processing_time: 18
  power_consumption: { electrical: 190 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_power_systems
  description: DC/AC inverter for mixed grids.
  ```

  ### era1_recipe_load_balancer — Load Balancer
  ```
  recipe_id: era1_recipe_load_balancer
  name: Load Balancer
  category: power
  machine: era1_machine_electronics_printer_mk3
  inputs:
    - { id: era1_power_grid_controller, amount: 1 }
- { id: era1_component_sensor_array, amount: 1 }
  outputs:
    - { id: era1_power_load_balancer, amount: 1 }
  waste_outputs:
    []
  processing_time: 22
  power_consumption: { electrical: 230 }
  purity_effect: 0
  grade_effect: precision
  technology_unlock: era1_tech_systems_science
  description: Balances draw across power networks.
  ```

  ### era1_recipe_emergency_cell — Emergency Energy Cell
  ```
  recipe_id: era1_recipe_emergency_cell
  name: Emergency Energy Cell
  category: power
  machine: era1_machine_battery_processor_mk1
  inputs:
    - { id: era1_material_carbon_powder, amount: 5 }
- { id: era1_material_conductive_ore, amount: 2 }
  outputs:
    - { id: era1_power_energy_cell, amount: 2 }
  waste_outputs:
    []
  processing_time: 8
  power_consumption: { electrical: 80 }
  purity_effect: -5
  grade_effect: crude
  technology_unlock: era1_tech_basic_recovery
  description: Crude early energy cells from raw materials.
  ```
