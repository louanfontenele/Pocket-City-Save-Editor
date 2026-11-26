# Steam progression catalogs

These catalogs were extracted from the Steam artifacts fingerprinted in the [save schema notes](STEAM_SAVE_SCHEMA.md). See [progression rules](STEAM_PROGRESSION.md) for how the game interprets these IDs.

## Pocket City quest IDs

The 66 quest registrations in the English bundle supply the IDs used by the four snapshot quest arrays. Titles are the English bundle labels. Eligibility depends on each quest’s own level, prerequisites and other conditions.

| ID | Quest |
| --- | --- |
| 1 | Growth Spurt |
| 2 | Way to Grow! |
| 3 | Getting Crowded |
| 4 | A Walk in the Park |
| 5 | Bigger is Better |
| 6 | Give Me Your Money |
| 7 | Tree-umph |
| 10 | Get the Lay of the Land |
| 11 | There's No Place Like Home |
| 12 | Make Bank |
| 13 | The Land Before Crime |
| 14 | Only You Can Prevent... |
| 15 | Grow it, Ship it |
| 16 | Getting Resourceful |
| 18 | Full House |
| 20 | Play it Safe |
| 23 | Kickstart the Economy |
| 24 | Make it Rain |
| 29 | Dolla Dolla Bill |
| 30 | Something to do |
| 32 | Grow Local |
| 33 | Party City |
| 34 | Baby, You're a Firework |
| 35 | Where's the Fire? |
| 36 | A Room with a View |
| 37 | A Mountain to Climb |
| 38 | Riding the Rail |
| 39 | Crime Doesn't Pay |
| 40 | Yard Sale |
| 41 | No More Worlds to Conquer |
| 42 | Perfect Storm |
| 43 | Treasure Hunter |
| 44 | Urban Sprawl |
| 45 | Hi! My Name Is... |
| 46 | Flying to the Rescue |
| 47 | You Can Run But You Can't Hide |
| 48 | Deeper Pockets |
| 49 | Get School'd |
| 50 | Mean Streets |
| 51 | Smooth Sailing |
| 52 | Take the Bus |
| 53 | Hidden Power |
| 54 | Hideout and Seek |
| 55 | Grand Theft Auto |
| 56 | Bad Eggs |
| 57 | Crime Crackdown |
| 58 | We Require More Minerals |
| 59 | Pedal to the Metal |
| 60 | You've got to be Kitten Me! |
| 61 | Gone Fishing |
| 62 | The Final Frontier |
| 63 | Busted Busker |
| 64 | Yoga to be Kidding Me |
| 65 | On the Map |
| 66 | Rush Hour |
| 67 | Game of Drones |
| 68 | In the Zone |
| 69 | Hot Dawg |
| 70 | Are you not entertained? |
| 71 | She's Gonna Blow! |
| 72 | Storm's a-brewin' |
| 73 | Tax Attack |
| 74 | Everyone Loves the Zoo |
| 75 | Green Incentive |
| 76 | Policy Maker |
| 77 | Neighbor City |

## Pocket City 2 quest possibilities

These are the 179 registered template IDs used as quest possibility keys. The constructor and referenced selectors identify the template. An explicit first NPC argument is shown where present. A dash indicates that the giver needs to be resolved through the template or runtime data. The references column covers constructor-line selectors and omits dynamic values and subsequent configuration.

| ID | Template | Explicit giver | Referenced selectors |
| --- | --- | --- | --- |
| 1 | `BuildFirstPossibility` | `GREEN_ENERGY` | `Tile.VALUE.LANDFILL_2_2` |
| 2 | `MetricReachSpecificValue` | `TOWN_HALL` | `CityMetrics.current.visiblePopulation` |
| 5 | `CountEventsPossibilityStaticDesc` | `ROBOTICS_EXPERT` | `EventManager.EVENT_TYPE.RESEARCH_COMPLETE` |
| 6 | `CountEventsPossibilityStaticDesc` | `CITIZENS` | `EventManager.EVENT_TYPE.CITY_EVENT_STARTED` |
| 7 | `BuildFirstPossibility` | `FINANCIAL` | `Tile.VALUE.BANK_1_1` |
| 8 | `BuildPossibility` | `PARKS_AND_REC` | `Tile.VALUE.TREES_PLANTED_1_1` |
| 10 | `BuildFirstPossibility` | `TRANSPORTATION` | `Tile.VALUE.LIGHT_RAIL_STOP_1_1` |
| 11 | `BuildFirstPossibility` | `TRANSPORTATION` | `Tile.VALUE.RAIL_STATION_1_1` |
| 12 | `CountEventsPossibilityStaticDesc` | `REALTOR` | `EventManager.EVENT_TYPE.UNLOCKED_LAND` |
| 13 | `BuildFirstPossibility` | `TRANSPORTATION` | `Tile.VALUE.SHIPPING_DOCK_2_2` |
| 15 | `CountEventsPossibilityStaticDesc` | `TOWN_HALL` | `EventManager.EVENT_TYPE.WON_CITY_COMPETITION` |
| 16 | `CityEventRatingPossibility` | `FIRE_DEPT` | `CityEvents.EVENT_ID.FARMERS_MARKET` |
| 17 | `CityEventRatingPossibility` | `FINANCIAL` | `CityEvents.EVENT_ID.HOUSE_PARTY` |
| 18 | `CityEventRatingPossibility` | `HEALTHCARE` | `CityEvents.EVENT_ID.BLOCK_PARTY` |
| 19 | `CountEventsPossibilityStaticDesc` | `POLICE_DEPT` | `EventManager.EVENT_TYPE.GIVE_FOOD_TO_POLICE` |
| 20 | `CityEventRatingPossibility` | `CITIZENS` | `CityEvents.EVENT_ID.DRIVER_MINIGAME` |
| 21 | `CityEventRatingPossibility` | `TRANSPORTATION` | `CityEvents.EVENT_ID.RACE_MINIGAME` |
| 22 | `CityEventRatingPossibility` | `ROBOTICS_EXPERT` | `CityEvents.EVENT_ID.LASER_TAG` |
| 23 | `MetricReachSpecificValue` | `FINANCIAL` | `CityMetrics.current.income` |
| 24 | `MetricReachSpecificValue` | `FINANCIAL` | `CityMetrics.current.income` |
| 25 | `MetricReachSpecificValue` | `FINANCIAL` | `CityMetrics.current.income` |
| 26 | `MetricReachSpecificValue` | `FINANCIAL` | `CityMetrics.current.income` |
| 27 | `MetricReachSpecificValue` | `FINANCIAL` | `CityMetrics.current.income` |
| 28 | `CityEventRatingPossibility` | `VET` | `CityEvents.EVENT_ID.PARADE` |
| 29 | `CityEventRatingPossibility` | `FIRE_DEPT` | `CityEvents.EVENT_ID.FIRE_HELI_MINIGAME` |
| 30 | `CityEventRatingPossibility` | `TRANSPORTATION` | `CityEvents.EVENT_ID.STUNT_PLANE_MINIGAME` |
| 31 | `BuildFirstPossibility` | `TRANSPORTATION` | `Tile.VALUE.SUBWAY_STATION_1_1` |
| 32 | `CountEventsPossibilityStaticDesc` | `ROBOTICS_EXPERT` | `EventManager.EVENT_TYPE.START_TORNADO` |
| 33 | `CountEventsPossibilityStaticDesc` | `ROBOTICS_EXPERT` | `EventManager.EVENT_TYPE.START_VOLCANO` |
| 34 | `CountEventsPossibilityStaticDesc` | `ROBOTICS_EXPERT` | `EventManager.EVENT_TYPE.START_FLOOD` |
| 35 | `CountEventsPossibilityStaticDesc` | `ROBOTICS_EXPERT` | `EventManager.EVENT_TYPE.START_KAIJU` |
| 36 | `CityEventRatingPossibility` | `FINANCIAL` | `CityEvents.EVENT_ID.STARTUP_MINIGAME` |
| 37 | `CityEventRatingPossibility` | `SPORTSWOMAN` | `CityEvents.EVENT_ID.BASKETBALL_MINIGAME` |
| 38 | `CityEventRatingPossibility` | `SPORTSWOMAN` | `CityEvents.EVENT_ID.MOTORCROSS_MINIGAME` |
| 39 | `CityEventRatingPossibility` | `CONTRACTORS` | `CityEvents.EVENT_ID.DEMOLISH_MINIGAME` |
| 40 | `CountEventsPossibilityStaticDesc` | `ROBOTICS_EXPERT` | `EventManager.EVENT_TYPE.START_METEOR_STRIKE` |
| 41 | `CountEventsPossibilityStaticDesc` | `ROBOTICS_EXPERT` | `EventManager.EVENT_TYPE.START_STORM` |
| 43 | `CityEventRatingPossibility` | `EDUCATION` | `CityEvents.EVENT_ID.KID_ESCAPE_MINIGAME` |
| 50 | `CountEventsPossibilityStaticDesc` | `VET` | `EventManager.EVENT_TYPE.PED_WAVE_AT_MAYOR_ON_HORSE` |
| 100 | `BuildFirstPossibility` | `PARKS_AND_REC` | `Tile.VALUE.PARK_SM_1_1` |
| 102 | `BuildFirstPossibility` | `PARKS_AND_REC` | `Tile.VALUE.PARK_LG_2_2` |
| 103 | `BuildFirstPossibility` | `HEALTHCARE` | `Tile.VALUE.HEALTH_CENTER_1_1` |
| 104 | `BuildFirstPossibility` | `HEALTHCARE` | `Tile.VALUE.HOSPITAL_2_2` |
| 105 | `BuildFirstPossibility` | `EDUCATION` | `Tile.VALUE.SCHOOL_2_2` |
| 106 | `BuildFirstPossibility` | `EDUCATION` | `Tile.VALUE.HIGH_SCHOOL_2_2` |
| 107 | `BuildFirstPossibility` | `EDUCATION` | `Tile.VALUE.UNIVERSITY_2_2` |
| 108 | `MetricReachPossibility` | `POLICE_DEPT` | `CityMetrics.current.crimeSafety` |
| 109 | `MetricReachPossibility` | `FIRE_DEPT` | `CityMetrics.current.fireSafety` |
| 110 | `DeliverResourceThenBuildRestaurant` | — | — |
| 111 | `BuildFirstPossibility` | `TRANSPORTATION` | `Tile.VALUE.SHIPPING_DOCK_2_2` |
| 112 | `BuildFirstPossibility` | `TOWN_HALL` | `Tile.VALUE.AIRPORT_4_4` |
| 113 | `BuildFirstPossibility` | `GREEN_ENERGY` | `Tile.VALUE.CARBON_CAPTURE_2_2` |
| 114 | `BuildFirstPossibility` | `GREEN_ENERGY` | `Tile.VALUE.ELECTRIC_CAR_FACTORY_2_2` |
| 115 | `MetricReachPossibility` | `REALTOR` | `CityMetrics.current.attraction` |
| 150 | `UpgradeBuildingPossibility` | — | `Tile.VALUE.HOSPITAL_2_2` |
| 151 | `UpgradeBuildingPossibility` | — | `Tile.VALUE.POLICE_STATION_2_2` |
| 152 | `UpgradeBuildingPossibility` | — | `Tile.VALUE.FIRE_DEPT_2_2` |
| 155 | `BuildOfTypePossibility` | `TOWN_HALL` | — |
| 156 | `BuildOfTypePossibility` | `REALTOR` | — |
| 157 | `BuildOfTypePossibility` | `FINANCIAL` | — |
| 158 | `BuildFirstPossibility` | `POLICE_DEPT` | `Tile.VALUE.POLICE_STATION_2_2` |
| 159 | `BuildFirstPossibility` | `FIRE_DEPT` | `Tile.VALUE.FIRE_DEPT_2_2` |
| 160 | `UpgradeBuildingPossibility` | — | `Tile.VALUE.WATER_TREATMENT_PLANT_2_2` |
| 163 | `BuildFirstPossibility` | `VET` | `Tile.VALUE.HORSE_RANCH_2_2` |
| 164 | `BuildFirstPossibility` | `PARKS_AND_REC` | `Tile.VALUE.COMMUNITY_GARDEN_1_1` |
| 165 | `BuildFirstPossibility` | `SPORTSWOMAN` | `Tile.VALUE.MARINA_2_2` |
| 166 | `BuildFirstPossibility` | `SPORTSWOMAN` | `Tile.VALUE.SKI_RESORT_4_4` |
| 167 | `BuildFirstPossibility` | `CITIZENS` | `Tile.VALUE.MOVIE_THEATRE_2_2` |
| 168 | `BuildFirstPossibility` | `STEEL_WORKER` | `Tile.VALUE.MMA_GYM_2_2` |
| 169 | `BuildFirstPossibility` | `EDUCATION` | `Tile.VALUE.CONCERT_HALL_2_2` |
| 171 | `BuildFirstPossibility` | `EDUCATION` | `Tile.VALUE.LIBRARY_2_2` |
| 172 | `BuildFirstPossibility` | `ROBOTICS_EXPERT` | `Tile.VALUE.ROBOTICS_FACTORY_2_2` |
| 173 | `BuildFirstPossibility` | `ROBOTICS_EXPERT` | `Tile.VALUE.SPACE_CENTER_4_4` |
| 174 | `BuildFirstPossibility` | `PARKS_AND_REC` | `Tile.VALUE.HEALTH_CENTER_1_1` |
| 175 | `BuildFirstPossibility` | `CITIZENS` | `Tile.VALUE.HOTEL_MODERATE_2_1` |
| 176 | `BuildFirstPossibility` | `FINANCIAL` | `Tile.VALUE.HOTEL_LUXURY_2_1` |
| 177 | `BuildFirstPossibility` | `GREEN_ENERGY` | `Tile.VALUE.WASTE_INCINERATOR_2_2` |
| 178 | `BuildFirstPossibility` | `CONTRACTORS` | `Tile.VALUE.HOME_IMPROVEMENT_CENTER_2_2` |
| 179 | `BuildFirstPossibility` | `GREEN_ENERGY` | `Tile.VALUE.POWER_FUSION_PLANT_2_2` |
| 313 | `JobsProvidedIntPossibility` | — | `CityMetrics.current.jobsProvided` |
| 314 | `MetricReachPossibility` | `CITIZENS` | `CityMetrics.current.recreation` |
| 315 | `MetricReachPossibility` | `TRANSPORTATION` | `CityMetrics.current.traffic` |
| 316 | `MetricReachPossibility` | `CITIZENS` | `CityMetrics.current.taxSatisfaction` |
| 317 | `MetricReachPossibility` | `PARKS_AND_REC` | `CityMetrics.current.cleanliness` |
| 318 | `MetricReachPossibility` | `PARKS_AND_REC` | `CityMetrics.current.environment` |
| 319 | `MetricReachPossibility` | `HEALTHCARE` | `CityMetrics.current.health` |
| 320 | `MetricReachIntPossibility` | `TOWN_HALL` | `CityMetrics.current.touristVisitors` |
| 321 | `MetricReachPossibility` | `CITIZENS` | `Tile.VALUE.PARLIAMENT_2_2`, `CityMetrics.current.community` |
| 322 | `MetricReachPossibility` | `CITIZENS` | `CityMetrics.current.happiness` |
| 412 | `BuildWhenMetricLowPercentPossibility` | `GREEN_ENERGY` | `Tile.VALUE.SEWAGE_DRAIN_1_1`, `CityMetrics.current.sewageManagement` |
| 413 | `BuildWhenMetricLowPercentPossibility` | `GREEN_ENERGY` | `Tile.VALUE.WATER_TREATMENT_PLANT_2_2`, `CityMetrics.current.sewageManagement` |
| 414 | `BuildWhenMetricLowPercentPossibility` | `GREEN_ENERGY` | `Tile.VALUE.LANDFILL_2_2`, `CityMetrics.current.cleanliness` |
| 415 | `MetricReachSpecificValue` | `TOWN_HALL` | `CityMetrics.current.visiblePopulation` |
| 416 | `MetricReachSpecificValue` | `TOWN_HALL` | `CityMetrics.current.visiblePopulation` |
| 417 | `MetricReachSpecificValue` | `TOWN_HALL` | `CityMetrics.current.visiblePopulation` |
| 418 | `MetricReachSpecificValue` | `TOWN_HALL` | `CityMetrics.current.visiblePopulation` |
| 419 | `MetricReachSpecificValue` | `TOWN_HALL` | `CityMetrics.current.visiblePopulation` |
| 420 | `MetricReachSpecificValue` | `TOWN_HALL` | `CityMetrics.current.visiblePopulation` |
| 421 | `MetricReachSpecificValue` | `TOWN_HALL` | `CityMetrics.current.visiblePopulation` |
| 422 | `MetricReachSpecificValue` | `TOWN_HALL` | `CityMetrics.current.visiblePopulation` |
| 450 | `FindTheEncounterPossibility` | — | `EventManager.EVENT_TYPE.CAT_FOUND` |
| 451 | `FindTheEncounterPossibility` | — | `EventManager.EVENT_TYPE.ILLEGAL_BUSINESS_REPORTED` |
| 452 | `FindTheEncounterPossibility` | — | `EventManager.EVENT_TYPE.HIKER_SAVED` |
| 453 | `CountEventsPossibilityStaticDesc` | `POLICE_DEPT` | `EventManager.EVENT_TYPE.PLAYER_STOPPED_CRIME` |
| 454 | `FindTheEncounterPossibility` | — | `EventManager.EVENT_TYPE.CONSTRUCTION_COMPANY_CHOSEN` |
| 455 | `FindTheEncounterPossibility` | — | `EventManager.EVENT_TYPE.DIGRUNTLED_CUSTOMER_CHOSEN` |
| 456 | `CountEventsPossibilityStaticDesc` | `POLICE_DEPT` | `EventManager.EVENT_TYPE.IMPOUND_VEHICLE` |
| 457 | `CountEventsPossibilityStaticDesc` | `STEEL_WORKER` | `EventManager.EVENT_TYPE.SCRAP_VEHICLE` |
| 458 | `CountEventsPossibilityStaticDesc` | `CITIZENS` | `EventManager.EVENT_TYPE.FILL_GAS_VEHICLE` |
| 511 | `CountEventsPossibilityStaticDesc` | `CITIZENS` | `EventManager.EVENT_TYPE.RACCOON_SHOOED` |
| 513 | `CountEventsPossibilityStaticDesc` | `REALTOR` | `EventManager.EVENT_TYPE.BUY_HOME` |
| 515 | `CountRepairRoadsPossibility` | — | — |
| 516 | `CountRepairRoadsPossibility` | — | — |
| 517 | `CountRepairRoadsPossibility` | — | — |
| 518 | `CountEventsPossibilityStaticDesc` | `SPORTSWOMAN` | `EventManager.EVENT_TYPE.CAR_JUMP_LANDED_1_SEC` |
| 519 | `CountEventsPossibilityStaticDesc` | `SPORTSWOMAN` | `EventManager.EVENT_TYPE.CAR_JUMP_LANDED_2_SEC` |
| 520 | `CountEventsPossibilityStaticDesc` | `SPORTSWOMAN` | `EventManager.EVENT_TYPE.PLAYER_FISH_COLLECTED` |
| 521 | `CountEventsPossibilityStaticDesc` | `CONTRACTORS` | `EventManager.EVENT_TYPE.PLAYER_LUMBER_COLLECTED` |
| 522 | `CountEventsPossibilityStaticDesc` | `WORKERS` | `EventManager.EVENT_TYPE.PLAYER_MINE_COLLECTED` |
| 523 | `CountEventsPossibilityStaticDesc` | `WORKERS` | `EventManager.EVENT_TYPE.PLAYER_FIX_COMPLETED` |
| 526 | `SellResourcePossibility` | `CONTRACTORS` | `Tile.VALUE.LUMBER_MILL_2_2`, `ResourceManager.RESOURCE_TYPE.WOOD` |
| 527 | `SellResourcePossibility` | `STEEL_WORKER` | `Tile.VALUE.STEEL_PLANT_2_2`, `ResourceManager.RESOURCE_TYPE.METAL` |
| 528 | `SellResourcePossibility` | `STEEL_WORKER` | `Tile.VALUE.ORE_MINE_4_4`, `ResourceManager.RESOURCE_TYPE.ORE` |
| 529 | `SellResourcePossibility` | `CONTRACTORS` | `Tile.VALUE.CLOTHING_FACTORY_2_2`, `ResourceManager.RESOURCE_TYPE.CONSUMER_GOODS` |
| 530 | `SellResourcePossibility` | `TRANSPORTATION` | `Tile.VALUE.FARM_2_2`, `ResourceManager.RESOURCE_TYPE.FOOD` |
| 550 | `RegionalCitiesCompletedLv50Possibility` | `REALTOR` | — |
| 551 | `RegionalCitiesCompletedLv50Possibility` | `REALTOR` | — |
| 552 | `RegionalCitiesCompletedLv50Possibility` | `REALTOR` | — |
| 553 | `RegionalCitiesCompletedLv50Possibility` | `REALTOR` | — |
| 554 | `RegionalCitiesCompletedLv50Possibility` | `REALTOR` | — |
| 616 | `BuildThenMetricReachPossibility` | `TRANSPORTATION` | `Tile.VALUE.TRAFFIC_GARAGE_2_2`, `CityMetrics.current.traffic` |
| 617 | `BuildThenMetricReachPossibility` | `TRANSPORTATION` | `Tile.VALUE.TRAFFIC_LIGHT_1_1`, `CityMetrics.current.traffic` |
| 618 | `BuildThenMetricReachPossibility` | `EDUCATION` | `Tile.VALUE.SCHOOL_2_2`, `CityMetrics.current.education` |
| 619 | `BuildThenMetricReachPossibility` | `EDUCATION` | `Tile.VALUE.HIGH_SCHOOL_2_2`, `CityMetrics.current.education` |
| 621 | `BuildThenMetricReachPossibility` | `SPORTSWOMAN` | `Tile.VALUE.COMMUNITY_CENTER_2_2`, `CityMetrics.current.community` |
| 713 | `CountEventsPossibilityStaticDesc` | `VET` | `EventManager.EVENT_TYPE.PET_ADOPTED`, `Tile.VALUE.ANIMAL_SHELTER_1_1` |
| 714 | `CountEventsPossibilityStaticDesc` | `VET` | `EventManager.EVENT_TYPE.PET_RAISE_TO_MAX_WELLBEING` |
| 715 | `CountEventsPossibilityStaticDesc` | `TOWN_HALL` | `EventManager.EVENT_TYPE.MEGA_PROJECT_START`, `Tile.VALUE.MEGA_AMUSEMENT_PARK_5_5` |
| 716 | `BuildOfTypePossibility` | `TOWN_HALL` | `Tile.VALUE.MEGA_ZOO_5_5` |
| 750 | `BuildZonesePossibility` | `REALTOR` | `Tile.TYPE.RES` |
| 751 | `BuildZonesePossibility` | `WORKERS` | `Tile.TYPE.IND` |
| 752 | `BuildZonesePossibility` | `SMALL_BUSINESS` | `Tile.TYPE.COM` |
| 753 | `UpgradeZonesPossibility` | `REALTOR` | `Tile.TYPE.RES` |
| 754 | `UpgradeZonesPossibility` | `WORKERS` | `Tile.TYPE.IND` |
| 755 | `UpgradeZonesPossibility` | `SMALL_BUSINESS` | `Tile.TYPE.COM` |
| 756 | `BuildZonesePossibilityHighDemand` | `REALTOR` | `Tile.TYPE.RES` |
| 757 | `BuildZonesePossibilityHighDemand` | `WORKERS` | `Tile.TYPE.IND` |
| 758 | `BuildZonesePossibilityHighDemand` | `SMALL_BUSINESS` | `Tile.TYPE.COM` |
| 831 | `ResearchPossibility` | `GREEN_ENERGY` | — |
| 833 | `ResearchPossibility` | `FINANCIAL` | — |
| 834 | `ResearchPossibility` | `SMALL_BUSINESS` | — |
| 835 | `ResearchPossibility` | `HEALTHCARE` | — |
| 836 | `ResearchPossibility` | `EDUCATION` | — |
| 837 | `ResearchPossibility` | `POLICE_DEPT` | `Research.RESEARCH.SAFETY_COMMUNITY_MENTORING` |
| 839 | `ResearchPossibility` | `ROBOTICS_EXPERT` | — |
| 840 | `ResearchPossibility` | `TRANSPORTATION` | `Research.RESEARCH.FOREIGN_RELATIONS` |
| 900 | `CountEventsPossibilityStaticDesc` | `WORKERS` | `EventManager.EVENT_TYPE.OPENED_LOOT_CRATE` |
| 901 | `CountEventsPossibilityStaticDesc` | `PARKS_AND_REC` | `EventManager.EVENT_TYPE.OPENED_LOOT_CHEST` |
| 902 | `CountEventsPossibilityStaticDesc` | `TRANSPORTATION` | `EventManager.EVENT_TYPE.BORROWED_CAR` |
| 905 | `FindLostItemQuest` | `EDUCATION` | `QuestLostItemSpawn.LOST_QUEST_ITEM_TYPE.TEDDY_BEAR`, `QuestLostItemSpawn.LOST_ITEM_LOCATION.ANIMALSHELTER_LOCATION` |
| 906 | `FindLostItemQuest` | `CONTRACTORS` | `QuestLostItemSpawn.LOST_QUEST_ITEM_TYPE.WALLET`, `QuestLostItemSpawn.LOST_ITEM_LOCATION.SUBWAY_TRAIN` |
| 907 | `FindLostItemQuest` | `EDUCATION` | `QuestLostItemSpawn.LOST_QUEST_ITEM_TYPE.TEDDY_BEAR`, `QuestLostItemSpawn.LOST_ITEM_LOCATION.ELEMENTARY_SCHOOL` |
| 908 | `FindLostItemQuest` | `FIRE_DEPT` | `QuestLostItemSpawn.LOST_QUEST_ITEM_TYPE.WALLET`, `QuestLostItemSpawn.LOST_ITEM_LOCATION.POLICESTATION_LOCATION` |
| 909 | `FindLostItemQuest` | `POLICE_DEPT` | `QuestLostItemSpawn.LOST_QUEST_ITEM_TYPE.PHONE`, `QuestLostItemSpawn.LOST_ITEM_LOCATION.SHIPPING_DOCK` |
| 910 | `FindLostItemQuest` | `GREEN_ENERGY` | `QuestLostItemSpawn.LOST_QUEST_ITEM_TYPE.PHONE`, `QuestLostItemSpawn.LOST_ITEM_LOCATION.FIRE_DEPT_LOCATION` |
| 911 | `FindLostItemQuest` | `VET` | `QuestLostItemSpawn.LOST_QUEST_ITEM_TYPE.WALLET`, `QuestLostItemSpawn.LOST_ITEM_LOCATION.ELEVATED_TRAIN` |
| 912 | `FindLostItemQuest` | `STEEL_WORKER` | `QuestLostItemSpawn.LOST_QUEST_ITEM_TYPE.PHONE`, `QuestLostItemSpawn.LOST_ITEM_LOCATION.HOSPITAL` |
| 1001 | `BuildZonesePossibility` | `TOWN_HALL` | `Tile.TYPE.RES` |
| 1002 | `JobsProvidedIntPossibility` | — | `CityMetrics.current.jobsProvided` |
| 1003 | `DeliverResourcePossibility` | `ROBOTICS_EXPERT` | `ResourceManager.RESOURCE_TYPE.METAL` |
| 1004 | `DeliverResourcePossibility` | `ROBOTICS_EXPERT` | `ResourceManager.RESOURCE_TYPE.METAL` |
| 1100 | `CityCompetitionWonPossibility` | `TOWN_HALL` | `CityEvents.EVENT_ID.CITY_COMPETE_WONDERPARK` |
| 1101 | `CityCompetitionWonPossibility` | `TOWN_HALL` | `CityEvents.EVENT_ID.CITY_COMPETE_MONEYHILL` |
| 1102 | `CityCompetitionWonPossibility` | `TOWN_HALL` | `CityEvents.EVENT_ID.CITY_COMPETE_COMMUTE` |
| 1103 | `CityCompetitionWonPossibility` | `TOWN_HALL` | `CityEvents.EVENT_ID.CITY_COMPETE_MERRY_SMOKES` |
| 1200 | `MetricReachSpecificValue` | `TOWN_HALL` | `CityMetrics.current.commuting` |
| 1201 | `CommuterAdditionalIntPossibility` | — | `CityMetrics.current.commuting` |
| 1300 | `CountEventsPossibilityStaticDesc` | `FINANCIAL` | `EventManager.EVENT_TYPE.THROW_HIT_MONEY` |
| 1301 | `CountEventsPossibilityStaticDesc` | `REALTOR` | `EventManager.EVENT_TYPE.APPLIED_MAYOR_HAT_NPC` |

## Pocket City 2 research

The 52 non-sentinel research IDs each have a registered definition. The table lists the parent and the costs checked for affordability. See the progression notes for the difference between checking costs and deducting them.

| ID | Research | Category | Parent ID | Points | Resource requirement |
| --- | --- | --- | --- | --- | --- |
| 1 | Improved Machinery | CONSTRUCT | None | 3 | None |
| 2 | Material Improvements | CONSTRUCT | 1 | 3 | 150 `METAL` |
| 3 | Recycle Materials | CONSTRUCT | 2 | 3 | 150 `METAL` |
| 4 | Improved Project Planning | CONSTRUCT | 1 | 8 | 1000 `WOOD` |
| 5 | Green Energy | ENERGY | None | 8 | None |
| 6 | Neon City | REC | None | 5 | 250 `METAL` |
| 7 | Crime Hotline | HEALTH | None | 1 | None |
| 8 | Community Mentoring | HEALTH | 7 | 3 | None |
| 9 | Logistics Improvements | FINANCE | None | 5 | None |
| 10 | Mental Health Awareness | HEALTH | None | 3 | None |
| 11 | Community Development | FINANCE | 9 | 5 | None |
| 12 | Small Business Income | FINANCE | 14 | 5 | None |
| 13 | Big Business Income | FINANCE | 12 | 5 | 100 `CONSUMER_GOODS` |
| 14 | Bank Income | FINANCE | None | 5 | None |
| 15 | Bank Investments | FINANCE | 14 | 5 | 2 `INTELLECTUAL_PROPERTY` |
| 16 | Curriculum Improvements | EDUCATION | None | 2 | None |
| 17 | Youth Programs | EDUCATION | None | 5 | 250 `WOOD` |
| 18 | Relocation Incentives | CITIZENS | None | 3 | None |
| 19 | Four Day Work Week | FINANCE | None | 3 | None |
| 20 | Quantum Computing | EDUCATION | 16 | 15 | None |
| 21 | Remote Work | FINANCE | 19 | 5 | None |
| 22 | Daycare Programs | EDUCATION | 17 | 3 | 100 `CONSUMER_GOODS` |
| 23 | Universal Healthcare | HEALTH | None | 15 | None |
| 24 | Foreign Relations | FINANCE | 9 | 10 | None |
| 50 | Landfill Park | ENERGY | None | 2 | 500 `WOOD` |
| 51 | Waste Shipping | ENERGY | 50 | 10 | None |
| 52 | Carbon Capture Plant | ENERGY | 5 | 5 | 1 `INTELLECTUAL_PROPERTY` |
| 53 | All Bridges | CONSTRUCT | 1 | 1 | 500 `WOOD` |
| 54 | Global Trade Deals | FINANCE | 24 | 10 | 250 `RARE_ORE` |
| 55 | Quality Roads | CONSTRUCT | 53 | 3 | 1000 `ORE` |
| 56 | Entrance Fees | REC | 64 | 5 | 1 `INTELLECTUAL_PROPERTY` |
| 57 | Space Mining | EDUCATION | None | 10 | 3 `INTELLECTUAL_PROPERTY` |
| 58 | EV Development | ENERGY | 5 | 5 | 50 `ELECTRONIC_COMPONENTS` |
| 59 | Sightseeing | REC | 66 | 8 | 500 `CONSUMER_GOODS` |
| 60 | Auto Upgrades | CONSTRUCT | 4 | 5 | 200 `RARE_ORE` |
| 61 | Skilled Realtors | CITIZENS | 18 | 5 | 100 `WOOD` |
| 62 | Conflict Resolution Training | HEALTH | 8 | 5 | 1 `INTELLECTUAL_PROPERTY` |
| 63 | Advanced Medical Technology | HEALTH | 23 | 10 | 50 `ELECTRONIC_COMPONENTS` |
| 64 | Recreation Novelty | REC | None | 3 | None |
| 65 | Retail Promotions | FINANCE | 11 | 5 | 500 `CONSUMER_GOODS` |
| 66 | Landmark Novelty | REC | None | 5 | None |
| 67 | Daily Fireworks | REC | 68 | 3 | 10 `ELECTRONIC_COMPONENTS` |
| 68 | Social Media Attraction | REC | 66 | 5 | None |
| 69 | Motivated Friends | CITIZENS | None | 15 | None |
| 70 | Hiring Bonus | FINANCE | 19 | 20 | 10 `INTELLECTUAL_PROPERTY` |
| 71 | Mega Project Employment | FINANCE | 70 | 25 | 5 `INTELLECTUAL_PROPERTY` |
| 72 | Water Treatment Sanitation Pump | ENERGY | 51 | 15 | 50 `ELECTRONIC_COMPONENTS` |
| 73 | Resource Building Jobs | FINANCE | None | 1 | 200 `FOOD` |
| 74 | Urban Walkability | CITIZENS | 61 | 10 | 500 `WOOD` |
| 75 | Multi Family Homes | CITIZENS | None | 25 | None |
| 76 | Mixed Use Commercial Zones | CITIZENS | 75 | 15 | 500 `METAL` |
| 77 | AI-Optimized Economy | EDUCATION | 20 | 15 | 250 `INTELLECTUAL_PROPERTY` |

## Pocket City 2 buildings tied to affinity

Each building below has a level-50 affinity requirement for the associated profile. Other building requirements can also apply.

| Affinity profile | Building identifier |
| --- | --- |
| `TOWN_HALL` | `MAYOR_MANSION_2_2` |
| `REALTOR` | `CIRCUS_2_2` |
| `FIRE_DEPT` | `IRON_TOWER_2_2` |
| `CONTRACTORS` | `GIANT_BEAVER_2_2` |
| `CITIZENS` | `GIANT_GOOSE_2_2` |
| `PARKS_AND_REC` | `GIANT_RACCOON_2_2` |
| `EDUCATION` | `TECHNOLOGY_TOWER_1_1` |
| `SMALL_BUSINESS` | `COMMERCE_TOWER_1_1` |
| `FINANCIAL` | `BILLIONAIRE_TOWER_1_1` |
| `WORKERS` | `EMPLOYMENT_CENTER_2_2` |
