# Pocket City 2 save lifecycle and sandbox rules

These observations describe the Windows Steam build with `Assembly-CSharp.dll` SHA-256 `2c2c55413241432ed6fe1aaa22d7f9879c4b174b55b38eecc997f6ab815e8fe6`. They come from REA's managed metadata and CIL, Ghidra reads of the same method bodies, targeted ILSpy output, and local save samples.

## Mode and consequences

Both `GameController.isSandbox(GameState)` and `GameController.isSandbox(GameStateStub)` return the logical OR of `unlockAll`, `infiniteMoney`, `disableQuests`, and `maxLevel`. A normal save can therefore start being treated as sandbox after changing only one option. `CreateCityController.ConfirmStartAdditionalCity` always enables `unlockAll` when creating sandbox cities, while the other three options come from the selected toggles. The original sandbox creation route also disables tutorial and survival.

| Field | Observed behavior |
| --- | --- |
| `unlockAll` | Enables the broad unlock path. Building-specific difficulty checks still apply to that path, and force-unlock entries have their own priority. New-city initialization also grants research points and furniture. |
| `infiniteMoney` | Sets the balance to `999999999999` in the running city and sets resources to their maxima during city startup. |
| `disableQuests` | Suppresses the automatic quest check in `QuestManager`. |
| `maxLevel` | Initializes a new city at level 999 and stops the normal accumulation of new XP. Existing saves load their stored level through a separate path. |

`CityRowUI.Sync` displays survival before considering sandbox and displays no level in either case. Survival is stored independently in `isSurvivalMode`. Its city-specific goal, lifeline and upgrade data are separate from the global survival settings. The editor protects the survival-mode flag and prevents enabling sandbox options on survival cities.

When a sandbox city loads, `StartCityAfterLoadTiles` attempts to unlock every vehicle. `UnlockCar` has further game-mode and vehicle predicates. `Steam.IsAchievableCity` excludes sandbox, competition, unsaved downloaded and collaborating cities. The editor reports sandbox status from the pending file values. Clearing a sandbox flag does not reverse changes previously applied by the game, including any vehicles it already granted.

Direct edits to balance, XP, level, research and `forceUnlockedBuildings` do not enter the sandbox predicate. Other gameplay conditions can still affect the outcome. Building placement, capacity and costs are distinct from the unlock test.

## Files, selection and recovery

- A city uses `city_<FILE_ID>.es3`. Automatic and manual archives append `_auto_` or `_manual_` and a locale-formatted timestamp. The name timestamp is for archive naming. Selection compares filesystem modification times.
- `IO.GetLatestFileName` first considers the primary file. If it is absent, the newest automatic archive becomes the candidate. A manual save replaces the candidate only when its modification time is strictly newer.
- `IO.GetSavedStateWithBackup` first loads the requested candidate. If that fails, it tries the game's archive-recovery path. The save-list path uses a `GameStateStub` to read the four sandbox flags, survival status and city summary without deserializing the whole city.
- Primary and manual writes stage an `_tmp` file and read it back before completing. On primary saves, an older primary may be rotated to an automatic archive after a gap exceeding five minutes. The game prunes the oldest autosave when there are more than 15. In Survival, the game refuses manual version selection and opens the latest available save.
- `global_settings` stores game preferences, difficulty records and global flags. `os_settings` stores rendering, graphics and camera controls. `survival_global_settings` stores records and upgrade spending for future survival runs.
- `deletionslog` is a comma-separated deletion journal. Steam cloud integration can replay it at startup. It is excluded from the editor.
- Map dimensions, regional links, the seed and tile grids form a coupled structure. The editor exposes dimensions and structural IDs as read-only.

## Additional collection semantics

- `resourceAutoSellKeepAmt` and `resourceAutoConvertKeepAmt` use a resource ID as the key and the stock amount to keep as the value. A missing key or value of `-1` disables the rule. Automation can edit a dictionary that is absent or empty in older saves.
- `researched` and `disabledResearch` both test key presence. The Progression tab pairs the dictionaries into locked, active, and unlocked with the effect disabled. The editor removes the disable key when enabling the effect. In legacy files it creates an empty disable dictionary as needed because the game reads it when testing active effects.
- `upgradesSpentCity` is separate from the global `upgradesSpent` dictionary. The city loader indexes available upgrade actions using the stored tier minus one, so city entries must use positive tiers. The editor exposes existing entries using the known per-upgrade limits.
- `Tooltips.TooltipScreen.CheckShow` treats a tooltip as recorded when its ID is present in either `globalFlags` or the city's `flags`. The stored Boolean is ignored for this check. IDs 20, 22 and 24 correspond to elevated rail construction, rail commuters from outside the city, and disaster reports and repairs respectively. The editor shows these entries as recorded tips with a short description and protects the internal flags from Boolean edits.

## Binary anchors

REA evidence `ev_bedf4aa67394ebd561271b1661c5196635da4c0657a1836b993106e0c815ef5e` records the managed methods. A Ghidra headless script read their CIL bodies from the imported PE and matched the SHA-256 hashes for both sandbox predicates, creation, city load, startup, version selection and recovery.

| Method | Ghidra address |
| --- | --- |
| `LoadExistingCityState` | `00515a94` |
| `StartCityAfterLoadTiles` | `00515c38` |
| `isSandbox(GameState)` | `00517330` |
| `isSandbox(GameStateStub)` | `00517352` |
| `ConfirmStartAdditionalCity` | `00553f94` |
| `GetLatestFileName` | `0058db2c` |
| `GetSavedStateWithBackup` | `0058dfb4` |

Settings and control types are covered by REA evidence `ev_0e1e15da71133d6094ba94914ab0296ae7d5853cf188e81f9c564c3613f3b4bc`. The sandbox classification and gameplay effects above are static findings. Editor tests exercise the saved flags and settings on copies. Steam cloud sync and live achievement behavior were outside those tests.
