# Save schema in the Windows Steam builds

This pass used the installed game files as inputs. The companion [field index](STEAM_SAVE_FIELDS.md) includes 33 PC1 policies, all 127 declared PC2 city fields, and PC2's preference, display, Survival and pet state types. See [save formats](SAVE_FORMATS.md) for the file wrappers and [PC2 save lifecycle](POCKET_CITY_2_SAVES.md) for archive recovery and sandbox classification.

| Steam build | Installed artifact | SHA-256 |
| --- | --- | --- |
| Pocket City `9765658` | `Pocket City.exe` | `c2222de98bdd0fa7825391c82d596fdea6b6f039ebe377a6fb099e12369d0ab3` |
| Pocket City `9765658` | `resources/app/app_en.js` | `9641add4d676057628e214bc85c7fa5289a59f2b67e00100912bc13352c3f2d7` |
| Pocket City 2 `17960799` | `Pocket City 2_Data/Managed/Assembly-CSharp.dll` | `2c2c55413241432ed6fe1aaa22d7f9879c4b174b55b38eecc997f6ab815e8fe6` |

## Pocket City

### What each snapshot contains

`City.createSaveSnapshot` writes `timestamp` as Unix milliseconds, then `metadata`, `stats` and `grids`. It adds `rng` when the city uses random terrain and the generator has a truthy seed. Both snapshot lists put the newest entry first and retain at most five entries each when the game pushes a snapshot. The first automatic snapshot wins an equal-timestamp tie with the first manual snapshot.

`City.loadSaveStateIntoCity` overlays saved values onto the city defaults, rebuilds the map, restores custom soil and bus stops, and refreshes usage counts. A missing or false `fastTerrain` uses the legacy terrain path. The default metadata has `name`, `creationDate`, `elapsedTime`, `size`, `isSandbox`, `fastTerrain`, and `sandboxWithMoney`. `isInfiniteMoney` requires both sandbox mode and the finite-money option being off. The in-game action for finite sandbox money also changes the balance to twenty times the normal starting cash.

| Snapshot statistics | Role in the game |
| --- | --- |
| `money`, `population`, `income` | Balance, population and last calculated income. The snapshot writer refreshes income from the simulation. |
| `level`, `exp`, `pendingRewardLevels` | Current level, experience and pending level rewards. These are separate values. |
| `finishedTutorial`, `internalVersion` | Tutorial completion and internal save revision. |
| `numTiles`, `unlocks`, `regions`, `regionType`, `isNeighbor` | Land expansion and the city's regional structure. Preserve these together with map dimensions and the generator seed. |
| `activeQuests`, `newQuests`, `completedQuests`, `archivedQuests` | Quest lists synchronized by the quest module before saving. |
| `unlockedEventsIds`, `metricModifiers` | Event unlocks and effects applied to city metrics. |
| `policies` | Approved policy IDs. The normal UI is available from level 28 in regular cities. |
| `globalSettings` | Optional data scoped to this city and snapshot. App-wide settings live in `_user.save`. |

The game rebuilds `activePolicies` from saved IDs. During ordinary approval it also checks the prerequisite and point cost, runs any approval callback, and records approval for quests. Some callbacks refresh systems such as road supply or traffic. An editor that writes the array bypasses that approval path. Keep the policy IDs unique and use the shipped catalog. The game indexes the catalog directly when adding up points spent.

### Optional city settings found in the shipped code

| Key under `stats.globalSettings` | What it controls |
| --- | --- |
| `difficulty` | `0` Normal, `1` Hard, `2` Expert. Absence defaults to Normal. |
| `hm_complete`, `em_complete` | Completion markers used by Hard and Expert unlock checks. |
| `taxes.sales`, `taxes.income`, `taxes.property` | Five tax multipliers `0.2`, `0.6`, `1`, `1.4`, `1.8`. A value of `1` is the standard rate. |
| `loans` | Outstanding loan IDs `0`, `1`, and `2`, representing 50,000, 200,000, and 500,000. The in-game take and repay actions also change cash. |
| `disasters`, `next_dstr` | Whether random disasters are enabled and the countdown to the next one. |
| `COOLDOWNS`, `D_REPORTS` | Event cooldowns and disaster report history copied in at snapshot time. |
| `stoppedcrimes`, `hide-bust` | Crime-stop count and elapsed city time at the last hideout bust. |
| `chest_saved`, `chestunlockbuildings` | Opened chests and building unlocks earned through chests. |
| `thframe`, `thunlocks` | Selected Town Hall appearance and its unlocks. |
| `cus-ter`, `SOIL`, `bus_stops` | Custom terrain, soil, and bus stop layers reconstructed when loading. |
| `hwunpowerednotif`, `msg-about-highway`, `msg-about-zone-upgrade` | Tutorial-style notices the player has already seen. |

The map serializer writes `R` for road, `H` for highway, `B` for bridge and `o` for cleared seeded rock. A zone uses `r`, `c` or `i` plus a building sheet identifier, with an initial `*` for an upgraded zone. Other structures use numeric IDs, and the `S` suffix adds elevated rail to the same cell. A multi-cell building stores its anchor and recreates its occupied neighboring cells on load. Raw map cell editing requires these placement rules as well as the separate map layers.

## Pocket City 2

### Creation, migration and map load

- The base `GetNewDefaultState` starts at level 1 with 0 XP, 50,000 cash, day 1, a 64-cell-wide map and Easy difficulty. The three tax rates are fractions set to `0.15`, or 15%. New-game routes can specialize the base state for tutorials, sandbox or Survival.
- `EnsureDefaultSetIfMissing` fills selected absent collections, including research-adjacent collections such as building unlocks, available quests and a range of map and event dictionaries. It always makes vehicle ID `0` available. It also regenerates missing regional biomes from `SEED`.
- Writing a city first copies the live main, terrain and elevated grids to the state. The same routine asks the game to save pending downloaded regional cities. Loading passes the main grid through the game's out-of-bounds cleanup and refreshes the roads on the elevated grid when present.
- `LoadGameStateIntoController` discards the available quest cache if its saved language differs from the selected language. That check treats a missing language as English. Active quests and completion records use separate fields.

### Pets and NPC relationships

`pets` records which pets belong to the city and whether each stays home (`0`) or follows the mayor (`1`). `petStates` stores each pet's care history. The pet screen indexes `petStates` using every owned ID in `pets`. Keep the same IDs in both dictionaries. The defaulting routine can create a missing `pets` dictionary, while the game initializes `petStates` during fresh city creation. The defaulting routine contains no per-pet repair for missing care records.

| Pet ID | Animal |
| --- | --- |
| 1 | Gray cat |
| 2 | Orange cat |
| 3 | Brown cat |
| 4 | Gray dog |
| 5 | Shiba |
| 7 | Large orange cat |
| 9 | Duck |

ID `0` is the game's no-pet sentinel. `SetBasePetState` starts an adopted pet at well-being `1` and initializes both care-day fields to `0`. Selecting a follower first marks all pets as staying home, then marks the chosen one as following. Feeding and petting record the current in-game `day` and add one point to `wellBeing` up to 10. The game compares `lastdayFed` and `lastdayPet` to `day` to show whether care was already given today. Normal adoption also increases veterinarian affinity and fires an adoption event. Reaching 10 through normal care fires its own event.

NPC affinity is separate from pet well-being. The `relationships` scores use the shared NPC profiles described in [save formats](SAVE_FORMATS.md). `lastVisitedDay`, `lastGaveGift`, `pendingGifts` and `completedQuests` are sibling collections. An affinity adjustment can leave those histories intact. It also bypasses the in-game code that queues gifts and notifications.

### Other fields with cross-system effects

| Field | Observed behavior |
| --- | --- |
| `loanTaken` | Outstanding principal. The bank borrows or repays in steps of 100,000, adjusts cash separately and charges 2% interest per day. Arbitrary principal values can leave an amount that the standard one-step repayment does not match. |
| `purchasedResourcesDay` | Units bought per resource during the current day, used to calculate the remaining shop quota. The resource manager clears the counts in its daily update. |
| `eventCooldowns` | Remaining game time in seconds by event ID. The standard trigger writes 60 seconds, or 120 in Survival. The runtime removes the key on expiry. |
| `bestEventRating` | Stored best result by event ID. Some building gates require exactly `1` for their specified competition. A five-star quest check uses `>= 5`. Preserve the distinction when changing an event record. |
| `forceUnlockedBuildings`, `baseUnlockedBuildingsAll` | An existing key in either dictionary bypasses the usual level unlock check. The stored Boolean is ignored. The game uses the base dictionary for grants such as regional buildings. Use the force-unlock dictionary for a targeted edit that does not set sandbox options. |
| `flags` and `globalFlags` | Internal records, including dismissed tutorial tips. The tooltip code checks key presence in either dictionary. |
| `mainGrid`, `terrainGrid`, `upperGrid`, and coordinate-keyed dictionaries | Map layers and attached state. Bridge type and rotation, upgrades, bus stops and streetlights are examples of state tied to a map coordinate. |

## Analysis evidence and limits

REA inspected the PC1 Electron startup and compression workers, plus the shipped renderer bundle. Evidence IDs are `ev_050a56a63128d58e61044b54530e87af52c6dd889afe3dbb0cf6dcfcf98f9660` and `ev_05e92a50254385f5b860c0f6ae8d9f6a81c563b24652f4b6e82f8d0f369a804a`. Ghidra independently identified the PC1 Steam executable and its fingerprint. The loader, grid and policy findings come from JavaScript shipped next to that executable.

For PC2, REA read the managed metadata and method bodies under `ev_ec2c517327a9c7b9f7f1ddab3b4afcdc2c8b96f2aa229b9d658e8eba093258cf`. Ghidra read the four CIL bodies below from the same PE and matched their SHA-256 hashes with REA. Targeted ILSpy output provided method-level context, including pet ownership and care.

| `GameStateHelper` method | Ghidra address | CIL body SHA-256 |
| --- | --- | --- |
| `GetNewDefaultState` | `00519d84` | `de4431cbab753a6943b862b180e638af598a2104d906b4b649fc0f54e68f8793` |
| `EnsureDefaultSetIfMissing` | `0051a4b0` | `2cea0ded770e42ffe130b802d172e380e60d635622dafa1a8c3201c8dcbdace7` |
| `LoadGameStateIntoController` | `0051b01c` | `d4b11372d7a3c13974f13e123fc61f1b8c020f34923b1cebbe703d538d46ef16` |
| `PrepGameStateForSave` | `0051b130` | `929e23b2ac7f2b710b660ef8227ded52dc33f7cab17991f6ec5546eb25e87eb7` |

This is a static inventory for the identified builds. It leaves nested quest and map payloads, legacy migration edge cases, and live reload of edited policy and pet data open for verification. The tables document the game schema and do not imply that each field is exposed in the editor.
