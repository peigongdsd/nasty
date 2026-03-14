from nasty.context import TestContext
from nasty.output import header, info


async def test_snapshots(ctx: TestContext):
    """Snapshot read_only flag and clone-to-subvolume."""
    header("Snapshot Tests")

    tag = ctx.tag
    sv_name = f"test-snap-parent-{tag}"

    info(f"Creating parent subvolume '{sv_name}'...")
    try:
        await ctx.client.call("subvolume.create", {
            "pool": ctx.pool,
            "name": sv_name,
            "subvolume_type": "filesystem",
        })
        ctx.record("snapshots: parent create", True)
    except Exception as e:
        ctx.record("snapshots: parent create", False, str(e))
        return

    # ── 1. Read-only snapshot ────────────────────────────────────

    ro_snap = f"snap-ro-{tag}"
    info(f"Creating read-only snapshot '{ro_snap}'...")
    try:
        snap = await ctx.client.call("snapshot.create", {
            "pool": ctx.pool,
            "subvolume": sv_name,
            "name": ro_snap,
            "read_only": True,
        })
        ctx.record("snapshots: create read-only", True)
    except Exception as e:
        ctx.record("snapshots: create read-only", False, str(e))
        snap = None

    if snap is not None:
        info("Verifying read_only=True in snapshot.list...")
        all_snaps = await ctx.client.call("snapshot.list", {"pool": ctx.pool})
        match = next(
            (s for s in all_snaps if s["name"] == ro_snap and s["subvolume"] == sv_name),
            None,
        )
        if match:
            is_ro = match.get("read_only") is True
            ctx.record("snapshots: read_only flag true", is_ro,
                       "" if is_ro else f"read_only={match.get('read_only')!r}")
        else:
            ctx.record("snapshots: read_only flag true", False, "snapshot not found in list")

    # ── 2. Writable snapshot ─────────────────────────────────────

    rw_snap = f"snap-rw-{tag}"
    info(f"Creating writable snapshot '{rw_snap}'...")
    try:
        snap_rw = await ctx.client.call("snapshot.create", {
            "pool": ctx.pool,
            "subvolume": sv_name,
            "name": rw_snap,
            "read_only": False,
        })
        ctx.record("snapshots: create writable", True)
    except Exception as e:
        ctx.record("snapshots: create writable", False, str(e))
        snap_rw = None

    if snap_rw is not None:
        info("Verifying read_only=False in snapshot.list...")
        all_snaps = await ctx.client.call("snapshot.list", {"pool": ctx.pool})
        match_rw = next(
            (s for s in all_snaps if s["name"] == rw_snap and s["subvolume"] == sv_name),
            None,
        )
        if match_rw:
            is_rw = match_rw.get("read_only") is False
            ctx.record("snapshots: read_only flag false", is_rw,
                       "" if is_rw else f"read_only={match_rw.get('read_only')!r}")
        else:
            ctx.record("snapshots: read_only flag false", False, "snapshot not found in list")

    # ── 3. Clone snapshot → new writable subvolume ───────────────

    clone_name = f"test-snap-clone-{tag}"
    if snap is not None:  # need a snapshot to clone; use the ro one
        info(f"Cloning '{ro_snap}' → new subvolume '{clone_name}'...")
        try:
            cloned = await ctx.client.call("snapshot.clone", {
                "pool": ctx.pool,
                "subvolume": sv_name,
                "snapshot": ro_snap,
                "new_name": clone_name,
            })
            ctx.record("snapshots: clone creates subvolume", True)
        except Exception as e:
            ctx.record("snapshots: clone creates subvolume", False, str(e))
            cloned = None

        if cloned is not None:
            info("Verifying clone appears in subvolume.list...")
            svs = await ctx.client.call("subvolume.list", {"pool": ctx.pool})
            found = any(s["name"] == clone_name for s in svs)
            ctx.record("snapshots: clone in subvolume list", found,
                       "" if found else f"'{clone_name}' not in list")

            if not ctx.skip_delete:
                try:
                    await ctx.client.call("subvolume.delete", {"pool": ctx.pool, "name": clone_name})
                except Exception:
                    pass

    # ── Cleanup ──────────────────────────────────────────────────

    if not ctx.skip_delete:
        for snap_name in [ro_snap, rw_snap]:
            try:
                await ctx.client.call("snapshot.delete", {
                    "pool": ctx.pool,
                    "subvolume": sv_name,
                    "name": snap_name,
                })
            except Exception:
                pass
        try:
            await ctx.client.call("subvolume.delete", {"pool": ctx.pool, "name": sv_name})
        except Exception:
            pass
