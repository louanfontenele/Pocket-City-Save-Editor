# Steam save field index

See [the save schema notes](STEAM_SAVE_SCHEMA.md) for the build fingerprints and the meaning and dependencies of individual fields.

## Pocket City policies

Each row describes one ID in the shipped policy catalog. The cost and prerequisite are the normal in-game approval rules. The save stores the approved IDs in an array.

| ID | Game identifier | Cost in points | Prerequisite ID |
| --- | --- | --- | --- |
| 0 | `BANK_MAX_1` | 2 | None |
| 1 | `BANK_MAX_2` | 5 | 0 |
| 2 | `BANK_MAX_3` | 5 | 1 |
| 3 | `BANK_SAFETY_NET_1` | 2 | None |
| 4 | `BANK_SAFETY_NET_2` | 5 | 3 |
| 5 | `DISASTER_INSURANCE_1` | 5 | None |
| 6 | `DISASTER_INSURANCE_2` | 10 | 5 |
| 7 | `HIGH_OCCUPANCY_1` | 10 | None |
| 8 | `HIGH_OCCUPANCY_2` | 10 | 7 |
| 9 | `HIGH_OCCUPANCY_3` | 10 | 8 |
| 10 | `CONTRACTOR_RES_DISCOUNT` | 5 | None |
| 11 | `CONTRACTOR_RES_DISCOUNT_2` | 5 | 10 |
| 12 | `IMPROVED_INTERIORS` | 2 | None |
| 13 | `ROAD_MAINTENANCE` | 2 | None |
| 14 | `HIGHWAY_SUPPLY` | 10 | 13 |
| 15 | `SKYRAIL_SPEED` | 2 | None |
| 16 | `SKYRAIL_REDUCED_TICKETS` | 5 | 15 |
| 17 | `CYCLING_INCENTIVE` | 4 | None |
| 18 | `SHIPPING_INCREASE_1` | 5 | None |
| 19 | `SHIPPING_INCREASE_2` | 10 | 18 |
| 20 | `REDUCE_INDUSTRY_POLLUTION` | 2 | None |
| 21 | `REDUCE_STEEL_POLLUTION` | 2 | 20 |
| 22 | `REMOTE_WORK` | 5 | None |
| 23 | `EVENT_XP_INCREASE` | 2 | None |
| 24 | `EVENT_MONEY_OPPORTUNITIES` | 5 | 23 |
| 25 | `LANDMARK_TOURS` | 5 | None |
| 26 | `FREE_LANDSCAPING` | 5 | 27 |
| 27 | `SCENIC_CITY` | 10 | None |
| 28 | `GREAT_OUTDOORS` | 2 | None |
| 29 | `FIRE_SAFETY` | 2 | None |
| 35 | `NO_EVENT_COOLDOWNS` | 5 | None |
| 36 | `EDUCATED_WORKFORCE` | 10 | None |
| 37 | `HAPPY_REWARDS` | 5 | None |

## Pocket City 2 field types

The tables inventory the managed fields declared in this Steam build. Individual files can omit fields, and some fields act as caches or have game-specific defaults. The declared types describe game data. Consult the format notes for the Easy Save representation.

## City state (`GameState`)

| Field | Declared type |
| --- | --- |
| `FILE_ID` | `string` |
| `SEED` | `int` |
| `isTutorialPreset` | `bool` |
| `population` | `int` |
| `level` | `int` |
| `XP` | `int` |
| `money` | `long` |
| `day` | `int` |
| `dayProgress` | `float` |
| `name` | `string` |
| `mapSize` | `int` |
| `incomePerSec` | `int` |
| `lastVehicleIndex` | `int` |
| `parentCity` | `string` |
| `regionIndexI` | `int` |
| `regionIndexJ` | `int` |
| `buildVersion` | `int` |
| `mayorPerk` | `MayorPerk.ID` |
| `showsTooltips` | `bool` |
| `difficulty` | `SettingsManager.DIFFICULITY` |
| `unlockAll` | `bool` |
| `infiniteMoney` | `bool` |
| `disableQuests` | `bool` |
| `maxLevel` | `bool` |
| `resources` | `Dictionary<ResourceManager.RESOURCE_TYPE, int>` |
| `purchasedResourcesDay` | `Dictionary<ResourceManager.RESOURCE_TYPE, int>` |
| `purchasedFurnitureDay` | `Dictionary<int, int>` |
| `purchasedItemDay` | `Dictionary<ItemManager.ITEM_TYPE, int>` |
| `researched` | `Dictionary<Research.RESEARCH, bool>` |
| `disabledResearch` | `Dictionary<Research.RESEARCH, bool>` |
| `lastVisitedDay` | `Dictionary<Relationships.ID, int>` |
| `historicalIncome` | `List<int>` |
| `historicalPopulation` | `List<int>` |
| `historicalHappiness` | `List<float>` |
| `shippingUsed` | `int` |
| `shippingUsedPreviousDay` | `int` |
| `shippingUsedPreviousDayTotal` | `int` |
| `milestones` | `Dictionary<MilestoneManager.MILESTONE, int>` |
| `forceUnlockedBuildings` | `Dictionary<Tile.VALUE, bool>` |
| `baseUnlockedBuildingsAll` | `Dictionary<Tile.VALUE, bool>` |
| `mineralDepositsSaved` | `Dictionary<(int, int), int>` |
| `harvestedAmt` | `Dictionary<(int, int), float>` |
| `regeneratingResource` | `Dictionary<(int, int), float>` |
| `landfillUsage` | `Dictionary<(int, int), int>` |
| `wasteOverflow` | `int` |
| `wasteSewageOverflow` | `int` |
| `buildingCooldowns` | `Dictionary<(int, int), float>` |
| `pendingResourcesCollected` | `Dictionary<(int, int), bool>` |
| `roughTerrain` | `Dictionary<(int, int), bool>` |
| `flags` | `Dictionary<GameStateHelper.FLAGS, bool>` |
| `fixedEncountersDone` | `Dictionary<RandomEcounters.FIXED_ENCOUNTER_TYPE, bool>` |
| `snapshotNoSpawnForNextDays` | `Dictionary<SnapshotXPManager.SNAPSHOT_TYPE, int>` |
| `incomeTax` | `float` |
| `propertyTax` | `float` |
| `salesTax` | `float` |
| `mainGrid` | `Tile.VALUE[,]` |
| `mainGridPCLoad` | `Tile.VALUE[,]` |
| `upperGrid` | `Tile.VALUE[,]` |
| `terrainGrid` | `Tile.VALUE[,]` |
| `preferredGroundTerrainGrid` | `Tile.VALUE[,]` |
| `hasStreetlight` | `Dictionary<(int, int), bool>` |
| `hasBusStop` | `Dictionary<(int, int), bool>` |
| `hasBridgeWithRotation` | `Dictionary<(int, int), int>` |
| `hasBridgeType` | `Dictionary<(int, int), Tile.VALUE>` |
| `unlockedBlocks` | `Dictionary<(int, int), bool>` |
| `preferredGroundTerrain` | `Dictionary<(int, int), Tile.VALUE>` |
| `subwayTunnels` | `Dictionary<(int, int), bool>` |
| `firstBuild` | `Dictionary<Tile.VALUE, (int, int)>` |
| `availableQuests` | `List<QuestManager.QuestSettings>` |
| `activeQuests` | `List<QuestManager.QuestSettings>` |
| `megaProjects` | `Dictionary<(int, int), MegaProjects.MegaProjectSetting>` |
| `upgrades` | `Dictionary<(int, int), int>` |
| `temporaryBuildings` | `Dictionary<(int, int), float>` |
| `customRotation` | `Dictionary<(int, int), int>` |
| `damagedRoots` | `Dictionary<(int, int), bool>` |
| `relationships` | `Dictionary<Relationships.ID, int>` |
| `pendingGifts` | `Dictionary<Relationships.ID, int>` |
| `completedQuests` | `Dictionary<Relationships.ID, Dictionary<int, bool>>` |
| `unlockedCars` | `Dictionary<int, bool>` |
| `daysRemaining` | `Dictionary<(int, int), float>` |
| `temporaryEffects` | `Dictionary<TemporaryEffectManager.TEMPORARY_EFFECT_TYPE, float>` |
| `items` | `Dictionary<ItemManager.ITEM_TYPE, int>` |
| `foodEatenToday` | `int` |
| `regionBioms` | `Dictionary<(int, int), Regions.REGION_BIOM>` |
| `regionFileIDs` | `Dictionary<(int, int), string>` |
| `biom` | `Regions.REGION_BIOM` |
| `avatarBody` | `AvatarEquip.BASE_BODY_TYPE` |
| `avatarSkin` | `AvatarEquip.BASE_SKIN_TYPE` |
| `avatarEyes` | `AvatarEquip.EYE_TYPE` |
| `avatarHair` | `AvatarEquip.HAIR_TYPE` |
| `avatarHairColor` | `AvatarEquip.HAIR_COLOR` |
| `avatarFacialHair` | `AvatarEquip.FACIAL_HAIR_TYPE` |
| `avatarFacialHairColor` | `AvatarEquip.HAIR_COLOR` |
| `avatarHat` | `ItemManager.ITEM_TYPE` |
| `accessoryActive` | `ItemManager.ITEM_TYPE` |
| `clothingTopActive` | `ItemManager.ITEM_TYPE` |
| `clothingBottomActive` | `ItemManager.ITEM_TYPE` |
| `clothingShoesActive` | `ItemManager.ITEM_TYPE` |
| `equipActive` | `Dictionary<ItemManager.ITEM_TYPE, int>` |
| `furnitureInventory` | `Dictionary<int, int>` |
| `lastMayorIndexDropped` | `(int, int)` |
| `researchPoints` | `int` |
| `loanTaken` | `int` |
| `residences` | `Dictionary<(int, int), VisitHome.Residence>` |
| `pets` | `Dictionary<PetManager.PET_TYPE, PetManager.PET_STATUS>` |
| `petStates` | `Dictionary<PetManager.PET_TYPE, PetManager.SavePetState>` |
| `lastGaveGift` | `Dictionary<Relationships.ID, int>` |
| `bestEventRating` | `Dictionary<CityEvents.EVENT_ID, int>` |
| `eventCooldowns` | `Dictionary<CityEvents.EVENT_ID, float>` |
| `resourceAutoSellKeepAmt` | `Dictionary<ResourceManager.RESOURCE_TYPE, int>` |
| `resourceAutoConvertKeepAmt` | `Dictionary<ResourceManager.RESOURCE_TYPE, int>` |
| `disasterReports` | `List<CityEvents.DisasterReport>` |
| `startingBoost` | `CityManagementController.STARTING_BOOST` |
| `nextRandomDisasterTimer` | `float` |
| `randomDisasters` | `bool` |
| `disabledQuestTypes` | `List<QuestPossibility.QUEST_TYPE>` |
| `isSurvivalMode` | `bool` |
| `timeZeroIncome` | `float` |
| `survivalGoalIndex` | `int` |
| `survivalModeCompleted` | `bool` |
| `startedSurvivalGoalDay` | `int` |
| `secondsSinceLastDisaster` | `float` |
| `survivalLifelines` | `int` |
| `survivalGoalDayPassed` | `int` |
| `maxMoneyReached` | `long` |
| `minesEverBuilt` | `Dictionary<(int, int), bool>` |
| `upgradesSpentCity` | `Dictionary<SurvivalMode.SURVIVAL_UPGRADE, int>` |

## Game preferences (`SettingsState`)

| Field | Declared type |
| --- | --- |
| `enableSfx` | `bool` |
| `enableMusic` | `bool` |
| `settingsDisableEdgeTerrain` | `bool` |
| `settingsEnableZoneSize` | `bool` |
| `settingsNightMode` | `bool` |
| `settingsZoomInOutBtns` | `bool` |
| `settingsSmallInterface` | `bool` |
| `settingsInverseControls` | `bool` |
| `settingsInverseControlsPad` | `bool` |
| `disableNPCChatter` | `bool` |
| `maxLevelReached` | `Dictionary<SettingsManager.DIFFICULITY, int>` |
| `maxPopulationReached` | `Dictionary<SettingsManager.DIFFICULITY, int>` |
| `globalFlags` | `Dictionary<GameStateHelper.FLAGS, bool>` |
| `desiredLang` | `string` |
| `controllerLayout` | `int` |

## Display and camera settings (`OSState`)

| Field | Declared type |
| --- | --- |
| `quality` | `int` |
| `distanceClip` | `int` |
| `settingsQualityLevel` | `int` |
| `enablePostProcessing` | `bool` |
| `settings30FPS` | `bool` |
| `settingsDisableShadows` | `bool` |
| `shadowDistance` | `float` |
| `edgeScrollSpeed` | `float` |
| `uiScrollSpeed` | `float` |
| `thirdPersonCamSensitivity` | `float` |
| `controllerMapMovementSpeed` | `float` |
| `settingsDisableGrass` | `bool` |
| `settingsDisableDistanceBlur` | `bool` |
| `settingsDisableNightBloom` | `bool` |
| `settingsDisableExtraNightLights` | `bool` |

## Global Survival records (`SurvivalSettingsStateGlobal`)

| Field | Declared type |
| --- | --- |
| `bestDayReachedEasy` | `int` |
| `bestDayReachedHard` | `int` |
| `bestDayReachedExpert` | `int` |
| `highestPopulationReachedEasy` | `int` |
| `highestPopulationReachedHard` | `int` |
| `highestPopulationReachedExpert` | `int` |
| `highestStarsReachedEasy` | `int` |
| `highestStarsReachedHard` | `int` |
| `highestStarsReachedExpert` | `int` |
| `lastDifficultySelection` | `int` |
| `totalUpgradePoints` | `int` |
| `disableUnspentUpgradeToCash` | `bool` |
| `upgradesSpent` | `Dictionary<SURVIVAL_UPGRADE, int>` |

## Pet state (`PetManager.SavePetState`)

| Field | Declared type |
| --- | --- |
| `wellBeing` | `int` |
| `lastdayFed` | `int` |
| `lastdayPet` | `int` |

## Pocket City 2 dictionary identifiers

Numeric keys in a save are interpreted through the corresponding game type. The identifiers below name the catalog entries. Their presence in the catalog alone does not establish whether the game uses them in every mode.

### Recorded tips and internal flags (`GameStateHelper.FLAGS`)

| ID | Game identifier |
| --- | --- |
| 1 | `TUTORIAL_CAR_BOARDS` |
| 2 | `CAR_UNLOCK_A` |
| 3 | `CAR_UNLOCK_B` |
| 4 | `CAR_UNLOCK_C` |
| 5 | `CAR_UNLOCK_D` |
| 6 | `CAR_UNLOCK_E` |
| 7 | `TOOLTIP_GARBAGE` |
| 8 | `TOOLTIP_FREE_ROAM` |
| 9 | `TOOLTIP_GLIDER` |
| 10 | `TOOLTIP_SHIPPING_DOCK_UNLOCKED` |
| 11 | `TOOLTIP_ITEMS_SCREEN` |
| 12 | `TOOLTIP_RELATIONSHIPS_SCREEN` |
| 13 | `TOOLTIP_REGION_SCREEN` |
| 14 | `TOOLTIP_WELCOME_SECOND_CITY_REGION_SCREEN` |
| 15 | `TOOLTIP_STATS_SCREEN` |
| 16 | `TOOLTIP_VISIT_SHIPPING_DOCK` |
| 17 | `TOOLTIP_TOURISM_MAX_REACHED` |
| 18 | `TOOLTIP_ROUGH_TERRAIN` |
| 19 | `TOOLTIP_BUSINESS_SIM` |
| 20 | `TOOLTIP_RAIL_USAGE` |
| 21 | `TOOLTIP_LIGHTRAIL_USAGE` |
| 22 | `TOOLTIP_COMMUTE_USAGE` |
| 23 | `TOOLTIP_SUBWAY_USAGE` |
| 24 | `TOOLTIP_DISASTER_COMPLETED` |
| 25 | `TOOLTIP_NO_POPULATION_LOW_ATTRACTION` |
| 26 | `TOOLTIP_SEWAGE_OVERFLOW` |
| 27 | `TOOLTIP_SURVIVAL_MODE` |
| 28 | `PREFER_NEW_CAR_CONTROLS` |

### NPC affinity profiles (`Relationships.ID`)

| ID | Game identifier |
| --- | --- |
| 0 | `TOWN_HALL` |
| 1 | `POLICE_DEPT` |
| 2 | `FIRE_DEPT` |
| 3 | `FINANCIAL` |
| 4 | `GREEN_ENERGY` |
| 5 | `SMALL_BUSINESS` |
| 6 | `EDUCATION` |
| 7 | `HEALTHCARE` |
| 9 | `CONTRACTORS` |
| 10 | `PARKS_AND_REC` |
| 11 | `TRANSPORTATION` |
| 12 | `VET` |
| 13 | `REALTOR` |
| 50 | `CITIZENS` |
| 51 | `MINNIE_CHEF` |
| 52 | `PASTA_CHEF` |
| 53 | `SPORTSWOMAN` |
| 54 | `ROBOTICS_EXPERT` |
| 75 | `WORKERS` |
| 76 | `STEEL_WORKER` |

### City events (`CityEvents.EVENT_ID`)

| ID | Game identifier |
| --- | --- |
| 1 | `FARMERS_MARKET` |
| 2 | `BLOCK_PARTY` |
| 3 | `FIREWORKS` |
| 4 | `HOUSE_PARTY` |
| 5 | `PARADE` |
| 6 | `MUSIC_CONCERT` |
| 7 | `MARATHON` |
| 8 | `SNOW_ICE_EVENT` |
| 20 | `TORNADO` |
| 21 | `VOLCANO` |
| 22 | `FLOOD` |
| 23 | `METEOR_STRIKE` |
| 24 | `EARTHQUAKE` |
| 25 | `STORM` |
| 32 | `KAIJU` |
| 40 | `CITY_COMPETE_WONDERPARK` |
| 41 | `CITY_COMPETE_MONEYHILL` |
| 42 | `CITY_COMPETE_COMMUTE` |
| 44 | `CITY_COMPETE_AUTUMN_OAKS` |
| 46 | `CITY_COMPETE_MERRY_SMOKES` |
| 47 | `CITY_COMPETE_COVE_ALBION` |
| 48 | `CITY_COMPETE_SEVENWATERS` |
| 49 | `CITY_COMPETE_DREAM_CITY` |
| 50 | `CITY_COMPETE_GALORANDIN` |
| 51 | `CITY_COMPETE_EVANGELINE` |
| 100 | `FREE_WALK` |
| 101 | `FREE_DRIVE` |
| 102 | `DRIVER_MINIGAME` |
| 103 | `RACE_MINIGAME` |
| 104 | `LASER_TAG` |
| 105 | `FIRE_HELI_MINIGAME` |
| 106 | `STUNT_PLANE_MINIGAME` |
| 107 | `STARTUP_MINIGAME` |
| 108 | `BASKETBALL_MINIGAME` |
| 109 | `MOTORCROSS_MINIGAME` |
| 110 | `DEMOLISH_MINIGAME` |
| 111 | `KID_ESCAPE_MINIGAME` |
| 112 | `UNDERCOVER_POLICE_MINIGAME` |

### Milestones (`MilestoneManager.MILESTONE`)

| ID | Game identifier |
| --- | --- |
| 0 | `QUESTS_COMPLETE` |
| 1 | `ROADS_BUILT` |
| 2 | `ZONES_BUILT` |
| 3 | `POPULATION_REACHED` |
| 4 | `INCOME_REACHED` |
| 5 | `CRIMES_REPORTED` |
| 6 | `BUILDINGS_UPGRADED` |
| 7 | `HOMES_BOUGHT` |
| 8 | `ENCOUNTERS_INTERACTIONS` |
| 9 | `FURNITURE_BOUGHT` |
| 10 | `RESOURCES_SOLD` |
| 11 | `SPECIAL_BUILDINGS_BUILT` |
| 12 | `EVENTS_STARTED` |
| 13 | `RELATIONSHIPS_INCREASED` |
| 14 | `INJURIES_HEALED` |
| 15 | `MINIGAMES_PLAYED` |

### Difficulty records (`SettingsManager.DIFFICULITY`)

| ID | Game identifier |
| --- | --- |
| 0 | `NOT_SPECIFIED` |
| 1 | `EASY` |
| 2 | `HARD` |
| 3 | `EXPERT` |

### Survival upgrades (`SurvivalMode.SURVIVAL_UPGRADE`)

| ID | Game identifier |
| --- | --- |
| 0 | `NONE` |
| 1 | `STARTING_MONEY` |
| 2 | `RESEARCH` |
| 3 | `ADDITIONAL_LIFELINES` |
| 4 | `LAND_COSTS` |
| 5 | `BANK_INTEREST_REDUCTION` |
| 6 | `TOURISM_EARNINGS` |
| 7 | `EXPORT_PRICES` |
| 8 | `HOUSING_PER_RES_ZONE` |
| 9 | `INFLATION_REDUCTION` |
