import { createRoot } from "react-dom/client"
import { App } from "./app"
import { ThemeProvider } from "./components/theme"
import "./index.css"

createRoot(document.getElementById("root")!).render(<ThemeProvider><App /></ThemeProvider>)
