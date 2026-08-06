# ERA 1 — Recipe Gap Fill B: R214–350
## Machine components, machines, logistics, research

### era1_recipe_machine_calibration_unit — Machine Calibration Unit
```
recipe_id: era1_recipe_machine_calibration_unit
name: Machine Calibration Unit
category: automation
machine: era1_machine_precision_fabricator_mk1
inputs:
  - { id: era1_component_calibration_chip, amount: 4 }
  - { id: era1_component_sensor_array, amount: 1 }
  - { id: era1_component_precision_housing, amount: 1 }
outputs:
  - { id: era1_component_machine_calibration_unit, amount: 1 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 180 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_precision_manufacturing
description: Calibration unit for precision machines.
```

### era1_recipe_precision_bearing — Precision Bearing
```
recipe_id: era1_recipe_precision_bearing
name: Precision Bearing
category: automation
machine: era1_machine_precision_fabricator_mk1
inputs:
  - { id: era1_component_bearing, amount: 4 }
  - { id: era1_material_precision_alloy, amount: 2 }
  - { id: era1_fluid_lubricant, amount: 2 }
outputs:
  - { id: era1_component_precision_bearing, amount: 4 }
waste_outputs:
  []
processing_time: 14
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_precision_manufacturing
description: High-tolerance bearings.
```

### era1_recipe_magnetic_actuator — Magnetic Actuator
```
recipe_id: era1_recipe_magnetic_actuator
name: Magnetic Actuator
category: automation
machine: era1_machine_component_fabricator_mk1
inputs:
  - { id: era1_component_magnetic_component, amount: 4 }
  - { id: era1_component_actuator, amount: 2 }
outputs:
  - { id: era1_component_magnetic_actuator, amount: 2 }
waste_outputs:
  []
processing_time: 16
power_consumption: { electrical: 150 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_robotics
description: Magnetic high-speed actuator.
```

### era1_recipe_heavy_robotics_frame — Heavy Robotics Frame
```
recipe_id: era1_recipe_heavy_robotics_frame
name: Heavy Robotics Frame
category: automation
machine: era1_machine_robotics_factory_mk1
inputs:
  - { id: era1_component_robotic_frame, amount: 2 }
  - { id: era1_material_hardened_steel, amount: 8 }
outputs:
  - { id: era1_component_heavy_robotics_frame, amount: 1 }
waste_outputs:
  []
processing_time: 25
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_robotics
description: Reinforced frame for industrial robots.
```

### era1_recipe_factory_control_module — Factory Control Module
```
recipe_id: era1_recipe_factory_control_module
name: Factory Control Module
category: automation
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_component_machine_controller, amount: 2 }
  - { id: era1_component_logistic_controller, amount: 1 }
outputs:
  - { id: era1_component_factory_control_module, amount: 1 }
waste_outputs:
  []
processing_time: 28
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Central factory control module.
```

### era1_recipe_thermal_regulator — Thermal Regulator
```
recipe_id: era1_recipe_thermal_regulator
name: Thermal Regulator
category: automation
machine: era1_machine_component_fabricator_mk1
inputs:
  - { id: era1_component_cooling_assembly, amount: 1 }
  - { id: era1_component_sensor, amount: 2 }
  - { id: era1_component_basic_circuit, amount: 2 }
outputs:
  - { id: era1_component_thermal_regulator, amount: 1 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 130 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Regulates machine thermal load.
```

### era1_recipe_pressure_regulator — Pressure Regulator
```
recipe_id: era1_recipe_pressure_regulator
name: Pressure Regulator
category: automation
machine: era1_machine_component_assembler_mk1
inputs:
  - { id: era1_component_industrial_valve, amount: 2 }
  - { id: era1_component_sensor, amount: 1 }
  - { id: era1_component_basic_circuit, amount: 1 }
outputs:
  - { id: era1_component_pressure_regulator, amount: 2 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 110 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Regulates fluid/gas pressure.
```

### era1_recipe_chem_resistant_housing — Chemical-Resistant Housing
```
recipe_id: era1_recipe_chem_resistant_housing
name: Chemical-Resistant Housing
category: automation
machine: era1_machine_component_fabricator_mk1
inputs:
  - { id: era1_component_machine_housing, amount: 2 }
  - { id: era1_material_industrial_coating, amount: 4 }
  - { id: era1_material_advanced_polymer, amount: 2 }
outputs:
  - { id: era1_component_chem_resistant_housing, amount: 2 }
waste_outputs:
  []
processing_time: 18
power_consumption: { electrical: 160 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_chemical_manufacturing
description: Housing for corrosive environments.
```

### era1_recipe_modular_machine_chassis — Modular Machine Chassis
```
recipe_id: era1_recipe_modular_machine_chassis
name: Modular Machine Chassis
category: automation
machine: era1_machine_heavy_assembler_mk1
inputs:
  - { id: era1_component_industrial_frame, amount: 1 }
  - { id: era1_component_precision_housing, amount: 2 }
  - { id: era1_component_control_bus, amount: 1 }
outputs:
  - { id: era1_component_modular_machine_chassis, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 250 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Universal modular chassis.
```

### era1_recipe_servo_cluster — Servo Cluster
```
recipe_id: era1_recipe_servo_cluster
name: Servo Cluster
category: automation
machine: era1_machine_motor_assembly_mk1
inputs:
  - { id: era1_component_servo_motor, amount: 4 }
  - { id: era1_component_precision_bearing, amount: 2 }
outputs:
  - { id: era1_component_servo_cluster, amount: 1 }
waste_outputs:
  []
processing_time: 16
power_consumption: { electrical: 150 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_industrial_automation
description: Cluster of coordinated servos.
```

### era1_recipe_tool_changer — Tool Changer
```
recipe_id: era1_recipe_tool_changer
name: Tool Changer
category: automation
machine: era1_machine_component_fabricator_mk1
inputs:
  - { id: era1_component_actuator, amount: 2 }
  - { id: era1_component_sensor, amount: 1 }
  - { id: era1_component_precision_mechanical_assembly, amount: 1 }
outputs:
  - { id: era1_component_tool_changer, amount: 1 }
waste_outputs:
  []
processing_time: 18
power_consumption: { electrical: 160 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_robotics
description: Automatic tool changer for fabricators.
```

### era1_recipe_vacuum_gripper — Vacuum Gripper
```
recipe_id: era1_recipe_vacuum_gripper
name: Vacuum Gripper
category: automation
machine: era1_machine_component_fabricator_mk1
inputs:
  - { id: era1_component_pneumatic_system, amount: 1 }
  - { id: era1_material_synthetic_rubber, amount: 2 }
outputs:
  - { id: era1_component_vacuum_gripper, amount: 2 }
waste_outputs:
  []
processing_time: 10
power_consumption: { electrical: 90 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_industrial_automation
description: Pneumatic part gripper.
```

### era1_recipe_linear_rail — Linear Rail
```
recipe_id: era1_recipe_linear_rail
name: Linear Rail
category: automation
machine: era1_machine_precision_fabricator_mk1
inputs:
  - { id: era1_material_hardened_steel, amount: 6 }
  - { id: era1_component_precision_bearing, amount: 2 }
outputs:
  - { id: era1_component_linear_rail, amount: 2 }
waste_outputs:
  []
processing_time: 14
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_precision_manufacturing
description: Precision linear motion rail.
```

### era1_recipe_encoder_module — Encoder Module
```
recipe_id: era1_recipe_encoder_module
name: Encoder Module
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_optical_sensor, amount: 1 }
  - { id: era1_component_basic_circuit, amount: 2 }
outputs:
  - { id: era1_component_encoder_module, amount: 2 }
waste_outputs:
  []
processing_time: 10
power_consumption: { electrical: 100 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_electronics
description: Position encoder for motors.
```

### era1_recipe_safety_interlock — Safety Interlock
```
recipe_id: era1_recipe_safety_interlock
name: Safety Interlock
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_sensor, amount: 2 }
  - { id: era1_component_relay_board, amount: 1 }
outputs:
  - { id: era1_component_safety_interlock, amount: 2 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 110 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Machine safety interlock.
```

### era1_recipe_vibration_damper — Vibration Damper
```
recipe_id: era1_recipe_vibration_damper
name: Vibration Damper
category: automation
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_material_synthetic_rubber, amount: 5 }
  - { id: era1_material_steel_composite, amount: 2 }
outputs:
  - { id: era1_component_vibration_damper, amount: 4 }
waste_outputs:
  []
processing_time: 8
power_consumption: { electrical: 70 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_industrial_automation
description: Dampers for high-speed machines.
```

### era1_recipe_cable_harness — Cable Harness
```
recipe_id: era1_recipe_cable_harness
name: Cable Harness
category: automation
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_material_conductive_wire, amount: 15 }
  - { id: era1_material_insulation, amount: 3 }
outputs:
  - { id: era1_component_cable_harness, amount: 3 }
waste_outputs:
  []
processing_time: 10
power_consumption: { electrical: 80 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_electronics
description: Pre-wired cable harnesses.
```

### era1_recipe_bus_coupler — Bus Coupler
```
recipe_id: era1_recipe_bus_coupler
name: Bus Coupler
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_control_bus, amount: 2 }
  - { id: era1_component_interface_module, amount: 1 }
outputs:
  - { id: era1_component_bus_coupler, amount: 2 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 120 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Couples machine control buses.
```

### era1_recipe_firmware_loader — Firmware Loader
```
recipe_id: era1_recipe_firmware_loader
name: Firmware Loader
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_firmware_module, amount: 2 }
  - { id: era1_component_interface_module, amount: 1 }
outputs:
  - { id: era1_component_firmware_loader, amount: 1 }
waste_outputs:
  []
processing_time: 14
power_consumption: { electrical: 130 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_electronics
description: Loads firmware onto machine controllers.
```

### era1_recipe_maintenance_drone_kit — Maintenance Drone Kit
```
recipe_id: era1_recipe_maintenance_drone_kit
name: Maintenance Drone Kit
category: automation
machine: era1_machine_robotics_factory_mk1
inputs:
  - { id: era1_component_drone_chassis, amount: 1 }
  - { id: era1_component_tool_changer, amount: 1 }
  - { id: era1_component_diagnostic_module, amount: 1 }
outputs:
  - { id: era1_component_maintenance_drone_kit, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 260 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_robotics
description: Kit for maintenance drones.
```

### era1_recipe_vision_module — Vision Module
```
recipe_id: era1_recipe_vision_module
name: Vision Module
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_optical_sensor, amount: 2 }
  - { id: era1_component_processor_core, amount: 1 }
outputs:
  - { id: era1_component_vision_module, amount: 1 }
waste_outputs:
  []
processing_time: 18
power_consumption: { electrical: 180 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_robotics
description: Machine vision package.
```

### era1_recipe_force_sensor — Force Sensor
```
recipe_id: era1_recipe_force_sensor
name: Force Sensor
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_sensor, amount: 2 }
  - { id: era1_component_precision_bearing, amount: 1 }
outputs:
  - { id: era1_component_force_sensor, amount: 2 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 120 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_precision_manufacturing
description: Force/torque sensor.
```

### era1_recipe_end_effector — End Effector
```
recipe_id: era1_recipe_end_effector
name: End Effector
category: automation
machine: era1_machine_robotics_component_printer_mk1
inputs:
  - { id: era1_component_vacuum_gripper, amount: 1 }
  - { id: era1_component_force_sensor, amount: 1 }
  - { id: era1_component_actuator, amount: 1 }
outputs:
  - { id: era1_component_end_effector, amount: 1 }
waste_outputs:
  []
processing_time: 16
power_consumption: { electrical: 150 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_robotics
description: Robotic end effector assembly.
```

### era1_recipe_motion_planner — Motion Planner Module
```
recipe_id: era1_recipe_motion_planner
name: Motion Planner Module
category: automation
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_component_robotics_controller, amount: 1 }
  - { id: era1_component_navigation_module, amount: 1 }
outputs:
  - { id: era1_component_motion_planner, amount: 1 }
waste_outputs:
  []
processing_time: 24
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_robotics
description: Plans multi-axis robot motion.
```

### era1_recipe_industrial_ai_lite — Industrial AI Lite Controller
```
recipe_id: era1_recipe_industrial_ai_lite
name: Industrial AI Lite Controller
category: automation
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_component_industrial_ai_assistant, amount: 1 }
  - { id: era1_component_factory_control_module, amount: 1 }
outputs:
  - { id: era1_component_industrial_ai_lite, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 350 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Lightweight industrial decision controller.
```

### era1_recipe_modular_io_rack — Modular I/O Rack
```
recipe_id: era1_recipe_modular_io_rack
name: Modular I/O Rack
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_interface_module, amount: 4 }
  - { id: era1_component_control_bus, amount: 2 }
outputs:
  - { id: era1_component_modular_io_rack, amount: 1 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Rack for factory I/O modules.
```

### era1_recipe_uptime_monitor — Uptime Monitor
```
recipe_id: era1_recipe_uptime_monitor
name: Uptime Monitor
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_diagnostic_module, amount: 1 }
  - { id: era1_component_data_storage_module, amount: 1 }
outputs:
  - { id: era1_component_uptime_monitor, amount: 1 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 120 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Tracks machine uptime and faults.
```

### era1_recipe_chemical_filter_machine — Chemical Filter Machine
```
recipe_id: era1_recipe_chemical_filter_machine
name: Chemical Filter Machine
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_chem_resistant_housing, amount: 2 }
  - { id: era1_component_chemical_filter, amount: 10 }
  - { id: era1_component_control_module, amount: 1 }
outputs:
  - { id: era1_machine_chemical_filter_machine, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 250 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Builds a chemical filtration machine.
```

### era1_recipe_gas_compressor — Gas Compressor
```
recipe_id: era1_recipe_gas_compressor
name: Gas Compressor
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_heavy_motor, amount: 1 }
  - { id: era1_component_pressure_chamber, amount: 2 }
  - { id: era1_component_pressure_regulator, amount: 2 }
outputs:
  - { id: era1_machine_gas_compressor, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Builds a gas compressor.
```

### era1_recipe_fluid_pump_mk1 — Fluid Pump Mk1
```
recipe_id: era1_recipe_fluid_pump_mk1
name: Fluid Pump Mk1
category: machines
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_component_industrial_motor, amount: 1 }
  - { id: era1_component_reinforced_pipe, amount: 4 }
  - { id: era1_component_industrial_valve, amount: 2 }
outputs:
  - { id: era1_machine_fluid_pump_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 150 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Builds a fluid pump.
```

### era1_recipe_storage_tank_machine — Storage Tank
```
recipe_id: era1_recipe_storage_tank_machine
name: Storage Tank
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_chemical_storage_parts, amount: 2 }
  - { id: era1_component_structural_panel, amount: 5 }
outputs:
  - { id: era1_machine_storage_tank, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Builds a large fluid storage tank.
```

### era1_recipe_precision_roller — Precision Roller
```
recipe_id: era1_recipe_precision_roller
name: Precision Roller
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_precision_housing, amount: 2 }
  - { id: era1_component_industrial_motor, amount: 1 }
  - { id: era1_component_precision_bearing, amount: 4 }
outputs:
  - { id: era1_machine_precision_roller_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_precision_manufacturing
description: Builds a precision roller for foils.
```

### era1_recipe_hydraulic_press — Hydraulic Press
```
recipe_id: era1_recipe_hydraulic_press
name: Hydraulic Press
category: machines
machine: era1_machine_heavy_assembler_mk1
inputs:
  - { id: era1_component_hydraulic_system, amount: 2 }
  - { id: era1_component_heavy_structural_frame, amount: 2 }
  - { id: era1_component_heavy_motor, amount: 1 }
outputs:
  - { id: era1_machine_hydraulic_press_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 45
power_consumption: { electrical: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_metallurgy
description: Builds a hydraulic press.
```

### era1_recipe_heat_treatment_furnace — Heat Treatment Furnace
```
recipe_id: era1_recipe_heat_treatment_furnace
name: Heat Treatment Furnace
category: machines
machine: era1_machine_heavy_assembler_mk1
inputs:
  - { id: era1_material_heat_resistant_ceramic, amount: 15 }
  - { id: era1_component_heavy_structural_frame, amount: 2 }
  - { id: era1_component_thermal_regulator, amount: 1 }
outputs:
  - { id: era1_machine_heat_treatment_furnace_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 50
power_consumption: { electrical: 350 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_metallurgy
description: Builds a heat treatment furnace.
```

### era1_recipe_composite_processor_machine — Composite Processor
```
recipe_id: era1_recipe_composite_processor_machine
name: Composite Processor
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_chem_resistant_housing, amount: 2 }
  - { id: era1_component_industrial_motor, amount: 1 }
  - { id: era1_component_control_module, amount: 1 }
outputs:
  - { id: era1_machine_composite_processor_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_polymer_science
description: Builds a composite processor.
```

### era1_recipe_recycling_plant — Recycling Plant
```
recipe_id: era1_recipe_recycling_plant
name: Recycling Plant
category: machines
machine: era1_machine_heavy_assembler_mk1
inputs:
  - { id: era1_component_heavy_structural_frame, amount: 2 }
  - { id: era1_component_chem_resistant_housing, amount: 2 }
  - { id: era1_component_machine_controller, amount: 1 }
outputs:
  - { id: era1_machine_recycling_plant_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 50
power_consumption: { electrical: 320 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_waste_recovery
description: Builds a general recycling plant.
```

### era1_recipe_recovery_facility — Recovery Facility
```
recipe_id: era1_recipe_recovery_facility
name: Recovery Facility
category: machines
machine: era1_machine_heavy_assembler_mk1
inputs:
  - { id: era1_component_pressure_chamber, amount: 2 }
  - { id: era1_component_industrial_valve, amount: 4 }
  - { id: era1_component_control_module, amount: 2 }
outputs:
  - { id: era1_machine_recovery_plant_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 45
power_consumption: { electrical: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_waste_recovery
description: Builds a metal/mineral recovery plant.
```

### era1_recipe_research_analyzer — Research Analyzer
```
recipe_id: era1_recipe_research_analyzer
name: Research Analyzer
category: machines
machine: era1_machine_electronics_printer_mk2
inputs:
  - { id: era1_component_research_processor, amount: 1 }
  - { id: era1_component_sensor_array, amount: 2 }
  - { id: era1_component_precision_housing, amount: 1 }
outputs:
  - { id: era1_machine_research_analyzer, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Builds a research analyzer.
```

### era1_recipe_laboratory_module — Laboratory Module
```
recipe_id: era1_recipe_laboratory_module
name: Laboratory Module
category: machines
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_building_research_laboratory, amount: 1 }
  - { id: era1_component_research_processor, amount: 1 }
  - { id: era1_component_modular_io_rack, amount: 1 }
outputs:
  - { id: era1_machine_laboratory_module, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 260 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Expandable lab module.
```

### era1_recipe_boiler_mk1 — Boiler Mk1
```
recipe_id: era1_recipe_boiler_mk1
name: Boiler Mk1
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_material_heat_resistant_ceramic, amount: 8 }
  - { id: era1_component_reinforced_pipe, amount: 6 }
  - { id: era1_component_pressure_chamber, amount: 1 }
outputs:
  - { id: era1_machine_boiler_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Builds a process steam boiler.
```

### era1_recipe_atmospheric_intake_machine — Atmospheric Intake
```
recipe_id: era1_recipe_atmospheric_intake_machine
name: Atmospheric Intake
category: machines
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_component_cooling_assembly, amount: 1 }
  - { id: era1_component_industrial_motor, amount: 1 }
  - { id: era1_component_filter_cartridge, amount: 2 }
outputs:
  - { id: era1_machine_atmospheric_intake_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 25
power_consumption: { electrical: 160 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Builds atmospheric intake compressor.
```

### era1_recipe_advanced_purifier — Advanced Purifier
```
recipe_id: era1_recipe_advanced_purifier
name: Advanced Purifier
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_pressure_chamber, amount: 2 }
  - { id: era1_component_chemical_filter, amount: 10 }
  - { id: era1_component_machine_controller, amount: 1 }
outputs:
  - { id: era1_machine_advanced_purifier_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_metallurgy
description: Builds advanced ore purifier.
```

### era1_recipe_reduction_furnace — Reduction Furnace
```
recipe_id: era1_recipe_reduction_furnace
name: Reduction Furnace
category: machines
machine: era1_machine_heavy_assembler_mk1
inputs:
  - { id: era1_material_reactor_lining, amount: 5 }
  - { id: era1_component_heavy_structural_frame, amount: 2 }
  - { id: era1_component_thermal_regulator, amount: 1 }
outputs:
  - { id: era1_machine_reduction_furnace_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 50
power_consumption: { electrical: 340 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_metallurgy
description: Builds reduction furnace.
```

### era1_recipe_fiber_processor — Fiber Processor
```
recipe_id: era1_recipe_fiber_processor
name: Fiber Processor
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_precision_housing, amount: 2 }
  - { id: era1_component_industrial_motor, amount: 1 }
  - { id: era1_component_control_module, amount: 1 }
outputs:
  - { id: era1_machine_fiber_processor_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 32
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_polymer_science
description: Builds fiber processor.
```

### era1_recipe_coating_unit — Coating Unit
```
recipe_id: era1_recipe_coating_unit
name: Coating Unit
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_chem_resistant_housing, amount: 1 }
  - { id: era1_component_reinforced_pipe, amount: 4 }
  - { id: era1_component_control_module, amount: 1 }
outputs:
  - { id: era1_machine_coating_unit_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_chemical_manufacturing
description: Builds industrial coating unit.
```

### era1_recipe_industrial_grinder — Industrial Grinder
```
recipe_id: era1_recipe_industrial_grinder
name: Industrial Grinder
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_heavy_motor, amount: 1 }
  - { id: era1_component_heavy_structural_frame, amount: 1 }
  - { id: era1_component_precision_bearing, amount: 4 }
outputs:
  - { id: era1_machine_industrial_grinder_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_material_processing
description: Builds industrial grinder.
```

### era1_recipe_motor_assembly_machine — Motor Assembly Machine
```
recipe_id: era1_recipe_motor_assembly_machine
name: Motor Assembly Machine
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_modular_machine_chassis, amount: 1 }
  - { id: era1_component_servo_cluster, amount: 1 }
  - { id: era1_component_machine_controller, amount: 1 }
outputs:
  - { id: era1_machine_motor_assembly_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 260 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_industrial_automation
description: Builds motor assembly machine.
```

### era1_recipe_robotics_printer — Robotics Component Printer
```
recipe_id: era1_recipe_robotics_printer
name: Robotics Component Printer
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_precision_housing, amount: 2 }
  - { id: era1_component_electronics_printer_parts, amount: 1 }
  - { id: era1_component_robotics_controller, amount: 1 }
outputs:
  - { id: era1_machine_robotics_component_printer_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 270 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_robotics
description: Builds robotics component printer.
```

### era1_recipe_fluid_processor — Fluid Processor
```
recipe_id: era1_recipe_fluid_processor
name: Fluid Processor
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_chem_resistant_housing, amount: 1 }
  - { id: era1_component_industrial_valve, amount: 4 }
  - { id: era1_component_control_module, amount: 1 }
outputs:
  - { id: era1_machine_fluid_processor_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 210 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_chemical_manufacturing
description: Builds fluid processor.
```

### era1_recipe_polymer_reactor_mk2 — Polymer Reactor Mk2
```
recipe_id: era1_recipe_polymer_reactor_mk2
name: Polymer Reactor Mk2
category: machines
machine: era1_machine_heavy_assembler_mk1
inputs:
  - { id: era1_component_pressure_chamber, amount: 3 }
  - { id: era1_component_chem_resistant_housing, amount: 2 }
  - { id: era1_component_machine_controller, amount: 1 }
outputs:
  - { id: era1_machine_polymer_reactor_mk2, amount: 1 }
waste_outputs:
  []
processing_time: 50
power_consumption: { electrical: 320 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_polymer_science
description: Builds upgraded polymer reactor.
```

### era1_recipe_electronics_printer_mk2 — Electronics Printer Mk2
```
recipe_id: era1_recipe_electronics_printer_mk2
name: Electronics Printer Mk2
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_electronics_printer_parts, amount: 2 }
  - { id: era1_component_precision_housing, amount: 2 }
  - { id: era1_component_processor_core, amount: 1 }
outputs:
  - { id: era1_machine_electronics_printer_mk2, amount: 1 }
waste_outputs:
  []
processing_time: 45
power_consumption: { electrical: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_electronics
description: Builds Mk2 electronics printer.
```

### era1_recipe_electronics_printer_mk3 — Electronics Printer Mk3
```
recipe_id: era1_recipe_electronics_printer_mk3
name: Electronics Printer Mk3
category: machines
machine: era1_machine_heavy_assembler_mk1
inputs:
  - { id: era1_machine_electronics_printer_mk2, amount: 1 }
  - { id: era1_component_research_processor, amount: 1 }
  - { id: era1_component_high_density_circuit, amount: 2 }
outputs:
  - { id: era1_machine_electronics_printer_mk3, amount: 1 }
waste_outputs:
  []
processing_time: 60
power_consumption: { electrical: 400 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_electronics
description: Builds Mk3 electronics printer.
```

### era1_recipe_assembler_mk2 — Assembler Mk2
```
recipe_id: era1_recipe_assembler_mk2
name: Assembler Mk2
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_assembler_parts, amount: 2 }
  - { id: era1_component_industrial_motor, amount: 2 }
  - { id: era1_component_machine_controller, amount: 1 }
outputs:
  - { id: era1_machine_assembler_mk2, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Builds Assembler Mk2.
```

### era1_recipe_assembler_mk3 — Assembler Mk3
```
recipe_id: era1_recipe_assembler_mk3
name: Assembler Mk3
category: machines
machine: era1_machine_heavy_assembler_mk1
inputs:
  - { id: era1_machine_assembler_mk2, amount: 1 }
  - { id: era1_component_servo_cluster, amount: 2 }
  - { id: era1_component_factory_control_module, amount: 1 }
outputs:
  - { id: era1_machine_assembler_mk3, amount: 1 }
waste_outputs:
  []
processing_time: 55
power_consumption: { electrical: 360 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Builds Assembler Mk3.
```

### era1_recipe_heavy_assembler — Heavy Assembler
```
recipe_id: era1_recipe_heavy_assembler
name: Heavy Assembler
category: machines
machine: era1_machine_machine_fabricator_mk1
inputs:
  - { id: era1_component_heavy_structural_frame, amount: 3 }
  - { id: era1_component_heavy_motor, amount: 2 }
  - { id: era1_component_machine_controller, amount: 1 }
outputs:
  - { id: era1_machine_heavy_assembler_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 50
power_consumption: { electrical: 340 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_metallurgy
description: Builds heavy assembler.
```

### era1_recipe_smart_storage_controller — Smart Storage Controller
```
recipe_id: era1_recipe_smart_storage_controller
name: Smart Storage Controller
category: logistics
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_control_module, amount: 1 }
  - { id: era1_component_sensor, amount: 2 }
outputs:
  - { id: era1_logistics_smart_storage_controller, amount: 1 }
waste_outputs:
  []
processing_time: 10
power_consumption: { electrical: 100 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Controller retrofit for storage.
```

### era1_recipe_filter_system — Belt Filter System
```
recipe_id: era1_recipe_filter_system
name: Belt Filter System
category: logistics
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_splitter, amount: 1 }
  - { id: era1_component_sensor_array, amount: 1 }
outputs:
  - { id: era1_logistics_filter_system, amount: 1 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 120 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Advanced belt filtering junction.
```

### era1_recipe_pump_jack_parts — Pump System
```
recipe_id: era1_recipe_pump_jack_parts
name: Pump System
category: logistics
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_machine_fluid_pump_mk1, amount: 1 }
  - { id: era1_component_pressure_regulator, amount: 1 }
outputs:
  - { id: era1_logistics_pump_system, amount: 1 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 100 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Integrated pump system module.
```

### era1_recipe_high_volume_pipe — High-Volume Pipe
```
recipe_id: era1_recipe_high_volume_pipe
name: High-Volume Pipe
category: logistics
machine: era1_machine_component_assembler_mk1
inputs:
  - { id: era1_component_reinforced_pipe, amount: 5 }
  - { id: era1_component_high_pressure_fitting, amount: 2 }
outputs:
  - { id: era1_logistics_high_volume_pipe, amount: 5 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 110 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Large-bore pipe segments.
```

### era1_recipe_conveyor_sensor — Conveyor Sensor
```
recipe_id: era1_recipe_conveyor_sensor
name: Conveyor Sensor
category: logistics
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_optical_sensor, amount: 1 }
  - { id: era1_component_basic_circuit, amount: 1 }
outputs:
  - { id: era1_logistics_conveyor_sensor, amount: 4 }
waste_outputs:
  []
processing_time: 8
power_consumption: { electrical: 80 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Belt presence/speed sensors.
```

### era1_recipe_automated_sorter — Automated Sorter
```
recipe_id: era1_recipe_automated_sorter
name: Automated Sorter
category: logistics
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_filter_system, amount: 1 }
  - { id: era1_component_vision_module, amount: 1 }
  - { id: era1_component_machine_controller, amount: 1 }
outputs:
  - { id: era1_logistics_automated_sorter, amount: 1 }
waste_outputs:
  []
processing_time: 25
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Vision-guided item sorter.
```

### era1_recipe_fluid_distributor — Fluid Distribution Module
```
recipe_id: era1_recipe_fluid_distributor
name: Fluid Distribution Module
category: logistics
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_high_volume_pipe, amount: 5 }
  - { id: era1_component_industrial_valve, amount: 4 }
  - { id: era1_component_pressure_regulator, amount: 2 }
outputs:
  - { id: era1_logistics_fluid_distributor, amount: 1 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 160 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Multi-output fluid distributor.
```

### era1_recipe_belt_balancer — Belt Balancer
```
recipe_id: era1_recipe_belt_balancer
name: Belt Balancer
category: logistics
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_splitter, amount: 2 }
  - { id: era1_logistics_merger, amount: 2 }
  - { id: era1_component_control_module, amount: 1 }
outputs:
  - { id: era1_logistics_belt_balancer, amount: 1 }
waste_outputs:
  []
processing_time: 18
power_consumption: { electrical: 150 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Lane balancer assembly.
```

### era1_recipe_stack_inserter — Stack Inserter
```
recipe_id: era1_recipe_stack_inserter
name: Stack Inserter
category: logistics
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_fast_inserter, amount: 1 }
  - { id: era1_component_vacuum_gripper, amount: 1 }
outputs:
  - { id: era1_logistics_stack_inserter, amount: 1 }
waste_outputs:
  []
processing_time: 16
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Multi-item stack inserter.
```

### era1_recipe_long_inserter — Long Inserter
```
recipe_id: era1_recipe_long_inserter
name: Long Inserter
category: logistics
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_logistics_inserter_arm, amount: 1 }
  - { id: era1_component_mechanical_shaft, amount: 2 }
outputs:
  - { id: era1_logistics_long_inserter, amount: 1 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 100 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_industrial_automation
description: Extended-reach inserter.
```

### era1_recipe_loader — Belt Loader
```
recipe_id: era1_recipe_loader
name: Belt Loader
category: logistics
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_conveyor_segment, amount: 4 }
  - { id: era1_logistics_fast_inserter, amount: 1 }
outputs:
  - { id: era1_logistics_loader, amount: 1 }
waste_outputs:
  []
processing_time: 14
power_consumption: { electrical: 120 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Bulk loader onto belts.
```

### era1_recipe_unloader — Belt Unloader
```
recipe_id: era1_recipe_unloader
name: Belt Unloader
category: logistics
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_loader, amount: 1 }
  - { id: era1_component_sensor, amount: 1 }
outputs:
  - { id: era1_logistics_unloader, amount: 1 }
waste_outputs:
  []
processing_time: 14
power_consumption: { electrical: 120 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Bulk unloader from belts.
```

### era1_recipe_warehouse_module — Warehouse Module
```
recipe_id: era1_recipe_warehouse_module
name: Warehouse Module
category: logistics
machine: era1_machine_assembler_mk3
inputs:
  - { id: era1_logistics_smart_storage_unit, amount: 4 }
  - { id: era1_logistics_logistic_controller, amount: 1 }
outputs:
  - { id: era1_logistics_warehouse_module, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Large smart warehouse module.
```

### era1_recipe_pipe_to_ground — Pipe to Ground
```
recipe_id: era1_recipe_pipe_to_ground
name: Pipe to Ground
category: logistics
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_component_reinforced_pipe, amount: 2 }
  - { id: era1_component_structural_frame, amount: 1 }
outputs:
  - { id: era1_logistics_pipe_to_ground, amount: 2 }
waste_outputs:
  []
processing_time: 8
power_consumption: { electrical: 70 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Underground pipe pair.
```

### era1_recipe_overflow_gate — Overflow Gate
```
recipe_id: era1_recipe_overflow_gate
name: Overflow Gate
category: logistics
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_logistics_splitter, amount: 1 }
  - { id: era1_component_basic_circuit, amount: 1 }
outputs:
  - { id: era1_logistics_overflow_gate, amount: 1 }
waste_outputs:
  []
processing_time: 10
power_consumption: { electrical: 90 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Overflow routing gate.
```

### era1_recipe_priority_splitter — Priority Splitter
```
recipe_id: era1_recipe_priority_splitter
name: Priority Splitter
category: logistics
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_splitter, amount: 1 }
  - { id: era1_component_control_module, amount: 1 }
outputs:
  - { id: era1_logistics_priority_splitter, amount: 1 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 110 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Priority output splitter.
```

### era1_recipe_fluid_meter — Fluid Meter
```
recipe_id: era1_recipe_fluid_meter
name: Fluid Meter
category: logistics
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_sensor, amount: 1 }
  - { id: era1_component_basic_circuit, amount: 1 }
  - { id: era1_component_reinforced_pipe, amount: 1 }
outputs:
  - { id: era1_logistics_fluid_meter, amount: 2 }
waste_outputs:
  []
processing_time: 8
power_consumption: { electrical: 80 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_fluid_engineering
description: Measures fluid flow.
```

### era1_recipe_belt_lift — Belt Lift
```
recipe_id: era1_recipe_belt_lift
name: Belt Lift
category: logistics
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_fast_conveyor_segment, amount: 4 }
  - { id: era1_component_industrial_motor, amount: 1 }
outputs:
  - { id: era1_logistics_belt_lift, amount: 1 }
waste_outputs:
  []
processing_time: 16
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Vertical belt lift.
```

### era1_recipe_drone_pad — Logistics Drone Pad
```
recipe_id: era1_recipe_drone_pad
name: Logistics Drone Pad
category: logistics
machine: era1_machine_assembler_mk3
inputs:
  - { id: era1_component_drone_chassis, amount: 1 }
  - { id: era1_logistics_logistic_controller, amount: 1 }
  - { id: era1_component_navigation_module, amount: 1 }
outputs:
  - { id: era1_logistics_drone_pad, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_robotics
description: Pad for logistics drones.
```

### era1_recipe_request_chest — Request Chest
```
recipe_id: era1_recipe_request_chest
name: Request Chest
category: logistics
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_smart_storage_unit, amount: 1 }
  - { id: era1_logistics_logistic_controller, amount: 1 }
outputs:
  - { id: era1_logistics_request_chest, amount: 1 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 130 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Logistics request storage.
```

### era1_recipe_provider_chest — Provider Chest
```
recipe_id: era1_recipe_provider_chest
name: Provider Chest
category: logistics
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_smart_storage_unit, amount: 1 }
  - { id: era1_component_control_module, amount: 1 }
outputs:
  - { id: era1_logistics_provider_chest, amount: 1 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 130 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Logistics provider storage.
```

### era1_recipe_buffer_chest — Buffer Chest
```
recipe_id: era1_recipe_buffer_chest
name: Buffer Chest
category: logistics
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_smart_storage_unit, amount: 1 }
  - { id: era1_component_data_storage_module, amount: 1 }
outputs:
  - { id: era1_logistics_buffer_chest, amount: 1 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 130 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Logistics buffer storage.
```

### era1_recipe_robo_port — Roboport Mk1
```
recipe_id: era1_recipe_robo_port
name: Roboport Mk1
category: logistics
machine: era1_machine_assembler_mk3
inputs:
  - { id: era1_logistics_drone_pad, amount: 1 }
  - { id: era1_power_storage_module, amount: 1 }
  - { id: era1_component_factory_control_module, amount: 1 }
outputs:
  - { id: era1_logistics_roboport_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 45
power_consumption: { electrical: 320 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_robotics
description: Era 1 logistics roboport.
```

### era1_recipe_construction_drone — Construction Drone
```
recipe_id: era1_recipe_construction_drone
name: Construction Drone
category: logistics
machine: era1_machine_robotics_factory_mk1
inputs:
  - { id: era1_component_maintenance_drone_kit, amount: 1 }
  - { id: era1_component_end_effector, amount: 1 }
outputs:
  - { id: era1_logistics_construction_drone, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 250 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_robotics
description: Construction/repair drone.
```

### era1_recipe_logistic_drone — Logistic Drone
```
recipe_id: era1_recipe_logistic_drone
name: Logistic Drone
category: logistics
machine: era1_machine_robotics_factory_mk1
inputs:
  - { id: era1_component_drone_chassis, amount: 1 }
  - { id: era1_component_vacuum_gripper, amount: 1 }
  - { id: era1_component_navigation_module, amount: 1 }
outputs:
  - { id: era1_logistics_logistic_drone, amount: 1 }
waste_outputs:
  []
processing_time: 28
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_robotics
description: Item transport drone.
```

### era1_recipe_wire_red — Red Circuit Wire
```
recipe_id: era1_recipe_wire_red
name: Red Circuit Wire
category: logistics
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_material_conductive_wire, amount: 2 }
  - { id: era1_material_polymer_resin, amount: 1 }
outputs:
  - { id: era1_logistics_red_wire, amount: 10 }
waste_outputs:
  []
processing_time: 5
power_consumption: { electrical: 40 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_electronics
description: Logistics/control red wire.
```

### era1_recipe_wire_green — Green Circuit Wire
```
recipe_id: era1_recipe_wire_green
name: Green Circuit Wire
category: logistics
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_material_conductive_wire, amount: 2 }
  - { id: era1_material_polymer_resin, amount: 1 }
outputs:
  - { id: era1_logistics_green_wire, amount: 10 }
waste_outputs:
  []
processing_time: 5
power_consumption: { electrical: 40 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_electronics
description: Logistics/control green wire.
```

### era1_recipe_combinator — Decider Combinator
```
recipe_id: era1_recipe_combinator
name: Decider Combinator
category: logistics
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_basic_circuit, amount: 4 }
  - { id: era1_component_logic_board, amount: 1 }
outputs:
  - { id: era1_logistics_decider_combinator, amount: 1 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 110 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Logic combinator for circuits.
```

### era1_recipe_arithmetic_combinator — Arithmetic Combinator
```
recipe_id: era1_recipe_arithmetic_combinator
name: Arithmetic Combinator
category: logistics
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_basic_circuit, amount: 4 }
  - { id: era1_component_processor_core, amount: 1 }
outputs:
  - { id: era1_logistics_arithmetic_combinator, amount: 1 }
waste_outputs:
  []
processing_time: 14
power_consumption: { electrical: 130 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Arithmetic combinator.
```

### era1_recipe_speaker — Programmable Speaker
```
recipe_id: era1_recipe_speaker
name: Programmable Speaker
category: logistics
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_component_basic_circuit, amount: 1 }
  - { id: era1_material_polymer_foam, amount: 1 }
outputs:
  - { id: era1_logistics_speaker, amount: 1 }
waste_outputs:
  []
processing_time: 8
power_consumption: { electrical: 60 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_electronics
description: Alert speaker for factory circuits.
```

### era1_recipe_research_module_frame — Research Module Frame
```
recipe_id: era1_recipe_research_module_frame
name: Research Module Frame
category: science
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_component_structural_frame, amount: 5 }
  - { id: era1_component_precision_housing, amount: 2 }
outputs:
  - { id: era1_science_research_module_frame, amount: 1 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 160 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Frame for lab expansion modules.
```

### era1_recipe_data_processor_rack — Data Processor Rack
```
recipe_id: era1_recipe_data_processor_rack
name: Data Processor Rack
category: science
machine: era1_machine_electronics_printer_mk2
inputs:
  - { id: era1_component_research_processor, amount: 2 }
  - { id: era1_component_data_storage_module, amount: 4 }
outputs:
  - { id: era1_science_data_processor_rack, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Rack for research data processing.
```

### era1_recipe_analysis_equipment — Analysis Equipment
```
recipe_id: era1_recipe_analysis_equipment
name: Analysis Equipment
category: science
machine: era1_machine_precision_fabricator_mk1
inputs:
  - { id: era1_component_sensor_array, amount: 2 }
  - { id: era1_component_optical_sensor, amount: 2 }
  - { id: era1_material_reinforced_glass, amount: 2 }
outputs:
  - { id: era1_science_analysis_equipment, amount: 1 }
waste_outputs:
  []
processing_time: 25
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Lab analysis instrument package.
```

### era1_recipe_simulation_core — Simulation Core
```
recipe_id: era1_recipe_simulation_core
name: Simulation Core
category: science
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_component_processor_core, amount: 3 }
  - { id: era1_science_computational_data, amount: 2 }
outputs:
  - { id: era1_science_simulation_core, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 350 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Simulates designs before deployment.
```

### era1_recipe_prototype_bench — Prototype Bench
```
recipe_id: era1_recipe_prototype_bench
name: Prototype Bench
category: science
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_science_research_module_frame, amount: 1 }
  - { id: era1_component_tool_changer, amount: 1 }
  - { id: era1_component_precision_mechanical_assembly, amount: 2 }
outputs:
  - { id: era1_science_prototype_bench, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Bench for building prototypes.
```

### era1_recipe_tech_validation_unit — Technology Validation Unit
```
recipe_id: era1_recipe_tech_validation_unit
name: Technology Validation Unit
category: science
machine: era1_machine_laboratory_module
inputs:
  - { id: era1_science_analysis_equipment, amount: 1 }
  - { id: era1_science_simulation_core, amount: 1 }
outputs:
  - { id: era1_science_tech_validation_unit, amount: 1 }
waste_outputs:
  []
processing_time: 45
power_consumption: { electrical: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Validates tech unlock conditions.
```

### era1_recipe_material_science_pack — Material Science Sample
```
recipe_id: era1_recipe_material_science_pack
name: Material Science Sample
category: science
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_material_precision_alloy, amount: 5 }
  - { id: era1_material_advanced_ceramic, amount: 5 }
  - { id: era1_component_data_storage_module, amount: 2 }
outputs:
  - { id: era1_science_material_sample, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_material_science
description: Sample pack boosting purity research.
```

### era1_recipe_chemical_science_pack — Chemical Science Sample
```
recipe_id: era1_recipe_chemical_science_pack
name: Chemical Science Sample
category: science
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_fluid_catalyst_solution, amount: 5 }
  - { id: era1_material_advanced_polymer, amount: 5 }
  - { id: era1_component_data_storage_module, amount: 2 }
outputs:
  - { id: era1_science_chemical_sample, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_chemical_manufacturing
description: Sample pack for chemical science.
```

### era1_recipe_mechanical_science_pack — Mechanical Science Sample
```
recipe_id: era1_recipe_mechanical_science_pack
name: Mechanical Science Sample
category: science
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_component_precision_bearing, amount: 4 }
  - { id: era1_component_industrial_motor, amount: 2 }
  - { id: era1_component_data_storage_module, amount: 2 }
outputs:
  - { id: era1_science_mechanical_sample, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_industrial_automation
description: Sample pack for mechanical science.
```

### era1_recipe_systems_science_pack — Systems Science Sample
```
recipe_id: era1_recipe_systems_science_pack
name: Systems Science Sample
category: science
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_component_factory_control_module, amount: 1 }
  - { id: era1_logistics_logistic_controller, amount: 1 }
  - { id: era1_component_data_storage_module, amount: 2 }
outputs:
  - { id: era1_science_systems_sample, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 250 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Sample pack for systems science.
```

### era1_recipe_field_scanner — Field Scanner
```
recipe_id: era1_recipe_field_scanner
name: Field Scanner
category: science
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_sensor_array, amount: 1 }
  - { id: era1_component_navigation_module, amount: 1 }
outputs:
  - { id: era1_science_field_scanner, amount: 1 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 160 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Portable deposit/enemy scanner.
```

### era1_recipe_purity_analyzer — Purity Analyzer
```
recipe_id: era1_recipe_purity_analyzer
name: Purity Analyzer
category: science
machine: era1_machine_research_analyzer
inputs:
  - { id: era1_science_analysis_equipment, amount: 1 }
  - { id: era1_component_quality_control_unit, amount: 1 }
outputs:
  - { id: era1_science_purity_analyzer, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_material_science
description: Analyzes material purity inline.
```

### era1_recipe_grade_assessor — Grade Assessor
```
recipe_id: era1_recipe_grade_assessor
name: Grade Assessor
category: science
machine: era1_machine_research_analyzer
inputs:
  - { id: era1_science_purity_analyzer, amount: 1 }
  - { id: era1_component_calibration_chip, amount: 4 }
outputs:
  - { id: era1_science_grade_assessor, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_material_science
description: Assesses manufacturing grade.
```

### era1_recipe_lab_power_adapter — Lab Power Adapter
```
recipe_id: era1_recipe_lab_power_adapter
name: Lab Power Adapter
category: science
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_component_power_regulator, amount: 1 }
  - { id: era1_power_capacitor, amount: 2 }
outputs:
  - { id: era1_science_lab_power_adapter, amount: 2 }
waste_outputs:
  []
processing_time: 10
power_consumption: { electrical: 80 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Stabilizes lab power draw.
```

### era1_recipe_sample_container — Sample Container
```
recipe_id: era1_recipe_sample_container
name: Sample Container
category: science
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_material_glass, amount: 2 }
  - { id: era1_material_polymer_resin, amount: 2 }
outputs:
  - { id: era1_science_sample_container, amount: 4 }
waste_outputs:
  []
processing_time: 6
power_consumption: { electrical: 50 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Containers for lab samples.
```

### era1_recipe_specimen_locker — Specimen Locker
```
recipe_id: era1_recipe_specimen_locker
name: Specimen Locker
category: science
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_science_sample_container, amount: 8 }
  - { id: era1_logistics_smart_storage_unit, amount: 1 }
outputs:
  - { id: era1_science_specimen_locker, amount: 1 }
waste_outputs:
  []
processing_time: 18
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Secured specimen storage.
```

### era1_recipe_experiment_loop — Experiment Loop Module
```
recipe_id: era1_recipe_experiment_loop
name: Experiment Loop Module
category: science
machine: era1_machine_laboratory_module
inputs:
  - { id: era1_science_prototype_bench, amount: 1 }
  - { id: era1_science_tech_validation_unit, amount: 1 }
outputs:
  - { id: era1_science_experiment_loop, amount: 1 }
waste_outputs:
  []
processing_time: 50
power_consumption: { electrical: 320 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Closed-loop experiment module.
```

### era1_recipe_data_archive — Data Archive
```
recipe_id: era1_recipe_data_archive
name: Data Archive
category: science
machine: era1_machine_electronics_printer_mk2
inputs:
  - { id: era1_component_data_storage_module, amount: 10 }
  - { id: era1_component_structural_frame, amount: 2 }
outputs:
  - { id: era1_science_data_archive, amount: 1 }
waste_outputs:
  []
processing_time: 25
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Long-term research data archive.
```

### era1_recipe_remote_lab_link — Remote Lab Link
```
recipe_id: era1_recipe_remote_lab_link
name: Remote Lab Link
category: science
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_communication_module, amount: 2 }
  - { id: era1_component_interface_module, amount: 1 }
outputs:
  - { id: era1_science_remote_lab_link, amount: 1 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 150 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Links distributed labs.
```

### era1_recipe_blueprint_projector — Blueprint Projector
```
recipe_id: era1_recipe_blueprint_projector
name: Blueprint Projector
category: science
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_optical_sensor, amount: 1 }
  - { id: era1_component_logic_board, amount: 1 }
  - { id: era1_material_reinforced_glass, amount: 1 }
outputs:
  - { id: era1_science_blueprint_projector, amount: 1 }
waste_outputs:
  []
processing_time: 16
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Projects construction blueprints.
```

### era1_recipe_training_simulator — Operator Training Simulator
```
recipe_id: era1_recipe_training_simulator
name: Operator Training Simulator
category: science
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_science_simulation_core, amount: 1 }
  - { id: era1_component_interface_module, amount: 2 }
outputs:
  - { id: era1_science_training_simulator, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Trains operators / reduces errors.
```

### era1_recipe_patent_register — Patent Register
```
recipe_id: era1_recipe_patent_register
name: Patent Register
category: science
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_science_engineering_data, amount: 5 }
  - { id: era1_science_chemical_data, amount: 5 }
  - { id: era1_science_computational_data, amount: 5 }
  - { id: era1_science_defense_data, amount: 5 }
outputs:
  - { id: era1_science_patent_register, amount: 1 }
waste_outputs:
  []
processing_time: 120
power_consumption: { electrical: 400 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Compiles cross-discipline patents (prestige/late unlock).
```

### era1_recipe_field_lab — Mobile Field Lab
```
recipe_id: era1_recipe_field_lab
name: Mobile Field Lab
category: science
machine: era1_machine_assembler_mk3
inputs:
  - { id: era1_building_research_laboratory, amount: 1 }
  - { id: era1_logistics_transport_frame, amount: 2 }
  - { id: era1_power_backup_system, amount: 1 }
outputs:
  - { id: era1_science_mobile_field_lab, amount: 1 }
waste_outputs:
  []
processing_time: 60
power_consumption: { electrical: 350 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Deployable field laboratory.
```

### era1_recipe_knowledge_matrix — Knowledge Matrix
```
recipe_id: era1_recipe_knowledge_matrix
name: Knowledge Matrix
category: science
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_science_data_archive, amount: 1 }
  - { id: era1_science_simulation_core, amount: 1 }
  - { id: era1_component_industrial_ai_lite, amount: 1 }
outputs:
  - { id: era1_science_knowledge_matrix, amount: 1 }
waste_outputs:
  []
processing_time: 80
power_consumption: { electrical: 450 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Integrates research archives for Nexus prep.
```

### era1_recipe_era_transition_dossier — Era Transition Dossier
```
recipe_id: era1_recipe_era_transition_dossier
name: Era Transition Dossier
category: science
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_science_patent_register, amount: 1 }
  - { id: era1_science_knowledge_matrix, amount: 1 }
outputs:
  - { id: era1_science_era_transition_dossier, amount: 1 }
waste_outputs:
  []
processing_time: 90
power_consumption: { electrical: 500 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Documents readiness for Era 2 unlock.
```
