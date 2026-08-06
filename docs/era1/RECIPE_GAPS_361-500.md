# ERA 1 — Recipe Gap Fill C: R361–500
## Military, defense, advanced automation, Nexus

### era1_recipe_weapon_cooling — Weapon Cooling Assembly
```
recipe_id: era1_recipe_weapon_cooling
name: Weapon Cooling Assembly
category: military
machine: era1_machine_military_fabricator_mk1
inputs:
  - { id: era1_component_cooling_assembly, amount: 1 }
  - { id: era1_component_heat_exchanger_plate, amount: 2 }
outputs:
  - { id: era1_military_weapon_cooling, amount: 1 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Cooling assembly for rapid-fire turrets.
```

### era1_recipe_ammo_loader — Ammunition Loader
```
recipe_id: era1_recipe_ammo_loader
name: Ammunition Loader
category: military
machine: era1_machine_military_fabricator_mk1
inputs:
  - { id: era1_component_servo_motor, amount: 2 }
  - { id: era1_component_sensor, amount: 1 }
  - { id: era1_material_hardened_steel, amount: 4 }
outputs:
  - { id: era1_military_ammo_loader, amount: 1 }
waste_outputs:
  []
processing_time: 18
power_consumption: { electrical: 160 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Automatic ammunition loader mechanism.
```

### era1_recipe_turret_rotation_motor — Turret Rotation Motor
```
recipe_id: era1_recipe_turret_rotation_motor
name: Turret Rotation Motor
category: military
machine: era1_machine_motor_assembly_mk1
inputs:
  - { id: era1_component_industrial_motor, amount: 1 }
  - { id: era1_component_precision_bearing, amount: 2 }
  - { id: era1_component_encoder_module, amount: 1 }
outputs:
  - { id: era1_military_turret_rotation_motor, amount: 1 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 180 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Heavy duty turret traverse motor.
```

### era1_recipe_radar_component — Radar Component
```
recipe_id: era1_recipe_radar_component
name: Radar Component
category: military
machine: era1_machine_electronics_printer_mk2
inputs:
  - { id: era1_component_signal_amplifier, amount: 2 }
  - { id: era1_component_sensor_array, amount: 1 }
  - { id: era1_material_conductive_foil, amount: 5 }
outputs:
  - { id: era1_military_radar_component, amount: 2 }
waste_outputs:
  []
processing_time: 16
power_consumption: { electrical: 170 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Core radar RF/sensor package.
```

### era1_recipe_combat_sensor — Combat Sensor
```
recipe_id: era1_recipe_combat_sensor
name: Combat Sensor
category: military
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_optical_sensor, amount: 2 }
  - { id: era1_component_sensor, amount: 2 }
  - { id: era1_component_basic_circuit, amount: 2 }
outputs:
  - { id: era1_military_combat_sensor, amount: 2 }
waste_outputs:
  []
processing_time: 14
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Threat detection sensor suite.
```

### era1_recipe_armor_reinforcement — Armor Reinforcement Kit
```
recipe_id: era1_recipe_armor_reinforcement
name: Armor Reinforcement Kit
category: military
machine: era1_machine_armor_processor_mk1
inputs:
  - { id: era1_military_armor_composite, amount: 5 }
  - { id: era1_material_carbon_composite, amount: 5 }
outputs:
  - { id: era1_military_armor_reinforcement, amount: 5 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Reinforcement kits for walls/vehicles.
```

### era1_recipe_blast_housing — Blast-Resistant Housing
```
recipe_id: era1_recipe_blast_housing
name: Blast-Resistant Housing
category: military
machine: era1_machine_military_fabricator_mk1
inputs:
  - { id: era1_component_machine_housing, amount: 2 }
  - { id: era1_military_armor_plate, amount: 6 }
outputs:
  - { id: era1_military_blast_housing, amount: 1 }
waste_outputs:
  []
processing_time: 22
power_consumption: { electrical: 190 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Housing that survives nearby blasts.
```

### era1_recipe_defense_computer — Defense Computer
```
recipe_id: era1_recipe_defense_computer
name: Defense Computer
category: military
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_military_military_control_unit, amount: 2 }
  - { id: era1_component_processor_core, amount: 1 }
outputs:
  - { id: era1_military_defense_computer, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_research
description: Central defense computation unit.
```

### era1_recipe_threat_analysis — Threat Analysis Module
```
recipe_id: era1_recipe_threat_analysis
name: Threat Analysis Module
category: military
machine: era1_machine_electronics_printer_mk2
inputs:
  - { id: era1_military_combat_sensor, amount: 2 }
  - { id: era1_component_logic_board, amount: 2 }
outputs:
  - { id: era1_military_threat_analysis_module, amount: 1 }
waste_outputs:
  []
processing_time: 24
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_research
description: Analyzes and prioritizes threats.
```

### era1_recipe_emergency_repair_pack — Emergency Repair Pack
```
recipe_id: era1_recipe_emergency_repair_pack
name: Emergency Repair Pack
category: military
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_component_assembler_parts, amount: 1 }
  - { id: era1_material_ferrite_plate, amount: 10 }
  - { id: era1_component_basic_circuit, amount: 2 }
outputs:
  - { id: era1_military_emergency_repair_pack, amount: 2 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 120 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Consumable emergency structure repairs.
```

### era1_recipe_ap_rounds — Armor Piercing Rounds
```
recipe_id: era1_recipe_ap_rounds
name: Armor Piercing Rounds
category: military
machine: era1_machine_ammunition_factory_mk1
inputs:
  - { id: era1_military_projectile_core, amount: 20 }
  - { id: era1_military_ammo_casing, amount: 20 }
  - { id: era1_fluid_ballistic_propellant, amount: 10 }
  - { id: era1_material_hardened_steel, amount: 5 }
outputs:
  - { id: era1_military_ap_ammunition, amount: 40 }
waste_outputs:
  []
processing_time: 25
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: AP ammunition for heavy ballistic turrets.
```

### era1_recipe_incendiary_rounds — Incendiary Rounds
```
recipe_id: era1_recipe_incendiary_rounds
name: Incendiary Rounds
category: military
machine: era1_machine_ammunition_factory_mk1
inputs:
  - { id: era1_military_standard_ammunition, amount: 20 }
  - { id: era1_fluid_fuel_oil, amount: 5 }
outputs:
  - { id: era1_military_incendiary_ammunition, amount: 20 }
waste_outputs:
  []
processing_time: 18
power_consumption: { electrical: 160 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Incendiary ammunition variant.
```

### era1_recipe_turret_ammo_box — Turret Ammo Box
```
recipe_id: era1_recipe_turret_ammo_box
name: Turret Ammo Box
category: military
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_military_standard_ammunition, amount: 100 }
  - { id: era1_material_ferrite_plate, amount: 4 }
outputs:
  - { id: era1_military_ammo_box, amount: 1 }
waste_outputs:
  []
processing_time: 10
power_consumption: { electrical: 80 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Boxed ammo for logistics to turrets.
```

### era1_recipe_smoke_canister — Smoke Canister
```
recipe_id: era1_recipe_smoke_canister
name: Smoke Canister
category: military
machine: era1_machine_chemical_reactor_mk1
inputs:
  - { id: era1_material_carbon_powder, amount: 5 }
  - { id: era1_fluid_chemical_additive, amount: 2 }
outputs:
  - { id: era1_military_smoke_canister, amount: 5 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 100 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Non-lethal smoke for retreat/cover.
```

### era1_recipe_flare — Signal Flare
```
recipe_id: era1_recipe_flare
name: Signal Flare
category: military
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_material_carbon_powder, amount: 2 }
  - { id: era1_fluid_oxidizer, amount: 1 }
outputs:
  - { id: era1_military_signal_flare, amount: 4 }
waste_outputs:
  []
processing_time: 6
power_consumption: { electrical: 50 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Signal flares for alerts.
```

### era1_recipe_mine_casing — Proximity Mine Casing
```
recipe_id: era1_recipe_mine_casing
name: Proximity Mine Casing
category: military
machine: era1_machine_military_fabricator_mk1
inputs:
  - { id: era1_material_hardened_steel, amount: 4 }
  - { id: era1_component_sensor, amount: 1 }
outputs:
  - { id: era1_military_mine_casing, amount: 2 }
waste_outputs:
  []
processing_time: 14
power_consumption: { electrical: 130 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Casings for defensive mines.
```

### era1_recipe_proximity_mine — Proximity Mine
```
recipe_id: era1_recipe_proximity_mine
name: Proximity Mine
category: military
machine: era1_machine_ammunition_factory_mk1
inputs:
  - { id: era1_military_mine_casing, amount: 2 }
  - { id: era1_fluid_ballistic_propellant, amount: 4 }
  - { id: era1_component_basic_circuit, amount: 1 }
outputs:
  - { id: era1_military_proximity_mine, amount: 2 }
waste_outputs:
  []
processing_time: 16
power_consumption: { electrical: 150 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Area denial proximity mine.
```

### era1_recipe_bunker_kit — Bunker Kit
```
recipe_id: era1_recipe_bunker_kit
name: Bunker Kit
category: military
machine: era1_machine_military_fabricator_mk1
inputs:
  - { id: era1_military_reinforced_wall, amount: 4 }
  - { id: era1_military_armor_composite, amount: 5 }
outputs:
  - { id: era1_military_bunker_kit, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Prefabricated bunker kit.
```

### era1_recipe_targeting_computer — Advanced Targeting Computer
```
recipe_id: era1_recipe_targeting_computer
name: Advanced Targeting Computer
category: military
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_military_targeting_module, amount: 2 }
  - { id: era1_military_threat_analysis_module, amount: 1 }
outputs:
  - { id: era1_military_targeting_computer, amount: 1 }
waste_outputs:
  []
processing_time: 28
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_research
description: Advanced fire-control computer.
```

### era1_recipe_repair_drone_core — Repair Drone Core
```
recipe_id: era1_recipe_repair_drone_core
name: Repair Drone Core
category: military
machine: era1_machine_robotics_factory_mk1
inputs:
  - { id: era1_component_industrial_robot_core, amount: 1 }
  - { id: era1_military_emergency_repair_pack, amount: 2 }
outputs:
  - { id: era1_military_repair_core, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 260 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Core for automated repair drones.
```

### era1_recipe_repair_drone_station — Repair Drone Station
```
recipe_id: era1_recipe_repair_drone_station
name: Repair Drone Station
category: defense
machine: era1_machine_defense_assembly_machine
inputs:
  - { id: era1_military_repair_core, amount: 1 }
  - { id: era1_logistics_drone_pad, amount: 1 }
  - { id: era1_power_storage_module, amount: 1 }
outputs:
  - { id: era1_military_repair_drone_station, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Station that deploys repair drones.
```

### era1_recipe_repair_tower — Automated Repair Tower
```
recipe_id: era1_recipe_repair_tower
name: Automated Repair Tower
category: defense
machine: era1_machine_defense_assembly_machine
inputs:
  - { id: era1_military_repair_drone_station, amount: 1 }
  - { id: era1_component_structural_frame, amount: 5 }
outputs:
  - { id: era1_military_repair_tower, amount: 1 }
waste_outputs:
  []
processing_time: 45
power_consumption: { electrical: 320 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Area repair tower.
```

### era1_recipe_ammo_logistics_hub — Ammo Logistics Hub
```
recipe_id: era1_recipe_ammo_logistics_hub
name: Ammo Logistics Hub
category: defense
machine: era1_machine_military_assembly_bay_mk1
inputs:
  - { id: era1_logistics_warehouse_module, amount: 1 }
  - { id: era1_military_ammo_loader, amount: 2 }
  - { id: era1_logistics_logistic_controller, amount: 1 }
outputs:
  - { id: era1_military_ammo_hub, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Hub distributing ammo to turrets.
```

### era1_recipe_defense_power_node — Defense Power Node
```
recipe_id: era1_recipe_defense_power_node
name: Defense Power Node
category: defense
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_power_switchgear, amount: 1 }
  - { id: era1_power_backup_system, amount: 1 }
outputs:
  - { id: era1_military_defense_power_node, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Hardened power node for defense grids.
```

### era1_recipe_radar_upgrade — Radar Upgrade Kit
```
recipe_id: era1_recipe_radar_upgrade
name: Radar Upgrade Kit
category: defense
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_military_radar_component, amount: 4 }
  - { id: era1_component_signal_amplifier, amount: 2 }
outputs:
  - { id: era1_military_radar_upgrade, amount: 1 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 180 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_research
description: Upgrades radar range/resolution.
```

### era1_recipe_shield_prototype — Shield Projector Prototype
```
recipe_id: era1_recipe_shield_prototype
name: Shield Projector Prototype
category: defense
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_component_capacitor_bank, amount: 2 }
  - { id: era1_power_energy_cell, amount: 20 }
  - { id: era1_component_high_density_circuit, amount: 2 }
outputs:
  - { id: era1_military_shield_prototype, amount: 1 }
waste_outputs:
  []
processing_time: 60
power_consumption: { electrical: 450 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_research
description: Non-combat shield projector prototype (research/defense utility).
```

### era1_recipe_emergency_barrier — Emergency Barrier
```
recipe_id: era1_recipe_emergency_barrier
name: Emergency Barrier
category: defense
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_military_defensive_wall_segment, amount: 2 }
  - { id: era1_component_pneumatic_system, amount: 1 }
outputs:
  - { id: era1_military_emergency_barrier, amount: 2 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 100 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Rapid-deploy emergency barrier.
```

### era1_recipe_auto_defense_controller — Automated Defense Controller
```
recipe_id: era1_recipe_auto_defense_controller
name: Automated Defense Controller
category: defense
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_military_defense_computer, amount: 1 }
  - { id: era1_military_defense_control_network, amount: 1 }
outputs:
  - { id: era1_military_auto_defense_controller, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 350 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_research
description: Coordinates all local defenses.
```

### era1_recipe_watchtower — Watchtower
```
recipe_id: era1_recipe_watchtower
name: Watchtower
category: defense
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_component_structural_frame, amount: 6 }
  - { id: era1_military_combat_sensor, amount: 1 }
outputs:
  - { id: era1_military_watchtower, amount: 1 }
waste_outputs:
  []
processing_time: 25
power_consumption: { electrical: 160 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Elevated sensor watchtower.
```

### era1_recipe_gate — Defensive Gate
```
recipe_id: era1_recipe_gate
name: Defensive Gate
category: defense
machine: era1_machine_military_fabricator_mk1
inputs:
  - { id: era1_military_reinforced_wall, amount: 2 }
  - { id: era1_component_actuator, amount: 2 }
outputs:
  - { id: era1_military_gate, amount: 1 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 150 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Openable defensive gate.
```

### era1_recipe_floodlight — Defense Floodlight
```
recipe_id: era1_recipe_floodlight
name: Defense Floodlight
category: defense
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_material_glass, amount: 2 }
  - { id: era1_power_energy_cell, amount: 2 }
  - { id: era1_component_basic_circuit, amount: 1 }
outputs:
  - { id: era1_military_floodlight, amount: 2 }
waste_outputs:
  []
processing_time: 8
power_consumption: { electrical: 70 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Illumination for night defense.
```

### era1_recipe_siren — Defense Siren
```
recipe_id: era1_recipe_siren
name: Defense Siren
category: defense
machine: era1_machine_assembler_mk1
inputs:
  - { id: era1_logistics_speaker, amount: 1 }
  - { id: era1_component_basic_circuit, amount: 1 }
outputs:
  - { id: era1_military_siren, amount: 1 }
waste_outputs:
  []
processing_time: 6
power_consumption: { electrical: 50 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Base alert siren.
```

### era1_recipe_bunker_turret_mount — Bunker Turret Mount
```
recipe_id: era1_recipe_bunker_turret_mount
name: Bunker Turret Mount
category: defense
machine: era1_machine_military_fabricator_mk1
inputs:
  - { id: era1_military_bunker_kit, amount: 1 }
  - { id: era1_military_ballistic_turret_frame, amount: 1 }
outputs:
  - { id: era1_military_bunker_turret_mount, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Hardpoint mount combining bunker and turret frame.
```

### era1_recipe_minefield_kit — Minefield Kit
```
recipe_id: era1_recipe_minefield_kit
name: Minefield Kit
category: defense
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_military_proximity_mine, amount: 10 }
  - { id: era1_component_sensor, amount: 2 }
outputs:
  - { id: era1_military_minefield_kit, amount: 1 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Deployable minefield package.
```

### era1_recipe_artillery_shell — Artillery Shell (Era1)
```
recipe_id: era1_recipe_artillery_shell
name: Artillery Shell (Era1)
category: defense
machine: era1_machine_ammunition_factory_mk1
inputs:
  - { id: era1_military_projectile_core, amount: 5 }
  - { id: era1_fluid_ballistic_propellant, amount: 10 }
  - { id: era1_material_hardened_steel, amount: 5 }
outputs:
  - { id: era1_military_artillery_shell, amount: 2 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Heavy shell for late Era 1 artillery.
```

### era1_recipe_artillery_frame — Artillery Frame
```
recipe_id: era1_recipe_artillery_frame
name: Artillery Frame
category: defense
machine: era1_machine_military_fabricator_mk1
inputs:
  - { id: era1_component_heavy_structural_frame, amount: 2 }
  - { id: era1_military_weapon_housing, amount: 2 }
outputs:
  - { id: era1_military_artillery_frame, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Frame for static artillery.
```

### era1_recipe_artillery_turret — Artillery Turret
```
recipe_id: era1_recipe_artillery_turret
name: Artillery Turret
category: defense
machine: era1_machine_military_assembly_bay_mk1
inputs:
  - { id: era1_military_artillery_frame, amount: 1 }
  - { id: era1_military_targeting_computer, amount: 1 }
  - { id: era1_military_artillery_shell, amount: 4 }
outputs:
  - { id: era1_military_artillery_turret, amount: 1 }
waste_outputs:
  []
processing_time: 50
power_consumption: { electrical: 350 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_research
description: Long-range ballistic artillery.
```

### era1_recipe_laser_lens — Laser Lens
```
recipe_id: era1_recipe_laser_lens
name: Laser Lens
category: defense
machine: era1_machine_precision_fabricator_mk1
inputs:
  - { id: era1_material_optical_silicon, amount: 2 }
  - { id: era1_material_reinforced_glass, amount: 2 }
outputs:
  - { id: era1_military_laser_lens, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_laser_defense
description: Optical lens requiring high purity silicon.
```

### era1_recipe_optical_silicon — Optical Silicon
```
recipe_id: era1_recipe_optical_silicon
name: Optical Silicon
category: defense
machine: era1_machine_precision_water_processor_mk1
inputs:
  - { id: era1_material_refined_silicon, amount: 10 }
  - { id: era1_fluid_ultra_pure_water, amount: 10 }
outputs:
  - { id: era1_material_optical_silicon, amount: 5 }
waste_outputs:
  []
processing_time: 45
power_consumption: { electrical: 400 }
purity_effect: 20
grade_effect: precision
technology_unlock: era1_tech_laser_defense
description: Ultra-pure optical silicon (min purity 98% enforced in engine).
```

### era1_recipe_laser_capacitor — Laser Capacitor Bank
```
recipe_id: era1_recipe_laser_capacitor
name: Laser Capacitor Bank
category: defense
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_power_capacitor_bank, amount: 2 }
  - { id: era1_power_energy_cell, amount: 10 }
outputs:
  - { id: era1_military_laser_capacitor, amount: 1 }
waste_outputs:
  []
processing_time: 25
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_laser_defense
description: Energy bank for laser turrets.
```

### era1_recipe_laser_emitter — Laser Emitter
```
recipe_id: era1_recipe_laser_emitter
name: Laser Emitter
category: defense
machine: era1_machine_precision_fabricator_mk1
inputs:
  - { id: era1_military_laser_lens, amount: 1 }
  - { id: era1_material_conductive_foil, amount: 10 }
  - { id: era1_component_heat_exchanger_plate, amount: 2 }
outputs:
  - { id: era1_military_laser_emitter, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_laser_defense
description: Laser emitter assembly.
```

### era1_recipe_laser_turret — Laser Turret
```
recipe_id: era1_recipe_laser_turret
name: Laser Turret
category: defense
machine: era1_machine_military_assembly_bay_mk1
inputs:
  - { id: era1_military_weapon_housing, amount: 1 }
  - { id: era1_military_laser_emitter, amount: 1 }
  - { id: era1_military_laser_capacitor, amount: 1 }
  - { id: era1_military_targeting_module, amount: 1 }
outputs:
  - { id: era1_military_laser_turret, amount: 1 }
waste_outputs:
  []
processing_time: 55
power_consumption: { electrical: 400 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_laser_defense
description: First true energy weapon of Era 1.
```

### era1_recipe_ammo_belt — Ammo Belt Feed
```
recipe_id: era1_recipe_ammo_belt
name: Ammo Belt Feed
category: defense
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_fast_conveyor_segment, amount: 4 }
  - { id: era1_military_ammo_loader, amount: 1 }
outputs:
  - { id: era1_military_ammo_belt, amount: 1 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 130 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Dedicated ammo belt feeder.
```

### era1_recipe_defense_planner — Defense Planner Console
```
recipe_id: era1_recipe_defense_planner
name: Defense Planner Console
category: defense
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_military_auto_defense_controller, amount: 1 }
  - { id: era1_component_interface_module, amount: 2 }
outputs:
  - { id: era1_military_defense_planner, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_research
description: Planning console for defense layouts.
```

### era1_recipe_fortress_core — Fortress Core
```
recipe_id: era1_recipe_fortress_core
name: Fortress Core
category: defense
machine: era1_machine_military_assembly_bay_mk1
inputs:
  - { id: era1_military_bunker_kit, amount: 2 }
  - { id: era1_military_defense_power_node, amount: 1 }
  - { id: era1_military_auto_defense_controller, amount: 1 }
outputs:
  - { id: era1_military_fortress_core, amount: 1 }
waste_outputs:
  []
processing_time: 70
power_consumption: { electrical: 450 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_research
description: Core module for fortified outposts.
```

### era1_recipe_drone_rearm_pad — Drone Rearm Pad
```
recipe_id: era1_recipe_drone_rearm_pad
name: Drone Rearm Pad
category: defense
machine: era1_machine_defense_assembly_machine
inputs:
  - { id: era1_logistics_drone_pad, amount: 1 }
  - { id: era1_military_ammo_hub, amount: 1 }
outputs:
  - { id: era1_military_drone_rearm_pad, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Rearms combat/repair drones.
```

### era1_recipe_sensor_fence — Sensor Fence
```
recipe_id: era1_recipe_sensor_fence
name: Sensor Fence
category: defense
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_military_combat_sensor, amount: 4 }
  - { id: era1_logistics_red_wire, amount: 20 }
outputs:
  - { id: era1_military_sensor_fence, amount: 4 }
waste_outputs:
  []
processing_time: 16
power_consumption: { electrical: 120 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Perimeter sensor fence segments.
```

### era1_recipe_ciws_turret — Point Defense Turret
```
recipe_id: era1_recipe_ciws_turret
name: Point Defense Turret
category: defense
machine: era1_machine_military_assembly_bay_mk1
inputs:
  - { id: era1_military_ballistic_turret, amount: 1 }
  - { id: era1_military_targeting_computer, amount: 1 }
  - { id: era1_military_weapon_cooling, amount: 1 }
outputs:
  - { id: era1_military_ciws_turret, amount: 1 }
waste_outputs:
  []
processing_time: 45
power_consumption: { electrical: 340 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_research
description: High-speed point defense turret.
```

### era1_recipe_missile_ammo_crate — Missile Ammo Crate
```
recipe_id: era1_recipe_missile_ammo_crate
name: Missile Ammo Crate
category: defense
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_military_guided_missile, amount: 5 }
  - { id: era1_material_structural_panel, amount: 2 }
outputs:
  - { id: era1_military_missile_crate, amount: 1 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 120 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_industry
description: Crate of guided missiles for logistics.
```

### era1_recipe_defense_manual — Defense Doctrine Manual
```
recipe_id: era1_recipe_defense_manual
name: Defense Doctrine Manual
category: defense
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_science_defense_data, amount: 10 }
  - { id: era1_military_defense_planner, amount: 1 }
outputs:
  - { id: era1_military_defense_doctrine, amount: 1 }
waste_outputs:
  []
processing_time: 90
power_consumption: { electrical: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_defense_research
description: Compiled defense doctrine (unlocks late wall/turret bonuses via research).
```

### era1_recipe_production_monitor_screen — Production Monitor Screen
```
recipe_id: era1_recipe_production_monitor_screen
name: Production Monitor Screen
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_production_monitoring_system, amount: 1 }
  - { id: era1_component_interface_module, amount: 1 }
outputs:
  - { id: era1_component_monitor_screen, amount: 1 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 110 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Display for production stats.
```

### era1_recipe_purity_probe — Inline Purity Probe
```
recipe_id: era1_recipe_purity_probe
name: Inline Purity Probe
category: automation
machine: era1_machine_research_analyzer
inputs:
  - { id: era1_science_purity_analyzer, amount: 1 }
  - { id: era1_component_sensor, amount: 2 }
outputs:
  - { id: era1_component_purity_probe, amount: 2 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 180 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_material_science
description: Inline purity probe for belts/pipes.
```

### era1_recipe_auto_balancer — Automated Balancing System
```
recipe_id: era1_recipe_auto_balancer
name: Automated Balancing System
category: automation
machine: era1_machine_assembler_mk3
inputs:
  - { id: era1_logistics_belt_balancer, amount: 2 }
  - { id: era1_component_factory_control_module, amount: 1 }
outputs:
  - { id: era1_component_auto_balancer, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Self-adjusting belt balancer.
```

### era1_recipe_smart_factory_node — Smart Factory Node
```
recipe_id: era1_recipe_smart_factory_node
name: Smart Factory Node
category: automation
machine: era1_machine_assembler_mk3
inputs:
  - { id: era1_component_factory_control_module, amount: 1 }
  - { id: era1_component_production_monitoring_system, amount: 1 }
  - { id: era1_logistics_logistic_controller, amount: 1 }
outputs:
  - { id: era1_component_smart_factory_node, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 320 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Node coordinating a factory district.
```

### era1_recipe_machine_upgrade_kit_speed — Machine Upgrade Kit (Speed)
```
recipe_id: era1_recipe_machine_upgrade_kit_speed
name: Machine Upgrade Kit (Speed)
category: automation
machine: era1_machine_research_fabricator
inputs:
  - { id: era1_component_optimization_module, amount: 1 }
  - { id: era1_component_industrial_motor, amount: 1 }
outputs:
  - { id: era1_upgrade_speed_module_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 25
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Speed upgrade module.
```

### era1_recipe_machine_upgrade_kit_efficiency — Machine Upgrade Kit (Efficiency)
```
recipe_id: era1_recipe_machine_upgrade_kit_efficiency
name: Machine Upgrade Kit (Efficiency)
category: automation
machine: era1_machine_research_fabricator
inputs:
  - { id: era1_component_optimization_module, amount: 1 }
  - { id: era1_component_power_regulator, amount: 1 }
outputs:
  - { id: era1_upgrade_efficiency_module_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 25
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Efficiency upgrade module.
```

### era1_recipe_machine_upgrade_kit_purity — Machine Upgrade Kit (Purity)
```
recipe_id: era1_recipe_machine_upgrade_kit_purity
name: Machine Upgrade Kit (Purity)
category: automation
machine: era1_machine_research_fabricator
inputs:
  - { id: era1_component_optimization_module, amount: 1 }
  - { id: era1_component_quality_control_unit, amount: 1 }
outputs:
  - { id: era1_upgrade_purity_module_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_material_science
description: Purity bonus upgrade module.
```

### era1_recipe_machine_upgrade_kit_waste — Machine Upgrade Kit (Waste Reduction)
```
recipe_id: era1_recipe_machine_upgrade_kit_waste
name: Machine Upgrade Kit (Waste Reduction)
category: automation
machine: era1_machine_research_fabricator
inputs:
  - { id: era1_component_optimization_module, amount: 1 }
  - { id: era1_fluid_waste_neutralizer, amount: 5 }
outputs:
  - { id: era1_upgrade_waste_module_mk1, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 220 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_waste_recovery
description: Waste reduction upgrade module.
```

### era1_recipe_ai_scheduler — Industrial AI Scheduler
```
recipe_id: era1_recipe_ai_scheduler
name: Industrial AI Scheduler
category: automation
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_component_industrial_ai_lite, amount: 1 }
  - { id: era1_component_smart_factory_node, amount: 1 }
outputs:
  - { id: era1_component_ai_scheduler, amount: 1 }
waste_outputs:
  []
processing_time: 45
power_consumption: { electrical: 360 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Schedules production across districts.
```

### era1_recipe_predictive_maint — Predictive Maintenance Unit
```
recipe_id: era1_recipe_predictive_maint
name: Predictive Maintenance Unit
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_uptime_monitor, amount: 1 }
  - { id: era1_component_diagnostic_module, amount: 1 }
outputs:
  - { id: era1_component_predictive_maintenance, amount: 1 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 180 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Predicts machine failures.
```

### era1_recipe_recipe_lock_module — Recipe Lock Module
```
recipe_id: era1_recipe_recipe_lock_module
name: Recipe Lock Module
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_firmware_module, amount: 1 }
  - { id: era1_component_basic_circuit, amount: 2 }
outputs:
  - { id: era1_component_recipe_lock, amount: 2 }
waste_outputs:
  []
processing_time: 10
power_consumption: { electrical: 90 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Locks a machine to one recipe.
```

### era1_recipe_priority_controller — Priority Controller
```
recipe_id: era1_recipe_priority_controller
name: Priority Controller
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_machine_controller, amount: 1 }
  - { id: era1_logistics_priority_splitter, amount: 1 }
outputs:
  - { id: era1_component_priority_controller, amount: 1 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 140 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Prioritizes machine power/inputs.
```

### era1_recipe_clock_sync — Factory Clock Sync
```
recipe_id: era1_recipe_clock_sync
name: Factory Clock Sync
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_component_clock_crystal, amount: 4 }
  - { id: era1_component_control_bus, amount: 2 }
outputs:
  - { id: era1_component_clock_sync, amount: 1 }
waste_outputs:
  []
processing_time: 12
power_consumption: { electrical: 120 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Synchronizes machine timing.
```

### era1_recipe_overflow_manager — Overflow Manager
```
recipe_id: era1_recipe_overflow_manager
name: Overflow Manager
category: automation
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_logistics_overflow_gate, amount: 2 }
  - { id: era1_component_control_module, amount: 1 }
outputs:
  - { id: era1_component_overflow_manager, amount: 1 }
waste_outputs:
  []
processing_time: 14
power_consumption: { electrical: 130 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Manages belt/pipe overflow intelligently.
```

### era1_recipe_batch_controller — Batch Controller
```
recipe_id: era1_recipe_batch_controller
name: Batch Controller
category: automation
machine: era1_machine_electronics_printer_mk2
inputs:
  - { id: era1_component_machine_controller, amount: 1 }
  - { id: era1_component_data_storage_module, amount: 2 }
outputs:
  - { id: era1_component_batch_controller, amount: 1 }
waste_outputs:
  []
processing_time: 18
power_consumption: { electrical: 160 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Controls batch chemical processes.
```

### era1_recipe_grade_sorter — Grade Sorter
```
recipe_id: era1_recipe_grade_sorter
name: Grade Sorter
category: automation
machine: era1_machine_assembler_mk3
inputs:
  - { id: era1_logistics_automated_sorter, amount: 1 }
  - { id: era1_science_grade_assessor, amount: 1 }
outputs:
  - { id: era1_component_grade_sorter, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_material_science
description: Sorts items by manufacturing grade.
```

### era1_recipe_purity_sorter — Purity Sorter
```
recipe_id: era1_recipe_purity_sorter
name: Purity Sorter
category: automation
machine: era1_machine_assembler_mk3
inputs:
  - { id: era1_logistics_automated_sorter, amount: 1 }
  - { id: era1_component_purity_probe, amount: 2 }
outputs:
  - { id: era1_component_purity_sorter, amount: 1 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 240 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_material_science
description: Sorts materials by purity band.
```

### era1_recipe_auto_researcher — Auto Research Assistant
```
recipe_id: era1_recipe_auto_researcher
name: Auto Research Assistant
category: automation
machine: era1_machine_laboratory_module
inputs:
  - { id: era1_component_ai_scheduler, amount: 1 }
  - { id: era1_science_data_processor_rack, amount: 1 }
outputs:
  - { id: era1_component_auto_researcher, amount: 1 }
waste_outputs:
  []
processing_time: 50
power_consumption: { electrical: 380 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_research_infrastructure
description: Assists lab queue management.
```

### era1_recipe_remote_terminal — Remote Factory Terminal
```
recipe_id: era1_recipe_remote_terminal
name: Remote Factory Terminal
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_science_remote_lab_link, amount: 1 }
  - { id: era1_component_interface_module, amount: 1 }
outputs:
  - { id: era1_component_remote_terminal, amount: 1 }
waste_outputs:
  []
processing_time: 16
power_consumption: { electrical: 150 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Remote terminal for factory control.
```

### era1_recipe_digital_twin — Digital Twin Module
```
recipe_id: era1_recipe_digital_twin
name: Digital Twin Module
category: automation
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_science_simulation_core, amount: 1 }
  - { id: era1_component_smart_factory_node, amount: 1 }
outputs:
  - { id: era1_component_digital_twin, amount: 1 }
waste_outputs:
  []
processing_time: 55
power_consumption: { electrical: 400 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Maintains a live digital twin of a district.
```

### era1_recipe_alert_matrix — Alert Matrix
```
recipe_id: era1_recipe_alert_matrix
name: Alert Matrix
category: automation
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_military_siren, amount: 1 }
  - { id: era1_component_monitor_screen, amount: 1 }
  - { id: era1_component_sensor_array, amount: 1 }
outputs:
  - { id: era1_component_alert_matrix, amount: 1 }
waste_outputs:
  []
processing_time: 14
power_consumption: { electrical: 130 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Centralized alert matrix.
```

### era1_recipe_spare_parts_kit — Spare Parts Kit
```
recipe_id: era1_recipe_spare_parts_kit
name: Spare Parts Kit
category: automation
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_component_assembler_parts, amount: 1 }
  - { id: era1_component_electronics_printer_parts, amount: 1 }
  - { id: era1_component_bearing, amount: 4 }
outputs:
  - { id: era1_component_spare_parts_kit, amount: 2 }
waste_outputs:
  []
processing_time: 20
power_consumption: { electrical: 150 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_advanced_automation
description: Generic spare parts for maintenance.
```

### era1_recipe_tooling_magazine — Tooling Magazine
```
recipe_id: era1_recipe_tooling_magazine
name: Tooling Magazine
category: automation
machine: era1_machine_assembler_mk2
inputs:
  - { id: era1_component_tool_changer, amount: 1 }
  - { id: era1_material_hardened_steel, amount: 5 }
outputs:
  - { id: era1_component_tooling_magazine, amount: 1 }
waste_outputs:
  []
processing_time: 15
power_consumption: { electrical: 130 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_precision_manufacturing
description: Stores tools for fabricators.
```

### era1_recipe_calib_station — Calibration Station
```
recipe_id: era1_recipe_calib_station
name: Calibration Station
category: automation
machine: era1_machine_precision_fabricator_mk1
inputs:
  - { id: era1_component_machine_calibration_unit, amount: 1 }
  - { id: era1_component_linear_rail, amount: 2 }
outputs:
  - { id: era1_machine_calibration_station, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 260 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_precision_manufacturing
description: Station that recalibrates modules.
```

### era1_recipe_factory_os — Factory OS Image
```
recipe_id: era1_recipe_factory_os
name: Factory OS Image
category: automation
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_component_firmware_module, amount: 4 }
  - { id: era1_science_systems_sample, amount: 1 }
outputs:
  - { id: era1_component_factory_os, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 320 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: OS image for smart factory nodes.
```

### era1_recipe_district_linker — District Linker
```
recipe_id: era1_recipe_district_linker
name: District Linker
category: automation
machine: era1_machine_assembler_mk3
inputs:
  - { id: era1_component_smart_factory_node, amount: 2 }
  - { id: era1_component_bus_coupler, amount: 4 }
outputs:
  - { id: era1_component_district_linker, amount: 1 }
waste_outputs:
  []
processing_time: 35
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_systems_science
description: Links multiple factory districts.
```

### era1_recipe_nexus_expansion_bay — Nexus Expansion Bay
```
recipe_id: era1_recipe_nexus_expansion_bay
name: Nexus Expansion Bay
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_component_heavy_structural_frame, amount: 20 }
  - { id: era1_material_steel_composite, amount: 40 }
outputs:
  - { id: era1_nexus_expansion_bay, amount: 1 }
waste_outputs:
  []
processing_time: 120
power_consumption: { electrical: 800 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Expansion bay for Nexus throughput.
```

### era1_recipe_nexus_data_array — Nexus Data Array
```
recipe_id: era1_recipe_nexus_data_array
name: Nexus Data Array
category: nexus
machine: era1_machine_advanced_electronics_printer
inputs:
  - { id: era1_science_data_archive, amount: 4 }
  - { id: era1_science_knowledge_matrix, amount: 1 }
outputs:
  - { id: era1_nexus_data_array, amount: 1 }
waste_outputs:
  []
processing_time: 100
power_consumption: { electrical: 700 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Planetary data array for Nexus.
```

### era1_recipe_nexus_control_array — Nexus Industrial Control Array
```
recipe_id: era1_recipe_nexus_control_array
name: Nexus Industrial Control Array
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_component_district_linker, amount: 4 }
  - { id: era1_component_ai_scheduler, amount: 2 }
outputs:
  - { id: era1_nexus_control_array, amount: 1 }
waste_outputs:
  []
processing_time: 110
power_consumption: { electrical: 750 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Control array coordinating Nexus production.
```

### era1_recipe_nexus_backup_power — Nexus Backup Power System
```
recipe_id: era1_recipe_nexus_backup_power
name: Nexus Backup Power System
category: nexus
machine: era1_machine_energy_facility
inputs:
  - { id: era1_power_backup_system, amount: 4 }
  - { id: era1_nexus_power_core, amount: 1 }
outputs:
  - { id: era1_nexus_backup_power, amount: 1 }
waste_outputs:
  []
processing_time: 90
power_consumption: { electrical: 600 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Redundant Nexus power system.
```

### era1_recipe_nexus_mfg_upgrade — Nexus Manufacturing Upgrade
```
recipe_id: era1_recipe_nexus_mfg_upgrade
name: Nexus Manufacturing Upgrade
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_nexus_manufacturing_module, amount: 1 }
  - { id: era1_upgrade_speed_module_mk1, amount: 10 }
  - { id: era1_upgrade_purity_module_mk1, amount: 10 }
outputs:
  - { id: era1_nexus_mfg_upgrade, amount: 1 }
waste_outputs:
  []
processing_time: 100
power_consumption: { electrical: 700 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Upgrades Nexus manufacturing throughput/quality.
```

### era1_recipe_era_transition_beacon — Era Transition Beacon
```
recipe_id: era1_recipe_era_transition_beacon
name: Era Transition Beacon
category: nexus
machine: era1_machine_construction_site
inputs:
  - { id: era1_nexus_planetary_fabrication_nexus, amount: 1 }
  - { id: era1_science_era_transition_dossier, amount: 1 }
  - { id: era1_nexus_data_array, amount: 1 }
outputs:
  - { id: era1_nexus_era_transition_beacon, amount: 1 }
waste_outputs:
  []
processing_time: 180
power_consumption: { electrical: 1000 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Beacon that finalizes Era 1 → Era 2 transition.
```

### era1_recipe_nexus_cooling_plant — Nexus Cooling Plant
```
recipe_id: era1_recipe_nexus_cooling_plant
name: Nexus Cooling Plant
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_component_power_cooling_loop, amount: 10 }
  - { id: era1_fluid_thermal_coolant, amount: 200 }
outputs:
  - { id: era1_nexus_cooling_plant, amount: 1 }
waste_outputs:
  []
processing_time: 90
power_consumption: { electrical: 650 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Massive cooling plant for Nexus.
```

### era1_recipe_nexus_resource_silo — Nexus Resource Silo
```
recipe_id: era1_recipe_nexus_resource_silo
name: Nexus Resource Silo
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_logistics_warehouse_module, amount: 4 }
  - { id: era1_nexus_resource_interface, amount: 1 }
outputs:
  - { id: era1_nexus_resource_silo, amount: 1 }
waste_outputs:
  []
processing_time: 80
power_consumption: { electrical: 600 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Bulk resource silo attached to Nexus.
```

### era1_recipe_nexus_security_grid — Nexus Security Grid
```
recipe_id: era1_recipe_nexus_security_grid
name: Nexus Security Grid
category: nexus
machine: era1_machine_military_assembly_bay_mk1
inputs:
  - { id: era1_military_fortress_core, amount: 2 }
  - { id: era1_military_auto_defense_controller, amount: 2 }
outputs:
  - { id: era1_nexus_security_grid, amount: 1 }
waste_outputs:
  []
processing_time: 100
power_consumption: { electrical: 700 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Defense grid protecting the Nexus.
```

### era1_recipe_nexus_lab_wing — Nexus Laboratory Wing
```
recipe_id: era1_recipe_nexus_lab_wing
name: Nexus Laboratory Wing
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_machine_laboratory_module, amount: 4 }
  - { id: era1_science_experiment_loop, amount: 2 }
outputs:
  - { id: era1_nexus_lab_wing, amount: 1 }
waste_outputs:
  []
processing_time: 100
power_consumption: { electrical: 700 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Research wing integrated into Nexus.
```

### era1_recipe_nexus_conduit — Nexus Power Conduit
```
recipe_id: era1_recipe_nexus_conduit
name: Nexus Power Conduit
category: nexus
machine: era1_machine_power_component_factory_mk1
inputs:
  - { id: era1_power_high_voltage_cable, amount: 50 }
  - { id: era1_power_advanced_transformer_parts, amount: 5 }
outputs:
  - { id: era1_nexus_conduit, amount: 5 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Heavy power conduits for Nexus.
```

### era1_recipe_nexus_data_cable — Nexus Data Cable
```
recipe_id: era1_recipe_nexus_data_cable
name: Nexus Data Cable
category: nexus
machine: era1_machine_electronics_assembler_mk1
inputs:
  - { id: era1_material_conductive_wire, amount: 40 }
  - { id: era1_material_optical_silicon, amount: 2 }
  - { id: era1_material_insulation, amount: 10 }
outputs:
  - { id: era1_nexus_data_cable, amount: 10 }
waste_outputs:
  []
processing_time: 25
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: High-bandwidth data cabling.
```

### era1_recipe_nexus_anchor — Nexus Structural Anchor
```
recipe_id: era1_recipe_nexus_anchor
name: Nexus Structural Anchor
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_material_reinforced_metal_block, amount: 20 }
  - { id: era1_component_heavy_structural_frame, amount: 10 }
outputs:
  - { id: era1_nexus_anchor, amount: 4 }
waste_outputs:
  []
processing_time: 50
power_consumption: { electrical: 400 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Anchors Nexus foundation to bedrock.
```

### era1_recipe_nexus_airlock — Nexus Airlock
```
recipe_id: era1_recipe_nexus_airlock
name: Nexus Airlock
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_military_gate, amount: 2 }
  - { id: era1_component_pressure_chamber, amount: 2 }
outputs:
  - { id: era1_nexus_airlock, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Pressurized Nexus airlock.
```

### era1_recipe_nexus_crane — Nexus Assembly Crane
```
recipe_id: era1_recipe_nexus_crane
name: Nexus Assembly Crane
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_component_heavy_motor, amount: 4 }
  - { id: era1_component_linear_rail, amount: 8 }
  - { id: era1_component_industrial_robot_core, amount: 2 }
outputs:
  - { id: era1_nexus_crane, amount: 1 }
waste_outputs:
  []
processing_time: 70
power_consumption: { electrical: 500 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Heavy crane for Nexus construction.
```

### era1_recipe_nexus_scaffold — Nexus Scaffold
```
recipe_id: era1_recipe_nexus_scaffold
name: Nexus Scaffold
category: nexus
machine: era1_machine_assembler_mk3
inputs:
  - { id: era1_component_structural_frame, amount: 20 }
  - { id: era1_material_ferrite_plate, amount: 40 }
outputs:
  - { id: era1_nexus_scaffold, amount: 10 }
waste_outputs:
  []
processing_time: 30
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Temporary scaffold for Nexus build site.
```

### era1_recipe_nexus_survey — Nexus Site Survey
```
recipe_id: era1_recipe_nexus_survey
name: Nexus Site Survey
category: nexus
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_science_field_scanner, amount: 2 }
  - { id: era1_science_engineering_data, amount: 5 }
outputs:
  - { id: era1_nexus_site_survey, amount: 1 }
waste_outputs:
  []
processing_time: 60
power_consumption: { electrical: 250 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Surveys and clears Nexus construction site.
```

### era1_recipe_nexus_permit — Planetary Construction Permit
```
recipe_id: era1_recipe_nexus_permit
name: Planetary Construction Permit
category: nexus
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_nexus_site_survey, amount: 1 }
  - { id: era1_science_era_transition_dossier, amount: 1 }
outputs:
  - { id: era1_nexus_construction_permit, amount: 1 }
waste_outputs:
  []
processing_time: 90
power_consumption: { electrical: 300 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Administrative/tech permit to begin Nexus.
```

### era1_recipe_nexus_foundation_pour — Nexus Foundation Pour
```
recipe_id: era1_recipe_nexus_foundation_pour
name: Nexus Foundation Pour
category: nexus
machine: era1_machine_construction_site
inputs:
  - { id: era1_nexus_construction_permit, amount: 1 }
  - { id: era1_nexus_anchor, amount: 4 }
  - { id: era1_material_mineral_binder, amount: 100 }
outputs:
  - { id: era1_nexus_foundation_pad, amount: 1 }
waste_outputs:
  []
processing_time: 120
power_consumption: { electrical: 200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Pours the Nexus foundation pad.
```

### era1_recipe_nexus_frame_erect — Erect Nexus Frame
```
recipe_id: era1_recipe_nexus_frame_erect
name: Erect Nexus Frame
category: nexus
machine: era1_machine_construction_site
inputs:
  - { id: era1_nexus_foundation_pad, amount: 1 }
  - { id: era1_nexus_foundation_frame, amount: 1 }
  - { id: era1_nexus_crane, amount: 1 }
outputs:
  - { id: era1_nexus_frame_erected, amount: 1 }
waste_outputs:
  []
processing_time: 150
power_consumption: { electrical: 500 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Erects the Nexus primary frame.
```

### era1_recipe_nexus_integrate_power — Integrate Nexus Power
```
recipe_id: era1_recipe_nexus_integrate_power
name: Integrate Nexus Power
category: nexus
machine: era1_machine_construction_site
inputs:
  - { id: era1_nexus_frame_erected, amount: 1 }
  - { id: era1_nexus_power_core, amount: 1 }
  - { id: era1_nexus_conduit, amount: 10 }
outputs:
  - { id: era1_nexus_power_integrated, amount: 1 }
waste_outputs:
  []
processing_time: 100
power_consumption: { electrical: 600 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Integrates power core into Nexus.
```

### era1_recipe_nexus_integrate_compute — Integrate Nexus Compute
```
recipe_id: era1_recipe_nexus_integrate_compute
name: Integrate Nexus Compute
category: nexus
machine: era1_machine_construction_site
inputs:
  - { id: era1_nexus_power_integrated, amount: 1 }
  - { id: era1_nexus_computational_core, amount: 1 }
  - { id: era1_nexus_data_cable, amount: 20 }
outputs:
  - { id: era1_nexus_compute_integrated, amount: 1 }
waste_outputs:
  []
processing_time: 100
power_consumption: { electrical: 600 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Integrates computational core.
```

### era1_recipe_nexus_integrate_mfg — Integrate Nexus Manufacturing
```
recipe_id: era1_recipe_nexus_integrate_mfg
name: Integrate Nexus Manufacturing
category: nexus
machine: era1_machine_construction_site
inputs:
  - { id: era1_nexus_compute_integrated, amount: 1 }
  - { id: era1_nexus_manufacturing_module, amount: 1 }
outputs:
  - { id: era1_nexus_mfg_integrated, amount: 1 }
waste_outputs:
  []
processing_time: 100
power_consumption: { electrical: 600 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Integrates manufacturing module.
```

### era1_recipe_nexus_integrate_resources — Integrate Nexus Resources
```
recipe_id: era1_recipe_nexus_integrate_resources
name: Integrate Nexus Resources
category: nexus
machine: era1_machine_construction_site
inputs:
  - { id: era1_nexus_mfg_integrated, amount: 1 }
  - { id: era1_nexus_resource_interface, amount: 1 }
  - { id: era1_nexus_resource_silo, amount: 1 }
outputs:
  - { id: era1_nexus_resources_integrated, amount: 1 }
waste_outputs:
  []
processing_time: 100
power_consumption: { electrical: 600 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Integrates resource interface and silos.
```

### era1_recipe_nexus_final_commission — Commission Nexus
```
recipe_id: era1_recipe_nexus_final_commission
name: Commission Nexus
category: nexus
machine: era1_machine_construction_site
inputs:
  - { id: era1_nexus_resources_integrated, amount: 1 }
  - { id: era1_nexus_control_array, amount: 1 }
  - { id: era1_nexus_cooling_plant, amount: 1 }
  - { id: era1_nexus_security_grid, amount: 1 }
outputs:
  - { id: era1_nexus_planetary_fabrication_nexus, amount: 1 }
waste_outputs:
  []
processing_time: 180
power_consumption: { electrical: 1000 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Final commissioning of the Planetary Fabrication Nexus.
```

### era1_recipe_nexus_diagnostic — Nexus Diagnostic Sweep
```
recipe_id: era1_recipe_nexus_diagnostic
name: Nexus Diagnostic Sweep
category: nexus
machine: era1_machine_research_analyzer
inputs:
  - { id: era1_nexus_planetary_fabrication_nexus, amount: 1 }
  - { id: era1_component_diagnostic_module, amount: 5 }
outputs:
  - { id: era1_nexus_diagnostic_report, amount: 1 }
waste_outputs:
  []
processing_time: 60
power_consumption: { electrical: 400 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Validates Nexus systems after commissioning.
```

### era1_recipe_nexus_calibration — Nexus Calibration
```
recipe_id: era1_recipe_nexus_calibration
name: Nexus Calibration
category: nexus
machine: era1_machine_calibration_station
inputs:
  - { id: era1_nexus_diagnostic_report, amount: 1 }
  - { id: era1_component_machine_calibration_unit, amount: 5 }
outputs:
  - { id: era1_nexus_calibrated, amount: 1 }
waste_outputs:
  []
processing_time: 80
power_consumption: { electrical: 450 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Calibrates Nexus manufacturing tolerances.
```

### era1_recipe_nexus_overclock_safe — Nexus Safe Overclock Profile
```
recipe_id: era1_recipe_nexus_overclock_safe
name: Nexus Safe Overclock Profile
category: nexus
machine: era1_machine_electronics_printer_mk3
inputs:
  - { id: era1_nexus_calibrated, amount: 1 }
  - { id: era1_component_factory_os, amount: 1 }
outputs:
  - { id: era1_nexus_overclock_profile, amount: 1 }
waste_outputs:
  []
processing_time: 50
power_consumption: { electrical: 350 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Safe performance profile for Nexus.
```

### era1_recipe_era2_key — Era 2 Access Key
```
recipe_id: era1_recipe_era2_key
name: Era 2 Access Key
category: nexus
machine: era1_machine_construction_site
inputs:
  - { id: era1_nexus_era_transition_beacon, amount: 1 }
  - { id: era1_nexus_overclock_profile, amount: 1 }
  - { id: era1_nexus_lab_wing, amount: 1 }
outputs:
  - { id: era1_nexus_era2_access_key, amount: 1 }
waste_outputs:
  []
processing_time: 200
power_consumption: { electrical: 1200 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Final key unlocking Era 2 content.
```

### era1_recipe_nexus_maintenance_drone — Nexus Maintenance Drone
```
recipe_id: era1_recipe_nexus_maintenance_drone
name: Nexus Maintenance Drone
category: nexus
machine: era1_machine_robotics_factory_mk1
inputs:
  - { id: era1_logistics_construction_drone, amount: 2 }
  - { id: era1_component_spare_parts_kit, amount: 4 }
outputs:
  - { id: era1_nexus_maintenance_drone, amount: 1 }
waste_outputs:
  []
processing_time: 40
power_consumption: { electrical: 280 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Heavy maintenance drone for Nexus.
```

### era1_recipe_nexus_spare_vault — Nexus Spare Parts Vault
```
recipe_id: era1_recipe_nexus_spare_vault
name: Nexus Spare Parts Vault
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_component_spare_parts_kit, amount: 20 }
  - { id: era1_logistics_warehouse_module, amount: 1 }
outputs:
  - { id: era1_nexus_spare_vault, amount: 1 }
waste_outputs:
  []
processing_time: 60
power_consumption: { electrical: 400 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Stores critical Nexus spares.
```

### era1_recipe_nexus_fire_suppression — Nexus Fire Suppression
```
recipe_id: era1_recipe_nexus_fire_suppression
name: Nexus Fire Suppression
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_fluid_coolant, amount: 100 }
  - { id: era1_component_pressure_chamber, amount: 4 }
outputs:
  - { id: era1_nexus_fire_suppression, amount: 1 }
waste_outputs:
  []
processing_time: 50
power_consumption: { electrical: 350 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Fire suppression network for Nexus.
```

### era1_recipe_nexus_atmosphere_plant — Nexus Atmosphere Plant
```
recipe_id: era1_recipe_nexus_atmosphere_plant
name: Nexus Atmosphere Plant
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_machine_atmospheric_intake_mk1, amount: 4 }
  - { id: era1_machine_atmospheric_separator_mk1, amount: 2 }
outputs:
  - { id: era1_nexus_atmosphere_plant, amount: 1 }
waste_outputs:
  []
processing_time: 70
power_consumption: { electrical: 450 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Atmosphere handling plant for Nexus habitation.
```

### era1_recipe_nexus_water_plant — Nexus Water Plant
```
recipe_id: era1_recipe_nexus_water_plant
name: Nexus Water Plant
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_machine_atmospheric_condenser_mk1, amount: 4 }
  - { id: era1_machine_water_purifier_mk1, amount: 4 }
outputs:
  - { id: era1_nexus_water_plant, amount: 1 }
waste_outputs:
  []
processing_time: 70
power_consumption: { electrical: 450 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Water synthesis plant supporting Nexus.
```

### era1_recipe_nexus_habitat — Nexus Habitat Module
```
recipe_id: era1_recipe_nexus_habitat
name: Nexus Habitat Module
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_material_structural_panel, amount: 40 }
  - { id: era1_material_reinforced_glass, amount: 20 }
  - { id: era1_nexus_airlock, amount: 2 }
outputs:
  - { id: era1_nexus_habitat, amount: 1 }
waste_outputs:
  []
processing_time: 80
power_consumption: { electrical: 500 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Crew habitat module for Nexus ops.
```

### era1_recipe_nexus_comm_array — Nexus Comm Array
```
recipe_id: era1_recipe_nexus_comm_array
name: Nexus Comm Array
category: nexus
machine: era1_machine_advanced_electronics_printer
inputs:
  - { id: era1_component_communication_module, amount: 10 }
  - { id: era1_military_radar_tower, amount: 1 }
outputs:
  - { id: era1_nexus_comm_array, amount: 1 }
waste_outputs:
  []
processing_time: 60
power_consumption: { electrical: 400 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Long-range communications array.
```

### era1_recipe_nexus_archive_wing — Nexus Archive Wing
```
recipe_id: era1_recipe_nexus_archive_wing
name: Nexus Archive Wing
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_science_data_archive, amount: 4 }
  - { id: era1_nexus_data_array, amount: 1 }
outputs:
  - { id: era1_nexus_archive_wing, amount: 1 }
waste_outputs:
  []
processing_time: 70
power_consumption: { electrical: 450 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Archives all Era 1 research at planetary scale.
```

### era1_recipe_nexus_victory_record — Planetary Recovery Record
```
recipe_id: era1_recipe_nexus_victory_record
name: Planetary Recovery Record
category: nexus
machine: era1_machine_research_laboratory
inputs:
  - { id: era1_nexus_era2_access_key, amount: 1 }
  - { id: era1_nexus_archive_wing, amount: 1 }
outputs:
  - { id: era1_nexus_planetary_recovery_record, amount: 1 }
waste_outputs:
  []
processing_time: 120
power_consumption: { electrical: 500 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Records completion of Planetary Recovery Era.
```

### era1_recipe_nexus_replica_core — Nexus Replica Core
```
recipe_id: era1_recipe_nexus_replica_core
name: Nexus Replica Core
category: nexus
machine: era1_machine_heavy_fabricator
inputs:
  - { id: era1_nexus_manufacturing_module, amount: 1 }
  - { id: era1_nexus_computational_core, amount: 1 }
outputs:
  - { id: era1_nexus_replica_core, amount: 1 }
waste_outputs:
  []
processing_time: 150
power_consumption: { electrical: 800 }
purity_effect: 0
grade_effect: industrial
technology_unlock: era1_tech_nexus_construction
description: Optional second Nexus core for megabase players.
```
