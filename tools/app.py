"""
RustyEcon scenario editor / runner / visualiser.
Run with:  streamlit run tools/app.py
"""

import math
import sys
from pathlib import Path

import plotly.graph_objects as go
import streamlit as st

sys.path.insert(0, str(Path(__file__).parent))
import ron
from ron import Bare, Tagged
from readers import ScenarioResults, load_scenario_results
from scenario import (
    REPO_ROOT,
    SCENARIOS_DIR,
    BINARY,
    HEX_D,
    NEIGHBOR_ANGLES,
    SNAP_EPS,
    attach_region,
    get_pos,
    list_scenarios,
    load_scenario,
    load_starting_state,
    save_starting_state,
    load_events,
    save_events,
    run_simulation,
    save_scenario,
    set_pos,
)

HEX_R = 1.0

COLORS = {
    "region_default":  "rgba(100, 149, 237, 0.6)",
    "region_selected": "rgba(255, 165, 0, 0.85)",
    "region_attach":   "rgba(50, 205, 50, 0.6)",
    "hex_border":      "#333333",
    "channel":         "#aaaaaa",
    "bg":              "#1a1a2e",
}

SHELF_LIFE_OPTIONS  = ["Indefinite", "None"]
MOVEMENT_OPTIONS    = ["Physical", "Financial", "Local"]
SCALING_OPTIONS     = ["Variable", "Fixed"]
TIER_OPTIONS        = ["Regional", "National", "World"]
CHANNEL_TYPE_OPTIONS = ["Trade", "Capital"]

# ── Hex geometry ──────────────────────────────────────────────────────────────

def hex_vertices(cx: float, cy: float, R: float = HEX_R):
    angles = [math.pi / 2 - i * math.pi / 3 for i in range(6)]
    xs = [cx + R * math.cos(a) for a in angles] + [cx + R * math.cos(angles[0])]
    ys = [cy + R * math.sin(a) for a in angles] + [cy + R * math.sin(angles[0])]
    return xs, ys

# ── RON value helpers ─────────────────────────────────────────────────────────

def bare_str(v) -> str:
    return v.value if isinstance(v, Bare) else str(v)


def str_to_bare(s: str):
    return Bare(s)


def tagged_inner_int(v: Tagged) -> int:
    return int(v.inner) if isinstance(v, Tagged) else int(v)

# ── Session state init ────────────────────────────────────────────────────────

def _init():
    if "scenario_name" not in st.session_state:
        scenarios = list_scenarios()
        st.session_state.scenario_name = scenarios[0] if scenarios else ""
    if st.session_state.scenario_name and "gd" not in st.session_state:
        name = st.session_state.scenario_name
        st.session_state.gd = load_scenario(name)
        st.session_state.ss = load_starting_state(name)
        st.session_state.ev = load_events(name)
    if "dirty" not in st.session_state:
        st.session_state.dirty = False
    if "sel_region" not in st.session_state:
        st.session_state.sel_region = None
    if "mode" not in st.session_state:
        st.session_state.mode = "normal"
    if "page" not in st.session_state:
        st.session_state.page = "Map"

# ── Sidebar ───────────────────────────────────────────────────────────────────

def render_sidebar():
    st.sidebar.title("RustyEcon")

    scenarios = list_scenarios()
    cur = st.session_state.scenario_name
    idx = scenarios.index(cur) if cur in scenarios else 0
    choice = st.sidebar.selectbox("Scenario", scenarios, index=idx)

    col_l, col_s = st.sidebar.columns(2)
    if col_l.button("Load", use_container_width=True):
        st.session_state.scenario_name = choice
        st.session_state.gd = load_scenario(choice)
        st.session_state.ss = load_starting_state(choice)
        st.session_state.ev = load_events(choice)
        st.session_state.dirty = False
        st.session_state.sel_region = None
        st.session_state.mode = "normal"
        st.session_state.viz_dashboard = _load_dashboard(choice)
        st.session_state.viz_series = []
        st.session_state.viz_results = None
        st.rerun()
    save_disabled = not st.session_state.get("dirty", False)
    if col_s.button("Save", disabled=save_disabled, use_container_width=True):
        name = st.session_state.scenario_name
        save_scenario(name, st.session_state.gd)
        save_starting_state(name, st.session_state.ss)
        save_events(name, st.session_state.ev)
        st.session_state.dirty = False
        st.rerun()

    if st.session_state.get("dirty"):
        st.sidebar.caption("⚠ Unsaved changes")

    st.sidebar.divider()

    for label in ["Map", "Goods", "Recipes", "Needs & Wealth", "Events", "Run", "Visualise"]:
        if st.sidebar.button(label, use_container_width=True,
                             type="primary" if st.session_state.page == label else "secondary"):
            st.session_state.page = label
            st.rerun()

# ── Map page ──────────────────────────────────────────────────────────────────

def build_map(gd: dict, sel_region: int | None, mode: str) -> go.Figure:
    regions = gd.get("regions", [])
    channels = gd.get("channels", [])
    n = len(regions)

    fig = go.Figure()
    fig.update_layout(
        margin=dict(l=0, r=0, t=0, b=0),
        showlegend=False,
        plot_bgcolor=COLORS["bg"],
        paper_bgcolor=COLORS["bg"],
        xaxis=dict(visible=False, scaleanchor="y", scaleratio=1),
        yaxis=dict(visible=False),
        clickmode="event+select",
        dragmode="select",
        uirevision="map",
    )

    node_to_region = {}
    for i, r in enumerate(regions):
        mid = r.get("market_node")
        if isinstance(mid, Tagged):
            node_to_region[mid.inner] = i

    for ch in channels:
        f_node = ch.get("from")
        t_node = ch.get("to")
        fi = node_to_region.get(f_node.inner if isinstance(f_node, Tagged) else None)
        ti = node_to_region.get(t_node.inner if isinstance(t_node, Tagged) else None)
        if fi is None or ti is None:
            continue
        fx, fy = get_pos(regions[fi], fi, n)
        tx, ty = get_pos(regions[ti], ti, n)
        fig.add_trace(go.Scatter(
            x=[fx, tx], y=[fy, ty],
            mode="lines",
            line=dict(color=COLORS["channel"], width=2, dash="dot"),
            hoverinfo="skip", showlegend=False,
        ))

    for i, region in enumerate(regions):
        cx, cy = get_pos(region, i, n)
        vx, vy = hex_vertices(cx, cy)

        if mode == "attach" and i != sel_region:
            fill = COLORS["region_attach"]
        elif i == sel_region:
            fill = COLORS["region_selected"]
        else:
            fill = COLORS["region_default"]

        fig.add_trace(go.Scatter(
            x=vx, y=vy,
            mode="lines", fill="toself", fillcolor=fill,
            line=dict(color=COLORS["hex_border"], width=1.5),
            hoverinfo="skip", showlegend=False,
        ))

    for i, region in enumerate(regions):
        cx, cy = get_pos(region, i, n)
        fig.add_annotation(
            x=cx, y=cy,
            text=region.get("name", f"R{i}"),
            showarrow=False,
            font=dict(color="white", size=11),
            xanchor="center", yanchor="middle",
        )

    if n:
        fig.add_trace(go.Scatter(
            x=[get_pos(r, i, n)[0] for i, r in enumerate(regions)],
            y=[get_pos(r, i, n)[1] for i, r in enumerate(regions)],
            mode="markers",
            marker=dict(size=max(30, int(HEX_D * 18)), opacity=0, color="white"),
            customdata=list(range(n)),
            hovertemplate="<b>%{customdata}</b><extra></extra>",
            showlegend=False,
        ))

    return fig


# ── Inspector helpers ─────────────────────────────────────────────────────────

def _id_int(v) -> int:
    """Extract int from a 1-tuple (n,) or plain int."""
    if isinstance(v, tuple) and len(v) == 1:
        return int(v[0])
    if isinstance(v, Tagged):
        return int(v.inner)
    return int(v)


def _make_id(n: int) -> tuple:
    """Build a RON newtype ID: Python 1-tuple that writes as (n)."""
    return (n,)


def _inv_to_dict(inv) -> dict:
    """Inventory (list of (good_name_or_id, qty) tuples) → {name: qty}."""
    if isinstance(inv, dict):
        return dict(inv)
    result = {}
    for entry in (inv or []):
        if isinstance(entry, (tuple, list)) and len(entry) == 2:
            result[str(_id_int(entry[0]) if not isinstance(entry[0], str) else entry[0])] = float(entry[1])
    return result


def _region_node_id(region: dict) -> int | None:
    mn = region.get("market_node")
    if isinstance(mn, Tagged):
        return _id_int(mn.inner)
    if mn is not None:
        return _id_int(mn)
    return None


# ── Region tab ────────────────────────────────────────────────────────────────

def _tab_region(region: dict, sel: int, regions: list):
    new_name = st.text_input("Name", value=region.get("name", ""), key="reg_name")
    if new_name != region.get("name", ""):
        region["name"] = new_name
        st.session_state.dirty = True
        st.rerun()

    n = len(regions)
    cx, cy = get_pos(region, sel, n)
    c1, c2 = st.columns(2)
    new_x = c1.number_input("X", value=round(cx, 3), step=HEX_D, format="%.3f", key="reg_x")
    new_y = c2.number_input("Y", value=round(cy, 3), step=HEX_D, format="%.3f", key="reg_y")
    if abs(new_x - cx) > 1e-6 or abs(new_y - cy) > 1e-6:
        set_pos(region, new_x, new_y)
        st.session_state.dirty = True
        st.rerun()

    st.divider()
    if st.button("Attach to region", use_container_width=True):
        st.session_state.mode = "attach"
        st.rerun()


# ── Buildings tab ─────────────────────────────────────────────────────────────

def _tab_buildings(sel_region_idx: int, gd: dict, ss: dict):
    recipes = [r.get("name", f"recipe{i}") for i, r in enumerate(gd.get("recipes", []))]
    goods   = [g.get("name", f"g{i}") for i, g in enumerate(gd.get("goods", []))]
    channels= gd.get("channels", [])

    buildings = ss.get("buildings", [])
    region_buildings = [b for b in buildings if _id_int(b.get("region", -1)) == sel_region_idx]

    for b in region_buildings:
        bid = _id_int(b.get("id", 0))
        label = f"Building {bid} — {b.get('recipe', '?')}  (size {b.get('recipe_size', 0)})"
        with st.expander(label):
            _edit_building(b, bid, recipes, goods, channels, ss)

    st.divider()
    if st.button("＋ Add building", key=f"add_bld_{sel_region_idx}"):
        new_id = len(buildings)
        buildings.append({
            "id":           _make_id(new_id),
            "region":       _make_id(sel_region_idx),
            "recipe":       recipes[0] if recipes else "",
            "recipe_size":  1.0,
            "chosen_size":  0.0,
            "efficiency":   1.0,
            "inventory":    ron.RonMap(),
            "transfer_target": None,
            "owners":       [],
            "channel":      None,
        })
        ss["buildings"] = buildings
        st.session_state.dirty = True
        st.rerun()


def _edit_building(b: dict, bid: int, recipes: list, goods: list, channels: list, ss: dict):
    c1, c2 = st.columns(2)
    recipe = b.get("recipe", recipes[0] if recipes else "")
    new_recipe = c1.selectbox("Recipe", recipes,
                              index=recipes.index(recipe) if recipe in recipes else 0,
                              key=f"bld_recipe_{bid}")
    if new_recipe != recipe:
        b["recipe"] = new_recipe
        st.session_state.dirty = True

    recipe_size = c2.number_input("Recipe size", value=float(b.get("recipe_size", 1.0)),
                                  min_value=0.0, step=1.0, key=f"bld_rsize_{bid}")
    if abs(recipe_size - float(b.get("recipe_size", 1.0))) > 1e-9:
        b["recipe_size"] = recipe_size
        st.session_state.dirty = True

    chosen = st.number_input("Chosen size", value=float(b.get("chosen_size", 0.0)),
                             step=1.0, key=f"bld_csize_{bid}")
    if abs(chosen - float(b.get("chosen_size", 0.0))) > 1e-9:
        b["chosen_size"] = chosen
        st.session_state.dirty = True

    # Channel
    ch_val = b.get("channel")
    ch_idx = _id_int(ch_val.inner) if isinstance(ch_val, Tagged) and ch_val.tag == "Some" else \
             (_id_int(ch_val[0]) if isinstance(ch_val, tuple) else None)
    ch_options = [None] + list(range(len(channels)))
    ch_fmt = lambda v: "None" if v is None else f"Channel {v}"
    new_ch = st.selectbox("Channel", ch_options, index=ch_options.index(ch_idx),
                          format_func=ch_fmt, key=f"bld_ch_{bid}")
    if new_ch != ch_idx:
        b["channel"] = Tagged("Some", _make_id(new_ch)) if new_ch is not None else None
        st.session_state.dirty = True

    # Inventory
    st.markdown("**Starting inventory**")
    inv = _inv_to_dict(b.get("inventory"))
    import pandas as pd
    inv_df = pd.DataFrame(list(inv.items()), columns=["good", "qty"]) if inv else \
             pd.DataFrame(columns=["good", "qty"])
    edited = st.data_editor(inv_df, num_rows="dynamic", key=f"bld_inv_{bid}",
                            column_config={"good": st.column_config.TextColumn("Good"),
                                           "qty": st.column_config.NumberColumn("Qty")},
                            use_container_width=True)
    new_inv = ron.RonMap({r["good"]: float(r["qty"]) for _, r in edited.iterrows() if r["good"]})
    if new_inv != b.get("inventory"):
        b["inventory"] = new_inv
        st.session_state.dirty = True

    if st.button("🗑 Delete building", key=f"del_bld_{bid}", type="secondary"):
        ss["buildings"] = [x for x in ss.get("buildings", []) if _id_int(x.get("id", -1)) != bid]
        st.session_state.dirty = True
        st.rerun()


# ── Pops tab ──────────────────────────────────────────────────────────────────

def _tab_pops(sel_region_idx: int, ss: dict):
    import pandas as pd
    pops = ss.get("pop_groups", [])
    region_pops = [p for p in pops if _id_int(p.get("region", -1)) == sel_region_idx]

    for p in region_pops:
        pid = _id_int(p.get("id", 0))
        with st.expander(f"Pop group {pid}  (size {p.get('size', 0)})"):
            _edit_pop(p, pid, ss)

    st.divider()
    if st.button("＋ Add pop group", key=f"add_pop_{sel_region_idx}"):
        new_id = len(pops)
        pops.append({
            "id":             _make_id(new_id),
            "region":         _make_id(sel_region_idx),
            "size":           1.0,
            "wealth":         0.0,
            "inventory":      ron.RonMap(),
            "savings_target": 0.5,
        })
        ss["pop_groups"] = pops
        st.session_state.dirty = True
        st.rerun()


def _edit_pop(p: dict, pid: int, ss: dict):
    import pandas as pd
    c1, c2, c3 = st.columns(3)
    size = c1.number_input("Size", value=float(p.get("size", 1.0)), min_value=0.0, step=1.0,
                           key=f"pop_size_{pid}")
    wealth = c2.number_input("Wealth", value=float(p.get("wealth", 0.0)), step=0.1,
                             key=f"pop_wealth_{pid}")
    savings = c3.number_input("Savings target", value=float(p.get("savings_target", 0.5)),
                              min_value=0.0, step=0.1, key=f"pop_sav_{pid}")

    for attr, val, key in [("size", size, "size"), ("wealth", wealth, "wealth"),
                            ("savings_target", savings, "savings_target")]:
        if abs(val - float(p.get(attr, 0.0))) > 1e-9:
            p[attr] = val
            st.session_state.dirty = True

    st.markdown("**Starting inventory**")
    inv = _inv_to_dict(p.get("inventory"))
    inv_df = pd.DataFrame(list(inv.items()), columns=["good", "qty"]) if inv else \
             pd.DataFrame(columns=["good", "qty"])
    edited = st.data_editor(inv_df, num_rows="dynamic", key=f"pop_inv_{pid}",
                            column_config={"good": st.column_config.TextColumn("Good"),
                                           "qty": st.column_config.NumberColumn("Qty")},
                            use_container_width=True)
    new_inv = ron.RonMap({r["good"]: float(r["qty"]) for _, r in edited.iterrows() if r["good"]})
    if new_inv != p.get("inventory"):
        p["inventory"] = new_inv
        st.session_state.dirty = True

    if st.button("🗑 Delete pop group", key=f"del_pop_{pid}", type="secondary"):
        ss["pop_groups"] = [x for x in ss.get("pop_groups", []) if _id_int(x.get("id", -1)) != pid]
        st.session_state.dirty = True
        st.rerun()


# ── Magic producers tab ───────────────────────────────────────────────────────

def _tab_magic_producers(region: dict, gd: dict, ss: dict):
    node_id = _region_node_id(region)
    goods   = [g.get("name", f"g{i}") for i, g in enumerate(gd.get("goods", []))]
    mps     = ss.get("magic_producers", [])
    region_mps = [m for m in mps if _id_int(m.get("node", -1)) == node_id]

    for m in region_mps:
        mid = _id_int(m.get("id", 0))
        with st.expander(f"Producer {mid} — {m.get('good', '?')}  ({m.get('qty_per_tick', 0)}/tick)"):
            _edit_magic_producer(m, mid, goods, ss)

    st.divider()
    if st.button("＋ Add magic producer", key=f"add_mp_{node_id}"):
        new_id = len(mps)
        mps.append({
            "id":           _make_id(new_id),
            "node":         _make_id(node_id if node_id is not None else 0),
            "good":         goods[0] if goods else "",
            "qty_per_tick": 1.0,
        })
        ss["magic_producers"] = mps
        st.session_state.dirty = True
        st.rerun()


def _edit_magic_producer(m: dict, mid: int, goods: list, ss: dict):
    good = m.get("good", goods[0] if goods else "")
    new_good = st.selectbox("Good", goods,
                            index=goods.index(good) if good in goods else 0,
                            key=f"mp_good_{mid}")
    if new_good != good:
        m["good"] = new_good
        st.session_state.dirty = True

    qty = st.number_input("Qty per tick", value=float(m.get("qty_per_tick", 1.0)),
                          min_value=0.0, step=0.5, key=f"mp_qty_{mid}")
    if abs(qty - float(m.get("qty_per_tick", 0.0))) > 1e-9:
        m["qty_per_tick"] = qty
        st.session_state.dirty = True

    if st.button("🗑 Delete producer", key=f"del_mp_{mid}", type="secondary"):
        ss["magic_producers"] = [x for x in ss.get("magic_producers", [])
                                 if _id_int(x.get("id", -1)) != mid]
        st.session_state.dirty = True
        st.rerun()


# ── Main map inspector ────────────────────────────────────────────────────────

def render_map_inspector(regions: list, gd: dict, ss: dict):
    sel = st.session_state.sel_region
    mode = st.session_state.mode

    if mode == "attach":
        st.info("Click another region to attach to it.")
        if st.button("Cancel"):
            st.session_state.mode = "normal"
            st.rerun()
        return

    if sel is None:
        st.caption("Click a region to select it.")
        return

    region = regions[sel]
    st.subheader(region.get("name", f"Region {sel}"))

    _tab_region(region, sel, regions)

def render_region_editor(regions: list, gd: dict, ss: dict):
    sel = st.session_state.sel_region
    if sel is None:
        return
    
    region = regions[sel]

    t_buildings, t_pops, t_mp = st.tabs(["Buildings", "Pops", "Magic Producers"])

    with t_buildings:
        _tab_buildings(sel, gd, ss)
    with t_pops:
        _tab_pops(sel, ss)
    with t_mp:
        _tab_magic_producers(region, gd, ss)


def render_map():
    gd = st.session_state.gd
    ss = st.session_state.get("ss", {})
    regions = gd.get("regions", [])

    map_col, info_col = st.columns([3, 1])

    with map_col:
        fig = build_map(gd, st.session_state.sel_region, st.session_state.mode)
        event = st.plotly_chart(fig, on_select="rerun", key="map_chart",
                                use_container_width=True)

        points = []
        try:
            points = event.selection.get("points", []) if event and event.selection else []
        except Exception:
            pass

        if points:
            clicked_idx = points[0].get("customdata")
            if isinstance(clicked_idx, (int, float)):
                clicked_idx = int(clicked_idx)
                if st.session_state.mode == "attach" and st.session_state.sel_region is not None \
                        and clicked_idx != st.session_state.sel_region:
                    ok = attach_region(regions, st.session_state.sel_region, clicked_idx)
                    st.session_state.mode = "normal"
                    if ok:
                        st.session_state.dirty = True
                    else:
                        st.warning("All sides of that region are occupied.")
                    st.rerun()
                else:
                    st.session_state.sel_region = clicked_idx
                    st.session_state.mode = "normal"
                    st.rerun()

    with info_col:
        render_map_inspector(regions, gd, ss)

    render_region_editor(regions, gd, ss)

# ── Goods page ────────────────────────────────────────────────────────────────

def render_goods():
    import pandas as pd
    gd = st.session_state.gd
    goods = gd.get("goods", [])

    st.subheader("Goods")

    rows = [{
        "name":              g.get("name", ""),
        "alpha":             float(g.get("alpha", 0.5)),
        "shelf_life":        bare_str(g.get("shelf_life", Bare("Indefinite"))),
        "movement_type":     bare_str(g.get("movement_type", Bare("Physical"))),
        "divisible":         bool(g.get("divisible", True)),
        "storage_cost_per_tick": float(g.get("storage_cost_per_tick", 0.0)),
    } for g in goods]

    edited = st.data_editor(
        pd.DataFrame(rows) if rows else pd.DataFrame(
            columns=["name", "alpha", "shelf_life", "movement_type", "divisible", "storage_cost_per_tick"]
        ),
        column_config={
            "name":              st.column_config.TextColumn("Name"),
            "alpha":             st.column_config.NumberColumn("Alpha", min_value=0.0, max_value=2.0, format="%.3f"),
            "shelf_life":        st.column_config.SelectboxColumn("Shelf Life", options=SHELF_LIFE_OPTIONS),
            "movement_type":     st.column_config.SelectboxColumn("Movement Type", options=MOVEMENT_OPTIONS),
            "divisible":         st.column_config.CheckboxColumn("Divisible"),
            "storage_cost_per_tick": st.column_config.NumberColumn("Storage Cost/Tick", format="%.4f"),
        },
        num_rows="dynamic",
        use_container_width=True,
        key="goods_editor",
    )

    if st.button("Apply goods changes"):
        new_goods = []
        for _, row in edited.iterrows():
            new_goods.append({
                "name":                  row["name"],
                "alpha":                 float(row["alpha"]),
                "shelf_life":            Bare(row["shelf_life"]),
                "movement_type":         Bare(row["movement_type"]),
                "divisible":             bool(row["divisible"]),
                "storage_cost_per_tick": float(row["storage_cost_per_tick"]),
            })
        gd["goods"] = new_goods
        st.session_state.dirty = True
        st.success("Goods updated.")

# ── Recipes page ──────────────────────────────────────────────────────────────

def render_recipes():
    import pandas as pd
    gd = st.session_state.gd
    recipes = gd.get("recipes", [])
    good_names = [g.get("name", "") for g in gd.get("goods", [])]

    st.subheader("Recipes")

    recipe_names = [r.get("name", f"recipe_{i}") for i, r in enumerate(recipes)]
    options = ["— new —"] + recipe_names

    # Track which recipe is selected in session state so switching resets editors
    if "recipe_sel" not in st.session_state:
        st.session_state.recipe_sel = "— new —"
    # If a recipe was just deleted or saved as new, reset to "— new —"
    if st.session_state.recipe_sel not in options:
        st.session_state.recipe_sel = "— new —"

    sel_name = st.selectbox("Select recipe", options,
                            index=options.index(st.session_state.recipe_sel),
                            key="recipe_sel")

    if sel_name == "— new —":
        recipe = {"name": "", "inputs": [], "outputs": [], "reversible": False}
        recipe_idx = None
    else:
        recipe_idx = recipe_names.index(sel_name)
        recipe = recipes[recipe_idx]

    # Use recipe_idx as part of widget keys so switching recipes resets the editors
    slot = recipe_idx if recipe_idx is not None else "new"

    name = st.text_input("Name", value=recipe.get("name", ""), key=f"recipe_name_{slot}")
    reversible = st.checkbox("Reversible", value=bool(recipe.get("reversible", False)),
                             key=f"recipe_rev_{slot}")

    st.markdown("**Inputs**")
    input_rows = [{
        "good":         inp.get("good", ""),
        "qty_per_unit": float(inp.get("qty_per_unit", 1.0)),
        "scaling":      bare_str(inp.get("scaling", Bare("Variable"))),
    } for inp in recipe.get("inputs", [])]
    edited_inputs = st.data_editor(
        pd.DataFrame(input_rows) if input_rows else
        pd.DataFrame(columns=["good", "qty_per_unit", "scaling"]),
        column_config={
            "good":         st.column_config.SelectboxColumn("Good", options=good_names),
            "qty_per_unit": st.column_config.NumberColumn("Qty/Unit", format="%.3f"),
            "scaling":      st.column_config.SelectboxColumn("Scaling", options=SCALING_OPTIONS),
        },
        num_rows="dynamic", use_container_width=True, key=f"recipe_inputs_{slot}",
    )

    st.markdown("**Outputs**")
    output_rows = [{
        "good":         out.get("good", ""),
        "qty_per_unit": float(out.get("qty_per_unit", 1.0)),
    } for out in recipe.get("outputs", [])]
    edited_outputs = st.data_editor(
        pd.DataFrame(output_rows) if output_rows else
        pd.DataFrame(columns=["good", "qty_per_unit"]),
        column_config={
            "good":         st.column_config.SelectboxColumn("Good", options=good_names),
            "qty_per_unit": st.column_config.NumberColumn("Qty/Unit", format="%.3f"),
        },
        num_rows="dynamic", use_container_width=True, key=f"recipe_outputs_{slot}",
    )

    col_save, col_del = st.columns([1, 1])
    if col_save.button("Save recipe", type="primary"):
        new_recipe = {
            "name":       name,
            "reversible": reversible,
            "inputs": [{
                "good":         row["good"],
                "qty_per_unit": float(row["qty_per_unit"]),
                "scaling":      Bare(row["scaling"]),
            } for _, row in edited_inputs.iterrows() if row.get("good")],
            "outputs": [{
                "good":         row["good"],
                "qty_per_unit": float(row["qty_per_unit"]),
            } for _, row in edited_outputs.iterrows() if row.get("good")],
        }
        if recipe_idx is None:
            recipes.append(new_recipe)
            st.session_state.recipe_sel = name  # jump to the new recipe
        else:
            recipes[recipe_idx] = new_recipe
        gd["recipes"] = recipes
        st.session_state.dirty = True
        st.rerun()

    if recipe_idx is not None:
        if col_del.button("Delete recipe", type="secondary"):
            recipes.pop(recipe_idx)
            gd["recipes"] = recipes
            st.session_state.recipe_sel = "— new —"
            st.session_state.dirty = True
            st.rerun()

# ── Needs & Wealth page ───────────────────────────────────────────────────────

def render_needs_wealth():
    import pandas as pd
    gd = st.session_state.gd
    good_names = [g.get("name", "") for g in gd.get("goods", [])]
    cats = gd.get("need_categories", [])
    wealth = gd.get("wealth_levels", [])

    # ── Need categories ────────────────────────────────────────────────────────
    st.subheader("Need categories")

    cat_names = [c.get("name", f"cat_{i}") for i, c in enumerate(cats)]
    cat_options = ["— new —"] + cat_names

    if "cat_sel" not in st.session_state:
        st.session_state.cat_sel = "— new —"
    if st.session_state.cat_sel not in cat_options:
        st.session_state.cat_sel = "— new —"

    sel_cat = st.selectbox("Category", cat_options,
                           index=cat_options.index(st.session_state.cat_sel),
                           key="cat_sel")

    if sel_cat == "— new —":
        cat, cat_idx = {"name": "", "entries": []}, None
    else:
        cat_idx = cat_names.index(sel_cat)
        cat = cats[cat_idx]

    slot = cat_idx if cat_idx is not None else "new"

    cat_name = st.text_input("Name", value=cat.get("name", ""), key=f"cat_name_{slot}")
    entry_rows = [{
        "good":           e.get("good", ""),
        "weight":         float(e.get("weight", 1.0)),
        "max_allocation": float(e.get("max_allocation", 1.0)),
        "min_allocation": float(e.get("min_allocation", 0.0)),
    } for e in cat.get("entries", [])]
    edited_entries = st.data_editor(
        pd.DataFrame(entry_rows) if entry_rows else
        pd.DataFrame(columns=["good", "weight", "max_allocation", "min_allocation"]),
        column_config={
            "good":           st.column_config.SelectboxColumn("Good", options=good_names),
            "weight":         st.column_config.NumberColumn("Weight", format="%.3f"),
            "max_allocation": st.column_config.NumberColumn("Max alloc", format="%.3f"),
            "min_allocation": st.column_config.NumberColumn("Min alloc", format="%.3f"),
        },
        num_rows="dynamic", use_container_width=True, key=f"cat_entries_{slot}",
    )

    col_save, col_del = st.columns([1, 1])
    if col_save.button("Save category", type="primary"):
        new_cat = {
            "name": cat_name,
            "entries": [{
                "good":           row["good"],
                "weight":         float(row["weight"]),
                "max_allocation": float(row["max_allocation"]),
                "min_allocation": float(row["min_allocation"]),
            } for _, row in edited_entries.iterrows() if row.get("good")],
        }
        if cat_idx is None:
            cats.append(new_cat)
            st.session_state.cat_sel = cat_name
        else:
            cats[cat_idx] = new_cat
        gd["need_categories"] = cats
        st.session_state.dirty = True
        st.rerun()

    if cat_idx is not None and col_del.button("Delete category", type="secondary"):
        cats.pop(cat_idx)
        gd["need_categories"] = cats
        st.session_state.cat_sel = "— new —"
        st.session_state.dirty = True
        st.rerun()

    st.divider()

    # ── Wealth levels ──────────────────────────────────────────────────────────
    st.subheader("Wealth levels")

    cat_names_real = [c.get("name", "") for c in cats]
    wealth_rows = []
    for wl in wealth:
        tier = wl.get("tier", 0)
        needs = wl.get("needs", {})
        row = {"tier": int(tier)}
        for cn in cat_names_real:
            row[cn] = float(needs.get(cn, 0.0)) if isinstance(needs, dict) else 0.0
        wealth_rows.append(row)

    col_config = {"tier": st.column_config.NumberColumn("Tier", min_value=0, step=1)}
    for cn in cat_names_real:
        col_config[cn] = st.column_config.NumberColumn(cn, format="%.3f")

    edited_wealth = st.data_editor(
        pd.DataFrame(wealth_rows) if wealth_rows else
        pd.DataFrame(columns=["tier"] + cat_names_real),
        column_config=col_config,
        num_rows="dynamic", use_container_width=True, key="wealth_editor",
    )

    if st.button("Apply wealth levels"):
        from ron import RonMap
        new_wealth = []
        for _, row in edited_wealth.iterrows():
            needs = RonMap({cn: float(row.get(cn, 0.0)) for cn in cat_names_real})
            new_wealth.append({"tier": int(row["tier"]), "needs": needs})
        new_wealth.sort(key=lambda w: w["tier"])
        gd["wealth_levels"] = new_wealth
        st.session_state.dirty = True
        st.success("Wealth levels updated.")

# ── Events page ──────────────────────────────────────────────────────────────

# Supported delta types for the editor
_DELTA_TYPES = [
    "SetMagicProducerQty",
    "AddToInventory",
    "RemoveFromInventory",
    "SetPrice",
]

_OWNER_TYPES = ["PopGroup", "Building", "Government", "MagicProducer"]


def _delta_summary(d) -> str:
    if isinstance(d, Tagged):
        inner = d.inner
        if isinstance(inner, dict):
            parts = [f"{k}={v}" for k, v in list(inner.items())[:3]]
            return f"{d.tag}({', '.join(parts)})"
        return f"{d.tag}({inner})"
    return str(d)


def _render_delta_editor(delta, idx: int, event_key: str, gd: dict) -> Tagged | None:
    """Edit one delta. Returns the updated Tagged delta, or None if deleted."""
    goods = [g.get("name", f"g{i}") for i, g in enumerate(gd.get("goods", []))]
    nodes = list(range(len(gd.get("market_nodes", []))))

    tag = delta.tag if isinstance(delta, Tagged) else _DELTA_TYPES[0]
    inner = delta.inner if isinstance(delta, Tagged) else {}

    cols = st.columns([3, 1])
    new_tag = cols[0].selectbox("Type", _DELTA_TYPES,
                                index=_DELTA_TYPES.index(tag) if tag in _DELTA_TYPES else 0,
                                key=f"{event_key}_tag_{idx}")
    delete = cols[1].button("✕", key=f"{event_key}_del_{idx}")
    if delete:
        return None

    result_inner: dict = {}

    if new_tag == "SetMagicProducerQty":
        mid = _id_int(inner.get("id", (0,))) if isinstance(inner, dict) else 0
        qty = float(inner.get("qty", 1.0)) if isinstance(inner, dict) else 1.0
        c1, c2 = st.columns(2)
        result_inner["id"] = _make_id(c1.number_input("Producer ID", value=mid, min_value=0,
                                                       step=1, key=f"{event_key}_mid_{idx}"))
        result_inner["qty"] = c2.number_input("Qty", value=qty, step=0.5,
                                              key=f"{event_key}_qty_{idx}")

    elif new_tag in ("AddToInventory", "RemoveFromInventory"):
        owner_val = inner.get("owner") if isinstance(inner, dict) else None
        owner_type = owner_val.tag if isinstance(owner_val, Tagged) else "PopGroup"
        owner_id = _id_int(owner_val.inner) if isinstance(owner_val, Tagged) else 0
        good_id = _id_int(inner.get("good", (0,))) if isinstance(inner, dict) else 0
        qty = float(inner.get("qty", 0.0)) if isinstance(inner, dict) else 0.0

        c1, c2, c3, c4 = st.columns(4)
        ot = c1.selectbox("Owner type", _OWNER_TYPES,
                          index=_OWNER_TYPES.index(owner_type) if owner_type in _OWNER_TYPES else 0,
                          key=f"{event_key}_ot_{idx}")
        oid = c2.number_input("Owner ID", value=owner_id, min_value=0, step=1,
                              key=f"{event_key}_oid_{idx}")
        gname = goods[good_id] if good_id < len(goods) else (goods[0] if goods else "")
        new_gname = c3.selectbox("Good", goods,
                                 index=goods.index(gname) if gname in goods else 0,
                                 key=f"{event_key}_good_{idx}")
        new_gid = goods.index(new_gname) if new_gname in goods else 0
        new_qty = c4.number_input("Qty", value=qty, step=1.0, key=f"{event_key}_qty_{idx}")
        result_inner["owner"] = Tagged(ot, _make_id(oid))
        result_inner["good"] = _make_id(new_gid)
        result_inner["qty"] = new_qty

    elif new_tag == "SetPrice":
        node = _id_int(inner.get("node", (0,))) if isinstance(inner, dict) else 0
        good_id = _id_int(inner.get("good", (0,))) if isinstance(inner, dict) else 0
        price = float(inner.get("price", 1.0)) if isinstance(inner, dict) else 1.0
        gname = goods[good_id] if good_id < len(goods) else (goods[0] if goods else "")
        c1, c2, c3 = st.columns(3)
        new_node = c1.number_input("Node ID", value=node, min_value=0, step=1,
                                   key=f"{event_key}_node_{idx}")
        new_gname = c2.selectbox("Good", goods,
                                 index=goods.index(gname) if gname in goods else 0,
                                 key=f"{event_key}_good_{idx}")
        new_gid = goods.index(new_gname) if new_gname in goods else 0
        new_price = c3.number_input("Price", value=price, min_value=0.0, step=0.1,
                                    key=f"{event_key}_price_{idx}")
        result_inner["node"] = _make_id(new_node)
        result_inner["good"] = _make_id(new_gid)
        result_inner["price"] = new_price

    return Tagged(new_tag, result_inner)


def _render_event_row(entry: tuple, entry_idx: int, section: str, ev_list: list, gd: dict,
                      label: str):
    tick_or_period, deltas = entry
    key = f"{section}_{entry_idx}"

    with st.expander(f"{label} {tick_or_period}  — {len(deltas)} delta(s)"):
        new_t = st.number_input(label, value=int(tick_or_period), min_value=1, step=1,
                                key=f"{key}_t")

        new_deltas = []
        for i, d in enumerate(deltas):
            st.markdown(f"**Delta {i}** — {_delta_summary(d)}")
            result = _render_delta_editor(d, i, key, gd)
            if result is not None:
                new_deltas.append(result)
            st.divider()

        if st.button("＋ Add delta", key=f"{key}_adddelta"):
            new_deltas.append(Tagged("SetMagicProducerQty", {"id": _make_id(0), "qty": 1.0}))
            st.session_state.dirty = True

        # Update the entry in place
        ev_list[entry_idx] = (new_t, new_deltas)
        if new_t != tick_or_period or new_deltas != list(deltas):
            st.session_state.dirty = True

        if st.button("🗑 Delete this event", key=f"{key}_delete", type="secondary"):
            ev_list.pop(entry_idx)
            st.session_state.dirty = True
            st.rerun()


def render_events():
    ev = st.session_state.get("ev", {})
    gd = st.session_state.gd

    st.subheader("Events")

    # One-shot
    st.markdown("### One-shot events  *(fire once at a given tick)*")
    one_shots = list(ev.get("events", []))

    for i, entry in enumerate(one_shots):
        _render_event_row(entry, i, "one", one_shots, gd, "Tick")

    if st.button("＋ Add one-shot event"):
        one_shots.append((1, [Tagged("SetMagicProducerQty", {"id": _make_id(0), "qty": 1.0})]))
        ev["events"] = one_shots
        st.session_state.ev = ev
        st.session_state.dirty = True
        st.rerun()

    ev["events"] = one_shots

    st.divider()

    # Recurring
    st.markdown("### Recurring events  *(fire every N ticks)*")
    recurring = list(ev.get("recurring", []))

    for i, entry in enumerate(recurring):
        _render_event_row(entry, i, "rec", recurring, gd, "Every N ticks")

    if st.button("＋ Add recurring event"):
        recurring.append((1, [Tagged("AddToInventory",
                                     {"owner": Tagged("PopGroup", _make_id(0)),
                                      "good": _make_id(0), "qty": 1.0})]))
        ev["recurring"] = recurring
        st.session_state.ev = ev
        st.session_state.dirty = True
        st.rerun()

    ev["recurring"] = recurring
    st.session_state.ev = ev


# ── Run page ──────────────────────────────────────────────────────────────────

def render_run():
    st.subheader("Run simulation")

    if not BINARY.exists():
        st.error(f"Binary not found: `{BINARY}`  —  run `cargo build` first.")
        return

    scenario = st.session_state.scenario_name
    default_out = str(REPO_ROOT / "tmp" / scenario)

    c1, c2, c3 = st.columns(3)
    ticks            = c1.number_input("Ticks", min_value=1, value=1000, step=100)
    telemetry_every  = c2.number_input("Sample telemetry every N ticks", min_value=1, value=1, step=1)
    output_dir       = c3.text_input("Output dir", value=default_out)

    if st.button("▶ Run"):
        with st.spinner("Running…"):
            result = run_simulation(
                scenario, ticks=ticks,
                output_dir=output_dir,
                record=True, certify=True,
                telemetry_every=int(telemetry_every),
            )
        if result.ok:
            note = " (certificate verdict: FAIL)" if result.certificate_failed else ""
            st.success(f"Done. Telemetry in `{output_dir}`{note}.")
        else:
            st.error(f"Simulation failed (exit {result.returncode}) - "
                     "output on disk is incomplete, do not analyse it.")
        if result.stdout:
            st.code(result.stdout[-3000:], language="text")
        if result.stderr:
            st.code(result.stderr[-2000:], language="text")


# ── Visualise page ────────────────────────────────────────────────────────────

# Metric catalogue ─────────────────────────────────────────────────────────────

# Each entry: label, unit, needs_good, needs_region, needs_building, needs_entity
_METRIC_DEFS = {
    "price":                ("Price",                  "price",  True,  True,  False, False),
    "supply":               ("Supply volume",          "volume", True,  True,  False, False),
    "demand":               ("Demand volume",          "volume", True,  True,  False, False),
    "imbalance":            ("Imbalance",              "ratio",  True,  True,  False, False),
    "building_balance":     ("Building balance (P&L)", "price",  False, False, True,  False),
    "building_chosen_size": ("Building chosen size",   "volume", False, False, True,  False),
    "building_recipe_size": ("Building recipe size",   "volume", False, False, True,  False),
    "building_margin":      ("Building margin",        "price",  False, False, True,  False),
    "pop_wealth":           ("Pop wealth (mean)",      "price",  False, True,  False, False),
    "pop_size":             ("Pop size",               "count",  False, True,  False, False),
    "inventory":            ("Inventory",              "qty",    True,  False, False, True),
    "price_index":          ("Price index",            "index",  False, False, False, False),
    "yoy_inflation":        ("YoY inflation %",        "pct",    False, False, False, False),
}

_DEFAULT_AXIS = {
    "price":  "L1", "index": "L1", "count": "L1",
    "volume": "R1", "ratio": "R1",
    "pct":    "R2",
}

_PALETTE = [
    "#1f77b4","#ff7f0e","#2ca02c","#d62728",
    "#9467bd","#8c564b","#e377c2","#7f7f7f",
]

_YAXIS_MAP = {"L1": "y", "R1": "y2", "L2": "y3", "R2": "y4"}
_AXIS_LABELS = ["L1", "R1", "L2", "R2"]


# ── Dashboard persistence ─────────────────────────────────────────────────────

import json as _json

def _dashboard_path(scenario: str) -> Path:
    return SCENARIOS_DIR / scenario / "dashboard.json"

def _save_dashboard(scenario: str, panels: list):
    _dashboard_path(scenario).write_text(_json.dumps({"panels": panels}, indent=2))

def _load_dashboard(scenario: str) -> list:
    p = _dashboard_path(scenario)
    return _json.loads(p.read_text()).get("panels", []) if p.exists() else []


# ── Building label helper ─────────────────────────────────────────────────────

def _make_bld_labeller(res: ScenarioResults):
    """Returns (bld_ids, label_fn) where label_fn maps building_id -> display string."""
    if res.buildings.empty:
        return [], lambda b: f"bld{b}"
    bld_snap = (res.buildings[["building_id", "region_id", "recipe_id", "channel_id"]]
                .drop_duplicates("building_id"))
    bld_lookup = {int(r["building_id"]): (int(r["recipe_id"]), int(r["region_id"]), int(r["channel_id"]))
                  for _, r in bld_snap.iterrows()}
    recipe_count: dict[int, int] = {}
    for rid, _, _ in bld_lookup.values():
        recipe_count[rid] = recipe_count.get(rid, 0) + 1

    def label(bid: int) -> str:
        if bid not in bld_lookup:
            return f"bld{bid}"
        rid, reg_id, c_id = bld_lookup[bid]
        rname = res.recipe_name.get(rid, f"bld{bid}")
        if recipe_count.get(rid, 1) > 1:
            if c_id != -1:
                from_id, to_id = res.channel_to_regions.get(c_id, (-1, -1))
                from_name = res.region_name.get(from_id, str(from_id))
                to_name = res.region_name.get(to_id, str(to_id))
                return f"{rname} · ({from_name}→{to_name})"
            return f"{rname} · {res.region_name.get(reg_id, str(reg_id))}"
        return rname

    return sorted(bld_lookup.keys()), label


# ── Series config helpers ─────────────────────────────────────────────────────

def _auto_label(cfg: dict, res: ScenarioResults) -> str:
    m = cfg["metric"]
    parts = [_METRIC_DEFS[m][0]]

    if m == "inventory":
        et = cfg.get("entity_type", "pop")
        if et == "building":
            _, label = _make_bld_labeller(res)
            parts.append(label(cfg.get("entity_id", 0)))
        else:
            eid = cfg.get("entity_id", 0)
            parts.append(f"{et} {eid}")
        if cfg.get("good_name"):
            parts.append(cfg["good_name"])
        return " · ".join(parts)

    if cfg.get("good_name"):
        parts.append(cfg["good_name"])
    if cfg.get("region_name"):
        parts.append(cfg["region_name"])
    elif _METRIC_DEFS[m][3]:
        parts.append("all regions")
    if cfg.get("building_id") is not None:
        _, label = _make_bld_labeller(res)
        parts.append(label(cfg["building_id"]))

    return " · ".join(parts)


def _resolve_region_id(name: str | None, res: ScenarioResults) -> int | None:
    if name is None:
        return None
    return next((k for k, v in res.region_name.items() if v == name), None)


def _get_series(cfg: dict, res: ScenarioResults):
    m = cfg["metric"]
    g = cfg.get("good_name")
    r = _resolve_region_id(cfg.get("region_name"), res)
    b = cfg.get("building_id")

    if m == "price":
        return res.prices_for(g, region_id=r)
    if m == "supply":
        return res.volume_for(g, "supply", region_id=r)
    if m == "demand":
        return res.volume_for(g, "demand", region_id=r)
    if m == "imbalance":
        return res.volume_for(g, "imbalance", region_id=r)
    if m == "building_balance":
        return res.building_series(b, "balance")
    if m == "building_chosen_size":
        return res.building_series(b, "chosen_size")
    if m == "building_recipe_size":
        return res.building_series(b, "recipe_size")
    if m == "building_margin":
        return res.building_series(b, "last_margin")
    if m == "pop_wealth":
        return res.pop_wealth_by_region(r)
    if m == "pop_size":
        df = res.pops if r is None else res.pops[res.pops["region_id"] == r]
        return df.groupby("tick")["size"].sum()
    if m == "inventory":
        return res.inventory_for(cfg["entity_type"], cfg["entity_id"], g)
    if m == "price_index":
        return res.price_index()
    if m == "yoy_inflation":
        return res.yoy_inflation()
    return None


def _build_figure(series_cfgs: list[dict], res: ScenarioResults) -> go.Figure:
    fig = go.Figure()
    axes_used: set[str] = set()

    for i, cfg in enumerate(series_cfgs):
        s = _get_series(cfg, res)
        if s is None or s.empty:
            continue
        ax = cfg.get("axis", "L1")
        axes_used.add(ax)
        fig.add_trace(go.Scatter(
            x=s.index.tolist(), y=s.tolist(),
            name=cfg.get("label", f"series {i}"),
            line=dict(color=cfg.get("color", _PALETTE[i % len(_PALETTE)])),
            yaxis=_YAXIS_MAP[ax],
        ))

    domain = [0.0, 1.0]
    if "L2" in axes_used:
        domain[0] = 0.12
    if "R2" in axes_used:
        domain[1] = 0.88

    common = dict(showgrid=False, zeroline=False)
    fig.update_layout(
        height=520,
        legend=dict(orientation="h", y=-0.15),
        xaxis=dict(title="tick", domain=domain, showgrid=True, gridcolor="#aaa"),
        yaxis=dict(title="L1", side="left", showgrid=True, gridcolor="#aaa"),
        yaxis2=dict(title="R1", overlaying="y", side="right", **common)
            if "R1" in axes_used else dict(visible=False),
        yaxis3=dict(title="L2", overlaying="y", side="left",
                    anchor="free", position=0.0, **common)
            if "L2" in axes_used else dict(visible=False),
        yaxis4=dict(title="R2", overlaying="y", side="right",
                    anchor="free", position=1.0, **common)
            if "R2" in axes_used else dict(visible=False),
    )
    return fig


def _viz_init():
    if "viz_output_dir" not in st.session_state:
        st.session_state.viz_output_dir = ""
    if "viz_results" not in st.session_state:
        st.session_state.viz_results = None
    if "viz_series" not in st.session_state:
        st.session_state.viz_series = []
    if "viz_add_open" not in st.session_state:
        st.session_state.viz_add_open = True
    if "viz_dashboard" not in st.session_state:
        st.session_state.viz_dashboard = _load_dashboard(
            st.session_state.get("scenario_name", ""))


def _filter_res(res: ScenarioResults, lo: int, hi: int) -> ScenarioResults:
    def f(df): return df[(df["tick"] >= lo) & (df["tick"] <= hi)]
    return ScenarioResults(
        good_name=res.good_name, recipe_name=res.recipe_name,
        region_name=res.region_name, node_to_region=res.node_to_region,
        node_currency=res.node_currency,
        channel_to_regions=res.channel_to_regions,
        prices=f(res.prices), buildings=f(res.buildings),
        pops=f(res.pops), inventories=f(res.inventories),
    )


def _render_panel(panel: dict, idx: int, res: ScenarioResults, tick_range: tuple):
    series = panel.get("series", [])
    c_title, c_del = st.columns([8, 1])
    c_title.markdown(f"**{panel.get('title', f'Panel {idx+1}')}**")
    if c_del.button("✕", key=f"dash_del_{idx}"):
        st.session_state.viz_dashboard.pop(idx)
        _save_dashboard(st.session_state.scenario_name, st.session_state.viz_dashboard)
        st.rerun()
    fres = _filter_res(res, *tick_range)
    fig = _build_figure(series, fres)
    fig.update_layout(height=360, margin=dict(t=10, b=40))
    st.plotly_chart(fig, use_container_width=True, key=f"dash_fig_{idx}")


def render_visualise():
    _viz_init()
    st.subheader("Visualise")

    # ── Output dir + load ─────────────────────────────────────────────────────
    scenario = st.session_state.scenario_name
    default_out = str(REPO_ROOT / "tmp" / scenario)

    c1, c2 = st.columns([4, 1])
    out_dir = c1.text_input("Output dir (containing telemetry.parquet)", value=default_out,
                            key="viz_out_input")
    if c2.button("Load results", use_container_width=True):
        with st.spinner("Loading telemetry…"):
            try:
                # The manifest carries the dimension tables, so no scenario needs
                # to be loaded first to name a good or a region.
                st.session_state.viz_results = load_scenario_results(out_dir)
                st.session_state.viz_output_dir = out_dir
                st.session_state.viz_series = []
                st.session_state.viz_dashboard = _load_dashboard(scenario)
                st.rerun()
            except Exception as e:
                st.error(f"Load failed: {e}")

    res: ScenarioResults | None = st.session_state.viz_results
    if res is None:
        st.info("Load a run output above to start visualising.")
        return

    if res.prices.empty:
        st.warning("That run's telemetry has no market rows. "
                   "Re-run the scenario with telemetry enabled.")
        return

    ticks = sorted(res.prices["tick"].unique())
    tick_range = st.slider("Tick range", min_value=int(ticks[0]), max_value=int(ticks[-1]),
                           value=(int(ticks[0]), int(ticks[-1])), key="viz_tick_range")

    st.caption(f"{len(ticks)} ticks · goods: {list(res.good_name.values())} "
               f"· regions: {list(res.region_name.values())}")

    # ── Active chart builder ──────────────────────────────────────────────────
    st.divider()
    st.markdown("**Chart builder**")

    series_cfgs: list[dict] = st.session_state.viz_series
    bld_ids, bld_label = _make_bld_labeller(res)

    for i, cfg in enumerate(series_cfgs):
        c_lbl, c_ax, c_del = st.columns([6, 1, 1])
        c_lbl.markdown(f"**{cfg.get('label', '?')}**  —  axis {cfg.get('axis','L1')}")
        new_ax = c_ax.selectbox("axis", _AXIS_LABELS,
                                index=_AXIS_LABELS.index(cfg.get("axis", "L1")),
                                key=f"ax_{i}", label_visibility="collapsed")
        if new_ax != cfg.get("axis"):
            series_cfgs[i]["axis"] = new_ax
        if c_del.button("✕", key=f"del_{i}"):
            series_cfgs.pop(i)
            st.rerun()

    # ── Add series ────────────────────────────────────────────────────────────
    with st.expander("＋ Add series", expanded=st.session_state.viz_add_open):
        metric_keys   = list(_METRIC_DEFS.keys())
        metric_labels = [_METRIC_DEFS[k][0] for k in metric_keys]

        sel_m = st.selectbox("Metric", metric_labels, key="viz_metric")
        metric = metric_keys[metric_labels.index(sel_m)]
        _, unit, needs_good, needs_region, needs_building, needs_entity = _METRIC_DEFS[metric]

        good_name   = None
        region_name = None
        building_id = None
        entity_type = None
        entity_id   = None

        if needs_good:
            good_name = st.selectbox("Good", list(res.good_name.values()), key="viz_good")

        if needs_region:
            region_opts = [None] + list(res.region_name.values())
            region_name = st.selectbox("Region", region_opts,
                format_func=lambda r: "All regions (mean)" if r is None else r,
                key="viz_region")

        if needs_building:
            building_id = st.selectbox("Building", bld_ids,
                                       format_func=bld_label, key="viz_building")

        if needs_entity:
            entity_type = st.selectbox("Entity type", ["pop", "building"], key="viz_etype")
            if entity_type == "pop":
                entity_id = st.number_input("Pop ID", min_value=0, step=1, value=0,
                                            key="viz_entity_pop")
            else:
                entity_id = st.selectbox("Building", bld_ids,
                                         format_func=bld_label, key="viz_entity_bld")

        suggested = _DEFAULT_AXIS.get(unit, "L1")
        for s in series_cfgs:
            if _METRIC_DEFS[s["metric"]][1] == unit:
                suggested = s.get("axis", suggested)
                break
        axis = st.selectbox("Y axis", _AXIS_LABELS, index=_AXIS_LABELS.index(suggested),
                            help="Same-unit metrics default to the same axis.",
                            key="viz_axis")
        color = st.color_picker("Colour", value=_PALETTE[len(series_cfgs) % len(_PALETTE)],
                                key=f"viz_color_{len(series_cfgs)}")

        if st.button("Add series", type="primary"):
            cfg = dict(metric=metric, good_name=good_name, region_name=region_name,
                       building_id=building_id, entity_type=entity_type,
                       entity_id=int(entity_id) if entity_id is not None else None,
                       axis=axis, color=color)
            cfg["label"] = _auto_label(cfg, res)
            series_cfgs.append(cfg)
            st.session_state.viz_series = series_cfgs
            st.session_state.viz_add_open = False
            st.rerun()

    if series_cfgs:
        fig = _build_figure(series_cfgs, _filter_res(res, *tick_range))
        st.plotly_chart(fig, use_container_width=True, key="viz_active_chart")

        c_pin, c_clear = st.columns([2, 1])
        if c_pin.button("📌 Add to dashboard", type="primary", use_container_width=True):
            title = " / ".join(c["label"] for c in series_cfgs[:2])
            if len(series_cfgs) > 2:
                title += f" + {len(series_cfgs)-2} more"
            st.session_state.viz_dashboard.append(
                {"title": title, "series": list(series_cfgs)})
            _save_dashboard(scenario, st.session_state.viz_dashboard)
            st.session_state.viz_series = []
            st.session_state.viz_add_open = True
            st.rerun()
        if c_clear.button("Clear", use_container_width=True):
            st.session_state.viz_series = []
            st.rerun()

    # ── Dashboard ─────────────────────────────────────────────────────────────
    panels: list[dict] = st.session_state.viz_dashboard
    if panels:
        st.divider()
        st.markdown("**Dashboard**")
        for i in range(0, len(panels), 2):
            cols = st.columns(2)
            for j, col in enumerate(cols):
                if i + j < len(panels):
                    with col:
                        _render_panel(panels[i + j], i + j, res, tick_range)

# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    st.set_page_config(page_title="RustyEcon", layout="wide")
    _init()
    render_sidebar()

    if "gd" not in st.session_state:
        st.info("Load a scenario from the sidebar.")
        return

    page = st.session_state.page
    if page == "Map":
        render_map()
    elif page == "Goods":
        render_goods()
    elif page == "Recipes":
        render_recipes()
    elif page == "Needs & Wealth":
        render_needs_wealth()
    elif page == "Events":
        render_events()
    elif page == "Run":
        render_run()
    elif page == "Visualise":
        render_visualise()


if __name__ == "__main__":
    main()
