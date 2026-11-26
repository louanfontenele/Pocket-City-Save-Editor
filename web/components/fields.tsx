import { useState } from "react"
import { ChevronLeft, ChevronRight, FileSearch, Lock, Search } from "lucide-react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia } from "@/components/ui/empty"
import { Field, FieldContent, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group"
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import type { Drafts, FieldView, Translate } from "@/lib/editor"

export function EmptyMessage({ message }: { message: string }) {
  return (
    <Empty className="py-10">
      <EmptyHeader>
        <EmptyMedia variant="icon"><FileSearch /></EmptyMedia>
        <EmptyDescription>{message}</EmptyDescription>
      </EmptyHeader>
    </Empty>
  )
}

export function SearchBox({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  return (
    <InputGroup className="min-w-36 max-w-72 flex-1">
      <InputGroupInput type="search" name="field-search" autoComplete="off" aria-label={label}
        placeholder={label} value={value} onChange={event => onChange(event.target.value)} />
      <InputGroupAddon><Search aria-hidden="true" /></InputGroupAddon>
    </InputGroup>
  )
}

export function FieldSelect({ id, label, value, options, onChange, disabled }: {
  id?: string
  label: string
  value: string
  options: [string, string][]
  onChange: (value: string) => void
  disabled?: boolean
}) {
  return (
    <Select value={value} onValueChange={onChange} disabled={disabled}>
      <SelectTrigger id={id} aria-label={label} className="min-w-0 max-w-full">
        <SelectValue>{options.find(option => option[0] === value)?.[1] ?? value}</SelectValue>
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          {options.map(([key, name]) => <SelectItem key={key} value={key}>{name}</SelectItem>)}
        </SelectGroup>
      </SelectContent>
    </Select>
  )
}

function RecordField({ field, value, update, t, busy, advanced }: {
  field: FieldView
  value: string
  update: (value: string) => void
  t: Translate
  busy: boolean
  advanced: boolean
}) {
  const id = "field-" + field.index
  const disabled = busy || field.locked
  const choice = field.options?.find(option => option[0] === value)?.[1]
  const displayValue = choice ?? (field.type === "boolean" ? (value === "true" ? t("yes") : t("no")) : value)
  const describedBy = field.help ? id + "-help" : undefined

  return (
    <Field orientation="responsive" className="border-b py-3 last:border-b-0">
      <FieldContent className="min-w-0">
        <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
          <FieldLabel htmlFor={field.locked ? undefined : id}>{field.name}</FieldLabel>
          {field.locked && <Badge variant="outline"><Lock data-icon="inline-start" />{t("protected")}</Badge>}
        </div>
        {field.help && <FieldDescription id={describedBy}>{field.help}</FieldDescription>}
        {advanced && <p className="break-all font-mono text-[11px] text-muted-foreground">{field.path}</p>}
      </FieldContent>
      <div className="min-w-0 max-w-full @md/field-group:w-64 @md/field-group:flex-none">
        {field.locked
          ? <output className="break-all text-sm text-muted-foreground tabular-nums">{displayValue || "—"}</output>
          : field.type === "boolean"
            ? <Switch id={id} checked={value === "true"} disabled={disabled} aria-describedby={describedBy}
                onCheckedChange={checked => update(String(checked))} />
            : field.options
              ? <FieldSelect id={id} label={field.name} value={value} options={field.options}
                  onChange={update} disabled={disabled} />
              : <Input id={id} name={field.path} value={value} type="text" autoComplete="off"
                  spellCheck={false} inputMode={field.type === "number" ? "decimal" : undefined}
                  aria-describedby={describedBy} disabled={disabled} onChange={event => update(event.target.value)}
                  className="w-full tabular-nums" />}
      </div>
    </Field>
  )
}

const pageSize = 32

export function Fields({ items, t, advanced = false, busy, drafts, setDraft }: {
  items: FieldView[]
  t: Translate
  advanced?: boolean
  busy: boolean
  drafts: Drafts
  setDraft: (field: FieldView, value: string) => void
}) {
  const [page, setPage] = useState(0)
  const lastPage = Math.max(0, Math.ceil(items.length / pageSize) - 1)
  const activePage = Math.min(page, lastPage)

  if (items.length === 0) return <EmptyMessage message={t("empty_fields")} />

  return (
    <div className="flex flex-col gap-3">
      {lastPage > 0 && (
        <div className="flex items-center justify-end gap-2">
          <Button variant="outline" size="icon-sm" aria-label={t("previous_page")}
            disabled={activePage === 0} onClick={() => setPage(activePage - 1)}><ChevronLeft /></Button>
          <span className="text-xs text-muted-foreground tabular-nums">{activePage + 1} / {lastPage + 1}</span>
          <Button variant="outline" size="icon-sm" aria-label={t("next_page")}
            disabled={activePage === lastPage} onClick={() => setPage(activePage + 1)}><ChevronRight /></Button>
        </div>
      )}
      <FieldGroup className="gap-0">
        {items.slice(activePage * pageSize, (activePage + 1) * pageSize).map(field => (
          <RecordField key={field.index} field={field}
            value={drafts[field.index] ?? field.value} busy={busy} advanced={advanced} t={t}
            update={value => setDraft(field, value)} />
        ))}
      </FieldGroup>
    </div>
  )
}
