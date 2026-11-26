import { createContext, useContext, useEffect, useState } from "react"
import type { ReactNode } from "react"
import { isTauri } from "@tauri-apps/api/core"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { Monitor, Moon, Sun } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu, DropdownMenuContent, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import type { Translate } from "@/lib/editor"

const themes = [
  ["light", "theme_light", Sun], ["dark", "theme_dark", Moon], ["system", "theme_system", Monitor],
] as const
type Theme = typeof themes[number][0]
const storageKey = "pocket-city-theme"
const ThemeContext = createContext<{ theme: Theme; setTheme: (theme: Theme) => void } | null>(null)

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<Theme>(() => {
    const saved = localStorage.getItem(storageKey)
    return themes.find(([value]) => value === saved)?.[0] ?? "light"
  })

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)")
    const apply = () => {
      const dark = theme === "dark" || (theme === "system" && media.matches)
      document.documentElement.classList.toggle("dark", dark)
    }
    apply()
    if (theme === "system") media.addEventListener("change", apply)
    if (isTauri()) {
      void getCurrentWindow().setTheme(theme === "system" ? null : theme).catch(console.error)
    }
    return () => media.removeEventListener("change", apply)
  }, [theme])

  return <ThemeContext.Provider value={{
    theme,
    setTheme: value => {
      localStorage.setItem(storageKey, value)
      setTheme(value)
    },
  }}>{children}</ThemeContext.Provider>
}

export function ThemeMenu({ t }: { t: Translate }) {
  const context = useContext(ThemeContext)
  if (!context) throw new Error("ThemeMenu requires ThemeProvider")
  const Icon = themes.find(([value]) => value === context.theme)?.[2] ?? Sun

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon-sm" aria-label={t("appearance")} title={t("appearance")}><Icon /></Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuRadioGroup value={context.theme} onValueChange={value => {
          const selected = themes.find(([theme]) => theme === value)
          if (selected) context.setTheme(selected[0])
        }}>
          {themes.map(([value, label, OptionIcon]) => (
            <DropdownMenuRadioItem key={value} value={value}><OptionIcon />{t(label)}</DropdownMenuRadioItem>
          ))}
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
