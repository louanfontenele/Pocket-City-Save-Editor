import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Slider } from "@/components/ui/slider"
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group"
import { EmptyMessage, FieldSelect, SearchBox } from "@/components/fields"
import type { ProgressAction, ProgressEdit, ProgressSection, Request, SliderView, Translate } from "@/lib/editor"

function ValueSlider({ slider, apply, busy }: { slider: SliderView; apply: (edit: ProgressEdit) => void; busy: boolean }) {
  const [preview, setPreview] = useState<number | null>(null)
  const value = preview ?? slider.value
  return (
    <div className="flex min-w-42 flex-col gap-1.5">
      <div className="flex min-h-7 items-center gap-4">
        <Slider disabled={busy} min={0} max={slider.max} step={1} value={[value]} aria-label={slider.label}
          onValueChange={([next]) => setPreview(next)}
          onValueCommit={([next]) => {
            setPreview(null)
            if (next !== slider.value) apply({ ...slider.edit, value: next })
          }} />
        <output className="min-w-11 text-right text-sm font-semibold text-primary tabular-nums">{value}{slider.unit}</output>
      </div>
      <div className="flex justify-between pr-15 text-xs text-muted-foreground tabular-nums">
        <span>0{slider.unit}</span><span>{slider.max}{slider.unit}</span>
      </div>
    </div>
  )
}

function ActionButton({ action, apply, busy }: { action: ProgressAction; apply: (edit: ProgressEdit) => void; busy: boolean }) {
  return (
    <Button size="sm" variant={action.selected ? "secondary" : "outline"}
      disabled={busy || !action.enabled} onClick={() => apply(action.edit)}>{action.label}</Button>
  )
}

function ProgressRows({ section, t, busy, apply }: {
  section: ProgressSection
  t: Translate
  busy: boolean
  apply: (edit: ProgressEdit) => void
}) {
  const [query, setQuery] = useState("")
  const value = query.trim().toLocaleLowerCase()
  const rows = section.rows.filter(row => !value || row.name.toLocaleLowerCase().includes(value) || String(row.id).includes(value))
  return (
    <div className="flex flex-col gap-4 pt-4">
      <p className="text-sm text-muted-foreground">{section.help}</p>
      {section.key !== "economy" && (
        <div className="flex flex-wrap items-center gap-2">
          <SearchBox label={t("search")} value={query} onChange={setQuery} />
          {section.bulk && <ActionButton action={section.bulk} busy={busy} apply={apply} />}
        </div>
      )}
      {rows.length === 0
        ? <EmptyMessage message={value ? t("empty_fields") : section.empty} />
        : <ul className="divide-y">
            {rows.map(row => (
              <li key={row.id} className="flex flex-wrap items-center justify-between gap-x-6 gap-y-2 py-3.5">
                <div className="min-w-40 flex-1">
                  <h3 className="font-medium">{row.name}</h3>
                  {row.help && <p className="mt-1 text-sm text-muted-foreground">{row.help}</p>}
                </div>
                <div className="flex max-w-full flex-wrap items-center gap-2">
                  {row.amount !== undefined && <output className="mr-2 font-semibold tabular-nums">{"$"}{row.amount}</output>}
                  {row.slider && <ValueSlider slider={row.slider} busy={busy} apply={apply} />}
                  {row.actions.length > 1
                    ? <ToggleGroup type="single" variant="outline" size="sm" disabled={busy} aria-label={row.name}
                        value={String(row.actions.findIndex(action => action.selected))}
                        onValueChange={next => { if (next) apply(row.actions[Number(next)].edit) }}>
                        {row.actions.map((action, index) => (
                          <ToggleGroupItem key={index} value={String(index)} disabled={!action.enabled}
                            className="h-auto min-h-8 whitespace-normal">{action.label}</ToggleGroupItem>
                        ))}
                      </ToggleGroup>
                    : row.actions.map(action => <ActionButton key={action.label} action={action} busy={busy} apply={apply} />)}
                </div>
              </li>
            ))}
          </ul>}
    </div>
  )
}

export function Progression({ sections, t, send, busy }: {
  sections: ProgressSection[]
  t: Translate
  send: (request: Request) => void
  busy: boolean
}) {
  const [selection, setSelection] = useState(sections[0].key)
  const section = sections.find(item => item.key === selection) ?? sections[0]
  return (
    <div>
      <FieldSelect label={t("progression_tab")} value={section.key}
        options={sections.map(item => [item.key, item.label])} onChange={setSelection} />
      <ProgressRows key={section.key} section={section} t={t} busy={busy}
        apply={edit => send({ kind: "progress", edit })} />
    </div>
  )
}
