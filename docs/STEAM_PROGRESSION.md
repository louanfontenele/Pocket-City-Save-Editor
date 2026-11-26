# Progression, unlocks and rewards in the Steam saves

These observations are tied to the two installed builds recorded in the [Steam save schema](STEAM_SAVE_SCHEMA.md). The [progression catalogs](STEAM_PROGRESSION_CATALOGS.md) list 66 PC1 quest IDs and, for PC2, 179 quest templates, 52 research nodes and 10 buildings with explicit affinity requirements.

## Pocket City

### Building access and treasure rewards

`Structures.isBuildingUnlocked` compares the building's minimum level with `City.getLevelWithCentral()`, the higher of the current city's level and its central city's level. Sandbox satisfies the level check. The three treasure structures use minimum level `-1` and also check their individual rewards. In a normal city, each one can be granted independently using its sheet string as a key in `stats.globalSettings.chestunlockbuildings`, with the value `1` used by the game. The lookup tests the value's truthiness.

| Structure | Building ID | Reward key |
| --- | --- | --- |
| Statue of Courage | 33 | `statuecourage` |
| Castle | 37 | `castle` |
| Obelisk | 48 | `obelisk` |

The reward key and the chest location record serve separate purposes. `chest_saved` uses `row_col` keys and controls which physical chests have already been opened. The normal acceptance action consumes the reward, then writes the location record. Granting a building through its reward key can leave every physical chest and its record intact. This path applies to the three treasure structures. Ordinary building access still uses the runtime level and other build conditions such as placement, capacity and cost.

### Missions and level rewards

The four mission lists contain numeric quest IDs. The game loads them into the quest module and writes them back together before a snapshot.

| Save field | Meaning |
| --- | --- |
| `stats.activeQuests` | Available objectives the game checks for completion. |
| `stats.newQuests` | Unread marker for an active quest. Automatic completion waits for the marker to clear. |
| `stats.completedQuests` | Objective met and reward ready to claim. |
| `stats.archivedQuests` | Quest filed away after reward acceptance. Prerequisite checks use this list. |

`finishCompleteQuest` archives the ID, removes it from the other lists, runs the reward callback and schedules an autosave. Inserting an ID in the archive by editing the file bypasses the callback. Use distinct actions for marking an objective ready to claim and recording that its reward has already been resolved. Unknown quest IDs are unsafe in active or completed arrays because those paths index the catalog directly.

`stats.pendingRewardLevels` counts unclaimed level-up reward screens. The next screen uses `level - (pendingRewardLevels - 1)` to select the reward level. A direct level adjustment should leave this counter alone unless the user specifically intends to change reward availability. The ordinary level-up routine separately increments the counter and updates difficulty records.

## Pocket City 2

### Affinity, visits and gifts

`Relationships.GetRelationship` resolves the incoming NPC to its affinity profile. If the profile key is absent, the effective value is `INT_BASE_RELATIONSHIP`. When present, the effective value is the saved score plus `INT_BASE_RELATIONSHIP - 1`. The shipped default base is 1. The increase routine clamps the stored number to 0–999. A stored 0 and a missing entry can therefore produce different results.

Ten buildings query the effective score at level 50 using the profiles in the catalog. Raising the stored score can satisfy the affinity test when the game evaluates it. The separate unlock animation, daily activity and gift queue are handled outside the setter for the saved value. The ordinary affinity routine counts multiples of five crossed, queues gifts outside Survival, updates a milestone and emits a relationship-change event.

| Save field | Meaning |
| --- | --- |
| `relationships[profile]` | Score used for affinity checks on the resolved profile. |
| `pendingGifts[profile]` | Number of rewards waiting to be claimed. The claim path directly reads the matching affinity entry and removes the queue key after the last claim. |
| `lastGaveGift[npc]` | City day on which the mayor gave an item. The repeat check compares it with the current game day and the shipped interval of one day. |
| `lastVisitedDay[npc]` | Day recorded by the building visit path. That path updates the date when at least two city days have elapsed. |

Use the profile selected by the game for score edits. Preserve the separate NPC keys in visit and gift histories. Keep the affinity entry present when a matching pending gift is retained.

### Research and other building gates

Both research dictionaries use key presence. A missing key in `researched` means the node is locked. A key in `researched` alone means it is unlocked and active. A key in both `researched` and `disabledResearch` means it is unlocked and disabled. A Boolean value of `false` still counts as present in either dictionary. The building research gate checks unlock status and can therefore pass even if the research effect is disabled.

The research UI checks the prerequisite and affordability. `CanAffordResearch` checks the configured research points and resource stock independently. `UnlockResearch` deducts the points if the point cost is positive and uses the resource deduction branch only otherwise. It also invokes feature callbacks, so inserting a key alone is a saved state change. The 52-row catalog describes affordability requirements as defined by the game.

The normal `Buildings.isUnlocked` path can apply an Expert-specific predicate, regional restriction, affinity, level, difficulty records, research, competition and Survival checks. Normal level checks use the higher of the current and parent city levels. Survival skips the normal level and competition checks and has a separate days-survived requirement for some buildings. A difficulty-record gate checks for both level 100 and population 50,000 at the specified difficulty. Force-unlock key presence returns early from these building unlock checks. Capacity and placement use separate predicates.

### Quest objective, turn-in and history

`availableQuests` and `activeQuests` each store quest objects. The objects carry a quest type, a registered possibility key, NPC identity, rewards, progress, source location, language and step flags. `QuestPossibility.InitQuestCreatorPossibleQuests` registers 179 distinct template IDs, which the generator copies into `questPossibilityKey`. `completedQuests` stores historical results under `completedQuests[relationNPC][questPossibilityKey]`, using the original quest NPC even when several NPCs share an affinity profile.

- `SetMetadataStep1Complete` marks the first step and can start step two. `SetMetadataCompleteQuest` sets the active object's `completed` flag and posts a completion event. The quest stays available for turn-in.
- The normal `CompleteAndremoveQuest(QuestSettings)` route calls the type-specific completion callback, removes the active entry, records `true` in history, and separately handles rewards and affinity. Writing history alone bypasses these game actions. Resource-delivery quest types can have inventory side effects in their type-specific completion callback.
- `IsQuestIDCompleted` checks key presence. `HasQuestBeenCompletedBefore` reads the stored Boolean. Both feed quest generation checks, so `false` has inconsistent results across paths. A coherent historical completion uses `true`, and absence represents a quest without a completion record.

## Verification

For PC1, REA analyzed the installed JS bundle under `ev_05e92a50254385f5b860c0f6ae8d9f6a81c563b24652f4b6e82f8d0f369a804a`. Focused source reads traced the registration, load, claim and save callers. The catalog resolves every `_pushQuest` registration to a unique numeric ID from that bundle.

For PC2, REA inspected managed members under `ev_b21ab439a113c0b09de76d08f8447180a496ec02df065207626e146ede99109f`. Focused ILSpy output exposed the surrounding control flow. Ghidra independently matched the REA CIL body hashes for these ten method addresses in the same Steam assembly.

| Method | Ghidra address |
| --- | --- |
| `QuestManager.IsQuestIDCompleted` | `004b2973` |
| `QuestManager.SetMetadataCompleteQuest` | `004b2a04` |
| `QuestManager.CompleteAndremoveQuest(QuestSettings)` | `004b2cb0` |
| `QuestManager.CompleteAndremoveQuest(int)` | `004b2dbc` |
| `QuestManager.HasQuestBeenCompletedBefore` | `004b2e6c` |
| `Relationships.GetRelationship` | `004bd060` |
| `Relationships.AddRelationship` (five parameters) | `004bd0d0` |
| `Research.isUnlocked` | `004bf54d` |
| `Research.isActive` | `004bf564` |
| `Research.UnlockResearch` | `004bf6f0` |

These checks establish static behavior in the fingerprinted builds. Reopening edited saves and claiming generated rewards in the game remain unverified. Creating PC2 quest instances would also require resolving each template's runtime arguments, placement and reward logic.

### Additional verification for the editor controls

- `PetManager.SetPetFollowing` at `00447abc` calls `SetAllPetsStopFollowing` at `00447a04` before assigning the follower. `RefreshOwnedPetList` at `00447f38` and `AddWellBeing` at `004483c8` read the separate care state. The editor mirrors the single-follower rule for existing pets and writes well-being without changing care dates. Ghidra matched these CIL bodies to REA evidence `ev_23445a6161f5d36609971e259c024091994c35873c7df4a17b913c48ba7cbc5e`.
- `CityEvents.IsInCooldown` tests `eventCooldowns.ContainsKey(id)`. The normal timer deletes the entry when its remaining time reaches zero. The editor removes the selected key to match that state, including when the file contains a stale zero. Event definitions, level, costs and other launch requirements stay under the game's own checks. REA inspected `CityEvents` and `EventRow` under `ev_910daa5dfa5ab75a1d67d69fc126ccdea1c3288bf727568fd1b6907f6581bc0f`.
