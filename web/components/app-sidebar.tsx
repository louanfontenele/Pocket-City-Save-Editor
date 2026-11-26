import { useState } from "react"
import {
  Archive, ArchiveRestore, Blocks, Building2, ChevronDown, ChevronRight, ChevronsUpDown, Clock3,
  FilePlus2, FolderPlus, Gauge, Globe2, History, LayoutGrid, RefreshCw, Settings2, SlidersHorizontal,
} from "lucide-react"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import {
  DropdownMenu, DropdownMenuContent,
  DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupAction, SidebarGroupLabel,
  SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarMenuSub,
  SidebarMenuSubButton, SidebarMenuSubItem, SidebarRail, useSidebar,
} from "@/components/ui/sidebar"
import { editorSections, gameTitle } from "@/lib/editor"
import type { EditorSection, EditorView, Entry, Game, Request, SaveKind, Translate } from "@/lib/editor"

const groups = [
  ["main", Building2], ["auto_archive", Clock3], ["manual_archive", Archive], ["global", Settings2],
] as const satisfies readonly (readonly [SaveKind, typeof Building2])[]
const sectionIcons = {
  overview: LayoutGrid,
  collections: ArchiveRestore,
  progression_tab: Gauge,
  advanced: SlidersHorizontal,
  backups: History,
}

function GameSwitcher({ game, setGame, t }: { game: Game; setGame: (value: Game) => void; t: Translate }) {
  const { isMobile } = useSidebar()
  const Icon = game === 1 ? Building2 : Blocks
  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton size="lg" tooltip={gameTitle(game)} aria-label={gameTitle(game)}>
              <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
                <Icon />
              </div>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-medium">{gameTitle(game)}</span>
                <span className="truncate text-xs">{t("app")}</span>
              </div>
              <ChevronsUpDown className="ml-auto" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start" side={isMobile ? "bottom" : "right"} sideOffset={4}>
            <DropdownMenuRadioGroup value={String(game)} onValueChange={value => setGame(value === "1" ? 1 : 2)}>
              <DropdownMenuRadioItem value="1">Pocket City</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="2">Pocket City 2</DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}

function SaveGroup({ items, current, label, Icon, send, busy, defaultOpen }: {
  items: Entry[]
  current: string | undefined
  label: string
  Icon: typeof Building2
  send: (request: Request) => void
  busy: boolean
  defaultOpen: boolean
}) {
  const { state, setOpen, isMobile } = useSidebar()
  const [expanded, setExpanded] = useState(defaultOpen)
  return (
    <Collapsible asChild open={expanded} onOpenChange={value => {
      if (!isMobile && state === "collapsed") {
        setOpen(true)
        setExpanded(true)
      } else {
        setExpanded(value)
      }
    }} className="group/collapsible">
      <SidebarMenuItem>
        <CollapsibleTrigger asChild>
          <SidebarMenuButton tooltip={label} aria-label={label}>
            <Icon /><span>{label}</span>
            <ChevronRight className="ml-auto transition-transform group-data-[state=open]/collapsible:rotate-90" />
          </SidebarMenuButton>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <SidebarMenuSub>
            {items.map(entry => {
              const active = current === entry.path
              const duplicate = items.some(other => other !== entry && other.title === entry.title)
              return (
                <SidebarMenuSubItem key={entry.path}>
                  <SidebarMenuSubButton asChild isActive={active} className={duplicate ? "h-auto" : undefined}>
                    <button type="button" className="w-full text-left" title={entry.path} disabled={busy}
                      aria-current={active ? "page" : undefined}
                      onClick={() => { if (!active) send({ kind: "openEntry", path: entry.path }) }}>
                      <span className="flex min-w-0 flex-1 flex-col">
                        <span className="truncate">{entry.title}</span>
                        {duplicate && <span className="truncate text-xs text-muted-foreground">{entry.filename}</span>}
                      </span>
                    </button>
                  </SidebarMenuSubButton>
                </SidebarMenuSubItem>
              )
            })}
          </SidebarMenuSub>
        </CollapsibleContent>
      </SidebarMenuItem>
    </Collapsible>
  )
}

export function AppSidebar({ view, game, setGame, section, setSection, send, busy, t }: {
  view: EditorView | null
  game: Game
  setGame: (game: Game) => void
  section: EditorSection
  setSection: (section: EditorSection) => void
  send: (request: Request) => void
  busy: boolean
  t: Translate
}) {
  const { isMobile, setOpenMobile } = useSidebar()
  const entries = view?.library.filter(entry => entry.game === game) ?? []
  return (
    <Sidebar collapsible="icon" variant="inset">
      <SidebarHeader>
        <GameSwitcher game={game} setGame={setGame} t={t} />
      </SidebarHeader>
      <SidebarContent role="navigation" aria-label={t("library")}>
        {view?.document && <SidebarGroup>
          <SidebarGroupLabel>{t("editor_sections")}</SidebarGroupLabel>
          <SidebarMenu>
            {editorSections.filter(key => key !== "progression_tab" || view.document?.progression).map(key => {
              const Icon = sectionIcons[key]
              return <SidebarMenuItem key={key}>
                <SidebarMenuButton tooltip={t(key)} aria-label={t(key)} isActive={section === key}
                  aria-current={section === key ? "page" : undefined}
                  onClick={() => {
                    setSection(key)
                    if (isMobile) setOpenMobile(false)
                  }}>
                  <Icon /><span>{t(key)}</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            })}
          </SidebarMenu>
        </SidebarGroup>}
        <SidebarGroup>
          <SidebarGroupLabel>{t("library")}</SidebarGroupLabel>
          <SidebarGroupAction aria-label={t("refresh")} title={t("refresh")} disabled={busy}
            onClick={() => send({ kind: "refresh" })}><RefreshCw /></SidebarGroupAction>
          {view && entries.length === 0 && (
            <p className="px-2 text-sm text-muted-foreground group-data-[collapsible=icon]:hidden">{t("no_files")}</p>
          )}
          <SidebarMenu>
            {groups.map(([kind, Icon]) => {
              const items = entries.filter(entry => entry.kind === kind)
              const current = view?.document?.path
              return items.length > 0 && (
                <SaveGroup key={[game, kind].join(":")} items={items} current={current} label={t(kind)}
                  Icon={Icon} send={send} busy={busy} defaultOpen={kind === "main" || items.some(item => item.path === current)} />
              )
            })}
          </SidebarMenu>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton disabled={busy} tooltip={t("open")} aria-label={t("open")}
              onClick={() => send({ kind: "openFile" })}>
              <FilePlus2 /><span>{t("open")}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton disabled={busy} tooltip={t("folder")} aria-label={t("folder")}
              onClick={() => send({ kind: "addFolder" })}>
              <FolderPlus /><span>{t("folder")}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <SidebarMenuButton disabled={!view || busy} tooltip={t("language")} aria-label={t("language")}>
                  <Globe2 /><span>{view?.languageName ?? t("language")}</span>
                  <ChevronDown className="ml-auto" />
                </SidebarMenuButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent side={isMobile ? "top" : "right"} align="end" sideOffset={4}>
                <DropdownMenuRadioGroup value={view?.locale} onValueChange={code => send({ kind: "language", code })}>
                  {view?.languages.map(([code, name]) => (
                    <DropdownMenuRadioItem key={code} value={code}>{name}</DropdownMenuRadioItem>
                  ))}
                </DropdownMenuRadioGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
      <SidebarRail aria-label={t("toggle_sidebar")} title={t("toggle_sidebar")} />
    </Sidebar>
  )
}
