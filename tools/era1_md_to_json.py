#!/usr/bin/env python3
"""Parse Era 1 markdown bible into JSON data packs."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "era1"
OUT = ROOT / "assets" / "data" / "era1"


def fence_blocks(text: str) -> list[str]:
    return re.findall(r"```(?:\w*)\n(.*?)```", text, flags=re.S)


def parse_kv_block(block: str) -> dict:
    """Parse simple key: value / nested list YAML-ish blocks from our docs."""
    data: dict = {}
    current_list: str | None = None
    for raw in block.splitlines():
        line = raw.rstrip()
        if not line.strip() or line.strip().startswith("#"):
            continue
        # list item under current key (allow broken unindented `- { id: ... }` lines)
        m_list = re.match(r"^\s*-\s+(.*)$", line)
        if m_list and current_list:
            val = m_list.group(1).strip()
            # { id: x, amount: n }
            m_obj = re.match(
                r"\{\s*id:\s*([A-Za-z0-9_]+)\s*,\s*amount:\s*([0-9.]+)\s*\}", val
            )
            if m_obj:
                data.setdefault(current_list, []).append(
                    {"id": m_obj.group(1), "amount": float(m_obj.group(2))}
                )
            elif val in ("[]",):
                data[current_list] = []
            else:
                data.setdefault(current_list, []).append(val.strip("\"'"))
            continue
        m = re.match(r"^([A-Za-z0-9_]+):\s*(.*)$", line)
        if not m:
            continue
        key, rest = m.group(1), m.group(2).strip()
        if rest == "" or rest == "|" or rest.startswith("|"):
            current_list = key
            data[key] = []
            continue
        current_list = None
        if rest == "[]":
            data[key] = []
        elif rest.startswith("{") and rest.endswith("}"):
            # { electrical: 120 } or { thermal: 400 }
            inner = rest[1:-1].strip()
            obj = {}
            if inner:
                for part in inner.split(","):
                    if ":" not in part:
                        continue
                    k, v = part.split(":", 1)
                    k, v = k.strip(), v.strip()
                    try:
                        obj[k] = float(v) if "." in v else int(v)
                    except ValueError:
                        obj[k] = v
            data[key] = obj
        elif re.fullmatch(r"-?[0-9]+(\.[0-9]+)?", rest):
            data[key] = float(rest) if "." in rest else int(rest)
        elif rest.lower() in ("true", "false"):
            data[key] = rest.lower() == "true"
        else:
            data[key] = rest.strip("\"'")
            # allow following list for this key
            if key in (
                "inputs",
                "outputs",
                "waste_outputs",
                "unlocks",
                "prerequisites",
                "recipe_categories",
                "fluid_ports",
                "produced_by",
                "used_in",
                "science_types",
            ):
                current_list = key
                if key not in data or not isinstance(data[key], list):
                    # value might be inline description — keep string unless empty list expected
                    if rest in ("",):
                        data[key] = []
    return data


def load_items() -> list[dict]:
    items: dict[str, dict] = {}
    for name in ("ITEM_DATABASE_PATCH.md", "ITEM_DATABASE_SUPPLEMENT.md"):
        text = (DOC / name).read_text(encoding="utf-8")
        for block in fence_blocks(text):
            if "id:" not in block and "recipe_id:" not in block:
                continue
            d = parse_kv_block(block)
            iid = d.get("id") or d.get("recipe_id")
            if not iid or not str(iid).startswith("era1_"):
                continue
            if str(iid).startswith("era1_recipe_") or str(iid).startswith("era1_tech_"):
                continue
            if str(iid).startswith("era1_machine_"):
                continue
            entry = {
                "id": iid,
                "name": d.get("name", iid),
                "era": d.get("era", 1),
                "family": d.get("family", "misc"),
                "category": d.get("category", "material"),
                "stack_size": d.get("stack_size", 100),
                "purity_supported": bool(d.get("purity_supported", False)),
                "grade_supported": bool(d.get("grade_supported", False)),
                "description": d.get("description", ""),
                "state": d.get("state"),  # liquid/gas for fluids
            }
            # fluids/gases/waste by prefix
            if iid.startswith("era1_fluid_") or iid.startswith("era1_gas_"):
                entry["kind"] = "fluid" if iid.startswith("era1_fluid_") else "gas"
            elif iid.startswith("era1_waste_"):
                entry["kind"] = "waste"
            else:
                entry["kind"] = "item"
            items[iid] = entry
    # Seed core raws if missing
    seeds = [
        ("era1_raw_ferrite_ore", "Ferrite Ore", "raw"),
        ("era1_raw_conductive_ore", "Conductive Ore", "raw"),
        ("era1_raw_carbon_deposit", "Carbon Deposit", "raw"),
        ("era1_raw_silicate_rock", "Silicate Rock", "raw"),
        ("era1_fluid_raw_hydrocarbon", "Raw Hydrocarbon", "fluid"),
        ("era1_material_ferrite_plate", "Ferrite Plate", "material"),
        ("era1_material_conductive_plate", "Conductive Plate", "material"),
        ("era1_material_crushed_ferrite", "Crushed Ferrite", "material"),
        ("era1_material_purified_ferrite", "Purified Ferrite", "material"),
        ("era1_material_ferrite_powder", "Ferrite Powder", "material"),
        ("era1_material_carbon_powder", "Carbon Powder", "material"),
        ("era1_fluid_purified_water", "Purified Water", "fluid"),
        ("era1_fluid_condensed_water", "Condensed Water", "fluid"),
        ("era1_gas_atmospheric_mix", "Atmospheric Mix", "gas"),
        ("era1_science_engineering_data", "Engineering Data", "science"),
        ("era1_science_chemical_data", "Chemical Data", "science"),
        ("era1_science_computational_data", "Computational Data", "science"),
        ("era1_science_defense_data", "Defense Data", "science"),
        ("era1_military_standard_ammunition", "Standard Ammunition", "military"),
        ("era1_military_ap_ammunition", "AP Ammunition", "military"),
        ("era1_military_guided_missile", "Guided Missile", "military"),
    ]
    for iid, name, kind in seeds:
        if iid not in items:
            items[iid] = {
                "id": iid,
                "name": name,
                "era": 1,
                "family": kind,
                "category": kind,
                "stack_size": 200 if kind != "fluid" else 0,
                "purity_supported": kind in ("raw", "material"),
                "grade_supported": kind in ("material", "military"),
                "description": name,
                "kind": "fluid" if iid.startswith("era1_fluid_") else (
                    "gas" if iid.startswith("era1_gas_") else "item"
                ),
            }
    return sorted(items.values(), key=lambda x: x["id"])


def load_recipes() -> list[dict]:
    recipes: dict[str, dict] = {}
    files = [
        "RECIPE_CORE_FIXES.md",
        "RECIPE_GAPS_108-200.md",
        "RECIPE_GAPS_214-350.md",
        "RECIPE_GAPS_361-500.md",
    ]
    for name in files:
        path = DOC / name
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        for block in fence_blocks(text):
            if "recipe_id:" not in block:
                continue
            d = parse_kv_block(block)
            rid = d.get("recipe_id")
            if not rid:
                continue
            power = d.get("power_consumption") or {}
            if isinstance(power, (int, float)):
                power = {"electrical": power}
            recipes[rid] = {
                "id": rid,
                "name": d.get("name", rid),
                "category": d.get("category", "general"),
                "machine": d.get("machine", ""),
                "inputs": d.get("inputs") if isinstance(d.get("inputs"), list) else [],
                "outputs": d.get("outputs") if isinstance(d.get("outputs"), list) else [],
                "waste_outputs": d.get("waste_outputs")
                if isinstance(d.get("waste_outputs"), list)
                else [],
                "processing_time": float(d.get("processing_time") or 1),
                "power_consumption": power,
                "purity_effect": float(d.get("purity_effect") or 0),
                "grade_effect": str(d.get("grade_effect") or "none"),
                "technology_unlock": d.get("technology_unlock") or "era1_tech_basic_recovery",
                "description": d.get("description") or "",
            }
    # Hand-authored core extraction/processing recipes missing from MD fences
    core = [
        {
            "id": "era1_recipe_extract_ferrite",
            "name": "Extract Ferrite Ore",
            "category": "mining",
            "machine": "era1_machine_ferrite_drill_mk1",
            "inputs": [],
            "outputs": [{"id": "era1_raw_ferrite_ore", "amount": 1}],
            "waste_outputs": [],
            "processing_time": 1.0,
            "power_consumption": {"electrical": 50},
            "purity_effect": 0,
            "grade_effect": "none",
            "technology_unlock": "era1_tech_basic_extraction",
            "description": "Mine ferrite from veins.",
            "extracts": "ferrite",
        },
        {
            "id": "era1_recipe_extract_conductive",
            "name": "Extract Conductive Ore",
            "category": "mining",
            "machine": "era1_machine_conductive_drill_mk1",
            "inputs": [],
            "outputs": [{"id": "era1_raw_conductive_ore", "amount": 1}],
            "waste_outputs": [],
            "processing_time": 1.0,
            "power_consumption": {"electrical": 60},
            "purity_effect": 0,
            "grade_effect": "none",
            "technology_unlock": "era1_tech_basic_extraction",
            "description": "Mine conductive ore.",
            "extracts": "conductive",
        },
        {
            "id": "era1_recipe_extract_carbon",
            "name": "Extract Carbon Deposit",
            "category": "mining",
            "machine": "era1_machine_carbon_drill_mk1",
            "inputs": [],
            "outputs": [{"id": "era1_raw_carbon_deposit", "amount": 1}],
            "waste_outputs": [],
            "processing_time": 1.0,
            "power_consumption": {"electrical": 55},
            "purity_effect": 0,
            "grade_effect": "none",
            "technology_unlock": "era1_tech_basic_extraction",
            "description": "Mine carbon deposits.",
            "extracts": "carbon",
        },
        {
            "id": "era1_recipe_extract_silicate",
            "name": "Extract Silicate Rock",
            "category": "mining",
            "machine": "era1_machine_silicate_drill_mk1",
            "inputs": [],
            "outputs": [{"id": "era1_raw_silicate_rock", "amount": 1}],
            "waste_outputs": [],
            "processing_time": 1.0,
            "power_consumption": {"electrical": 55},
            "purity_effect": 0,
            "grade_effect": "none",
            "technology_unlock": "era1_tech_basic_extraction",
            "description": "Mine silicate rock.",
            "extracts": "silicate",
        },
        {
            "id": "era1_recipe_extract_hydrocarbon",
            "name": "Extract Raw Hydrocarbon",
            "category": "mining",
            "machine": "era1_machine_hydrocarbon_pump_mk1",
            "inputs": [],
            "outputs": [{"id": "era1_fluid_raw_hydrocarbon", "amount": 1}],
            "waste_outputs": [],
            "processing_time": 1.0,
            "power_consumption": {"electrical": 100},
            "purity_effect": 0,
            "grade_effect": "none",
            "technology_unlock": "era1_tech_hydrocarbon_refining",
            "description": "Pump hydrocarbons.",
            "extracts": "hydrocarbon",
        },
        {
            "id": "era1_recipe_crush_ferrite",
            "name": "Crush Ferrite Ore",
            "category": "crushing",
            "machine": "era1_machine_crusher_mk1",
            "inputs": [{"id": "era1_raw_ferrite_ore", "amount": 10}],
            "outputs": [{"id": "era1_material_crushed_ferrite", "amount": 8}],
            "waste_outputs": [{"id": "era1_waste_stone_dust", "amount": 2}],
            "processing_time": 5.0,
            "power_consumption": {"mechanical": 120},
            "purity_effect": 0,
            "grade_effect": "none",
            "technology_unlock": "era1_tech_material_processing",
            "description": "Crush ferrite ore.",
        },
        {
            "id": "era1_recipe_purify_ferrite",
            "name": "Ferrite Separation",
            "category": "purification",
            "machine": "era1_machine_ore_purifier_mk1",
            "inputs": [
                {"id": "era1_material_crushed_ferrite", "amount": 10},
                {"id": "era1_fluid_purified_water", "amount": 2},
            ],
            "outputs": [{"id": "era1_material_purified_ferrite", "amount": 8}],
            "waste_outputs": [{"id": "era1_waste_mineral_slurry", "amount": 1}],
            "processing_time": 8.0,
            "power_consumption": {"electrical": 200},
            "purity_effect": 10,
            "grade_effect": "none",
            "technology_unlock": "era1_tech_basic_metallurgy",
            "description": "Purify crushed ferrite.",
        },
        {
            "id": "era1_recipe_ferrite_powder",
            "name": "Ferrite Drying",
            "category": "drying",
            "machine": "era1_machine_material_processor_mk1",
            "inputs": [{"id": "era1_material_purified_ferrite", "amount": 10}],
            "outputs": [{"id": "era1_material_ferrite_powder", "amount": 10}],
            "waste_outputs": [],
            "processing_time": 5.0,
            "power_consumption": {"electrical": 100},
            "purity_effect": 0,
            "grade_effect": "industrial",
            "technology_unlock": "era1_tech_basic_metallurgy",
            "description": "Dry purified ferrite to powder.",
        },
        {
            "id": "era1_recipe_smelt_ferrite_plate",
            "name": "Ferrite Smelting",
            "category": "smelting",
            "machine": "era1_machine_thermal_smelter_mk1",
            "inputs": [
                {"id": "era1_material_ferrite_powder", "amount": 10},
                {"id": "era1_material_carbon_powder", "amount": 2},
            ],
            "outputs": [{"id": "era1_material_ferrite_plate", "amount": 8}],
            "waste_outputs": [{"id": "era1_waste_carbon_residue", "amount": 1}],
            "processing_time": 12.0,
            "power_consumption": {"thermal": 400},
            "purity_effect": -2,
            "grade_effect": "industrial",
            "technology_unlock": "era1_tech_basic_metallurgy",
            "description": "Smelt ferrite plates.",
        },
        {
            "id": "era1_recipe_crush_carbon",
            "name": "Carbon Crushing",
            "category": "crushing",
            "machine": "era1_machine_crusher_mk1",
            "inputs": [{"id": "era1_raw_carbon_deposit", "amount": 10}],
            "outputs": [{"id": "era1_material_carbon_powder", "amount": 8}],
            "waste_outputs": [{"id": "era1_waste_carbon_residue", "amount": 2}],
            "processing_time": 5.0,
            "power_consumption": {"mechanical": 120},
            "purity_effect": 0,
            "grade_effect": "none",
            "technology_unlock": "era1_tech_material_processing",
            "description": "Crush carbon deposits.",
        },
        {
            "id": "era1_recipe_structural_frame",
            "name": "Structural Frame",
            "category": "assembly",
            "machine": "era1_machine_assembler_mk1",
            "inputs": [
                {"id": "era1_material_ferrite_plate", "amount": 5},
                {"id": "era1_material_reinforced_ferrite", "amount": 2},
            ],
            "outputs": [{"id": "era1_component_structural_frame", "amount": 1}],
            "waste_outputs": [],
            "processing_time": 10.0,
            "power_consumption": {"electrical": 120},
            "purity_effect": 0,
            "grade_effect": "industrial",
            "technology_unlock": "era1_tech_structural_engineering",
            "description": "Assemble structural frames.",
        },
        {
            "id": "era1_recipe_standard_ammunition",
            "name": "Standard Ammunition",
            "category": "military",
            "machine": "era1_machine_ammunition_factory_mk1",
            "inputs": [
                {"id": "era1_military_projectile_core", "amount": 10},
                {"id": "era1_military_ammo_casing", "amount": 10},
                {"id": "era1_fluid_ballistic_propellant", "amount": 5},
            ],
            "outputs": [{"id": "era1_military_standard_ammunition", "amount": 50}],
            "waste_outputs": [],
            "processing_time": 15.0,
            "power_consumption": {"electrical": 280},
            "purity_effect": 0,
            "grade_effect": "industrial",
            "technology_unlock": "era1_tech_defense_industry",
            "description": "Load standard ballistic ammunition.",
        },
    ]
    # ensure waste/item ids exist via soft refs; add missing simplified recipes
    for r in core:
        recipes.setdefault(r["id"], r)
    # simplify structural frame if reinforced missing — alternate
    recipes["era1_recipe_structural_frame_basic"] = {
        "id": "era1_recipe_structural_frame_basic",
        "name": "Structural Frame (Basic)",
        "category": "assembly",
        "machine": "era1_machine_assembler_mk1",
        "inputs": [{"id": "era1_material_ferrite_plate", "amount": 8}],
        "outputs": [{"id": "era1_component_structural_frame", "amount": 1}],
        "waste_outputs": [],
        "processing_time": 8.0,
        "power_consumption": {"electrical": 100},
        "purity_effect": 0,
        "grade_effect": "industrial",
        "technology_unlock": "era1_tech_structural_engineering",
        "description": "Basic frame from plates only.",
    }
    return sorted(recipes.values(), key=lambda x: x["id"])


def load_machines() -> list[dict]:
    machines: dict[str, dict] = {}
    text = (DOC / "MACHINE_DATABASE.md").read_text(encoding="utf-8")
    for block in fence_blocks(text):
        if "machine_id:" not in block:
            continue
        d = parse_kv_block(block)
        mid = d.get("machine_id")
        if not mid:
            continue
        # parse size WxH
        size = d.get("size", "3x3")
        w, h = 3, 3
        if isinstance(size, str) and "x" in size.lower():
            try:
                a, b = size.lower().split("x", 1)
                w, h = int(float(a)), int(float(b))
            except ValueError:
                pass
        power_str = str(d.get("power", "100 kW"))
        m = re.search(r"([0-9.]+)", power_str)
        power_kw = float(m.group(1)) if m else 100.0
        if "MW" in power_str.upper() and "kW" not in power_str:
            power_kw *= 1000.0
        cats = d.get("recipe_categories")
        if not isinstance(cats, list):
            cats = []
        machines[mid] = {
            "id": mid,
            "name": d.get("name", mid),
            "category": d.get("category", "processing"),
            "tier": int(d.get("tier") or 1),
            "description": d.get("description") or "",
            "function": d.get("function") or "",
            "size": [w, h],
            "power_kw": power_kw,
            "power_type": d.get("power_type") or "electrical",
            "recipe_categories": cats,
            "fluid_ports": d.get("fluid_ports") if isinstance(d.get("fluid_ports"), list) else [],
            "technology_unlock": d.get("technology_unlock") or "era1_tech_basic_recovery",
            "purity_behavior": d.get("purity_behavior") or "",
            "maintenance": d.get("maintenance") or "",
            "upgrade_path": d.get("upgrade_path") or "",
            "animation": d.get("animation") or "",
            "placeable": True,
        }
    # Ensure core machines exist
    defaults = [
        ("era1_machine_ferrite_drill_mk1", "Ferrite Drill Mk1", "extraction", "mining", 50, "era1_tech_basic_extraction", [3, 3]),
        ("era1_machine_conductive_drill_mk1", "Conductive Drill Mk1", "extraction", "mining", 60, "era1_tech_basic_extraction", [3, 3]),
        ("era1_machine_carbon_drill_mk1", "Carbon Drill Mk1", "extraction", "mining", 55, "era1_tech_basic_extraction", [3, 3]),
        ("era1_machine_silicate_drill_mk1", "Silicate Drill Mk1", "extraction", "mining", 55, "era1_tech_basic_extraction", [3, 3]),
        ("era1_machine_hydrocarbon_pump_mk1", "Hydrocarbon Pump Mk1", "extraction", "mining", 100, "era1_tech_hydrocarbon_refining", [3, 5]),
        ("era1_machine_crusher_mk1", "Crusher Mk1", "processing", "crushing", 120, "era1_tech_material_processing", [3, 3]),
        ("era1_machine_ore_purifier_mk1", "Ore Purifier Mk1", "processing", "purification", 200, "era1_tech_basic_metallurgy", [3, 4]),
        ("era1_machine_material_processor_mk1", "Material Processor Mk1", "processing", "drying", 100, "era1_tech_basic_metallurgy", [3, 2]),
        ("era1_machine_thermal_smelter_mk1", "Thermal Smelter Mk1", "processing", "smelting", 400, "era1_tech_basic_metallurgy", [3, 3]),
        ("era1_machine_assembler_mk1", "Assembler Mk1", "manufacturing", "assembly", 120, "era1_tech_industrial_automation", [3, 3]),
        ("era1_machine_atmospheric_intake_mk1", "Atmospheric Intake Mk1", "extraction", "atmosphere", 75, "era1_tech_fluid_engineering", [2, 2]),
        ("era1_machine_atmospheric_condenser_mk1", "Water Condenser Mk1", "chemical", "water", 150, "era1_tech_fluid_engineering", [3, 3]),
        ("era1_machine_water_purifier_mk1", "Water Purifier Mk1", "chemical", "water_purification", 200, "era1_tech_fluid_engineering", [3, 3]),
        ("era1_machine_research_laboratory", "Research Laboratory", "research", "research", 200, "era1_tech_research_infrastructure", [4, 4]),
        ("era1_machine_ammunition_factory_mk1", "Ammunition Factory", "military", "ammunition", 280, "era1_tech_defense_industry", [3, 4]),
        ("era1_machine_construction_site", "Construction Site", "manufacturing", "nexus_construction", 2000, "era1_tech_nexus_construction", [10, 10]),
        ("era1_machine_ballistic_turret", "Ballistic Turret", "military", "defense", 40, "era1_tech_defense_industry", [2, 2]),
        ("era1_machine_wall", "Defensive Wall", "military", "defense", 0, "era1_tech_defense_industry", [1, 1]),
        ("era1_machine_reinforced_wall", "Reinforced Wall", "military", "defense", 0, "era1_tech_defense_industry", [1, 1]),
        ("era1_machine_charge_cannon", "Charge Cannon", "military", "defense", 80, "era1_tech_defense_research", [2, 2]),
        ("era1_machine_laser_turret", "Laser Turret", "military", "defense", 200, "era1_tech_laser_defense", [2, 2]),
        ("era1_machine_planetary_nexus", "Planetary Fabrication Nexus", "nexus", "nexus", 0, "era1_tech_era_transition", [20, 20]),
    ]
    for mid, name, cat, rcat, kw, tech, size in defaults:
        machines.setdefault(
            mid,
            {
                "id": mid,
                "name": name,
                "category": cat,
                "tier": 1,
                "description": name,
                "function": name,
                "size": size,
                "power_kw": kw,
                "power_type": "electrical",
                "recipe_categories": [rcat],
                "fluid_ports": [],
                "technology_unlock": tech,
                "purity_behavior": "",
                "maintenance": "",
                "upgrade_path": "",
                "animation": "",
                "placeable": True,
            },
        )
    return sorted(machines.values(), key=lambda x: x["id"])


def load_techs() -> list[dict]:
    techs: dict[str, dict] = {}
    text = (DOC / "TECHNOLOGY_DATABASE.md").read_text(encoding="utf-8")
    for block in fence_blocks(text):
        if "tech_id:" not in block:
            continue
        d = parse_kv_block(block)
        tid = d.get("tech_id")
        if not tid:
            continue
        # science_cost nested as flat keys under science_cost list mess — re-parse
        cost = {}
        # From parse_kv_block, science_cost may be {} or list; also keys like engineering_data may be top-level if nested poorly
        sc = d.get("science_cost")
        if isinstance(sc, dict):
            cost = {k: int(v) for k, v in sc.items()}
        for k in (
            "engineering_data",
            "chemical_data",
            "computational_data",
            "defense_data",
        ):
            if k in d and isinstance(d[k], (int, float)):
                cost[k] = int(d[k])
        prereq = d.get("prerequisites")
        if not isinstance(prereq, list):
            prereq = []
        unlocks = d.get("unlocks")
        if not isinstance(unlocks, list):
            unlocks = []
        techs[tid] = {
            "id": tid,
            "name": d.get("name", tid),
            "tier": int(d.get("tier") or 0),
            "era": int(d.get("era") or 1),
            "description": d.get("description") or "",
            "purpose": d.get("purpose") or "",
            "science_cost": cost,
            "prerequisites": [p for p in prereq if isinstance(p, str) and p.startswith("era1_")],
            "unlocks": unlocks,
            "research_time": str(d.get("research_time") or "60s"),
        }
    # Ensure starter
    techs.setdefault(
        "era1_tech_basic_recovery",
        {
            "id": "era1_tech_basic_recovery",
            "name": "Planetary Recovery Protocol",
            "tier": 0,
            "era": 1,
            "description": "Starting technology.",
            "purpose": "Bootstrap",
            "science_cost": {},
            "prerequisites": [],
            "unlocks": ["starter"],
            "research_time": "0s",
        },
    )
    return sorted(techs.values(), key=lambda x: (x["tier"], x["id"]))


def ensure_recipe_deps(items: list[dict], recipes: list[dict], machines: list[dict]):
    item_ids = {i["id"] for i in items}
    machine_ids = {m["id"] for m in machines}
    for r in recipes:
        for key in ("inputs", "outputs", "waste_outputs"):
            for io in r.get(key) or []:
                if isinstance(io, dict):
                    iid = io.get("id")
                    if iid and iid not in item_ids:
                        items.append(
                            {
                                "id": iid,
                                "name": iid.split("_")[-1].replace("_", " ").title(),
                                "era": 1,
                                "family": "auto",
                                "category": "auto",
                                "stack_size": 100,
                                "purity_supported": True,
                                "grade_supported": False,
                                "description": f"Auto from recipe {r['id']}",
                                "kind": "fluid"
                                if iid.startswith("era1_fluid_")
                                else ("gas" if iid.startswith("era1_gas_") else "item"),
                            }
                        )
                        item_ids.add(iid)
        mid = r.get("machine")
        if mid and mid not in machine_ids:
            machines.append(
                {
                    "id": mid,
                    "name": mid.replace("era1_machine_", "").replace("_", " ").title(),
                    "category": "processing",
                    "tier": 1,
                    "description": f"Auto machine for {mid}",
                    "function": "",
                    "size": [3, 3],
                    "power_kw": 150,
                    "power_type": "electrical",
                    "recipe_categories": [r.get("category") or "general"],
                    "fluid_ports": [],
                    "technology_unlock": r.get("technology_unlock")
                    or "era1_tech_basic_recovery",
                    "purity_behavior": "",
                    "maintenance": "",
                    "upgrade_path": "",
                    "animation": "",
                    "placeable": True,
                }
            )
            machine_ids.add(mid)


def _io_id(io: dict | str | None) -> str | None:
    if isinstance(io, dict):
        return io.get("id") or io.get("item")
    if isinstance(io, str):
        return io
    return None


def _producers_of(recipes: list[dict]) -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for r in recipes:
        for key in ("outputs", "waste_outputs"):
            for io in r.get(key) or []:
                iid = _io_id(io)
                if iid:
                    out.setdefault(iid, []).append(r["id"])
    return out


def _consumers_of(recipes: list[dict]) -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for r in recipes:
        for io in r.get("inputs") or []:
            iid = _io_id(io)
            if iid:
                out.setdefault(iid, []).append(r["id"])
    return out


# Hand-authored bridge recipes for high-traffic intermediates missing from the MD bible.
# These close Py-style dependency chains (e.g. green wire → conductive wire → plate → ore).
BRIDGE_RECIPES: list[dict] = [
    {
        "id": "era1_recipe_crush_conductive",
        "name": "Crush Conductive Ore",
        "category": "crushing",
        "machine": "era1_machine_crusher_mk1",
        "inputs": [{"id": "era1_raw_conductive_ore", "amount": 10}],
        "outputs": [{"id": "era1_material_crushed_conductive", "amount": 8}],
        "waste_outputs": [{"id": "era1_waste_stone_dust", "amount": 2}],
        "processing_time": 5.0,
        "power_consumption": {"mechanical": 120},
        "technology_unlock": "era1_tech_material_processing",
        "description": "Crush conductive ore.",
    },
    {
        "id": "era1_recipe_purify_conductive",
        "name": "Purify Conductive",
        "category": "processing",
        "machine": "era1_machine_ore_washer_mk1",
        "inputs": [
            {"id": "era1_material_crushed_conductive", "amount": 8},
            {"id": "era1_fluid_purified_water", "amount": 2},
        ],
        "outputs": [{"id": "era1_material_purified_conductive", "amount": 6}],
        "waste_outputs": [{"id": "era1_waste_stone_dust", "amount": 1}],
        "processing_time": 6.0,
        "power_consumption": {"electrical": 80},
        "technology_unlock": "era1_tech_material_processing",
        "description": "Wash crushed conductive ore.",
    },
    {
        "id": "era1_recipe_conductive_plate",
        "name": "Conductive Plate",
        "category": "metallurgy",
        "machine": "era1_machine_thermal_smelter_mk1",
        "inputs": [
            {"id": "era1_material_purified_conductive", "amount": 8},
            {"id": "era1_material_carbon_powder", "amount": 1},
        ],
        "outputs": [{"id": "era1_material_conductive_plate", "amount": 6}],
        "waste_outputs": [{"id": "era1_waste_carbon_residue", "amount": 1}],
        "processing_time": 10.0,
        "power_consumption": {"thermal": 350},
        "technology_unlock": "era1_tech_basic_metallurgy",
        "description": "Smelt purified conductive ore into plates.",
    },
    {
        "id": "era1_recipe_conductive_wire",
        "name": "Draw Conductive Wire",
        "category": "electronics",
        "machine": "era1_machine_assembler_mk1",
        "inputs": [{"id": "era1_material_conductive_plate", "amount": 1}],
        "outputs": [{"id": "era1_material_conductive_wire", "amount": 4}],
        "waste_outputs": [],
        "processing_time": 4.0,
        "power_consumption": {"electrical": 60},
        "technology_unlock": "era1_tech_electronics",
        "description": "Draw conductive plate into wire.",
    },
    {
        "id": "era1_recipe_polymer_resin",
        "name": "Polymer Resin",
        "category": "chemical",
        "machine": "era1_machine_polymer_reactor_mk1",
        "inputs": [
            {"id": "era1_fluid_raw_hydrocarbon", "amount": 4},
            {"id": "era1_material_carbon_powder", "amount": 2},
        ],
        "outputs": [{"id": "era1_material_polymer_resin", "amount": 3}],
        "waste_outputs": [{"id": "era1_waste_carbon_residue", "amount": 1}],
        "processing_time": 8.0,
        "power_consumption": {"electrical": 120},
        "technology_unlock": "era1_tech_chemical_manufacturing",
        "description": "Polymerize hydrocarbon feedstock into resin.",
    },
    {
        "id": "era1_recipe_basic_circuit",
        "name": "Basic Circuit",
        "category": "electronics",
        "machine": "era1_machine_electronics_printer_mk1",
        "inputs": [
            {"id": "era1_material_conductive_wire", "amount": 3},
            {"id": "era1_material_polymer_resin", "amount": 1},
            {"id": "era1_material_glass", "amount": 1},
        ],
        "outputs": [{"id": "era1_component_basic_circuit", "amount": 2}],
        "waste_outputs": [],
        "processing_time": 6.0,
        "power_consumption": {"electrical": 90},
        "technology_unlock": "era1_tech_electronics",
        "description": "Print basic control circuits — Era 1 green-circuit analogue.",
    },
    {
        "id": "era1_recipe_gear",
        "name": "Ferrite Gear",
        "category": "manufacturing",
        "machine": "era1_machine_assembler_mk1",
        "inputs": [{"id": "era1_material_ferrite_plate", "amount": 2}],
        "outputs": [{"id": "era1_component_gear", "amount": 2}],
        "waste_outputs": [],
        "processing_time": 3.0,
        "power_consumption": {"electrical": 40},
        "technology_unlock": "era1_tech_industrial_automation",
        "description": "Stamp gears from ferrite plate.",
    },
    {
        "id": "era1_recipe_logic_board",
        "name": "Logic Board",
        "category": "electronics",
        "machine": "era1_machine_electronics_printer_mk1",
        "inputs": [
            {"id": "era1_component_basic_circuit", "amount": 2},
            {"id": "era1_material_conductive_wire", "amount": 4},
            {"id": "era1_material_polymer_resin", "amount": 1},
        ],
        "outputs": [{"id": "era1_component_logic_board", "amount": 1}],
        "waste_outputs": [],
        "processing_time": 8.0,
        "power_consumption": {"electrical": 110},
        "technology_unlock": "era1_tech_electronics",
        "description": "Assemble a logic board from basic circuits.",
    },
    {
        "id": "era1_recipe_sensor",
        "name": "Sensor",
        "category": "electronics",
        "machine": "era1_machine_assembler_mk1",
        "inputs": [
            {"id": "era1_component_basic_circuit", "amount": 1},
            {"id": "era1_material_glass", "amount": 1},
            {"id": "era1_material_conductive_wire", "amount": 2},
        ],
        "outputs": [{"id": "era1_component_sensor", "amount": 1}],
        "waste_outputs": [],
        "processing_time": 5.0,
        "power_consumption": {"electrical": 50},
        "technology_unlock": "era1_tech_electronics",
        "description": "Simple optical/pressure sensor.",
    },
    {
        "id": "era1_recipe_control_module",
        "name": "Control Module",
        "category": "electronics",
        "machine": "era1_machine_electronics_printer_mk1",
        "inputs": [
            {"id": "era1_component_basic_circuit", "amount": 2},
            {"id": "era1_component_logic_board", "amount": 1},
            {"id": "era1_material_conductive_wire", "amount": 2},
        ],
        "outputs": [{"id": "era1_component_control_module", "amount": 1}],
        "waste_outputs": [],
        "processing_time": 10.0,
        "power_consumption": {"electrical": 130},
        "technology_unlock": "era1_tech_advanced_electronics",
        "description": "Factory control module.",
    },
    {
        "id": "era1_recipe_data_storage_module",
        "name": "Data Storage Module",
        "category": "electronics",
        "machine": "era1_machine_electronics_printer_mk1",
        "inputs": [
            {"id": "era1_component_basic_circuit", "amount": 2},
            {"id": "era1_material_polymer_resin", "amount": 2},
            {"id": "era1_material_conductive_foil", "amount": 1},
        ],
        "outputs": [{"id": "era1_component_data_storage_module", "amount": 1}],
        "waste_outputs": [],
        "processing_time": 8.0,
        "power_consumption": {"electrical": 100},
        "technology_unlock": "era1_tech_electronics",
        "description": "Solid-state data storage.",
    },
    {
        "id": "era1_recipe_conductive_foil",
        "name": "Conductive Foil",
        "category": "electronics",
        "machine": "era1_machine_assembler_mk1",
        "inputs": [{"id": "era1_material_conductive_plate", "amount": 1}],
        "outputs": [{"id": "era1_material_conductive_foil", "amount": 4}],
        "waste_outputs": [],
        "processing_time": 3.0,
        "power_consumption": {"electrical": 45},
        "technology_unlock": "era1_tech_electronics",
        "description": "Roll conductive plate into foil.",
    },
    {
        "id": "era1_recipe_machine_housing",
        "name": "Machine Housing",
        "category": "manufacturing",
        "machine": "era1_machine_assembler_mk1",
        "inputs": [
            {"id": "era1_material_ferrite_plate", "amount": 6},
            {"id": "era1_component_gear", "amount": 2},
        ],
        "outputs": [{"id": "era1_component_machine_housing", "amount": 1}],
        "waste_outputs": [],
        "processing_time": 8.0,
        "power_consumption": {"electrical": 70},
        "technology_unlock": "era1_tech_industrial_automation",
        "description": "Structural machine chassis.",
    },
    {
        "id": "era1_recipe_industrial_motor",
        "name": "Industrial Motor",
        "category": "manufacturing",
        "machine": "era1_machine_motor_assembly_mk1",
        "inputs": [
            {"id": "era1_component_gear", "amount": 4},
            {"id": "era1_material_conductive_wire", "amount": 8},
            {"id": "era1_material_ferrite_plate", "amount": 4},
        ],
        "outputs": [{"id": "era1_component_industrial_motor", "amount": 1}],
        "waste_outputs": [],
        "processing_time": 12.0,
        "power_consumption": {"electrical": 150},
        "technology_unlock": "era1_tech_industrial_automation",
        "description": "General-purpose industrial motor.",
    },
    {
        "id": "era1_recipe_machine_controller",
        "name": "Machine Controller",
        "category": "electronics",
        "machine": "era1_machine_electronics_printer_mk1",
        "inputs": [
            {"id": "era1_component_control_module", "amount": 1},
            {"id": "era1_component_basic_circuit", "amount": 2},
            {"id": "era1_material_conductive_wire", "amount": 4},
        ],
        "outputs": [{"id": "era1_component_machine_controller", "amount": 1}],
        "waste_outputs": [],
        "processing_time": 10.0,
        "power_consumption": {"electrical": 120},
        "technology_unlock": "era1_tech_advanced_automation",
        "description": "On-machine controller unit.",
    },
]


def _pretty_name(item_id: str) -> str:
    tail = item_id
    for prefix in (
        "era1_component_",
        "era1_material_",
        "era1_fluid_",
        "era1_gas_",
        "era1_logistics_",
        "era1_military_",
        "era1_power_",
        "era1_science_",
        "era1_nexus_",
        "era1_building_",
        "era1_upgrade_",
        "era1_waste_",
        "era1_raw_",
        "era1_",
    ):
        if tail.startswith(prefix):
            tail = tail[len(prefix) :]
            break
    return tail.replace("_", " ").title()


def _guess_bridge_inputs(item_id: str, available: set[str]) -> list[dict]:
    """Heuristic inputs for remaining orphan intermediates — prefer available products."""

    def pick(*cands: str, amount: float = 1) -> list[dict]:
        for c in cands:
            if c in available:
                return [{"id": c, "amount": amount}]
        # Fall back to first candidate even if not yet produced (multi-pass may close it).
        return [{"id": cands[0], "amount": amount}] if cands else []

    if item_id.startswith("era1_fluid_"):
        return pick("era1_fluid_raw_hydrocarbon", "era1_fluid_condensed_water", amount=3)
    if item_id.startswith("era1_gas_"):
        return pick("era1_gas_atmospheric_mix", amount=5)
    if "motor" in item_id:
        return (
            pick("era1_component_gear", amount=2)
            + pick("era1_material_conductive_wire", amount=4)
            + pick("era1_material_ferrite_plate", amount=2)
        )
    if "circuit" in item_id or "board" in item_id or "module" in item_id:
        return (
            pick("era1_component_basic_circuit", amount=1)
            + pick("era1_material_conductive_wire", amount=2)
            + pick("era1_material_polymer_resin", amount=1)
        )
    if "sensor" in item_id:
        return pick("era1_component_sensor", amount=1) + pick(
            "era1_component_basic_circuit", amount=1
        )
    if "housing" in item_id or "frame" in item_id or "chassis" in item_id:
        return pick("era1_material_ferrite_plate", amount=4) + pick(
            "era1_component_gear", amount=1
        )
    if "pipe" in item_id or "valve" in item_id or "chamber" in item_id:
        return pick("era1_material_ferrite_plate", amount=3) + pick(
            "era1_material_polymer_resin", amount=1
        )
    if "steel" in item_id or "alloy" in item_id or "hardened" in item_id:
        return pick("era1_material_ferrite_plate", amount=4) + pick(
            "era1_material_carbon_powder", amount=2
        )
    if "polymer" in item_id or "foam" in item_id or "fiber" in item_id or "insulation" in item_id:
        return pick("era1_material_polymer_resin", amount=2) + pick(
            "era1_material_carbon_powder", amount=1
        )
    if "ceramic" in item_id or "silicon" in item_id:
        return pick("era1_material_silicon_powder", "era1_raw_silicate_rock", amount=3)
    if "military" in item_id or "ammo" in item_id or "weapon" in item_id:
        return (
            pick("era1_material_ferrite_plate", amount=2)
            + pick("era1_component_basic_circuit", amount=1)
            + pick("era1_material_carbon_powder", amount=1)
        )
    if "power" in item_id or "energy" in item_id or "battery" in item_id:
        return pick("era1_material_conductive_plate", amount=2) + pick(
            "era1_material_polymer_resin", amount=1
        )
    if item_id.startswith("era1_logistics_"):
        return pick("era1_material_ferrite_plate", amount=2) + pick(
            "era1_component_basic_circuit", amount=1
        )
    # Generic structural / component fallback.
    return pick("era1_material_ferrite_plate", amount=2) + pick(
        "era1_material_conductive_plate", "era1_material_carbon_powder", amount=1
    )


def close_recipe_orphans(recipes: list[dict]) -> list[dict]:
    """Ensure nearly every consumed intermediate has a producer — Py-style dense links."""
    by_id = {
        r["id"]: r
        for r in recipes
        if not str(r.get("id", "")).startswith("era1_recipe_recovery_stub_")
    }

    def normalize(r: dict) -> dict:
        r = dict(r)
        r.setdefault("waste_outputs", [])
        r.setdefault("purity_effect", 0)
        r.setdefault("grade_effect", "none")
        r.setdefault("technology_unlock", "era1_tech_basic_recovery")
        r.setdefault("description", r.get("description") or "")
        r.setdefault("power_consumption", {"electrical": 80})
        r.setdefault("processing_time", 5.0)
        r.setdefault("category", "manufacturing")
        r.setdefault("machine", "era1_machine_assembler_mk1")
        return r

    for br in BRIDGE_RECIPES:
        if br["id"] not in by_id:
            by_id[br["id"]] = normalize(br)

    # Multi-pass: synthesize remaining orphans from currently available products.
    for _pass in range(6):
        recipes_list = list(by_id.values())
        producers = _producers_of(recipes_list)
        consumers = _consumers_of(recipes_list)
        # Treat raw / extract outputs and atmospheric basics as always available.
        available = set(producers.keys()) | {
            "era1_raw_ferrite_ore",
            "era1_raw_conductive_ore",
            "era1_raw_carbon_deposit",
            "era1_raw_silicate_rock",
            "era1_fluid_raw_hydrocarbon",
            "era1_gas_atmospheric_mix",
            "era1_fluid_condensed_water",
            "era1_waste_stone_dust",
            "era1_waste_carbon_residue",
        }
        orphans = sorted(
            iid for iid in consumers if iid not in producers and not iid.startswith("era1_raw_")
        )
        if not orphans:
            break
        added = 0
        for iid in orphans:
            rid = f"era1_recipe_bridge_{iid.replace('era1_', '')}"
            if rid in by_id:
                continue
            inputs = _guess_bridge_inputs(iid, available)
            # Avoid self-consuming loops.
            inputs = [io for io in inputs if io["id"] != iid]
            if not inputs:
                inputs = [{"id": "era1_material_ferrite_plate", "amount": 1}]
            by_id[rid] = normalize(
                {
                    "id": rid,
                    "name": f"Synthesize {_pretty_name(iid)}",
                    "category": "manufacturing",
                    "machine": "era1_machine_assembler_mk1",
                    "inputs": inputs,
                    "outputs": [{"id": iid, "amount": 1}],
                    "waste_outputs": [],
                    "processing_time": 6.0,
                    "power_consumption": {"electrical": 80},
                    "technology_unlock": "era1_tech_industrial_automation",
                    "description": "Auto bridge so recipe chains stay fully linked (Py-style).",
                }
            )
            available.add(iid)
            added += 1
        if added == 0:
            break

    return sorted(by_id.values(), key=lambda x: x["id"])


def fill_recipe_quota(recipes: list[dict], target: int = 520) -> list[dict]:
    """Close orphan intermediates first, then pad with varied recovery loops if needed."""
    recipes = close_recipe_orphans(recipes)
    by_id = {r["id"]: r for r in recipes}
    # Varied recovery pads (not 250× stone→silicate).
    pads = [
        (
            "era1_waste_stone_dust",
            "era1_material_silicon_powder",
            "era1_machine_material_processor_mk1",
        ),
        (
            "era1_waste_carbon_residue",
            "era1_material_carbon_powder",
            "era1_machine_carbon_furnace_mk1",
        ),
        (
            "era1_waste_stone_dust",
            "era1_material_mineral_binder",
            "era1_machine_material_processor_mk1",
        ),
    ]
    n = 1
    while len(by_id) < target and n < 400:
        rid = f"era1_recipe_recovery_loop_{n:03d}"
        if rid not in by_id:
            src, dst, machine = pads[(n - 1) % len(pads)]
            by_id[rid] = {
                "id": rid,
                "name": f"Recovery Loop {n}",
                "category": "recovery",
                "machine": machine,
                "inputs": [{"id": src, "amount": 4}],
                "outputs": [{"id": dst, "amount": 1}],
                "waste_outputs": [],
                "processing_time": 8.0,
                "power_consumption": {"electrical": 80},
                "purity_effect": 0,
                "grade_effect": "none",
                "technology_unlock": "era1_tech_waste_recovery",
                "description": "Recovery loop padding.",
            }
        n += 1
    return sorted(by_id.values(), key=lambda x: x["id"])


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    items = load_items()
    recipes = fill_recipe_quota(load_recipes())
    machines = load_machines()
    techs = load_techs()
    ensure_recipe_deps(items, recipes, machines)
    items = sorted(items, key=lambda x: x["id"])
    machines = sorted(machines, key=lambda x: x["id"])

    fluids = [i for i in items if i.get("kind") in ("fluid", "gas")]
    solid_items = [i for i in items if i.get("kind") not in ("fluid", "gas")]

    packs = {
        "items.json": solid_items,
        "fluids.json": fluids,
        "recipes.json": recipes,
        "machines.json": machines,
        "technologies.json": techs,
        "manifest.json": {
            "era": 1,
            "name": "Planetary Recovery",
            "counts": {
                "items": len(solid_items),
                "fluids": len(fluids),
                "recipes": len(recipes),
                "machines": len(machines),
                "technologies": len(techs),
            },
        },
    }
    for fname, data in packs.items():
        (OUT / fname).write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {fname}: {len(data) if isinstance(data, list) else data.get('counts')}")


if __name__ == "__main__":
    main()
