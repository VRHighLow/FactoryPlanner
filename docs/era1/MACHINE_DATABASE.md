# ERA 1 — MACHINE DATABASE v1.1
## Planetary Recovery Era (complete fill)

Source: User Machine DB v1.0 (M001–M008 authored) + gap fill for M009–M075  
IDs reconciled to recipe database (`era1_machine_*`).

### Balance rules
- Early: 50–200 kW · Mid: 200–800 kW · Late: 800 kW–2 MW
- Mk upgrades change purity/waste/recipes/efficiency — not only speed
- Power is never an item

---

# SECTION 1 — EXTRACTION (M001–M008)

### M001 — Ferrite Extraction Drill Mk1
```
machine_id: era1_machine_ferrite_drill_mk1
name: Ferrite Extraction Drill Mk1
category: extraction
tier: 1
description: Heavy drill head that taps ferrite veins and ejects ore onto belts.
function: Extract ferrite ore from planetary veins.
size: 3x3
power: 50 kW
power_type: electrical
inputs:
  - Ferrite Vein (world deposit)
outputs:
  - era1_raw_ferrite_ore
recipe_categories:
  - mining
  - ferrite_extraction
fluid_ports:
  []
technology_unlock: era1_tech_basic_recovery
purity_behavior: Deposit purity copied to ore. High purity = more concentrate potential, less tailings after crush.
maintenance: Wear rises on low-stability veins; downtime spikes when stability < 40%.
upgrade_path: era1_machine_ferrite_drill_mk2 — +yield, +4% purity preservation, lower wear
animation: Rotating excavation head + dust plume
```
### M002 — Conductive Extraction Drill Mk1
```
machine_id: era1_machine_conductive_drill_mk1
name: Conductive Extraction Drill Mk1
category: extraction
tier: 1
description: Specialized drill for conductive mineral veins.
function: Extract conductive ore.
size: 3x3
power: 60 kW
power_type: electrical
inputs:
  - Conductive Vein
outputs:
  - era1_raw_conductive_ore
recipe_categories:
  - mining
  - conductive_extraction
fluid_ports:
  []
technology_unlock: era1_tech_basic_recovery
purity_behavior: Vein purity maps to ore purity for later acid purification bonus.
maintenance: Standard mining wear; conductive dust increases filter clog rate.
upgrade_path: Mk2 — higher richness utilization, less mineral dust byproduct downstream
animation: Spiral bit + conductive spark accents
```
### M003 — Carbon Extraction Drill Mk1
```
machine_id: era1_machine_carbon_drill_mk1
name: Carbon Extraction Drill Mk1
category: extraction
tier: 1
description: Cuts and lifts dense carbon deposits.
function: Extract carbon deposits.
size: 3x3
power: 55 kW
power_type: electrical
inputs:
  - Carbon Vein
outputs:
  - era1_raw_carbon_deposit
recipe_categories:
  - mining
  - carbon_extraction
fluid_ports:
  []
technology_unlock: era1_tech_basic_recovery
purity_behavior: Affects graphite conversion efficiency later.
maintenance: Carbon grit abrades conveyors; more frequent bit drain on low stability.
upgrade_path: Mk2 — cleaner cut (less carbon residue on crush)
animation: Oscillating cutter head
```
### M004 — Silicate Extraction Drill Mk1
```
machine_id: era1_machine_silicate_drill_mk1
name: Silicate Extraction Drill Mk1
category: extraction
tier: 1
description: Rock drill for silicate formations.
function: Extract silicate rock.
size: 3x3
power: 55 kW
power_type: electrical
inputs:
  - Silicate Vein
outputs:
  - era1_raw_silicate_rock
recipe_categories:
  - mining
  - silicate_extraction
fluid_ports:
  []
technology_unlock: era1_tech_basic_recovery
purity_behavior: Higher purity improves glass/ceramic yields.
maintenance: High vibration; unstable deposits cause intermittent jams.
upgrade_path: Mk2 — reduced stone dust fraction after crush
animation: Percussive hammer drill
```
### M005 — Hydrocarbon Pump Mk1
```
machine_id: era1_machine_hydrocarbon_pump_mk1
name: Hydrocarbon Pump Mk1
category: extraction
tier: 1
description: Pumpjack-style extractor for underground hydrocarbon seams.
function: Extract raw hydrocarbon fluid.
size: 3x5
power: 100 kW
power_type: electrical
inputs:
  - Hydrocarbon Deposit
outputs:
  - era1_fluid_raw_hydrocarbon
recipe_categories:
  - fluid_extraction
  - hydrocarbon
fluid_ports:
  - out:raw_hydrocarbon
technology_unlock: era1_tech_fluid_engineering
purity_behavior: Controls fraction quality after distillation (light/medium/heavy balance).
maintenance: Seals degrade faster on low-stability seams; leak events raise hazard.
upgrade_path: Mk2 — higher flow, better fraction purity bias
animation: Nodding pump arm + pipe pulse
```
### M006 — Atmospheric Intake Mk1
```
machine_id: era1_machine_atmospheric_intake_mk1
name: Atmospheric Intake Mk1
category: extraction
tier: 1
description: Compresses ambient atmosphere into processable gas mix.
function: Collect atmospheric mix for separation/condensation.
size: 2x2
power: 75 kW
power_type: electrical
inputs:
  - World atmosphere
outputs:
  - era1_gas_atmospheric_mix
recipe_categories:
  - atmosphere
  - gas_intake
fluid_ports:
  - out:atmospheric_mix
technology_unlock: era1_tech_fluid_engineering
purity_behavior: No material purity; filter clogging simulated via maintenance.
maintenance: Intake filters clog in fog/storm; throughput drops until cleaned.
upgrade_path: Mk2 — storm-resistant filters, +flow
animation: Turbine intake swirl
```
### M007 — Water Condenser Mk1
```
machine_id: era1_machine_atmospheric_condenser_mk1
name: Water Condenser Mk1
category: extraction
tier: 1
description: Condenses atmospheric mix / moisture into condensed water. Alias: atmospheric condenser.
function: Industrial water synthesis step 1.
size: 3x3
power: 150 kW
power_type: electrical
inputs:
  - era1_gas_atmospheric_mix
outputs:
  - era1_fluid_condensed_water
recipe_categories:
  - water
  - condensation
fluid_ports:
  - in:atmospheric_mix
  - out:condensed_water
technology_unlock: era1_tech_fluid_engineering
purity_behavior: Output water starts low purity; requires purification chain.
maintenance: Cooling fins foul; efficiency drops in heat/storm.
upgrade_path: Mk2 — lower kWh/water, optional direct moisture path
animation: Frosting coils + drip
```
### M008 — Waste Recovery Extractor Mk1
```
machine_id: era1_machine_waste_extractor_mk1
name: Waste Recovery Extractor Mk1
category: extraction
tier: 1
description: Reclaims scattered waste piles / slag fields into processable waste items.
function: Extract mixed waste for recovery plants.
size: 3x3
power: 80 kW
power_type: electrical
inputs:
  - Waste field (world)
outputs:
  - era1_waste_metallic_tailings
  - era1_waste_stone_dust
  - era1_waste_chemical_residue
recipe_categories:
  - waste_extraction
fluid_ports:
  []
technology_unlock: era1_tech_waste_recovery
purity_behavior: Recovered streams start low purity; recovery recipes raise usable fraction.
maintenance: Abrasion high; tool wear scales with waste hardness mix.
upgrade_path: Mk2 — selective extraction (choose waste type)
animation: Scoop + sorter grate
```

# SECTION 2 — PROCESSING (M009–M023)

### M009 — Crusher Mk1
```
machine_id: era1_machine_crusher_mk1
name: Crusher Mk1
category: processing
tier: 1
description: Jaw/impact crusher that breaks raw ore into crushed feedstock and mineral dust.
function: Primary size reduction for all solid ores.
size: 3x3
power: 120 kW
power_type: mechanical
inputs:
  - raw ores
  - carbon deposit
  - silicate rock
outputs:
  - crushed materials
  - mineral/stone dust
  - carbon residue
recipe_categories:
  - crushing
fluid_ports:
  []
technology_unlock: era1_tech_material_processing
purity_behavior: No purity gain. Splits mass into product + waste dust.
maintenance: Jaw plates wear; throughput falls until replaced.
upgrade_path: Mk2 — +throughput, -waste dust, unlocks finer crush recipes
animation: Oscillating jaws + dust burst
```
### M010 — Industrial Grinder Mk1
```
machine_id: era1_machine_industrial_grinder_mk1
name: Industrial Grinder Mk1
category: processing
tier: 1
description: Fine grinding mill for powders (silicon, metal, ferrite).
function: Powder production for smelting and electronics.
size: 3x3
power: 150 kW
power_type: mechanical
inputs:
  - refined silicon
  - steel composite
  - purified ferrite
outputs:
  - powders
recipe_categories:
  - grinding
fluid_ports:
  []
technology_unlock: era1_tech_material_processing
purity_behavior: Preserves purity; slight loss if overfed.
maintenance: Media balls wear; contamination risk if mixed recipes without flush.
upgrade_path: Mk2 — +purity preservation, auto-flush
animation: Rotating drum
```
### M011 — Ore Purifier Mk1
```
machine_id: era1_machine_ore_purifier_mk1
name: Ore Purifier Mk1
category: processing
tier: 1
description: Washes/separates crushed ore using water or acid.
function: Raises purity and ejects slurry/tailings.
size: 3x4
power: 200 kW
power_type: electrical
inputs:
  - crushed ores
  - purified water or acid
outputs:
  - purified concentrates
  - mineral slurry
  - chemical waste
recipe_categories:
  - purification
fluid_ports:
  - in:fluid
  - out:slurry
technology_unlock: era1_tech_material_processing
purity_behavior: +8 to +12% purity depending on recipe.
maintenance: Filter cakes clog; acid recipes raise corrosion wear.
upgrade_path: Advanced Purifier Mk1
animation: Agitated tanks + overflow weir
```
### M012 — Advanced Ore Purifier Mk1
```
machine_id: era1_machine_advanced_purifier_mk1
name: Advanced Ore Purifier Mk1
category: processing
tier: 2
description: Catalyzed multi-stage purifier for high-efficiency concentrates.
function: High purity ferrite/conductive preparation.
size: 4x4
power: 450 kW
power_type: electrical
inputs:
  - crushed/purified intermediates
  - acid
  - catalyst
outputs:
  - concentrates
  - metallic tailings
recipe_categories:
  - advanced_purification
fluid_ports:
  - in:acid
  - in:catalyst
  - out:waste
technology_unlock: era1_tech_advanced_metallurgy
purity_behavior: +18 to +22% purity. Required path for precision alloys.
maintenance: Catalyst beds foul; scheduled bake-out reduces downtime.
upgrade_path: Mk2 — +purity, lower catalyst use
animation: Multi-column cascade
```
### M013 — Thermal Smelter Mk1
```
machine_id: era1_machine_thermal_smelter_mk1
name: Thermal Smelter Mk1
category: processing
tier: 1
description: Thermal furnace for plates from powders/concentrates.
function: Core metallurgy machine.
size: 3x3
power: 400 kW
power_type: thermal
inputs:
  - ferrite powder
  - conductive concentrate
  - carbon powder
outputs:
  - plates
  - carbon residue
recipe_categories:
  - smelting
fluid_ports:
  []
technology_unlock: era1_tech_basic_metallurgy
purity_behavior: Slight purity loss (-1 to -3%) from oxidation unless reducing agents present.
maintenance: Refractory wear; overheat if under-cooled.
upgrade_path: Mk2 — less purity loss, slag recovery recipe
animation: Glow core + pour spout
```
### M014 — Alloy Furnace Mk1
```
machine_id: era1_machine_alloy_furnace_mk1
name: Alloy Furnace Mk1
category: processing
tier: 2
description: Combines metals/carbon into steels and alloy blanks.
function: Steel composite, alloy blanks, reinforced ferrite.
size: 4x3
power: 600 kW
power_type: thermal
inputs:
  - plates
  - graphite
  - oxygen
  - carbon composite
outputs:
  - steels
  - alloys
  - reinforced metals
recipe_categories:
  - alloying
fluid_ports:
  - in:oxygen
technology_unlock: era1_tech_advanced_metallurgy
purity_behavior: Can raise grade chance; purity floor enforced on premium recipes.
maintenance: Crucible stress cracks on rapid thermal cycling.
upgrade_path: Precision Alloy Furnace
animation: Tilting crucible
```
### M015 — Heat Treatment Furnace Mk1
```
machine_id: era1_machine_heat_treatment_furnace_mk1
name: Heat Treatment Furnace Mk1
category: processing
tier: 2
description: Hardens and stress-relieves alloys.
function: Hardened steel / heat-treated paths.
size: 4x3
power: 700 kW
power_type: thermal
inputs:
  - steel composite
  - carbon fiber
outputs:
  - hardened steel
  - heat treated alloy
recipe_categories:
  - heat_treatment
fluid_ports:
  []
technology_unlock: era1_tech_advanced_metallurgy
purity_behavior: Improves grade; minimal purity change.
maintenance: Quench media contamination if shared with chem lines.
upgrade_path: Mk2 — tighter grade control
animation: Chamber doors + quench hiss
```
### M016 — Precision Alloy Furnace Mk1
```
machine_id: era1_machine_precision_alloy_furnace_mk1
name: Precision Alloy Furnace Mk1
category: processing
tier: 3
description: Tight-tolerance alloy furnace for precision/heat-treated alloys.
function: Late metallurgy specialty products.
size: 4x4
power: 900 kW
power_type: thermal
inputs:
  - hardened steel
  - alloy blank
  - catalyst
outputs:
  - precision alloy
  - heat treated alloy
recipe_categories:
  - precision_alloying
fluid_ports:
  - in:catalyst
technology_unlock: era1_tech_precision_manufacturing
purity_behavior: +grade; rejects batches below purity floor.
maintenance: High calibration drift — needs calibration station periodically.
upgrade_path: Mk2 — auto-calibrate
animation: Sealed vacuum glow
```
### M017 — Ceramic Furnace Mk1
```
machine_id: era1_machine_ceramic_furnace_mk1
name: Ceramic Furnace Mk1
category: processing
tier: 1
description: Fires glass, ceramics, reinforced glass.
function: Silicate finishing.
size: 3x3
power: 350 kW
power_type: thermal
inputs:
  - silicon sand
  - silicon powder
  - glass
  - polymer resin
  - mineral binder
outputs:
  - glass
  - ceramic
  - reinforced glass
recipe_categories:
  - ceramics
  - glass
fluid_ports:
  []
technology_unlock: era1_tech_ceramic_engineering
purity_behavior: Purity of silicon feeds optical quality later.
maintenance: Kiln bricks erode; glass recipes leave residue.
upgrade_path: Ceramic Furnace Mk2
animation: Kiln glow
```
### M018 — Ceramic Furnace Mk2
```
machine_id: era1_machine_ceramic_furnace_mk2
name: Ceramic Furnace Mk2
category: processing
tier: 2
description: High-temp ceramic furnace for advanced/heat-resistant ceramics.
function: Advanced ceramic production.
size: 4x3
power: 800 kW
power_type: thermal
inputs:
  - ceramic
  - mineral binder
  - ultra pure water
  - advanced ceramic
outputs:
  - advanced ceramic
  - heat resistant ceramic
  - reactor lining
recipe_categories:
  - advanced_ceramics
fluid_ports:
  - in:ultra_pure_water
technology_unlock: era1_tech_advanced_ceramics
purity_behavior: +5% purity on advanced ceramic recipe.
maintenance: Extreme thermal stress; schedule cooldown cycles.
upgrade_path: Mk3 (Era 2)
animation: White-hot chamber
```
### M019 — Material Dryer / Processor Mk1
```
machine_id: era1_machine_material_processor_mk1
name: Material Dryer / Processor Mk1
category: processing
tier: 1
description: Dries purified solids into powders (legacy name: Material Processor).
function: Ferrite drying and general solids prep.
size: 3x2
power: 100 kW
power_type: electrical
inputs:
  - purified ferrite
outputs:
  - ferrite powder
recipe_categories:
  - drying
  - material_prep
fluid_ports:
  []
technology_unlock: era1_tech_material_processing
purity_behavior: Preserves purity.
maintenance: Heater coils scale with wet feed.
upgrade_path: Mk2 — faster dry, less energy
animation: Conveyor through heated tunnel
```
### M020 — Industrial Press / Hydraulic Press Mk1
```
machine_id: era1_machine_hydraulic_press_mk1
name: Industrial Press / Hydraulic Press Mk1
category: processing
tier: 2
description: Compacts powders into blocks and structural forms.
function: Metal powder compaction and panel pressing.
size: 3x3
power: 300 kW
power_type: mechanical
inputs:
  - metal powder
  - polymer resin
outputs:
  - reinforced metal block
recipe_categories:
  - pressing
fluid_ports:
  - in:lubricant optional
technology_unlock: era1_tech_advanced_metallurgy
purity_behavior: Grade rises with slow press profiles.
maintenance: Hydraulic leaks on poor maintenance.
upgrade_path: Mk2 — dual die, higher grade chance
animation: Ram stamp
```
### M021 — Material Roller / Precision Roller Mk1
```
machine_id: era1_machine_precision_roller_mk1
name: Material Roller / Precision Roller Mk1
category: processing
tier: 2
description: Rolls plates into foils and sheets.
function: Conductive foil and sheet products.
size: 3x4
power: 220 kW
power_type: electrical
inputs:
  - conductive plate
outputs:
  - conductive foil
recipe_categories:
  - rolling
fluid_ports:
  []
technology_unlock: era1_tech_electronics
purity_behavior: High purity in → high purity foil out.
maintenance: Roll scoring if contaminated feed.
upgrade_path: Mk2 — thinner gauges, less scrap
animation: Paired rollers
```
### M022 — Recovery Plant Mk1
```
machine_id: era1_machine_recovery_plant_mk1
name: Recovery Plant Mk1
category: processing
tier: 2
description: Recovers metals from tailings using acid chemistry.
function: Tailings → ferrite dust / conductive trace.
size: 4x4
power: 350 kW
power_type: electrical
inputs:
  - metallic tailings
  - acid
outputs:
  - ferrite dust
  - conductive trace
  - waste slurry
recipe_categories:
  - recovery
fluid_ports:
  - in:acid
  - out:slurry
technology_unlock: era1_tech_waste_recovery
purity_behavior: Outputs low-mid purity; useful feedstock, not premium.
maintenance: Acid-proof liners wear; hazard events if ignored.
upgrade_path: Mk2 — higher recovery %, less slurry
animation: Bubbling leach tanks
```
### M023 — Recycling Plant Mk1
```
machine_id: era1_machine_recycling_plant_mk1
name: Recycling Plant Mk1
category: processing
tier: 2
description: General recycler for carbon residue and polymer scrap.
function: Waste loop closer.
size: 4x3
power: 300 kW
power_type: electrical
inputs:
  - carbon residue
  - polymer scrap
outputs:
  - carbon powder
  - polymer resin
recipe_categories:
  - recycling
fluid_ports:
  []
technology_unlock: era1_tech_waste_recovery
purity_behavior: Recycled outputs slightly lower grade.
maintenance: Shredder jams on mixed feeds.
upgrade_path: Mk2 — auto-sort intake
animation: Shredder + melt pot
```

# SECTION 3 — CHEMICAL (M024–M035)

### M024 — Chemical Reactor Mk1
```
machine_id: era1_machine_chemical_reactor_mk1
name: Chemical Reactor Mk1
category: chemical
tier: 1
description: General stirred reactor for acids, additives, electrolytes.
function: Core chemistry.
size: 3x3
power: 250 kW
power_type: electrical
inputs:
  - fluids
  - powders
outputs:
  - chemical fluids
  - residues
recipe_categories:
  - chemistry
fluid_ports:
  - in:a
  - in:b
  - out:product
  - out:waste
technology_unlock: era1_tech_chemical_manufacturing
purity_behavior: Recipe-defined purity effects.
maintenance: Fouling/pressure events.
upgrade_path: Mk2
animation: Agitator
```
### M025 — Chemical Reactor Mk2
```
machine_id: era1_machine_chemical_reactor_mk2
name: Chemical Reactor Mk2
category: chemical
tier: 2
description: Pressurized reactor for propellant and advanced reactions.
function: Advanced chemistry.
size: 4x3
power: 500 kW
power_type: electrical
inputs:
  - advanced feeds
outputs:
  - propellant
  - advanced chemicals
recipe_categories:
  - advanced_chemistry
fluid_ports:
  - in:a
  - in:b
  - out:product
technology_unlock: era1_tech_chemical_manufacturing
purity_behavior: Better conversion.
maintenance: Overpressure risk.
upgrade_path: Mk3 Era2
animation: Pressurized vessel
```
### M026 — Polymer Reactor Mk1
```
machine_id: era1_machine_polymer_reactor_mk1
name: Polymer Reactor Mk1
category: chemical
tier: 1
description: Polymerizes feedstock into resin/rubber/mixes.
function: Polymer backbone.
size: 3x4
power: 280 kW
power_type: electrical
inputs:
  - feedstock
  - catalyst
outputs:
  - resin
  - rubber
  - mix
recipe_categories:
  - polymers
fluid_ports:
  - in:feed
  - out:polymer
technology_unlock: era1_tech_polymer_science
purity_behavior: Clean feed → higher grade.
maintenance: Viscous fouling.
upgrade_path: Mk2
animation: Extruder
```
### M027 — Polymer Reactor Mk2
```
machine_id: era1_machine_polymer_reactor_mk2
name: Polymer Reactor Mk2
category: chemical
tier: 2
description: Advanced polymer blends with fiber.
function: Advanced polymers.
size: 4x4
power: 550 kW
power_type: electrical
inputs:
  - mix
  - fiber
outputs:
  - advanced polymer
recipe_categories:
  - advanced_polymers
fluid_ports:
  - in:feed
  - out:polymer
technology_unlock: era1_tech_polymer_science
purity_behavior: +grade.
maintenance: Die swaps.
upgrade_path: Mk3
animation: Multi-die
```
### M028 — Distillation Tower Mk1
```
machine_id: era1_machine_distillation_tower_mk1
name: Distillation Tower Mk1
category: chemical
tier: 2
description: Fractions raw hydrocarbon.
function: Oil refining.
size: 3x6
power: 350 kW
power_type: thermal
inputs:
  - raw hydrocarbon
outputs:
  - fractions
  - gas
recipe_categories:
  - distillation
fluid_ports:
  - in:raw
  - out:heavy
  - out:medium
  - out:light
  - out:gas
technology_unlock: era1_tech_hydrocarbon_refining
purity_behavior: Purity shifts ratios.
maintenance: Tray fouling/fire hazard.
upgrade_path: Mk2
animation: Tall tower
```
### M029 — Gas Separator Mk1
```
machine_id: era1_machine_atmospheric_separator_mk1
name: Gas Separator Mk1
category: chemical
tier: 1
description: Splits atmospheric mix to N2/O2/CO2.
function: Industrial gases.
size: 3x3
power: 400 kW
power_type: electrical
inputs:
  - atmospheric mix
outputs:
  - N2
  - O2
  - CO2
recipe_categories:
  - gas_separation
fluid_ports:
  - in:mix
  - out:n2
  - out:o2
  - out:co2
technology_unlock: era1_tech_fluid_engineering
purity_behavior: Output quality via maintenance.
maintenance: Membrane wear.
upgrade_path: Mk2
animation: Columns
```
### M030 — Electrochemical Separator Mk1
```
machine_id: era1_machine_electrochemical_separator_mk1
name: Electrochemical Separator Mk1
category: chemical
tier: 2
description: Water electrolysis to H2/O2.
function: Electrolysis.
size: 3x3
power: 450 kW
power_type: electrical
inputs:
  - purified water
outputs:
  - hydrogen
  - oxygen
recipe_categories:
  - electrolysis
fluid_ports:
  - in:water
  - out:h2
  - out:o2
technology_unlock: era1_tech_fluid_engineering
purity_behavior: Needs clean water.
maintenance: Electrode wear.
upgrade_path: Mk2
animation: Electrolyzer glow
```
### M031 — Fluid Processor Mk1
```
machine_id: era1_machine_fluid_processor_mk1
name: Fluid Processor Mk1
category: chemical
tier: 1
description: Blends coolants and conditioned fluids.
function: Fluid finishing.
size: 3x2
power: 180 kW
power_type: electrical
inputs:
  - fluids
  - additives
outputs:
  - coolants
recipe_categories:
  - fluid_processing
fluid_ports:
  - in:a
  - in:b
  - out:product
technology_unlock: era1_tech_chemical_manufacturing
purity_behavior: Slight purity raise.
maintenance: Seal wear.
upgrade_path: Mk2
animation: Inline mixer
```
### M032 — Water Purifier Mk1
```
machine_id: era1_machine_water_purifier_mk1
name: Water Purifier Mk1
category: chemical
tier: 1
description: Condensed → purified water.
function: Water purification.
size: 3x3
power: 200 kW
power_type: electrical
inputs:
  - condensed water
  - chemical filter
outputs:
  - purified water
  - slurry
recipe_categories:
  - water_purification
fluid_ports:
  - in:water
  - out:pure
  - out:slurry
technology_unlock: era1_tech_fluid_engineering
purity_behavior: +15 purity.
maintenance: Filter clogs.
upgrade_path: Precision Water Processor
animation: Filter carousel
```
### M033 — Gas Compressor Mk1
```
machine_id: era1_machine_gas_compressor
name: Gas Compressor Mk1
category: chemical
tier: 2
description: Compresses gases for storage/reactors.
function: Gas pressure boosting.
size: 2x3
power: 300 kW
power_type: electrical
inputs:
  - gas
outputs:
  - pressurized gas
recipe_categories:
  - gas_compression
fluid_ports:
  - in:gas
  - out:gas
technology_unlock: era1_tech_fluid_engineering
purity_behavior: N/A
maintenance: Overheat if blocked.
upgrade_path: Mk2
animation: Turbine
```
### M034 — Chemical Filter Unit Mk1
```
machine_id: era1_machine_chemical_filter_machine
name: Chemical Filter Unit Mk1
category: chemical
tier: 1
description: Scrubs fluids/gases with filter media.
function: Filtration skid.
size: 3x2
power: 150 kW
power_type: electrical
inputs:
  - dirty streams
  - filters
outputs:
  - clean streams
recipe_categories:
  - filtration
fluid_ports:
  - in:dirty
  - out:clean
technology_unlock: era1_tech_fluid_engineering
purity_behavior: Raises fluid purity.
maintenance: Consumes filters.
upgrade_path: Mk2
animation: Canister swap
```
### M035 — Waste Treatment Plant Mk1
```
machine_id: era1_machine_waste_treatment_plant
name: Waste Treatment Plant Mk1
category: chemical
tier: 2
description: Neutralizes chemical waste.
function: Waste treatment.
size: 4x3
power: 320 kW
power_type: electrical
inputs:
  - chemical residue
  - neutralizer
outputs:
  - treated slurry
recipe_categories:
  - waste_treatment
fluid_ports:
  - in:waste
  - out:treated
technology_unlock: era1_tech_waste_recovery
purity_behavior: Minor recovery possible.
maintenance: Hazard if starved of neutralizer.
upgrade_path: Mk2
animation: Scrubbers
```

# SECTION 4 — MANUFACTURING (M036–M050)

### M036 — Assembler Mk1
```
machine_id: era1_machine_assembler_mk1
name: Assembler Mk1
category: manufacturing
tier: 1
description: General discrete assembler for frames, gears, logistics parts.
function: Core manufacturing.
size: 3x3
power: 120 kW
power_type: electrical
inputs:
  - plates
  - components
outputs:
  - mechanical parts
  - logistics parts
recipe_categories:
  - assembly
  - mechanical
fluid_ports:
  []
technology_unlock: era1_tech_industrial_automation
purity_behavior: Grade industrial default; purity usually N/A.
maintenance: Arm wear.
upgrade_path: Mk2
animation: Articulated arms
```
### M037 — Assembler Mk2
```
machine_id: era1_machine_assembler_mk2
name: Assembler Mk2
category: manufacturing
tier: 2
description: Faster assembler with smart logistics recipes.
function: Mid automation crafting.
size: 3x3
power: 220 kW
power_type: electrical
inputs:
  - components
  - modules
outputs:
  - smart logistics
  - walls
  - lab frames
recipe_categories:
  - assembly
  - advanced_assembly
fluid_ports:
  []
technology_unlock: era1_tech_advanced_automation
purity_behavior: Can accept quality modules.
maintenance: Needs firmware updates.
upgrade_path: Mk3
animation: Dual-arm
```
### M038 — Heavy Assembler
```
machine_id: era1_machine_heavy_assembler_mk1
name: Heavy Assembler
category: manufacturing
tier: 2
description: Builds oversized frames, crushers, heavy machines.
function: Heavy manufacturing.
size: 4x4
power: 400 kW
power_type: electrical
inputs:
  - heavy frames
  - motors
outputs:
  - heavy machines
  - heavy frames
recipe_categories:
  - heavy_assembly
fluid_ports:
  []
technology_unlock: era1_tech_advanced_metallurgy
purity_behavior: N/A
maintenance: Crane stress.
upgrade_path: Mk2
animation: Overhead crane
```
### M039 — Component Fabricator Mk1
```
machine_id: era1_machine_component_fabricator_mk1
name: Component Fabricator Mk1
category: manufacturing
tier: 1
description: Fabricates precision mechanical subsystems (hydraulics, housings, filters).
function: Component fabrication.
size: 3x3
power: 180 kW
power_type: electrical
inputs:
  - gears
  - bearings
  - servos
  - pipes
outputs:
  - precision assemblies
  - filters
  - housings
recipe_categories:
  - component_fab
fluid_ports:
  []
technology_unlock: era1_tech_industrial_automation
purity_behavior: Grade rises with precision parts.
maintenance: Tooling magazine swaps.
upgrade_path: Precision Fabricator
animation: Tool head swap
```
### M040 — Precision Fabricator Mk1
```
machine_id: era1_machine_precision_fabricator_mk1
name: Precision Fabricator Mk1
category: manufacturing
tier: 2
description: Tight-tolerance parts: heat exchangers, lenses, precision housings.
function: Precision manufacturing.
size: 3x3
power: 280 kW
power_type: electrical
inputs:
  - alloys
  - coatings
  - optics feeds
outputs:
  - precision components
  - laser lenses
recipe_categories:
  - precision_fab
fluid_ports:
  []
technology_unlock: era1_tech_precision_manufacturing
purity_behavior: Enforces purity floors on optical recipes.
maintenance: Calibration drift.
upgrade_path: Mk2
animation: Laser etcher accents
```
### M041 — Heavy Fabricator
```
machine_id: era1_machine_heavy_fabricator
name: Heavy Fabricator
category: manufacturing
tier: 3
description: Constructs Nexus-scale modules and mega parts.
function: Late/heavy fabrication.
size: 5x5
power: 1.2 MW
power_type: electrical
inputs:
  - heavy frames
  - steel
  - nexus parts
outputs:
  - nexus modules
  - expansion bays
recipe_categories:
  - nexus_fab
  - heavy_fab
fluid_ports:
  []
technology_unlock: era1_tech_nexus_construction
purity_behavior: N/A
maintenance: Requires cooling loop nearby.
upgrade_path: Era2 fabricator
animation: Gantry system
```
### M042 — Machine Fabricator
```
machine_id: era1_machine_machine_fabricator_mk1
name: Machine Fabricator
category: manufacturing
tier: 2
description: Builds other production machines from housings/controllers.
function: Machine replication.
size: 4x4
power: 350 kW
power_type: electrical
inputs:
  - housings
  - motors
  - control modules
outputs:
  - tier1/2 machines
recipe_categories:
  - machine_fab
fluid_ports:
  []
technology_unlock: era1_tech_industrial_automation
purity_behavior: N/A
maintenance: Complex recipe setups.
upgrade_path: Mk2
animation: Internal assembler bay
```
### M043 — Robotics Factory Mk1
```
machine_id: era1_machine_robotics_factory_mk1
name: Robotics Factory Mk1
category: manufacturing
tier: 2
description: Assembles drones, robot cores, chassis.
function: Robotics production.
size: 4x4
power: 400 kW
power_type: electrical
inputs:
  - robotic frames
  - controllers
outputs:
  - drones
  - robot cores
recipe_categories:
  - robotics
fluid_ports:
  []
technology_unlock: era1_tech_robotics
purity_behavior: Grade precision preferred.
maintenance: Firmware mismatches cause scrap.
upgrade_path: Mk2
animation: Robot weld sparks
```
### M044 — Robotics Component Printer
```
machine_id: era1_machine_robotics_component_printer_mk1
name: Robotics Component Printer
category: manufacturing
tier: 2
description: Prints joints, frames, end effectors.
function: Robotics components.
size: 3x3
power: 260 kW
power_type: electrical
inputs:
  - actuators
  - composites
  - sensors
outputs:
  - joints
  - frames
  - effectors
recipe_categories:
  - robotics_components
fluid_ports:
  []
technology_unlock: era1_tech_robotics
purity_behavior: N/A
maintenance: Print bed leveling.
upgrade_path: Mk2
animation: Print head
```
### M045 — Armor Processor
```
machine_id: era1_machine_armor_processor_mk1
name: Armor Processor
category: manufacturing
tier: 2
description: Forms and layers armor plates/composites.
function: Military armor processing.
size: 3x3
power: 300 kW
power_type: electrical
inputs:
  - hardened steel
  - ceramics
  - carbon composite
outputs:
  - armor plates
  - composites
recipe_categories:
  - armor
fluid_ports:
  []
technology_unlock: era1_tech_defense_industry
purity_behavior: Higher grade armor from better alloys.
maintenance: Press dies wear.
upgrade_path: Mk2
animation: Hydraulic stamp
```
### M046 — Military Fabricator
```
machine_id: era1_machine_military_fabricator_mk1
name: Military Fabricator
category: manufacturing
tier: 2
description: Builds weapon housings, frames, wall kits.
function: Military structures/parts.
size: 4x3
power: 320 kW
power_type: electrical
inputs:
  - hardened steel
  - frames
outputs:
  - weapon housing
  - turret frames
  - walls
recipe_categories:
  - military_fab
fluid_ports:
  []
technology_unlock: era1_tech_defense_industry
purity_behavior: N/A
maintenance: Secure recipe lock optional.
upgrade_path: Mk2
animation: Armored bay doors
```
### M047 — Ammunition Factory
```
machine_id: era1_machine_ammunition_factory_mk1
name: Ammunition Factory
category: manufacturing
tier: 2
description: Loads casings, cores, propellant into ammo.
function: Munitions.
size: 3x4
power: 280 kW
power_type: electrical
inputs:
  - projectile cores
  - casings
  - propellant
outputs:
  - ammunition
  - mines
recipe_categories:
  - ammunition
fluid_ports:
  - in:propellant
technology_unlock: era1_tech_defense_industry
purity_behavior: N/A
maintenance: Explosion hazard if propellant mishandled.
upgrade_path: Mk2
animation: Belt-fed loaders
```
### M048 — Missile Factory
```
machine_id: era1_machine_missile_factory_mk1
name: Missile Factory
category: manufacturing
tier: 3
description: Assembles guided missiles.
function: Missile production.
size: 4x4
power: 450 kW
power_type: electrical
inputs:
  - advanced polymer
  - targeting
  - propellant
outputs:
  - guided missiles
recipe_categories:
  - missiles
fluid_ports:
  - in:propellant
technology_unlock: era1_tech_defense_industry
purity_behavior: Requires targeting modules of sufficient grade.
maintenance: High security maintenance.
upgrade_path: Mk2
animation: Rail assembly
```
### M049 — Construction Fabricator
```
machine_id: era1_machine_construction_fabricator
name: Construction Fabricator
category: manufacturing
tier: 2
description: Produces construction site kits, scaffolds, barriers.
function: Construction parts.
size: 3x3
power: 200 kW
power_type: electrical
inputs:
  - frames
  - panels
outputs:
  - scaffold
  - barriers
  - kits
recipe_categories:
  - construction
fluid_ports:
  []
technology_unlock: era1_tech_advanced_automation
purity_behavior: N/A
maintenance: Standard.
upgrade_path: Mk2
animation: Panel stapler
```
### M050 — Modular / Military Assembly Bay
```
machine_id: era1_machine_military_assembly_bay_mk1
name: Modular / Military Assembly Bay
category: manufacturing
tier: 3
description: Final-assembles turrets and defense systems (also modular bay role).
function: Defense final assembly.
size: 5x5
power: 600 kW
power_type: electrical
inputs:
  - frames
  - targeting
  - ammo
outputs:
  - turrets
  - fortress cores
recipe_categories:
  - military_assembly
  - modular_assembly
fluid_ports:
  []
technology_unlock: era1_tech_defense_industry
purity_behavior: N/A
maintenance: Large footprint power spikes.
upgrade_path: Mk2
animation: Multi-pad bay
```

# SECTION 5 — ELECTRONICS (M051–M058)

### M051 — Electronics Printer Mk1
```
machine_id: era1_machine_electronics_printer_mk1
name: Electronics Printer Mk1
category: electronics
tier: 1
description: Prints basic substrates and simple modules.
function: Entry electronics.
size: 3x3
power: 180 kW
power_type: electrical
inputs:
  - silicon
  - foil
  - wire
outputs:
  - substrates
  - basic modules
recipe_categories:
  - electronics_print
fluid_ports:
  - in:upw optional
technology_unlock: era1_tech_electronics
purity_behavior: Prefers higher purity silicon.
maintenance: Nozzle clog.
upgrade_path: Mk2
animation: Print gantry
```
### M052 — Electronics Printer Mk2
```
machine_id: era1_machine_electronics_printer_mk2
name: Electronics Printer Mk2
category: electronics
tier: 2
description: Logic boards, data storage, controllers.
function: Mid electronics.
size: 3x3
power: 320 kW
power_type: electrical
inputs:
  - circuits
  - silicon
  - modules
outputs:
  - logic boards
  - controllers
recipe_categories:
  - advanced_electronics
fluid_ports:
  - in:cleaning_fluid
technology_unlock: era1_tech_advanced_electronics
purity_behavior: Purity floors on wafers.
maintenance: Cleanroom filters.
upgrade_path: Mk3
animation: Multi-layer print
```
### M053 — Circuit Printer Mk1
```
machine_id: era1_machine_circuit_printer_mk1
name: Circuit Printer Mk1
category: electronics
tier: 1
description: Prints basic circuits from substrates and wire.
function: Basic circuits.
size: 2x3
power: 150 kW
power_type: electrical
inputs:
  - substrate
  - wire
outputs:
  - basic circuit
recipe_categories:
  - circuits
fluid_ports:
  []
technology_unlock: era1_tech_electronics
purity_behavior: N/A
maintenance: Trace defects if dirty.
upgrade_path: Circuit Printer Mk2
animation: Trace laser
```
### M054 — Circuit Printer Mk2 / Electronics Printer Mk3
```
machine_id: era1_machine_electronics_printer_mk3
name: Circuit Printer Mk2 / Electronics Printer Mk3
category: electronics
tier: 3
description: High-density circuits and late modules (covers Mk2 circuit + Mk3 electronics roles).
function: Late electronics.
size: 4x3
power: 700 kW
power_type: electrical
inputs:
  - wafers
  - traces
  - UPW
outputs:
  - high density circuits
  - AI modules
recipe_categories:
  - high_density_electronics
fluid_ports:
  - in:upw
technology_unlock: era1_tech_advanced_electronics
purity_behavior: Strict purity.
maintenance: Expensive calibration.
upgrade_path: Era2 lithography
animation: Cleanroom flash
```
### M055 — Semiconductor Processor
```
machine_id: era1_machine_semiconductor_processor
name: Semiconductor Processor
category: electronics
tier: 2
description: Wafer processing (logic/memory/sensor wafers).
function: Semiconductors.
size: 3x3
power: 400 kW
power_type: electrical
inputs:
  - refined silicon
  - powders
  - UPW
outputs:
  - wafers
recipe_categories:
  - semiconductors
fluid_ports:
  - in:upw
  - in:cleaning
technology_unlock: era1_tech_advanced_electronics
purity_behavior: +purity on success; scrap on fail.
maintenance: Yield variance.
upgrade_path: Mk2
animation: Wafer arm
```
### M056 — Data Processor / Research Analyzer
```
machine_id: era1_machine_research_analyzer
name: Data Processor / Research Analyzer
category: electronics
tier: 2
description: Processes research samples and diagnostics (dual-use data processor).
function: Data analysis hardware.
size: 3x2
power: 250 kW
power_type: electrical
inputs:
  - sensors
  - samples
  - storage
outputs:
  - analysis results
  - analyzers
recipe_categories:
  - data_processing
  - research_support
fluid_ports:
  []
technology_unlock: era1_tech_research_infrastructure
purity_behavior: N/A
maintenance: Fan dust.
upgrade_path: Mk2
animation: Server blink
```
### M057 — Battery Processor
```
machine_id: era1_machine_battery_processor_mk1
name: Battery Processor
category: electronics
tier: 1
description: Forms energy cells from graphite, plates, electrolyte.
function: Battery cells.
size: 3x3
power: 200 kW
power_type: electrical
inputs:
  - graphite
  - conductive plate
  - electrolyte
outputs:
  - energy cells
recipe_categories:
  - batteries
fluid_ports:
  - in:electrolyte
technology_unlock: era1_tech_power_systems
purity_behavior: Electrolyte purity affects cell grade.
maintenance: Acid corrosion.
upgrade_path: Battery Assembler
animation: Cell stacker
```
### M058 — Power Component Factory
```
machine_id: era1_machine_power_component_factory_mk1
name: Power Component Factory
category: electronics
tier: 2
description: Transformers, relays, HV parts, conduits.
function: Power components.
size: 3x4
power: 280 kW
power_type: electrical
inputs:
  - steel
  - wire
  - insulation
outputs:
  - transformer cores
  - relays
  - HV cable
recipe_categories:
  - power_components
fluid_ports:
  []
technology_unlock: era1_tech_power_systems
purity_behavior: N/A
maintenance: Copper winding scrap.
upgrade_path: Mk2
animation: Coil winders
```

# SECTION 6 — LOGISTICS (M059–M066)

### M059 — Storage Container
```
machine_id: era1_machine_storage_container
name: Storage Container
category: logistics
tier: 1
description: Solid item storage building (crafted entity).
function: Item buffering.
size: 2x2
power: 5 kW
power_type: electrical
inputs:
  - items
outputs:
  - items
recipe_categories:
  - storage
fluid_ports:
  []
technology_unlock: era1_tech_industrial_automation
purity_behavior: N/A
maintenance: None.
upgrade_path: Smart Storage
animation: Idle
```
### M060 — Fluid Storage Tank
```
machine_id: era1_machine_storage_tank
name: Fluid Storage Tank
category: logistics
tier: 1
description: Stores fluids/gases.
function: Fluid buffering.
size: 3x3
power: 10 kW
power_type: electrical
inputs:
  - fluids
outputs:
  - fluids
recipe_categories:
  - fluid_storage
fluid_ports:
  - in/out:fluid
technology_unlock: era1_tech_fluid_engineering
purity_behavior: Can track fluid purity averages.
maintenance: Leak risk if damaged.
upgrade_path: Mk2 larger
animation: Tank gauge
```
### M061 — Conveyor Fabricator
```
machine_id: era1_machine_conveyor_fabricator
name: Conveyor Fabricator
category: logistics
tier: 1
description: Crafts belt segments and belt accessories (optional dedicated crafter; recipes also in assemblers).
function: Logistics part crafting.
size: 3x2
power: 100 kW
power_type: electrical
inputs:
  - plates
  - rubber
outputs:
  - conveyor segments
  - splitters parts
recipe_categories:
  - logistics_fab
fluid_ports:
  []
technology_unlock: era1_tech_industrial_automation
purity_behavior: N/A
maintenance: Standard.
upgrade_path: Mk2
animation: Belt extrude
```
### M062 — Logistics Controller Hub
```
machine_id: era1_machine_logistics_controller
name: Logistics Controller Hub
category: logistics
tier: 2
description: Placeable controller building hosting logistic controller logic.
function: Logistics network brain.
size: 2x2
power: 80 kW
power_type: electrical
inputs:
  - signals
outputs:
  - network control
recipe_categories:
  - logistics_control
fluid_ports:
  []
technology_unlock: era1_tech_systems_science
purity_behavior: N/A
maintenance: Firmware.
upgrade_path: Mk2
animation: Antenna blink
```
### M063 — Sorting Machine
```
machine_id: era1_machine_sorting_machine
name: Sorting Machine
category: logistics
tier: 2
description: Vision/filter sorter for items by type/purity/grade.
function: Automated sorting.
size: 3x3
power: 160 kW
power_type: electrical
inputs:
  - mixed items
outputs:
  - sorted lanes
recipe_categories:
  - sorting
fluid_ports:
  []
technology_unlock: era1_tech_advanced_automation
purity_behavior: Can sort by purity/grade with probes.
maintenance: Camera fouling.
upgrade_path: Mk2
animation: Divert arms
```
### M064 — Pump Station
```
machine_id: era1_machine_fluid_pump_mk1
name: Pump Station
category: logistics
tier: 1
description: Moves fluids through pipes; boosts pressure.
function: Fluid transport.
size: 1x2
power: 60 kW
power_type: electrical
inputs:
  - fluid
outputs:
  - fluid
recipe_categories:
  - pumping
fluid_ports:
  - in:fluid
  - out:fluid
technology_unlock: era1_tech_fluid_engineering
purity_behavior: N/A
maintenance: Cavitation if starved.
upgrade_path: Mk2
animation: Impeller
```
### M065 — Pipe Junction
```
machine_id: era1_machine_pipe_junction
name: Pipe Junction
category: logistics
tier: 1
description: Multi-way pipe junction building.
function: Pipe routing.
size: 1x1
power: 0 kW
power_type: none
inputs:
  - fluid
outputs:
  - fluid
recipe_categories:
  - piping
fluid_ports:
  - multi
technology_unlock: era1_tech_fluid_engineering
purity_behavior: N/A
maintenance: Clogs.
upgrade_path: Smart junction
animation: Flow arrows
```
### M066 — Transport Hub
```
machine_id: era1_machine_transport_hub
name: Transport Hub
category: logistics
tier: 2
description: Hub for loaders/unloaders/drone pads linkage.
function: Transport nexus.
size: 4x4
power: 200 kW
power_type: electrical
inputs:
  - items
outputs:
  - items
recipe_categories:
  - transport
fluid_ports:
  []
technology_unlock: era1_tech_robotics
purity_behavior: N/A
maintenance: Traffic jams if undersized.
upgrade_path: Mk2
animation: Depot anim
```

# SECTION 7 — MILITARY (M067–M072)

### M067 — Military Assembly Bay
```
machine_id: era1_machine_military_assembly_bay_mk1
name: Military Assembly Bay
category: military
tier: 2
description: Final assembly for turrets (shared ID with M050 role; primary military assembler).
function: Turret/defense assembly.
size: 5x5
power: 600 kW
power_type: electrical
inputs:
  - frames
  - electronics
  - ammo
outputs:
  - turrets
recipe_categories:
  - military_assembly
fluid_ports:
  []
technology_unlock: era1_tech_defense_industry
purity_behavior: N/A
maintenance: Security lockdown mode.
upgrade_path: Mk2
animation: Assembly gantry
```
### M068 — Defense Fabricator
```
machine_id: era1_machine_defense_fabricator
name: Defense Fabricator
category: military
tier: 2
description: Builds walls, gates, non-turret defense structures.
function: Defense structures.
size: 3x3
power: 240 kW
power_type: electrical
inputs:
  - armor
  - frames
outputs:
  - walls
  - gates
  - barriers
recipe_categories:
  - defense_fab
fluid_ports:
  []
technology_unlock: era1_tech_defense_industry
purity_behavior: N/A
maintenance: Standard.
upgrade_path: Mk2
animation: Panel press
```
### M069 — Turret Factory
```
machine_id: era1_machine_turret_factory
name: Turret Factory
category: military
tier: 3
description: Specialized high-throughput turret line (optional specialized bay).
function: Turret mass production.
size: 4x5
power: 700 kW
power_type: electrical
inputs:
  - turret frames
  - targeting
outputs:
  - ballistic/missile/laser turrets
recipe_categories:
  - turret_fab
fluid_ports:
  []
technology_unlock: era1_tech_defense_research
purity_behavior: N/A
maintenance: High power draw.
upgrade_path: Mk2
animation: Rotary fixtures
```
### M070 — Radar Construction Unit / Defense Assembly Machine
```
machine_id: era1_machine_defense_assembly_machine
name: Radar Construction Unit / Defense Assembly Machine
category: military
tier: 2
description: Builds radar, repair stations, defense support buildings.
function: Defense support construction.
size: 4x3
power: 350 kW
power_type: electrical
inputs:
  - sensors
  - frames
  - power parts
outputs:
  - radar
  - repair stations
recipe_categories:
  - defense_support
fluid_ports:
  []
technology_unlock: era1_tech_defense_industry
purity_behavior: N/A
maintenance: Standard.
upgrade_path: Mk2
animation: Dish assemble
```
### M071 — Repair System Factory
```
machine_id: era1_machine_repair_system_factory
name: Repair System Factory
category: military
tier: 2
description: Produces repair drone cores/stations/packs.
function: Repair systems.
size: 3x3
power: 260 kW
power_type: electrical
inputs:
  - robot cores
  - repair packs
outputs:
  - repair infrastructure
recipe_categories:
  - repair_systems
fluid_ports:
  []
technology_unlock: era1_tech_defense_industry
purity_behavior: N/A
maintenance: Standard.
upgrade_path: Mk2
animation: Drone dock
```
### M072 — Ammunition Logistics Factory
```
machine_id: era1_machine_ammo_logistics_factory
name: Ammunition Logistics Factory
category: military
tier: 2
description: Boxes ammo and builds ammo hubs/belts.
function: Ammo logistics.
size: 3x4
power: 220 kW
power_type: electrical
inputs:
  - ammunition
  - plates
outputs:
  - ammo boxes
  - ammo hubs
recipe_categories:
  - ammo_logistics
fluid_ports:
  []
technology_unlock: era1_tech_defense_industry
purity_behavior: N/A
maintenance: Explosion-safe zoning.
upgrade_path: Mk2
animation: Crate line
```

# SECTION 8 — RESEARCH (M073–M075)

### M073 — Research Laboratory Mk1
```
machine_id: era1_machine_research_laboratory
name: Research Laboratory Mk1
category: research
tier: 1
description: Consumes industrial samples + data storage to produce science data packs.
function: Primary research.
size: 4x4
power: 200 kW
power_type: electrical
inputs:
  - specific components
  - data storage modules
outputs:
  - engineering/chemical/computational/defense data
recipe_categories:
  - research
fluid_ports:
  []
technology_unlock: era1_tech_research_infrastructure
purity_behavior: N/A
maintenance: Instrument calibration.
upgrade_path: Lab Module / Analyzer
animation: Hologram samples
```
### M074 — Data Analysis Core / Laboratory Module
```
machine_id: era1_machine_laboratory_module
name: Data Analysis Core / Laboratory Module
category: research
tier: 2
description: Expandable analysis core for advanced research recipes.
function: Advanced research.
size: 3x3
power: 350 kW
power_type: electrical
inputs:
  - processors
  - samples
outputs:
  - advanced science items
  - validation units
recipe_categories:
  - advanced_research
fluid_ports:
  []
technology_unlock: era1_tech_research_infrastructure
purity_behavior: N/A
maintenance: Heat density.
upgrade_path: Mk2
animation: Core glow
```
### M075 — Technology Nexus Interface
```
machine_id: era1_machine_tech_nexus_interface
name: Technology Nexus Interface
category: research
tier: 3
description: Late research interface used with Nexus commissioning and Era transition.
function: Era transition research interface.
size: 3x3
power: 500 kW
power_type: electrical
inputs:
  - dossiers
  - nexus diagnostics
outputs:
  - era transition keys
  - records
recipe_categories:
  - nexus_research
  - era_transition
fluid_ports:
  []
technology_unlock: era1_tech_nexus_construction
purity_behavior: N/A
maintenance: Must be near Nexus.
upgrade_path: Era2 uplink
animation: Beam to Nexus
```

# APPENDIX A — Additional machines required by recipe DB

These appear in filled recipes but sit outside the canonical 75 slots.
Keep them — they prevent orphan recipes.

### A01 — Precision Water Processor Mk1
```
machine_id: era1_machine_precision_water_processor_mk1
name: Precision Water Processor Mk1
category: chemical
tier: 2
description: Ultra pure water / optical silicon wash.
function: Precision water.
size: 3x3
power: 350 kW
power_type: electrical
inputs:
  - purified water
  - cartridges
outputs:
  - ultra pure water
  - optical silicon
recipe_categories:
  - precision_water
fluid_ports:
  - in:water
  - out:upw
technology_unlock: era1_tech_precision_chemistry
purity_behavior: +25 purity
maintenance: Cartridge logistics
upgrade_path: Mk2
animation: Sterile chamber
```
### A02 — Reduction Furnace Mk1
```
machine_id: era1_machine_reduction_furnace_mk1
name: Reduction Furnace Mk1
category: processing
tier: 2
description: Produces reduced ferrite with carbon compound gas.
function: Reduction metallurgy.
size: 4x3
power: 650 kW
power_type: thermal
inputs:
  - purified ferrite
  - carbon gas
outputs:
  - reduced ferrite
recipe_categories:
  - reduction
fluid_ports:
  - in:gas
technology_unlock: era1_tech_advanced_metallurgy
purity_behavior: +5 purity
maintenance: Gas seals
upgrade_path: Mk2
animation: Reduction glow
```
### A03 — Carbon Furnace Mk1
```
machine_id: era1_machine_carbon_furnace_mk1
name: Carbon Furnace Mk1
category: processing
tier: 1
description: Graphite and activated carbon.
function: Carbon thermal processing.
size: 3x3
power: 500 kW
power_type: thermal
inputs:
  - carbon powder
  - steam
outputs:
  - graphite
  - activated carbon
recipe_categories:
  - carbon_thermal
fluid_ports:
  - in:steam optional
technology_unlock: era1_tech_carbon_processing
purity_behavior: +purity on graphite
maintenance: Refractory wear
upgrade_path: Mk2
animation: Furnace glow
```
### A04 — Composite Processor Mk1
```
machine_id: era1_machine_composite_processor_mk1
name: Composite Processor Mk1
category: processing
tier: 2
description: Carbon/ceramic composites.
function: Composites.
size: 3x3
power: 240 kW
power_type: electrical
inputs:
  - fiber
  - resin
  - ceramic
outputs:
  - composites
recipe_categories:
  - composites
fluid_ports:
  []
technology_unlock: era1_tech_polymer_science
purity_behavior: Grade sensitive
maintenance: Layup waste
upgrade_path: Mk2
animation: Layup table
```
### A05 — Coating Unit Mk1
```
machine_id: era1_machine_coating_unit_mk1
name: Coating Unit Mk1
category: chemical
tier: 2
description: Applies industrial coatings to plates.
function: Surface coating.
size: 3x2
power: 200 kW
power_type: electrical
inputs:
  - plates
  - coatings
  - solvent
outputs:
  - protected plates
  - coatings
recipe_categories:
  - coating
fluid_ports:
  - in:solvent
technology_unlock: era1_tech_chemical_manufacturing
purity_behavior: N/A
maintenance: VOC hazard
upgrade_path: Mk2
animation: Spray booth
```
### A06 — Chemical Processor Mk1
```
machine_id: era1_machine_chemical_processor_mk1
name: Chemical Processor Mk1
category: chemical
tier: 1
description: Mineral compound/binder and simple chem prep.
function: Prep chemistry.
size: 3x2
power: 140 kW
power_type: electrical
inputs:
  - mineral dust
  - solutions
outputs:
  - mineral compound
  - binder
  - catalyst solution
recipe_categories:
  - chem_prep
fluid_ports:
  - in:fluid
  - out:fluid
technology_unlock: era1_tech_chemical_manufacturing
purity_behavior: +purity on compounds
maintenance: Fouling
upgrade_path: Mk2
animation: Mixer
```
### A07 — Boiler Mk1
```
machine_id: era1_machine_boiler_mk1
name: Boiler Mk1
category: chemical
tier: 1
description: Makes process steam from purified water.
function: Steam.
size: 2x3
power: 300 kW
power_type: thermal
inputs:
  - purified water
outputs:
  - steam
recipe_categories:
  - steam
fluid_ports:
  - in:water
  - out:steam
technology_unlock: era1_tech_fluid_engineering
purity_behavior: N/A
maintenance: Scale buildup
upgrade_path: Mk2
animation: Boiler flame
```
### A08 — Fiber Processor Mk1
```
machine_id: era1_machine_fiber_processor_mk1
name: Fiber Processor Mk1
category: chemical
tier: 2
description: Spins synthetic fiber.
function: Fibers.
size: 3x3
power: 220 kW
power_type: electrical
inputs:
  - resin
  - carbon fiber
outputs:
  - synthetic fiber
recipe_categories:
  - fibers
fluid_ports:
  []
technology_unlock: era1_tech_polymer_science
purity_behavior: N/A
maintenance: Spinneret clogs
upgrade_path: Mk2
animation: Spools
```
### A09 — Component Assembler Mk1
```
machine_id: era1_machine_component_assembler_mk1
name: Component Assembler Mk1
category: manufacturing
tier: 1
description: Pipes, valves, fittings, storage parts.
function: Mech components.
size: 3x2
power: 130 kW
power_type: electrical
inputs:
  - plates
  - pipes
outputs:
  - pipes
  - valves
  - fittings
recipe_categories:
  - component_assembly
fluid_ports:
  []
technology_unlock: era1_tech_fluid_engineering
purity_behavior: N/A
maintenance: Standard
upgrade_path: Mk2
animation: Arm
```
### A10 — Component Processor Mk1
```
machine_id: era1_machine_component_processor_mk1
name: Component Processor Mk1
category: manufacturing
tier: 1
description: Wire/foil/trace bundling and simple formative recipes.
function: Light components.
size: 2x3
power: 100 kW
power_type: electrical
inputs:
  - plates
  - wire
outputs:
  - wire
  - foil bundles
recipe_categories:
  - component_process
fluid_ports:
  []
technology_unlock: era1_tech_electronics
purity_behavior: N/A
maintenance: Standard
upgrade_path: Mk2
animation: Roller
```
### A11 — Motor Assembly Machine Mk1
```
machine_id: era1_machine_motor_assembly_mk1
name: Motor Assembly Machine Mk1
category: manufacturing
tier: 2
description: Builds industrial/heavy motors and rotation motors.
function: Motors.
size: 3x3
power: 200 kW
power_type: electrical
inputs:
  - servos
  - wire
  - assemblies
outputs:
  - motors
recipe_categories:
  - motors
fluid_ports:
  []
technology_unlock: era1_tech_industrial_automation
purity_behavior: Grade precision helps
maintenance: Balancing needed
upgrade_path: Mk2
animation: Rotor spin
```
### A12 — Precision Component Fabricator Mk1
```
machine_id: era1_machine_precision_component_fabricator_mk1
name: Precision Component Fabricator Mk1
category: manufacturing
tier: 2
description: Electronics printer parts and precision kits.
function: Precision kits.
size: 3x3
power: 200 kW
power_type: electrical
inputs:
  - housings
  - circuits
  - foil
outputs:
  - printer parts
recipe_categories:
  - precision_kits
fluid_ports:
  []
technology_unlock: era1_tech_electronics
purity_behavior: Precision grade
maintenance: Calibration
upgrade_path: Mk2
animation: Micro arms
```
### A13 — Electronics Assembler Mk1
```
machine_id: era1_machine_electronics_assembler_mk1
name: Electronics Assembler Mk1
category: electronics
tier: 1
description: Assembles modules from circuits (sensors, regulators, boards).
function: Electronics assembly.
size: 3x3
power: 160 kW
power_type: electrical
inputs:
  - circuits
  - plates
  - glass
outputs:
  - modules
recipe_categories:
  - electronics_assembly
fluid_ports:
  []
technology_unlock: era1_tech_electronics
purity_behavior: N/A
maintenance: ESD care
upgrade_path: Mk2
animation: Pick-and-place
```
### A14 — Battery Assembler Mk1
```
machine_id: era1_machine_battery_assembler_mk1
name: Battery Assembler Mk1
category: electronics
tier: 1
description: Packs cells into battery packs/storage modules.
function: Battery packs.
size: 3x2
power: 150 kW
power_type: electrical
inputs:
  - energy cells
  - frames
outputs:
  - battery packs
  - storage modules
recipe_categories:
  - battery_assembly
fluid_ports:
  []
technology_unlock: era1_tech_power_systems
purity_behavior: Cell grade→pack grade
maintenance: Thermal paste
upgrade_path: Mk2
animation: Pack line
```
### A15 — Construction Site
```
machine_id: era1_machine_construction_site
name: Construction Site
category: manufacturing
tier: 3
description: World site for Nexus multi-stage construction recipes.
function: Nexus construction.
size: 10x10
power: 2 MW
power_type: electrical
inputs:
  - nexus parts
outputs:
  - nexus stages
  - PFN
recipe_categories:
  - nexus_construction
fluid_ports:
  []
technology_unlock: era1_tech_nexus_construction
purity_behavior: N/A
maintenance: Scaffolding phases
upgrade_path: —
animation: Cranes + hologram
```
### A16 — Energy Facility
```
machine_id: era1_machine_energy_facility
name: Energy Facility
category: power
tier: 3
description: Builds Nexus power cores and heavy energy systems.
function: Nexus energy crafting.
size: 6x6
power: 1 MW
power_type: electrical
inputs:
  - transformers
  - cells
  - relays
outputs:
  - nexus power core
  - backup systems
recipe_categories:
  - nexus_power
fluid_ports:
  []
technology_unlock: era1_tech_nexus_construction
purity_behavior: N/A
maintenance: High voltage safety
upgrade_path: —
animation: Arc flashes
```
### A17 — Advanced Electronics Printer
```
machine_id: era1_machine_advanced_electronics_printer
name: Advanced Electronics Printer
category: electronics
tier: 3
description: Nexus computational cores and top electronics.
function: Top electronics.
size: 4x4
power: 900 kW
power_type: electrical
inputs:
  - logic boards
  - data
  - controllers
outputs:
  - nexus compute
  - comm arrays
recipe_categories:
  - nexus_electronics
fluid_ports:
  - in:upw
technology_unlock: era1_tech_nexus_construction
purity_behavior: Strict purity
maintenance: Cleanroom
upgrade_path: —
animation: Lithography
```
### A18 — Research Fabricator
```
machine_id: era1_machine_research_fabricator
name: Research Fabricator
category: research
tier: 2
description: Crafts optimization modules and research hardware.
function: Research manufacturing.
size: 3x3
power: 220 kW
power_type: electrical
inputs:
  - data
  - controllers
outputs:
  - optimization modules
recipe_categories:
  - research_fab
fluid_ports:
  []
technology_unlock: era1_tech_systems_science
purity_behavior: N/A
maintenance: Standard
upgrade_path: Mk2
animation: Proto bench
```
### A19 — Calibration Station
```
machine_id: era1_machine_calibration_station
name: Calibration Station
category: manufacturing
tier: 2
description: Recalibrates precision modules/machines.
function: Calibration.
size: 2x3
power: 120 kW
power_type: electrical
inputs:
  - calibration units
outputs:
  - calibrated modules
  - nexus calibration
recipe_categories:
  - calibration
fluid_ports:
  []
technology_unlock: era1_tech_precision_manufacturing
purity_behavior: Restores grade potential
maintenance: Reference standards drift
upgrade_path: Mk2
animation: Laser jig
```
### A20 — Compact Generator
```
machine_id: era1_machine_compact_generator
name: Compact Generator
category: power
tier: 2
description: Placeable generator (crafted power building).
function: Power generation.
size: 3x3
power: 0 kW gen / uses fuel recipes
power_type: electrical_output
inputs:
  - fuel or kinetic feeds
outputs:
  - electrical power
recipe_categories:
  - power_gen
fluid_ports:
  []
technology_unlock: era1_tech_power_systems
purity_behavior: N/A
maintenance: Bearing wear
upgrade_path: Mk2
animation: Rotor
```
### A21 — Solar Panel Mk1
```
machine_id: era1_machine_solar_panel_mk1
name: Solar Panel Mk1
category: power
tier: 1
description: Placeable solar generator.
function: Solar power.
size: 4x3
power: 0 (produces ~12+ e/s class)
power_type: electrical_output
inputs:
  - sunlight
outputs:
  - power
recipe_categories:
  - solar
fluid_ports:
  []
technology_unlock: era1_tech_power_systems
purity_behavior: N/A
maintenance: Dust on panels
upgrade_path: Mk2
animation: Sun track
```

# ID ALIASES (recipe reconciliation)

| User / Display Name | Canonical machine_id |
|---|---|
| Water Condenser Mk1 | `era1_machine_atmospheric_condenser_mk1` |
| Gas Separator Mk1 | `era1_machine_atmospheric_separator_mk1` |
| Material Dryer | `era1_machine_material_processor_mk1` |
| Industrial Press | `era1_machine_hydraulic_press_mk1` |
| Material Roller | `era1_machine_precision_roller_mk1` |
| Modular Assembly Bay | `era1_machine_military_assembly_bay_mk1` |
| Data Analysis Core | `era1_machine_laboratory_module` |
| Circuit Printer Mk2 | covered by `era1_machine_electronics_printer_mk3` |
| Ferrite Extraction Drill | `era1_machine_ferrite_drill_mk1` (recipes may still say extract_* — bind in data) |

# COUNTS

| Section | Slots |
|---|---|
| Extraction M001–008 | 8 |
| Processing M009–023 | 15 |
| Chemical M024–035 | 12 |
| Manufacturing M036–050 | 15 |
| Electronics M051–058 | 8 |
| Logistics M059–066 | 8 |
| Military M067–072 | 6 |
| Research M073–075 | 3 |
| **Canonical total** | **75** |
| Appendix extras | 21 |
| **Grand total defined** | **96** |
