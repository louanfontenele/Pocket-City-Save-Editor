import { invoke } from "@tauri-apps/api/core"
import dictionary from "../../src/locales.txt?raw"

export type Game = 1 | 2
export type SaveKind = "main" | "auto_archive" | "manual_archive" | "global"
export const editorSections = ["overview", "collections", "progression_tab", "advanced", "backups"] as const
export type EditorSection = typeof editorSections[number]
export type Entry = { path: string; title: string; filename: string; game: Game; kind: SaveKind }
export type FieldView = {
  index: number
  path: string
  name: string
  value: string
  type: "boolean" | "number" | "text"
  locked: boolean
  changed: boolean
  overview: boolean
  collection: string | null
  help: string
  options: [string, string][] | null
}
export type ProgressEdit =
  | { kind: "treasure"; index: number }
  | { kind: "policy" | "quest" | "follower" | "cooldown"; id: number }
  | { kind: "research"; id: number; choice: "locked" | "active" | "disabled" }
  | { kind: "researchAll" | "debt" }
  | { kind: "pet"; id: number; value: number }
  | { kind: "taxPreset"; index: number; preset: number }
  | { kind: "taxPercent"; index: number; value: number }

export type ProgressAction = { label: string; edit: ProgressEdit; selected: boolean; enabled: boolean }
export type SliderView = {
  label: string
  max: number
  unit: string
  value: number
  edit: Extract<ProgressEdit, { kind: "taxPercent" | "pet" }>
}
export type ProgressSection = {
  key: string
  label: string
  help: string
  empty: string
  bulk?: ProgressAction
  rows: { id: number; name: string; help: string; actions: ProgressAction[]; slider?: SliderView; amount?: string }[]
}
export type Backup = { path: string; label: string }
export type DocumentView = Entry & {
  gameTitle: string
  mode: string | null
  archiveNote: boolean
  pending: number
  snapshot: number
  snapshots: string[]
  fields: FieldView[]
  collections: { key: string; label: string; help: string; action: string | null }[]
  level: number | null
  xp: number | null
  nextXp: number | null
  progression: ProgressSection[] | null
  backups: Backup[]
  backupError: string | null
}
export type EditorView = {
  locale: string
  languageName: string
  languages: [string, string][]
  library: Entry[]
  generation: number
  document: DocumentView | null
  notice: { error: boolean; message: string } | null
}
export type Request =
  | { kind: "view" | "refresh" | "openFile" | "addFolder" | "openDrop" | "resetXp" | "save" | "reload" | "export" }
  | { kind: "openEntry" | "restore"; path: string }
  | { kind: "language"; code: string }
  | { kind: "snapshot"; index: number }
  | { kind: "collection"; key: string }
  | { kind: "progress"; edit: ProgressEdit }
export type Drafts = Record<number, string>
export type Translate = (key: string) => string

export const gameTitle = (game: Game) => game === 1 ? "Pocket City" : "Pocket City 2"

const localeCodes = ["pt-BR", "en", "ja", "fr", "it", "de", "es-ES", "cs", "ko", "ru", "zh-Hans", "es-419", "uk"]
const strings = new Map(dictionary.trimEnd().split(/\r?\n/).map(line => {
  const [key, ...values] = line.split("|")
  return [key, values]
}))

export function translator(locale: string): Translate {
  const index = Math.max(0, localeCodes.indexOf(locale))
  return key => strings.get(key)?.[index] ?? key
}

export function requestEditor(request: Request, generation?: number, drafts: Drafts = {}) {
  return invoke<EditorView>("editor_request", {
    request,
    generation: generation ?? null,
    drafts: Object.entries(drafts).map(([index, value]) => ({ index: Number(index), value })),
  })
}
