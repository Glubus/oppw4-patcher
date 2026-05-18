# Rewards Director Plugin Idea

Date: 2026-05-18

## Goal

Create a future `rewards_director` plugin that changes mission rewards based on
the selected difficulty. The plugin should modify the final awarded values, not
the result-screen display.

## Intended Behavior

The clean target is the end-of-mission reward transaction:

1. The game finishes a mission.
2. It computes base rewards.
3. It prepares the final reward/result data.
4. The plugin reads current difficulty.
5. The plugin multiplies selected reward values.
6. The game saves/applies the already-modified values normally.

Reward categories to investigate:

- character XP
- Berry
- medals
- souls

## Proposed Lua Shape

```lua
local rewards = require("rewards_director")

rewards.scale_by_difficulty({
  easy = 1.0,
  normal = 1.0,
  hard = 1.5,
  ultra_hard = 2.0,

  berry = true,
  character_xp = true,
  medals = true,
  souls = true,
})
```

Alternative callback shape:

```lua
local rewards = require("rewards_director")

rewards.on_mission_clear(function(ctx)
  local multiplier = ({
    easy = 1.0,
    normal = 1.0,
    hard = 1.5,
    ultra_hard = 2.0,
  })[ctx.difficulty] or 1.0

  ctx:scale({
    berry = multiplier,
    character_xp = multiplier,
    medals = multiplier,
    souls = multiplier,
  })
end)
```

## Reverse Targets

- Current difficulty state in RAM.
- Mission-clear/result finalization function.
- Temporary result/reward structure before save/profile write.
- Individual writers for Berry, character XP, medals, and souls.

## Important Constraint

Do not patch UI display values. Patch the final reward values immediately before
they are committed to the player profile/save data, otherwise the result screen
and actual saved values may diverge.

## Suggested Implementation Order

1. Add a no-op `rewards_director` plugin skeleton with config and Lua module.
2. Add debug logging for candidate mission-clear hooks once reverse targets are
   found.
3. Implement Berry only as the first real reward multiplier.
4. Extend to character XP, medals, and souls after confirming they share the same
   transaction path or after locating their separate commit functions.
