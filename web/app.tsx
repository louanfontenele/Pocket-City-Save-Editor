import { useCallback, useEffect, useRef, useState } from "react"
import { listen } from "@tauri-apps/api/event"
import {
  ArrowUpRight, CircleAlert, CircleCheck, DatabaseBackup, FileUp, RotateCcw, Save,
} from "lucide-react"
import { AppSidebar } from "@/components/app-sidebar"
import { EmptyMessage, Fields, FieldSelect, SearchBox } from "@/components/fields"
import { Progression } from "@/components/progression"
import { ThemeMenu } from "@/components/theme"
import { Alert, AlertDescription } from "@/components/ui/alert"
import {
  AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent,
  AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger,
} from "@/components/ui/alert-dialog"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Breadcrumb, BreadcrumbItem, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "@/components/ui/breadcrumb"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"
import { FieldLabel } from "@/components/ui/field"
import { Progress } from "@/components/ui/progress"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Separator } from "@/components/ui/separator"
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { Skeleton } from "@/components/ui/skeleton"
import { TooltipProvider } from "@/components/ui/tooltip"
import { gameTitle, requestEditor, translator } from "@/lib/editor"
import type { DocumentView, Drafts, EditorSection, EditorView, FieldView, Game, Request, Translate } from "@/lib/editor"

function CitySummary({ doc, drafts, t, locale }: {
  doc: DocumentView; drafts: Drafts; t: Translate; locale: string
}) {
  const metrics = [
    ["level", ["level", "stats.level"]],
    ["money", ["money", "stats.money"]],
    ["population", ["population", "stats.population"]],
  ] as const
  const activeMetrics = metrics.flatMap(([label, paths]) => {
    const field = doc.fields.find(item => paths.some(path => item.path === path))
    return field ? [{ label, value: drafts[field.index] ?? field.value }] : []
  })
  if (!activeMetrics.length) return null
  const format = new Intl.NumberFormat(locale)
  return (
    <div className="grid grid-cols-1 gap-3 @md/main:grid-cols-2 @xl/main:grid-cols-3">
      {activeMetrics.map(metric => {
        const integer = /^-?\d+$/.test(metric.value) ? BigInt(metric.value) : null
        return (
          <Card key={metric.label} size="sm">
            <CardHeader>
              <CardDescription>{t(metric.label)}</CardDescription>
              <CardTitle className="break-words tabular-nums">
                {metric.label === "money" && "$"}{integer === null ? metric.value || "—" : format.format(integer)}
              </CardTitle>
            </CardHeader>
          </Card>
        )
      })}
    </div>
  )
}

function CityProgress({ doc, drafts, t, locale, send, busy }: {
  doc: DocumentView; drafts: Drafts; t: Translate; locale: string; send: (request: Request) => void; busy: boolean
}) {
  const stagedValue = (field: FieldView) => drafts[field.index] ?? field.value
  const sandboxFields = ["unlockAll", "infiniteMoney", "disableQuests", "maxLevel"]
  const mode = doc.mode === "regular_city" || doc.mode === "sandbox"
    ? doc.fields.some(field => sandboxFields.includes(field.path) && stagedValue(field) === "true") ? "sandbox" : "regular_city"
    : doc.mode
  const xpField = doc.fields.find(field => field.path === "XP" || field.path === "stats.exp")
  const xp = xpField && drafts[xpField.index] !== undefined ? Number(drafts[xpField.index]) : doc.xp
  const nextXp = doc.nextXp
  const hasTarget = xp !== null && Number.isFinite(xp) && nextXp !== null && nextXp > 0
  if (!mode && !hasTarget) return null
  const format = new Intl.NumberFormat(locale)
  return (
    <div className="flex flex-col gap-3">
      {mode && <div className="flex flex-wrap items-center gap-2">
        <Badge variant="outline">{t(mode)}</Badge>
        {mode === "sandbox" && <p className="text-sm text-muted-foreground">{t("sandbox_effects")}</p>}
      </div>}
      {hasTarget && <div className="flex flex-wrap items-center gap-3">
        <div className="flex min-w-40 max-w-md flex-1 flex-col gap-2">
          <p className="text-sm text-muted-foreground">{t("xp_progress")
            .replace("{xp}", format.format(xp)).replace("{total}", format.format(nextXp))}</p>
          <Progress value={Math.max(0, Math.min(100, xp / nextXp * 100))} aria-label={t("xp")} />
        </div>
        <Button variant="outline" size="sm" disabled={busy || xp === 0} onClick={() => send({ kind: "resetXp" })}>
          {t("reset_xp")}
        </Button>
      </div>}
      <Separator />
    </div>
  )
}

function BackupList({ doc, t, busy, pending, send }: {
  doc: DocumentView; t: Translate; busy: boolean; pending: number; send: (request: Request) => void
}) {
  if (doc.backupError) return <Alert variant="destructive"><CircleAlert /><AlertDescription>{doc.backupError}</AlertDescription></Alert>
  if (doc.backups.length === 0) return <EmptyMessage message={t("backup_help")} />
  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm text-muted-foreground">{t("backup_help")}</p>
      <ul className="divide-y">
        {doc.backups.map(backup => (
          <li key={backup.path} className="flex flex-wrap items-center justify-between gap-3 py-3">
            <span className="flex min-w-0 items-center gap-2 break-all">
              <DatabaseBackup className="size-4 shrink-0 text-muted-foreground" />{backup.label}
            </span>
            <AlertDialog>
              <AlertDialogTrigger asChild>
                <Button size="sm" variant="outline" disabled={busy || pending !== 0}>{t("restore_open")}</Button>
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>{t("restore_backup")}</AlertDialogTitle>
                  <AlertDialogDescription>{t("restore_help")}</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel>{t("cancel")}</AlertDialogCancel>
                  <AlertDialogAction onClick={() => send({ kind: "restore", path: backup.path })}>{t("restore")}</AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          </li>
        ))}
      </ul>
    </div>
  )
}

function Document({ doc, section, t, send, busy, pending, notice, drafts, setDraft, locale }: {
  doc: DocumentView
  section: EditorSection
  t: Translate
  send: (request: Request) => void
  busy: boolean
  pending: number
  notice: EditorView["notice"]
  drafts: Drafts
  setDraft: (field: FieldView, value: string) => void
  locale: string
}) {
  const [query, setQuery] = useState("")
  const [collection, setCollection] = useState("")
  const activeCollection = doc.collections.find(item => item.key === collection) ?? doc.collections[0]
  const normalized = query.trim().toLocaleLowerCase(locale)
  const fields = doc.fields.filter(field => (
    (section === "overview" ? field.overview
      : section === "collections" ? field.collection === activeCollection?.key : true)
    && (!normalized || field.name.toLocaleLowerCase(locale).includes(normalized) || field.path.toLowerCase().includes(normalized))
  ))
  const fieldProps = { items: fields, drafts, setDraft, t, busy }
  return (
      <ScrollArea className="min-h-0 flex-1">
        <section aria-labelledby="editor-section-title">
        <div className="@container/main flex flex-col gap-4 px-4 py-4 lg:px-6">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <h1 id="editor-section-title" className="text-2xl font-semibold text-balance">{t(section)}</h1>
            {doc.snapshots.length > 1 && <div className="flex flex-wrap items-center gap-2">
              <FieldLabel htmlFor="city-version">{t("version")}</FieldLabel>
              <FieldSelect id="city-version" label={t("version")} value={String(doc.snapshot)}
                options={doc.snapshots.map((label, index) => [String(index), label])}
                disabled={busy} onChange={value => { setQuery(""); send({ kind: "snapshot", index: Number(value) }) }} />
            </div>}
          </div>
          {pending > 0 && <p role="status" className="text-xs font-medium text-primary">{t("pending").replace("{n}", String(pending))}</p>}
          {notice && <Alert aria-live="polite" variant={notice.error ? "destructive" : "default"}>
            {notice.error ? <CircleAlert /> : <CircleCheck />}<AlertDescription>{notice.message}</AlertDescription>
          </Alert>}
          {doc.archiveNote && <p className="text-sm text-muted-foreground">{t("archive_edit_help")}</p>}
          {section === "overview" && <>
            <CitySummary doc={doc} drafts={drafts} t={t} locale={locale} />
            <Card size="sm">
              <CardHeader><CardTitle>{t("city_data")}</CardTitle></CardHeader>
              <CardContent>
                <CityProgress doc={doc} drafts={drafts} t={t} locale={locale} send={send} busy={busy} />
                <Fields key={doc.snapshot} {...fieldProps} />
              </CardContent>
            </Card>
          </>}
          {section === "collections" && <>
            {activeCollection
              ? <>
                  <div className="flex flex-wrap items-center gap-2">
                    <FieldSelect label={t("collections")} value={activeCollection.key}
                      options={doc.collections.map(item => [item.key, item.label])}
                      onChange={value => { setCollection(value); setQuery("") }} disabled={busy} />
                    <SearchBox label={t("search")} value={query} onChange={setQuery} />
                    {activeCollection.action && (
                      <Button variant="outline" size="sm" disabled={busy}
                        onClick={() => send({ kind: "collection", key: activeCollection.key })}>{activeCollection.action}</Button>
                    )}
                  </div>
                  {activeCollection.help && <p className="text-sm text-muted-foreground">{activeCollection.help}</p>}
                  <Fields key={[doc.snapshot, activeCollection.key, query].join(":")} {...fieldProps} />
                </>
              : <EmptyMessage message={t("empty_fields")} />}
          </>}
          {section === "progression_tab" && doc.progression && (
            <Progression key={doc.snapshot} sections={doc.progression} t={t} send={send} busy={busy} />
          )}
          {section === "advanced" && <>
            <SearchBox label={t("search")} value={query} onChange={setQuery} />
            <Fields key={[doc.snapshot, query].join(":")} {...fieldProps} advanced />
          </>}
          {section === "backups" && (
            <BackupList doc={doc} t={t} send={send} busy={busy} pending={pending} />
          )}
        </div>
        </section>
      </ScrollArea>
  )
}

export function App() {
  const [view, setView] = useState<EditorView | null>(null)
  const [game, setGame] = useState<Game>(1)
  const [section, setSection] = useState<EditorSection>("overview")
  const [drafts, setDrafts] = useState<Drafts>({})
  const [busy, setBusy] = useState(false)
  const [ipcError, setIpcError] = useState("")
  const viewRef = useRef<EditorView | null>(null)
  const draftRef = useRef<Drafts>({})
  const inFlight = useRef(false)
  const queuedDrop = useRef(false)
  const locale = view?.locale ?? "pt-BR"
  const t = translator(locale)
  const doc = view?.document
  const activeSection = section === "progression_tab" && !doc?.progression ? "overview" : section
  const pending = (doc?.pending ?? 0) + (doc?.fields.filter(field => !field.changed && drafts[field.index] !== undefined).length ?? 0)

  useEffect(() => {
    document.documentElement.lang = locale
  }, [locale])

  const send = useCallback(async (request: Request): Promise<void> => {
    if (inFlight.current) {
      if (request.kind === "openDrop") queuedDrop.current = true
      return
    }
    inFlight.current = true
    setBusy(true)
    setIpcError("")
    const previous = viewRef.current
    const sentDrafts = draftRef.current
    try {
      const next = await requestEditor(request, previous?.generation, sentDrafts)
      if (next.document && next.document.path !== previous?.document?.path) {
        setGame(next.document.game)
        setSection(next.document.fields.some(field => field.overview) ? "overview" : "advanced")
      }
      const nextFields = new Map(next.document?.fields.map(field => [String(field.index), field.value]))
      const remaining = next.generation === previous?.generation
        ? Object.fromEntries(Object.entries(sentDrafts).filter(([index, value]) => nextFields.get(index) !== value))
        : {}
      draftRef.current = remaining
      setDrafts(remaining)
      viewRef.current = next
      setView(next)
    } catch (error) {
      setIpcError(String(error))
    } finally {
      setBusy(false)
      inFlight.current = false
      if (queuedDrop.current) {
        queuedDrop.current = false
        void send({ kind: "openDrop" })
      }
    }
  }, [])

  useEffect(() => {
    void send({ kind: "view" })
    const subscribed = listen("editor-updated", () => { void send({ kind: "openDrop" }) })
    void subscribed.catch(error => setIpcError(String(error)))
    return () => { void subscribed.then(unlisten => unlisten(), () => undefined) }
  }, [send])

  function setDraft(field: FieldView, value: string) {
    if (inFlight.current) return
    const draft = { ...draftRef.current }
    if (value === field.value) delete draft[field.index]
    else draft[field.index] = value
    draftRef.current = draft
    setDrafts(draft)
  }

  const notice = ipcError ? { error: true, message: ipcError } : view?.notice ?? null

  return (
    <TooltipProvider>
      <SidebarProvider className="h-svh overflow-hidden">
        <Button asChild variant="outline" className="sr-only focus:not-sr-only focus:absolute focus:top-2 focus:left-2">
          <a href="#main-content">{t("skip_to_content")}</a>
        </Button>
        <AppSidebar view={view} game={game} setGame={setGame} section={activeSection} setSection={setSection}
          send={send} busy={busy} t={t} />
        <SidebarInset id="main-content" tabIndex={-1} className="h-svh min-w-0 overflow-hidden md:h-[calc(100svh-1rem)]">
          <header className="flex min-h-12 shrink-0 flex-wrap items-center gap-2 border-b px-4 py-2">
            <SidebarTrigger className="-ml-1 shrink-0" aria-label={t("toggle_sidebar")} />
            <Separator orientation="vertical" className="mr-2 max-h-4 self-center" />
            <Breadcrumb className="min-w-0 flex-1">
              <BreadcrumbList className="flex-nowrap">
                <BreadcrumbItem>{doc?.gameTitle ?? gameTitle(game)}</BreadcrumbItem>
                {doc && <>
                  <BreadcrumbSeparator />
                  <BreadcrumbItem className="hidden xl:inline-flex">{t(doc.kind)}</BreadcrumbItem>
                  <BreadcrumbSeparator className="hidden xl:block" />
                  <BreadcrumbItem className="min-w-0">
                    <BreadcrumbPage className="truncate" title={doc.path}>{doc.title}</BreadcrumbPage>
                  </BreadcrumbItem>
                </>}
              </BreadcrumbList>
            </Breadcrumb>
            {doc && <div className="flex flex-wrap items-center gap-2">
              {pending > 0
                ? <AlertDialog>
                    <AlertDialogTrigger asChild>
                      <Button size="sm" variant="ghost" disabled={busy}>
                        <RotateCcw data-icon="inline-start" />{t("discard")}
                      </Button>
                    </AlertDialogTrigger>
                    <AlertDialogContent>
                      <AlertDialogHeader>
                        <AlertDialogTitle>{t("discard")}</AlertDialogTitle>
                        <AlertDialogDescription>{t("pending").replace("{n}", String(pending))}</AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel>{t("cancel")}</AlertDialogCancel>
                        <AlertDialogAction onClick={() => send({ kind: "reload" })}>{t("discard")}</AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                : <Button size="sm" variant="ghost" disabled={busy} onClick={() => send({ kind: "reload" })}>
                    <RotateCcw data-icon="inline-start" />{t("reload")}
                  </Button>}
              <Button size="sm" variant="outline" disabled={busy} onClick={() => send({ kind: "export" })}>
                <ArrowUpRight data-icon="inline-start" />{t("export")}
              </Button>
              <Button size="sm" disabled={busy || !pending} onClick={() => send({ kind: "save" })}>
                <Save data-icon="inline-start" />{t("save")}
              </Button>
            </div>}
            <ThemeMenu t={t} />
          </header>
          {doc
            ? <Document key={doc.path + ":" + activeSection} doc={doc} section={activeSection} t={t} send={send} busy={busy} pending={pending}
                notice={notice} drafts={drafts} setDraft={setDraft} locale={locale} />
            : <div className="flex min-h-0 flex-1 flex-col justify-center gap-4 p-4">
                {notice && <Alert variant="destructive"><CircleAlert /><AlertDescription>{notice.message}</AlertDescription></Alert>}
                {!view && !ipcError ? <Skeleton className="h-40 w-full" /> : (
                  <Empty className="border">
                    <EmptyHeader>
                      <EmptyMedia variant="icon"><FileUp /></EmptyMedia>
                      <EmptyTitle>{gameTitle(game)}</EmptyTitle>
                      <EmptyDescription>{t("choose")}</EmptyDescription>
                    </EmptyHeader>
                    <EmptyContent>
                      <Button disabled={busy} onClick={() => send({ kind: "openFile" })}>{t("open")}</Button>
                    </EmptyContent>
                  </Empty>
                )}
              </div>}
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  )
}
